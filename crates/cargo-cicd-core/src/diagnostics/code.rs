//! Diagnostic code enum.

/// Diagnostic code for a cicd finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CicdCode {
    BoundaryPublicApiLeak,
    EvidenceHardcodedTimestamp,
    EvidenceMissing,
    EvidenceMissingCaseId,
    EvidenceStale,
    FalseCloseRisk,
    GgenCustomRegionMissing,
    GgenDriftDetected,
    GgenRenderedSurfaceDrift,
    GitDirtyTreeBlocksClose,
    GitUntrackedArtifacts,
    PublicPrivateTermLeak,
    PublishDryRunWithoutReceipt,
    PublishNoCicdToml,
    PublishNoReceipt,
    TargetDirOversize,
    TestsImpactUnknown,
    TestsStaleMapping,
    WpmCommandUnavailable,
    WpmRuntimeCourtNotInvoked,
    WpmUnconfirmedReceiptCourt,
}

impl CicdCode {
    /// Return the stable string identifier for this code.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak => "CICD-PUBLIC-001",
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
            Self::PublicPrivateTermLeak => "CICD-PUBLIC-002",
            Self::PublishDryRunWithoutReceipt => "CICD-PUBLISH-002",
            Self::PublishNoCicdToml => "CICD-PUBLISH-001",
            Self::PublishNoReceipt => "CICD-PUBLISH-003",
            Self::TargetDirOversize => "CICD-TARGET-001",
            Self::TestsImpactUnknown => "CICD-TESTS-002",
            Self::TestsStaleMapping => "CICD-TESTS-001",
            Self::WpmCommandUnavailable => "CICD-WPM-001",
            Self::WpmRuntimeCourtNotInvoked => "CICD-WPM-003",
            Self::WpmUnconfirmedReceiptCourt => "CICD-WPM-002",
        }
    }

    /// Return a short human-readable title for this code.
    pub fn title(self) -> &'static str {
        match self {
            Self::BoundaryPublicApiLeak => "Public API boundary leak",
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
            Self::PublicPrivateTermLeak => "Private term in public surface",
            Self::PublishDryRunWithoutReceipt => "Dry-run without receipt",
            Self::PublishNoCicdToml => "No cicd.toml for publish",
            Self::PublishNoReceipt => "Package changed after dry-run",
            Self::TargetDirOversize => "Target directory oversize",
            Self::TestsImpactUnknown => "Test impact unknown",
            Self::TestsStaleMapping => "Test-to-source mapping stale",
            Self::WpmCommandUnavailable => "wpm command unavailable",
            Self::WpmRuntimeCourtNotInvoked => "Runtime court not invoked",
            Self::WpmUnconfirmedReceiptCourt => "wpm receipt court unconfirmed",
        }
    }
}
