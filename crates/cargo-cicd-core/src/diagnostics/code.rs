//! Diagnostic code enum.

/// Diagnostic code for a cicd finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CicdCode {
    BoundaryPublicApiLeak,
    // GIT family
    BranchBehindRemote,
    EvidenceHardcodedTimestamp,
    EvidenceMissing,
    EvidenceMissingCaseId,
    // EVIDENCE family
    EvidenceStale,
    ReceiptBeforeCourt,
    FalseCloseRisk,
    GgenCustomRegionMissing,
    GgenDriftDetected,
    GgenRenderedSurfaceDrift,
    GitDirtyTreeBlocksClose,
    GitUntrackedArtifacts,
    // PIPELINE family
    NoCicdTomlFound,
    PipelineStageFailed,
    PublicPrivateTermLeak,
    PublishDryRunWithoutReceipt,
    PublishNoCicdToml,
    PublishNoReceipt,
    // SPEC family
    SpecMissingForChange,
    TargetDirOversize,
    // TARGET family
    TargetPruneRequiresDryRun,
    // SPEC family (continued)
    TaskDoneWithoutEvidence,
    // TEST family
    TestFailuresBlockClose,
    TestsImpactUnknown,
    TestsStaleMapping,
    TrybuildFixtureChanged,
    WpmCommandUnavailable,
    WpmRuntimeCourtNotInvoked,
    WpmUnconfirmedReceiptCourt,
    /// CICD-WPM-004: The external court emitted a verdict field that the consuming
    /// audit surface does not read — e.g. court emits `overall_fitness` but reader
    /// looks for `fitness`. Verdict silently degrades to zero.
    WpmVerdictKeyMismatch,
    // WORKSPACE family
    WorkspaceStructureInvalid,
}

impl CicdCode {
    /// Return the stable string identifier for this code.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak => "CICD-PUBLIC-001",
            Self::BranchBehindRemote => "CICD-GIT-003",
            Self::EvidenceHardcodedTimestamp => "CICD-EVIDENCE-003",
            Self::EvidenceMissing => "CICD-EVIDENCE-001",
            Self::EvidenceMissingCaseId => "CICD-EVIDENCE-004",
            Self::EvidenceStale => "CICD-EVIDENCE-002",
            Self::FalseCloseRisk => "CICD-CLOSE-001",
            Self::GgenCustomRegionMissing => "CICD-GGEN-001",
            Self::GgenDriftDetected => "CICD-GGEN-002",
            Self::GgenRenderedSurfaceDrift => "CICD-GGEN-003",
            Self::GitDirtyTreeBlocksClose => "CICD-GIT-001",
            Self::GitUntrackedArtifacts => "CICD-GIT-002",
            Self::NoCicdTomlFound => "CICD-PIPELINE-002",
            Self::PipelineStageFailed => "CICD-PIPELINE-001",
            Self::PublicPrivateTermLeak => "CICD-PUBLIC-002",
            Self::PublishDryRunWithoutReceipt => "CICD-PUBLISH-002",
            Self::PublishNoCicdToml => "CICD-PUBLISH-001",
            Self::PublishNoReceipt => "CICD-PUBLISH-003",
            Self::ReceiptBeforeCourt => "CICD-EVIDENCE-005",
            Self::SpecMissingForChange => "CICD-SPEC-001",
            Self::TargetDirOversize => "CICD-TARGET-001",
            Self::TargetPruneRequiresDryRun => "CICD-TARGET-002",
            Self::TaskDoneWithoutEvidence => "CICD-SPEC-002",
            Self::TestFailuresBlockClose => "CICD-TEST-001",
            Self::TestsImpactUnknown => "CICD-TESTS-002",
            Self::TestsStaleMapping => "CICD-TESTS-001",
            Self::TrybuildFixtureChanged => "CICD-TEST-002",
            Self::WpmCommandUnavailable => "CICD-WPM-001",
            Self::WpmRuntimeCourtNotInvoked => "CICD-WPM-003",
            Self::WpmUnconfirmedReceiptCourt => "CICD-WPM-002",
            Self::WpmVerdictKeyMismatch => "CICD-WPM-004",
            Self::WorkspaceStructureInvalid => "CICD-WORKSPACE-001",
        }
    }

    /// Return a short human-readable title for this code.
    pub fn title(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak => "Public API boundary leak",
            Self::BranchBehindRemote => "Branch behind remote",
            Self::EvidenceHardcodedTimestamp => "Hardcoded evidence timestamp",
            Self::EvidenceMissing => "Evidence file missing",
            Self::EvidenceMissingCaseId => "Evidence missing case_id",
            Self::EvidenceStale => "Evidence file is stale",
            Self::FalseCloseRisk => "False-close risk",
            Self::GgenCustomRegionMissing => "ggen custom region missing",
            Self::GgenDriftDetected => "ggen drift detected",
            Self::GgenRenderedSurfaceDrift => "ggen rendered surface drift",
            Self::GitDirtyTreeBlocksClose => "Dirty git tree blocks close",
            Self::GitUntrackedArtifacts => "Untracked git artifacts",
            Self::NoCicdTomlFound => "No cicd.toml found",
            Self::PipelineStageFailed => "Pipeline stage failed",
            Self::PublicPrivateTermLeak => "Private term in public surface",
            Self::PublishDryRunWithoutReceipt => "Dry-run without receipt",
            Self::PublishNoCicdToml => "No cicd.toml for publish",
            Self::PublishNoReceipt => "Package changed after dry-run",
            Self::ReceiptBeforeCourt => "Receipt written before court adjudication",
            Self::SpecMissingForChange => "Spec missing for changed file",
            Self::TargetDirOversize => "Target directory oversize",
            Self::TargetPruneRequiresDryRun => "Target prune requires dry-run",
            Self::TaskDoneWithoutEvidence => "Task done without evidence",
            Self::TestFailuresBlockClose => "Test failures block close",
            Self::TestsImpactUnknown => "Test impact unknown",
            Self::TestsStaleMapping => "Test-to-source mapping stale",
            Self::TrybuildFixtureChanged => "Trybuild fixture changed",
            Self::WpmCommandUnavailable => "wpm command unavailable",
            Self::WpmRuntimeCourtNotInvoked => "Runtime court not invoked",
            Self::WpmUnconfirmedReceiptCourt => "wpm receipt court unconfirmed",
            Self::WpmVerdictKeyMismatch => {
                "Verdict key mismatch between court output and audit reader"
            }
            Self::WorkspaceStructureInvalid => "Workspace structure invalid",
        }
    }

    /// Returns a short human-readable description of this diagnostic code.
    pub fn description(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak => "a private/forbidden term was found in a public-facing document",
            Self::BranchBehindRemote => "local branch is behind its remote tracking branch",
            Self::EvidenceHardcodedTimestamp => "evidence contains a hardcoded timestamp instead of a real UTC time",
            Self::EvidenceMissing => "no process evidence directory found",
            Self::EvidenceMissingCaseId => "evidence events lack a session/case identifier",
            Self::EvidenceStale => "evidence is older than the last source change",
            Self::FalseCloseRisk => "one or more serious diagnostics are active; phase closure would be premature",
            Self::GgenCustomRegionMissing => "a ggen-managed file is missing its custom region markers",
            Self::GgenDriftDetected => "a ggen-rendered block differs from its source law",
            Self::GgenRenderedSurfaceDrift => "rendered surface is out of date with its source law",
            Self::GitDirtyTreeBlocksClose => "the working tree has uncommitted changes",
            Self::GitUntrackedArtifacts => "untracked files exist that may represent unintended output",
            Self::NoCicdTomlFound => "cicd.toml not found at workspace root",
            Self::PipelineStageFailed => "a pipeline stage reported failure",
            Self::PublicPrivateTermLeak => "a private/forbidden term was found in a public-facing document",
            Self::PublishDryRunWithoutReceipt => "publish dry-run completed but no adjudicated receipt exists",
            Self::PublishNoCicdToml => "cargo publish was attempted without a cicd.toml",
            Self::PublishNoReceipt => "no publish receipt exists for this version",
            Self::ReceiptBeforeCourt => "a receipt was written before wpm adjudicated the evidence",
            Self::SpecMissingForChange => "changed files have no corresponding spec entry",
            Self::TargetDirOversize => "target directory is large and may need pruning",
            Self::TargetPruneRequiresDryRun => "target prune should be reviewed with --dry-run before --apply",
            Self::TaskDoneWithoutEvidence => "a task is marked complete but no fresh evidence exists",
            Self::TestFailuresBlockClose => "test failures must be resolved before phase closure",
            Self::TestsImpactUnknown => "changed test files have not been run",
            Self::TestsStaleMapping => "test-to-source mapping is stale; re-run test changed",
            Self::TrybuildFixtureChanged => "trybuild fixtures were modified; re-run to confirm",
            Self::WorkspaceStructureInvalid => "workspace Cargo.toml structure has validation errors",
            Self::WpmCommandUnavailable => "wpm binary not found or command failed",
            Self::WpmRuntimeCourtNotInvoked => "wpm receipt doctor has not been called for current evidence",
            Self::WpmUnconfirmedReceiptCourt => "wpm binary not found or receipt doctor not confirmed",
            Self::WpmVerdictKeyMismatch => "court verdict key mismatch: audit reads wrong key",
        }
    }

    /// Returns all known variants of `CicdCode`.
    pub fn all_variants() -> Vec<Self> {
        vec![
            Self::BoundaryPublicApiLeak,
            Self::BranchBehindRemote,
            Self::EvidenceHardcodedTimestamp,
            Self::EvidenceMissing,
            Self::EvidenceMissingCaseId,
            Self::EvidenceStale,
            Self::FalseCloseRisk,
            Self::GgenCustomRegionMissing,
            Self::GgenDriftDetected,
            Self::GgenRenderedSurfaceDrift,
            Self::GitDirtyTreeBlocksClose,
            Self::GitUntrackedArtifacts,
            Self::NoCicdTomlFound,
            Self::PipelineStageFailed,
            Self::PublicPrivateTermLeak,
            Self::PublishDryRunWithoutReceipt,
            Self::PublishNoCicdToml,
            Self::PublishNoReceipt,
            Self::ReceiptBeforeCourt,
            Self::SpecMissingForChange,
            Self::TargetDirOversize,
            Self::TargetPruneRequiresDryRun,
            Self::TaskDoneWithoutEvidence,
            Self::TestFailuresBlockClose,
            Self::TestsImpactUnknown,
            Self::TestsStaleMapping,
            Self::TrybuildFixtureChanged,
            Self::WpmCommandUnavailable,
            Self::WpmRuntimeCourtNotInvoked,
            Self::WpmUnconfirmedReceiptCourt,
            Self::WpmVerdictKeyMismatch,
            Self::WorkspaceStructureInvalid,
        ]
    }

    /// Returns a short repair hint for this diagnostic code.
    pub fn repair_hint(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak | Self::PublicPrivateTermLeak => "remove or replace the forbidden term in the public-facing file",
            Self::BranchBehindRemote => "run git pull --rebase to sync with remote",
            Self::EvidenceHardcodedTimestamp => "use SystemTime::now() in evidence emission code",
            Self::EvidenceMissing => "run any cargo cicd command to emit process evidence",
            Self::EvidenceMissingCaseId => "ensure case_id is set on all emitted ProcessEvents",
            Self::EvidenceStale => "run cargo cicd test changed; cargo cicd workspace doctor",
            Self::FalseCloseRisk => "resolve all Error-severity diagnostics before claiming phase closure",
            Self::GgenCustomRegionMissing => "add the expected custom block markers to the ggen-managed file",
            Self::GgenDriftDetected | Self::GgenRenderedSurfaceDrift => "run ggen sync to regenerate rendered surfaces",
            Self::GitDirtyTreeBlocksClose => "run cargo cicd git status then commit or stash changes",
            Self::GitUntrackedArtifacts => "stage or .gitignore untracked files",
            Self::NoCicdTomlFound => "run cargo cicd publish run to generate cicd.toml",
            Self::PipelineStageFailed => "run cargo cicd pipeline run and address reported failures",
            Self::PublishDryRunWithoutReceipt => "run cargo cicd evidence doctor then cargo cicd publish",
            Self::PublishNoCicdToml | Self::PublishNoReceipt => "run cargo cicd publish run to generate the required artifacts",
            Self::ReceiptBeforeCourt => "run cargo cicd evidence doctor to adjudicate before writing receipt",
            Self::SpecMissingForChange => "add a spec entry for changed files or run /speckit.specify",
            Self::TargetDirOversize => "run cargo cicd target show then cargo cicd target prune",
            Self::TargetPruneRequiresDryRun => "run cargo cicd target prune then add --apply to execute",
            Self::TaskDoneWithoutEvidence => "run the manufacturing pipeline to produce fresh evidence",
            Self::TestFailuresBlockClose => "run cargo cicd test run and fix failing tests",
            Self::TestsImpactUnknown => "run cargo cicd test changed",
            Self::TestsStaleMapping => "run cargo cicd test changed to refresh the test mapping",
            Self::TrybuildFixtureChanged => "re-run trybuild to confirm fixtures still compile as expected",
            Self::WorkspaceStructureInvalid => "run cargo cicd workspace validate to diagnose structural issues",
            Self::WpmCommandUnavailable | Self::WpmUnconfirmedReceiptCourt => "install wasm4pm or set WPM_BIN env var",
            Self::WpmRuntimeCourtNotInvoked => "run cargo cicd evidence doctor",
            Self::WpmVerdictKeyMismatch => "align court output schema with audit reader",
        }
    }
}

impl std::fmt::Display for CicdCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<CicdCode> for String {
    fn from(c: CicdCode) -> String {
        c.as_str().to_string()
    }
}

impl std::str::FromStr for CicdCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::all_variants() {
            if variant.as_str() == s {
                return Ok(variant);
            }
        }
        Err(format!("unknown CicdCode: {}", s))
    }
}

/// Return a human-readable explanation string for a diagnostic code string like "CICD-GIT-001".
///
/// Returns `None` if the code string is not recognized.
pub fn explain_code(code_str: &str) -> Option<String> {
    // Find the CicdCode variant matching this string.
    let code = [
        CicdCode::BoundaryPublicApiLeak,
        CicdCode::BranchBehindRemote,
        CicdCode::EvidenceHardcodedTimestamp,
        CicdCode::EvidenceMissing,
        CicdCode::EvidenceMissingCaseId,
        CicdCode::EvidenceStale,
        CicdCode::FalseCloseRisk,
        CicdCode::GgenCustomRegionMissing,
        CicdCode::GgenDriftDetected,
        CicdCode::GgenRenderedSurfaceDrift,
        CicdCode::GitDirtyTreeBlocksClose,
        CicdCode::GitUntrackedArtifacts,
        CicdCode::NoCicdTomlFound,
        CicdCode::PipelineStageFailed,
        CicdCode::PublicPrivateTermLeak,
        CicdCode::PublishDryRunWithoutReceipt,
        CicdCode::PublishNoCicdToml,
        CicdCode::PublishNoReceipt,
        CicdCode::ReceiptBeforeCourt,
        CicdCode::SpecMissingForChange,
        CicdCode::TargetDirOversize,
        CicdCode::TargetPruneRequiresDryRun,
        CicdCode::TaskDoneWithoutEvidence,
        CicdCode::TestFailuresBlockClose,
        CicdCode::TestsImpactUnknown,
        CicdCode::TestsStaleMapping,
        CicdCode::TrybuildFixtureChanged,
        CicdCode::WorkspaceStructureInvalid,
        CicdCode::WpmCommandUnavailable,
        CicdCode::WpmRuntimeCourtNotInvoked,
        CicdCode::WpmUnconfirmedReceiptCourt,
        CicdCode::WpmVerdictKeyMismatch,
    ]
    .into_iter()
    .find(|c| c.as_str() == code_str)?;

    Some(format!(
        "Code:    {}\nSummary: {}\nRepair:  {}",
        code.as_str(),
        code.description(),
        code.repair_hint()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cicd_code_display_matches_as_str() {
        let code = CicdCode::GitDirtyTreeBlocksClose;
        assert_eq!(format!("{}", code), code.as_str());
    }

    #[test]
    fn cicd_code_from_str_roundtrip() {
        let code = CicdCode::TargetDirOversize;
        let s = code.as_str();
        let back: CicdCode = s.parse().unwrap();
        assert_eq!(back.as_str(), s);
    }

    #[test]
    fn all_variants_returns_non_empty() {
        assert!(!CicdCode::all_variants().is_empty());
    }
}
