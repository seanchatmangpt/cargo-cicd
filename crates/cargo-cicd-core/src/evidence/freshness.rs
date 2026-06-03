//! Evidence freshness check.

use std::path::Path;
use std::time::{Duration, SystemTime};

/// Maximum age before evidence is considered stale (1 hour).
const STALE_AGE: Duration = Duration::from_secs(3600);

/// Freshness verdict for a piece of evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshnessVerdict {
    Fresh,
    Stale,
    Missing,
}

/// State of evidence on disk.
pub struct EvidenceState {
    pub exists: bool,
    pub freshness: FreshnessVerdict,
}

impl Default for EvidenceState {
    fn default() -> Self {
        Self {
            exists: false,
            freshness: FreshnessVerdict::Missing,
        }
    }
}

impl EvidenceState {
    /// Derive evidence state from the evidence directory and workspace root.
    pub fn from_dir(evidence_dir: &Path, _workspace_root: &Path) -> Self {
        let jsonl = evidence_dir.join("events.jsonl");
        if !jsonl.exists() {
            return Self::default();
        }
        // First check filesystem mtime.
        let fs_stale = match jsonl.metadata().and_then(|m| m.modified()).ok() {
            None => true,
            Some(mtime) => SystemTime::now()
                .duration_since(mtime)
                .map(|age| age > STALE_AGE)
                .unwrap_or(true),
        };

        // Also check if newest event timestamp in content is stale.
        let content_stale = std::fs::read_to_string(&jsonl)
            .map(|c| is_content_stale(&c))
            .unwrap_or(false);

        let freshness = if fs_stale || content_stale {
            FreshnessVerdict::Stale
        } else {
            FreshnessVerdict::Fresh
        };
        Self {
            exists: true,
            freshness,
        }
    }
}

/// Returns true when the newest ISO-8601 timestamp found in JSONL content is
/// older than [`STALE_AGE`].
fn is_content_stale(content: &str) -> bool {
    let newest = content
        .lines()
        .filter_map(|line| {
            let key = "\"timestamp\":\"";
            let start = line.find(key)? + key.len();
            let rest = &line[start..];
            let end = rest.find('"')?;
            parse_iso8601_to_system_time(&rest[..end])
        })
        .max();

    match newest {
        None => false,
        Some(t) => SystemTime::now()
            .duration_since(t)
            .map(|age| age > STALE_AGE)
            .unwrap_or(false),
    }
}

/// Minimal ISO-8601 parser for `YYYY-MM-DDTHH:MM:SS`.
fn parse_iso8601_to_system_time(ts: &str) -> Option<SystemTime> {
    use std::time::UNIX_EPOCH;
    if ts.len() < 19 {
        return None;
    }
    let year: i64 = ts[0..4].parse().ok()?;
    let month: i64 = ts[5..7].parse().ok()?;
    let day: i64 = ts[8..10].parse().ok()?;
    let hour: i64 = ts[11..13].parse().ok()?;
    let min: i64 = ts[14..16].parse().ok()?;
    let sec: i64 = ts[17..19].parse().ok()?;

    let years_since_1970 = year - 1970;
    let leap_days = years_since_1970 / 4;
    let days = years_since_1970 * 365 + leap_days + days_before_month(year, month) + (day - 1);
    let secs = days * 86400 + hour * 3600 + min * 60 + sec;
    if secs < 0 {
        return None;
    }
    Some(UNIX_EPOCH + Duration::from_secs(secs as u64))
}

fn days_before_month(year: i64, month: i64) -> i64 {
    const DAYS: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let m = (month - 1).clamp(0, 11) as usize;
    let leap = if month > 2 && year % 4 == 0 { 1 } else { 0 };
    DAYS[m] + leap
}
