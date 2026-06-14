//! LSP backend — implements tower_lsp::LanguageServer.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use cargo_cicd_core::diagnostics::CicdFinding;

use crate::analyzers::run_all;
use crate::protocol::code_action_map::finding_to_actions;
use crate::protocol::diagnostic_map::finding_to_lsp;
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
}

impl Backend {
    /// Create a new Backend instance.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_root: Arc::new(RwLock::new(None)),
            diagnostic_store: Arc::new(RwLock::new(DiagnosticStore::new())),
            capability_cache: Arc::new(RwLock::new(CapabilityCache::new())),
            receipt_index: Arc::new(RwLock::new(ReceiptIndex::new())),
            workspace_state: Arc::new(RwLock::new(WorkspaceState::new())),
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

        // Persist findings into the diagnostic store for this URI.
        {
            let mut store = self.diagnostic_store.write().await;
            store.clear_uri(&uri.to_string());
            for f in &findings {
                store.insert(uri.to_string(), f.clone());
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
        let mut cache = self.capability_cache.write().await;
        if wpm_available {
            cache.set_available("unknown");
        } else {
            cache.set_unavailable();
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.refresh_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.refresh_diagnostics(&params.text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.refresh_diagnostics(&params.text_document.uri).await;
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
            Ok(Some(actions))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let root = {
            let lock = self.workspace_root.read().await;
            lock.clone()
        };
        let Some(ref path) = root else {
            return Ok(None);
        };

        let findings = run_all(&WorkspaceSnapshot::from_path(path));

        // Used to identify which file was hovered; no per-range filtering since
        // findings don't carry precise source positions.
        let _uri = &params.text_document_position_params.text_document.uri;

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

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
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
