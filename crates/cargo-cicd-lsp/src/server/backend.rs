//! LSP backend — implements tower_lsp::LanguageServer.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use cargo_cicd_core::diagnostics::CicdFinding;

use crate::analyzers::run_all;
use crate::commands::execute_permitted;
use crate::lifecycle::raise;
use crate::protocol::code_action_map::finding_to_actions;
use crate::protocol::diagnostic_map::finding_to_lsp;
use crate::protocol::hover_map::hover_card_map;
use crate::server::capabilities::build_server_capabilities;
use crate::state::{CapabilityCache, DiagnosticStore, ReceiptIndex, WorkspaceState};
use cargo_cicd_core::workspace::WorkspaceSnapshot;

/// The LSP backend for cargo-cicd-lsp.
pub struct Backend {
    pub client: Client,
    pub workspace_root: Arc<RwLock<Option<PathBuf>>>,
    pub diagnostic_store: Arc<RwLock<DiagnosticStore>>,
    pub capability_cache: Arc<RwLock<CapabilityCache>>,
    pub receipt_index: Arc<RwLock<ReceiptIndex>>,
    pub workspace_state: Arc<RwLock<WorkspaceState>>,
    /// Sender for debounced file-watcher events.
    pub watcher_tx: mpsc::Sender<()>,
}

impl Backend {
    /// Create a new Backend instance.
    pub fn new(client: Client) -> Self {
        let (watcher_tx, _watcher_rx) = mpsc::channel(16);
        Self {
            client,
            workspace_root: Arc::new(RwLock::new(None)),
            diagnostic_store: Arc::new(RwLock::new(DiagnosticStore::new())),
            capability_cache: Arc::new(RwLock::new(CapabilityCache::new())),
            receipt_index: Arc::new(RwLock::new(ReceiptIndex::new())),
            workspace_state: Arc::new(RwLock::new(WorkspaceState::new())),
            watcher_tx,
        }
    }

    /// Run all analyzers against the workspace root and publish diagnostics.
    async fn refresh_diagnostics(&self, uri: &Url) {
        let root = {
            let lock = self.workspace_root.read().await;
            lock.clone()
        };

        let findings: Vec<CicdFinding> = match root {
            Some(ref path) => run_all(&WorkspaceSnapshot::from_path(path)),
            None => Vec::new(),
        };

        // Update the diagnostic store via lifecycle functions.
        {
            let mut store = self.diagnostic_store.write().await;
            // Clear all existing findings for this URI before raising new ones.
            store.clear_uri(uri.as_ref());
            for finding in &findings {
                raise(&mut store, uri.to_string(), finding.clone());
            }
        }

        let diagnostics: Vec<Diagnostic> = findings.iter().map(finding_to_lsp).collect();

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri {
            if let Ok(path) = root_uri.to_file_path() {
                let mut lock = self.workspace_root.write().await;
                *lock = Some(path.clone());

                let mut ws = self.workspace_state.write().await;
                ws.set_root(path);
            }
        }
        Ok(InitializeResult {
            capabilities: build_server_capabilities(),
            server_info: Some(ServerInfo {
                name: "cargo-cicd-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("cargo-cicd-lsp ready");
        self.client
            .log_message(MessageType::INFO, "cargo-cicd-lsp ready")
            .await;

        // Check wpm availability and populate the capability cache.
        let wpm_available = std::process::Command::new("wpm")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let watcher_already_registered = {
            let cache = self.capability_cache.read().await;
            cache.watcher_registered
        };

        {
            let mut cache = self.capability_cache.write().await;
            if wpm_available {
                cache.set_available("unknown");
            } else {
                cache.set_unavailable();
            }
        }

        // Register file watchers for evidence directory and receipts (F-T4).
        if !watcher_already_registered {
            let registration = Registration {
                id: "cargo-cicd-file-watcher".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(
                    serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String(
                                    "**/target/cargo-cicd/evidence/**".to_string(),
                                ),
                                kind: None,
                            },
                            FileSystemWatcher {
                                glob_pattern: GlobPattern::String(
                                    "**/*.receipt".to_string(),
                                ),
                                kind: None,
                            },
                        ],
                    })
                    .unwrap(),
                ),
            };

            if self
                .client
                .register_capability(vec![registration])
                .await
                .is_ok()
            {
                let mut cache = self.capability_cache.write().await;
                cache.watcher_registered = true;
            }
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // Populate document cache (F-T2).
        {
            let mut ws = self.workspace_state.write().await;
            ws.set_document_text(
                params.text_document.uri.to_string(),
                params.text_document.text.clone(),
            );
        }
        self.refresh_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Populate document cache with the latest full-text sync (F-T2).
        if let Some(last) = params.content_changes.last() {
            let mut ws = self.workspace_state.write().await;
            ws.set_document_text(
                params.text_document.uri.to_string(),
                last.text.clone(),
            );
        }
        self.refresh_diagnostics(&params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.refresh_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut store = self.diagnostic_store.write().await;
            store.clear_uri(uri.as_ref());
        }
        {
            let mut ws = self.workspace_state.write().await;
            ws.remove_document_text(uri.as_ref());
        }
        // Publish empty diagnostics to clear squiggles in the editor.
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        // F-T4: filter for evidence/receipt files and refresh diagnostics.
        let relevant = params.changes.iter().any(|e| {
            let path = e.uri.path();
            path.contains("events.jsonl")
                || path.ends_with(".receipt")
                || path.contains("cargo-cicd/evidence")
        });

        if relevant {
            // Signal the debounce channel.
            let _ = self.watcher_tx.try_send(());

            // Refresh diagnostics for the workspace cicd.toml.
            let cicd_toml_uri = {
                let lock = self.workspace_root.read().await;
                lock.as_ref().and_then(|root| {
                    Url::from_file_path(root.join("cicd.toml")).ok()
                })
            };
            if let Some(uri) = cicd_toml_uri {
                self.refresh_diagnostics(&uri).await;
            }
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let root = {
            let lock = self.workspace_root.read().await;
            lock.clone()
        };

        let findings: Vec<CicdFinding> = match root {
            Some(ref path) => run_all(&WorkspaceSnapshot::from_path(path)),
            None => return Ok(None),
        };

        let uri = &params.text_document.uri;
        let actions: Vec<CodeActionOrCommand> = findings
            .iter()
            .flat_map(|f| finding_to_actions(f, uri, None))
            .map(CodeActionOrCommand::CodeAction)
            .collect();

        if actions.is_empty() {
            Ok(None)
        } else {
            // Minimal lifecycle hook: ensure the store lock is cleanly released after
            // acknowledging that repair actions exist for this URI. mark_pending is
            // called per-finding via the lifecycle module when repair is initiated.
            {
                let store = self.diagnostic_store.write().await;
                drop(store);
            }
            Ok(Some(actions))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Retrieve document text from cache (F-T2).
        let doc_text = {
            let ws = self.workspace_state.read().await;
            ws.get_document_text(uri.as_ref()).map(|s| s.to_string())
        };

        // Try per-field hover card if we have the document text.
        if let Some(text) = doc_text {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(line_text) = lines.get(position.line as usize) {
                // Extract the TOML key: take the part before '=' and trim.
                let key = line_text
                    .split('=')
                    .next()
                    .map(|k| k.trim().trim_matches('"'))
                    .unwrap_or("")
                    .to_string();

                if !key.is_empty() {
                    let map = hover_card_map();
                    if let Some(card) = map.get(key.as_str()) {
                        // Check DiagnosticStore for active findings with this code.
                        let active_finding = {
                            let store = self.diagnostic_store.read().await;
                            store
                                .get_all(uri.as_ref())
                                .iter()
                                .any(|f| f.code.as_str() == card.code)
                        };

                        let active_note = if active_finding {
                            format!("\n\n> **Active finding:** `{}`", card.code)
                        } else {
                            String::new()
                        };

                        let md = format!(
                            "### `{}` — {}\n\n**Section:** `{}`  \n**Controls:** {}  \n**Repair:** {}{}\n",
                            card.field,
                            card.code,
                            card.section,
                            card.controls,
                            card.repair_hint,
                            active_note,
                        );

                        // Range covers the key token on the hovered line.
                        let key_start = line_text
                            .find(key.as_str())
                            .unwrap_or(0) as u32;
                        let key_end = key_start + key.len() as u32;
                        let range = Range {
                            start: Position {
                                line: position.line,
                                character: key_start,
                            },
                            end: Position {
                                line: position.line,
                                character: key_end,
                            },
                        };

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: md,
                            }),
                            range: Some(range),
                        }));
                    }
                }
            }
        }

        // Fallback: show all active workspace findings.
        let root = {
            let lock = self.workspace_root.read().await;
            lock.clone()
        };
        let Some(ref path) = root else {
            return Ok(None);
        };

        let findings = run_all(&WorkspaceSnapshot::from_path(path));

        if findings.is_empty() {
            return Ok(None);
        }

        let mut lines = vec!["**cargo-cicd diagnostics**\n".to_string()];
        for finding in &findings {
            lines.push(format!(
                "- `{}` **{}**: {}",
                finding.code.as_str(),
                format!("{:?}", finding.severity).to_lowercase(),
                finding.message
            ));
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lines.join("\n"),
            }),
            range: None,
        }))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        if params.command != "cargo-cicd.execute" {
            return Ok(None);
        }

        let cmd = params
            .arguments
            .into_iter()
            .next()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        match execute_permitted(&cmd).await {
            Ok(stdout) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("cargo-cicd command succeeded: {}\n{}", cmd, stdout),
                    )
                    .await;
            }
            Err(stderr) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("cargo-cicd command failed: {}\n{}", cmd, stderr),
                    )
                    .await;
            }
        }

        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        use cargo_cicd_core::diagnostics::CicdCode;

        let codes = [
            CicdCode::GitDirtyTreeBlocksClose,
            CicdCode::GitUntrackedArtifacts,
            CicdCode::EvidenceMissing,
            CicdCode::EvidenceStale,
            CicdCode::EvidenceHardcodedTimestamp,
            CicdCode::EvidenceMissingCaseId,
            CicdCode::WpmUnconfirmedReceiptCourt,
            CicdCode::WpmCommandUnavailable,
            CicdCode::WpmRuntimeCourtNotInvoked,
            CicdCode::WpmVerdictKeyMismatch,
            CicdCode::TargetDirOversize,
            CicdCode::PublishDryRunWithoutReceipt,
            CicdCode::FalseCloseRisk,
            CicdCode::GgenRenderedSurfaceDrift,
            CicdCode::PublicPrivateTermLeak,
        ];

        let items: Vec<CompletionItem> = codes
            .iter()
            .map(|code| CompletionItem {
                label: code.as_str().to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some(code.description().to_string()),
                documentation: Some(Documentation::String(code.repair_hint().to_string())),
                ..CompletionItem::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }
}
