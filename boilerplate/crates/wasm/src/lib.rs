/// project-wasm: WASM-bindgen entry points.
///
/// All public functions follow the JSON-in / JSON-out convention:
///   - Inputs arrive as `&str` (serialised JSON).
///   - Outputs leave as `Result<String, JsValue>` (serialised JSON or a JS error).
///
/// Domain logic lives in [`api`] and [`arena`] so it stays testable without a
/// WASM runtime.
pub mod api;
pub mod arena;

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

/// Called automatically when the WASM module is instantiated.
///
/// Sets up [`console_error_panic_hook`] so that Rust panics are surfaced as
/// readable messages in the browser console rather than an opaque
/// `RuntimeError: unreachable`.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert any `Display` value into a [`JsValue`] error string.
#[inline]
fn to_js_error(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Deserialise a JSON `&str` into `T`, mapping errors to [`JsValue`].
fn parse_input<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, JsValue> {
    serde_json::from_str(json).map_err(to_js_error)
}

// ---------------------------------------------------------------------------
// Public WASM API
// ---------------------------------------------------------------------------

/// Return the crate version string (from `Cargo.toml`).
///
/// ```js
/// import init, { get_version } from "./pkg/project_wasm.js";
/// await init();
/// console.log(get_version()); // "0.1.0"
/// ```
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Analyse a dataset supplied as JSON.
///
/// # Input JSON shape
/// ```json
/// { "data": [1.0, 2.5, 3.7] }
/// ```
///
/// # Output JSON shape
/// ```json
/// { "count": 3, "sum": 7.2, "mean": 2.4, "min": 1.0, "max": 3.7 }
/// ```
///
/// Returns a JS `Error` string on malformed input or domain errors.
#[wasm_bindgen]
pub fn analyze(input_json: &str) -> Result<String, JsValue> {
    let input: api::AnalysisInput = parse_input(input_json)?;
    let output = api::analyze(input).map_err(to_js_error)?;
    serde_json::to_string(&output).map_err(to_js_error)
}

/// Process a batch of labelled items supplied as JSON.
///
/// # Input JSON shape
/// ```json
/// [
///   { "id": "a", "value": 1.0 },
///   { "id": "b", "value": 2.0 }
/// ]
/// ```
///
/// # Output JSON shape
/// ```json
/// {
///   "processed": 2,
///   "results": [
///     { "id": "a", "output": 2.0, "status": "ok" },
///     { "id": "b", "output": 4.0, "status": "ok" }
///   ]
/// }
/// ```
#[wasm_bindgen]
pub fn process_batch(items_json: &str) -> Result<String, JsValue> {
    let items: Vec<api::BatchItem> = parse_input(items_json)?;
    let output = api::process_batch(items);
    serde_json::to_string(&output).map_err(to_js_error)
}

// ---------------------------------------------------------------------------
// Tests (run with `wasm-pack test --headless --firefox` or `--chrome`)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Tests that do not use browser APIs can also run with `cargo test` via rlib.

    #[test]
    fn version_is_non_empty() {
        assert!(!get_version().is_empty());
    }

    #[test]
    fn analyze_happy_path() {
        let json = r#"{"data":[1.0,2.0,3.0]}"#;
        let result = analyze(json).expect("analyze should succeed");
        assert!(result.contains("\"count\":3"));
        assert!(result.contains("\"sum\":6.0"));
    }

    #[test]
    fn analyze_empty_data_returns_error() {
        let json = r#"{"data":[]}"#;
        assert!(analyze(json).is_err(), "empty data should be an error");
    }

    #[test]
    fn analyze_bad_json_returns_error() {
        assert!(analyze("not json").is_err());
    }

    #[test]
    fn process_batch_happy_path() {
        let json = r#"[{"id":"x","value":5.0}]"#;
        let result = process_batch(json).expect("process_batch should succeed");
        assert!(result.contains("\"processed\":1"));
        assert!(result.contains("\"id\":\"x\""));
    }

    #[test]
    fn process_batch_empty_array() {
        let json = r#"[]"#;
        let result = process_batch(json).expect("empty batch should succeed");
        assert!(result.contains("\"processed\":0"));
    }

    #[test]
    fn to_js_error_formats_display() {
        let err = to_js_error("boom");
        assert_eq!(err.as_string().unwrap(), "boom");
    }

    #[test]
    fn parse_input_valid_json() {
        let s = r#"{"data":[1.0]}"#;
        let parsed: api::AnalysisInput = parse_input(s).expect("valid json");
        assert_eq!(parsed.data, vec![1.0_f64]);
    }

    #[test]
    fn parse_input_invalid_json_returns_err() {
        let result: Result<api::AnalysisInput, _> = parse_input("{bad}");
        assert!(result.is_err());
    }

    // Browser-only test — only runs under wasm-bindgen-test runner
    #[wasm_bindgen_test]
    fn version_in_browser() {
        assert!(!get_version().is_empty());
    }
}
