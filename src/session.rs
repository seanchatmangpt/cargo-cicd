//! Session trace grouping.
//!
//! A *session* identifies a single invocation context (binary execution + pid).
//! All process events emitted during a session share the same `case_id`, so
//! wasm4pm can correlate them into a single XES `<trace>` for process mining.

use std::path::Path;
use std::time::SystemTime;

/// Read an existing session ID from `<evidence_dir>/.session`, or generate a
/// fresh one and persist it.
///
/// The generated ID has the form `"sess-{hex_ns}-{hex_pid}"` and is guaranteed
/// to be unique across concurrent processes on the same host.
pub fn read_or_create_session_id(evidence_dir: &Path) -> String {
    let f = evidence_dir.join(".session");
    if let Ok(id) = std::fs::read_to_string(&f) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return id;
        }
    }
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let id = format!(
        "sess-{:016x}-{:08x}",
        ts as u64, // lower 64 bits of nanoseconds
        pid
    );
    let _ = std::fs::create_dir_all(evidence_dir);
    let _ = std::fs::write(&f, &id);
    id
}
