/// Domain logic for the WASM API surface.
///
/// This module is intentionally WASM-agnostic: no `wasm_bindgen` imports, no
/// `JsValue`.  All types are plain Rust structs that can be unit-tested with
/// `cargo test` on any host platform.
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can be returned by domain functions.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("input data is empty — at least one element is required")]
    EmptyData,
    #[error("invalid value: {0}")]
    InvalidValue(String),
}

// ---------------------------------------------------------------------------
// Analysis API
// ---------------------------------------------------------------------------

/// Input for [`analyze`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalysisInput {
    /// Non-empty slice of floating-point values to summarise.
    pub data: Vec<f64>,
}

/// Descriptive statistics returned by [`analyze`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalysisOutput {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

/// Compute descriptive statistics over `input.data`.
///
/// Returns [`CoreError::EmptyData`] when the slice is empty.
///
/// # Example
/// ```
/// use project_wasm::api::{analyze, AnalysisInput};
/// let out = analyze(AnalysisInput { data: vec![1.0, 2.0, 3.0] }).unwrap();
/// assert_eq!(out.count, 3);
/// assert!((out.mean - 2.0).abs() < f64::EPSILON);
/// ```
pub fn analyze(input: AnalysisInput) -> Result<AnalysisOutput, CoreError> {
    if input.data.is_empty() {
        return Err(CoreError::EmptyData);
    }

    let count = input.data.len();
    let sum: f64 = input.data.iter().sum();
    let mean = sum / count as f64;
    // Unwraps are safe: slice is non-empty and f64 implements PartialOrd.
    let min = input
        .data
        .iter()
        .copied()
        .reduce(f64::min)
        .expect("non-empty");
    let max = input
        .data
        .iter()
        .copied()
        .reduce(f64::max)
        .expect("non-empty");

    Ok(AnalysisOutput {
        count,
        sum,
        mean,
        min,
        max,
    })
}

// ---------------------------------------------------------------------------
// Batch-processing API
// ---------------------------------------------------------------------------

/// A single item in a batch request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchItem {
    /// Caller-supplied identifier; echoed back in [`BatchResult`].
    pub id: String,
    /// Numeric payload to transform.
    pub value: f64,
}

/// Aggregate result for a batch operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchOutput {
    /// Number of items that were processed.
    pub processed: usize,
    /// Per-item results in input order.
    pub results: Vec<BatchResult>,
}

/// Per-item result within a [`BatchOutput`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchResult {
    /// Identifier echoed from the corresponding [`BatchItem`].
    pub id: String,
    /// Transformed output value (currently `value * 2.0`).
    pub output: f64,
    /// `"ok"` on success; a human-readable error string otherwise.
    pub status: String,
}

/// Process a batch of items, doubling each value.
///
/// This function is infallible: every item produces a result.  If a value is
/// `NaN` or infinite the status will be `"invalid"` and `output` will be `0.0`.
///
/// # Example
/// ```
/// use project_wasm::api::{process_batch, BatchItem};
/// let out = process_batch(vec![BatchItem { id: "a".into(), value: 3.0 }]);
/// assert_eq!(out.processed, 1);
/// assert!((out.results[0].output - 6.0).abs() < f64::EPSILON);
/// ```
pub fn process_batch(items: Vec<BatchItem>) -> BatchOutput {
    let processed = items.len();
    let results = items
        .into_iter()
        .map(|item| {
            if item.value.is_finite() {
                BatchResult {
                    id: item.id,
                    output: item.value * 2.0,
                    status: "ok".to_owned(),
                }
            } else {
                BatchResult {
                    id: item.id,
                    output: 0.0,
                    status: format!("invalid: value is {}", item.value),
                }
            }
        })
        .collect();

    BatchOutput { processed, results }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- analyze ---

    #[test]
    fn analyze_single_element() {
        let out = analyze(AnalysisInput { data: vec![7.0] }).unwrap();
        assert_eq!(out.count, 1);
        assert!((out.sum - 7.0).abs() < f64::EPSILON);
        assert!((out.mean - 7.0).abs() < f64::EPSILON);
        assert!((out.min - 7.0).abs() < f64::EPSILON);
        assert!((out.max - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_multiple_elements() {
        let out = analyze(AnalysisInput {
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        })
        .unwrap();
        assert_eq!(out.count, 5);
        assert!((out.sum - 15.0).abs() < f64::EPSILON);
        assert!((out.mean - 3.0).abs() < f64::EPSILON);
        assert!((out.min - 1.0).abs() < f64::EPSILON);
        assert!((out.max - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_empty_returns_error() {
        let err = analyze(AnalysisInput { data: vec![] }).unwrap_err();
        assert!(matches!(err, CoreError::EmptyData));
    }

    #[test]
    fn analyze_negative_values() {
        let out = analyze(AnalysisInput {
            data: vec![-3.0, -1.0, 0.0, 1.0, 3.0],
        })
        .unwrap();
        assert_eq!(out.count, 5);
        assert!((out.sum - 0.0).abs() < f64::EPSILON);
        assert!((out.mean - 0.0).abs() < f64::EPSILON);
        assert!((out.min - (-3.0)).abs() < f64::EPSILON);
        assert!((out.max - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_identical_values() {
        let out = analyze(AnalysisInput {
            data: vec![4.0, 4.0, 4.0],
        })
        .unwrap();
        assert!((out.min - 4.0).abs() < f64::EPSILON);
        assert!((out.max - 4.0).abs() < f64::EPSILON);
        assert!((out.mean - 4.0).abs() < f64::EPSILON);
    }

    // --- process_batch ---

    #[test]
    fn process_batch_empty() {
        let out = process_batch(vec![]);
        assert_eq!(out.processed, 0);
        assert!(out.results.is_empty());
    }

    #[test]
    fn process_batch_doubles_values() {
        let items = vec![
            BatchItem {
                id: "a".into(),
                value: 1.0,
            },
            BatchItem {
                id: "b".into(),
                value: 2.5,
            },
        ];
        let out = process_batch(items);
        assert_eq!(out.processed, 2);
        assert!((out.results[0].output - 2.0).abs() < f64::EPSILON);
        assert!((out.results[1].output - 5.0).abs() < f64::EPSILON);
        assert_eq!(out.results[0].status, "ok");
        assert_eq!(out.results[1].status, "ok");
    }

    #[test]
    fn process_batch_preserves_ids() {
        let items = vec![BatchItem {
            id: "unique-42".into(),
            value: 10.0,
        }];
        let out = process_batch(items);
        assert_eq!(out.results[0].id, "unique-42");
    }

    #[test]
    fn process_batch_nan_marked_invalid() {
        let items = vec![BatchItem {
            id: "nan".into(),
            value: f64::NAN,
        }];
        let out = process_batch(items);
        assert!(out.results[0].status.starts_with("invalid"));
        assert!((out.results[0].output - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn process_batch_infinity_marked_invalid() {
        let items = vec![BatchItem {
            id: "inf".into(),
            value: f64::INFINITY,
        }];
        let out = process_batch(items);
        assert!(out.results[0].status.starts_with("invalid"));
    }

    // --- serialisation round-trips ---

    #[test]
    fn analysis_input_round_trip() {
        let input = AnalysisInput {
            data: vec![1.0, 2.0],
        };
        let json = serde_json::to_string(&input).unwrap();
        let decoded: AnalysisInput = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.data, input.data);
    }

    #[test]
    fn batch_output_round_trip() {
        let items = vec![BatchItem {
            id: "z".into(),
            value: 3.0,
        }];
        let out = process_batch(items);
        let json = serde_json::to_string(&out).unwrap();
        let decoded: BatchOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.processed, out.processed);
        assert_eq!(decoded.results[0].id, out.results[0].id);
    }
}
