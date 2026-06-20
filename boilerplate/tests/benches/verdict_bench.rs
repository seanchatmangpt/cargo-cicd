//! Criterion benchmarks for `Verdict` parsing and display.
//!
//! These benchmarks measure the hot path for verdict string operations.
//! Run with:
//!   cargo bench --bench verdict_bench
//!
//! HTML reports are written to `target/criterion/` when the `html_reports`
//! feature is enabled (see Cargo-test-additions.toml).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use project_core::Verdict;

// ---------------------------------------------------------------------------
// Parsing benchmarks
// ---------------------------------------------------------------------------

fn bench_verdict_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_from_str");

    for input in &["PASS", "WARN", "FAIL", "BLOCKED"] {
        group.bench_with_input(
            BenchmarkId::from_parameter(input),
            input,
            |b, &s| {
                b.iter(|| {
                    // black_box prevents the compiler from optimising away the
                    // parse entirely.
                    black_box(s).parse::<Verdict>().unwrap()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark case-insensitive parsing (lowercase variants).
fn bench_verdict_parse_lowercase(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_from_str_lowercase");

    for input in &["pass", "warn", "fail", "blocked"] {
        group.bench_with_input(
            BenchmarkId::from_parameter(input),
            input,
            |b, &s| {
                b.iter(|| black_box(s).parse::<Verdict>().unwrap())
            },
        );
    }

    group.finish();
}

/// Benchmark rejection of invalid strings (Err path).
fn bench_verdict_parse_invalid(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_from_str_invalid");

    for input in &["UNKNOWN", "OK", "", "PASS_EXTRA", "1234"] {
        group.bench_with_input(
            BenchmarkId::from_parameter(if input.is_empty() { "<empty>" } else { input }),
            input,
            |b, &s| {
                b.iter(|| {
                    let _ = black_box(s).parse::<Verdict>();
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Display benchmarks
// ---------------------------------------------------------------------------

fn bench_verdict_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_display");

    let variants = [
        ("Pass",    Verdict::Pass),
        ("Warn",    Verdict::Warn),
        ("Fail",    Verdict::Fail),
        ("Blocked", Verdict::Blocked),
    ];

    for (name, variant) in &variants {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            variant,
            |b, v| {
                b.iter(|| format!("{}", black_box(v)))
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Roundtrip benchmark (parse → display → parse)
// ---------------------------------------------------------------------------

fn bench_verdict_roundtrip(c: &mut Criterion) {
    c.bench_function("verdict_roundtrip_pass", |b| {
        b.iter(|| {
            let v: Verdict = black_box("PASS").parse().unwrap();
            let s = format!("{}", black_box(&v));
            let _: Verdict = black_box(s.as_str()).parse().unwrap();
        })
    });
}

// ---------------------------------------------------------------------------
// label() benchmark
// ---------------------------------------------------------------------------

fn bench_verdict_label(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_label");

    let variants = [
        ("Pass",    Verdict::Pass),
        ("Warn",    Verdict::Warn),
        ("Fail",    Verdict::Fail),
        ("Blocked", Verdict::Blocked),
    ];

    for (name, variant) in &variants {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            variant,
            |b, v| {
                b.iter(|| black_box(v).label())
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// is_ok() benchmark
// ---------------------------------------------------------------------------

fn bench_verdict_is_ok(c: &mut Criterion) {
    let mut group = c.benchmark_group("verdict_is_ok");

    let variants = [
        ("Pass",    Verdict::Pass),
        ("Warn",    Verdict::Warn),
        ("Fail",    Verdict::Fail),
        ("Blocked", Verdict::Blocked),
    ];

    for (name, variant) in &variants {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            variant,
            |b, v| {
                b.iter(|| black_box(v).is_ok())
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_verdict_parse,
    bench_verdict_parse_lowercase,
    bench_verdict_parse_invalid,
    bench_verdict_display,
    bench_verdict_roundtrip,
    bench_verdict_label,
    bench_verdict_is_ok,
);
criterion_main!(benches);
