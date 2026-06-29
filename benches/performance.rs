use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cargo_cicd::ocel::{detect_drift, page_hinkley_test, OcelLog};
use cargo_cicd::barrier::detect_barriers;
use tempfile::TempDir;

fn bench_drift_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("drift_detection");
    
    // Test page-hinkley change detection on a sequence of 1000 items
    group.bench_function("page_hinkley_flat", |b| {
        let sequence = vec![0.5f64; 1000];
        b.iter(|| {
            let _ = black_box(page_hinkley_test(black_box(&sequence), 10.0, 0.1));
        });
    });

    group.bench_function("detect_drift_no_shift", |b| {
        let w1 = vec![1.0; 100];
        let w2 = vec![1.0; 100];
        b.iter(|| {
            let _ = black_box(detect_drift(black_box(&w1), black_box(&w2)));
        });
    });

    group.finish();
}

fn bench_barrier_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("barrier_scanning");
    
    // Scan a temporary directory with nested files to bench directory traversal
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("src");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("main.rs"), "fn main() { assert!(true); }").unwrap();
    std::fs::write(sub.join("lib.rs"), "fn lib() {}").unwrap();
    
    group.bench_function("detect_barriers_empty", |b| {
        b.iter(|| {
            let _ = black_box(detect_barriers(black_box(dir.path())));
        });
    });

    group.finish();
}

fn bench_ocel_log_types(c: &mut Criterion) {
    c.bench_function("ocel_log_cargo_object_types", |b| {
        b.iter(|| {
            let _ = black_box(OcelLog::cargo_object_types());
        });
    });
}

criterion_group!(
    benches,
    bench_drift_detection,
    bench_barrier_scanning,
    bench_ocel_log_types
);
criterion_main!(benches);
