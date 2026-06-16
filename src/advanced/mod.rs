//! Advanced capabilities for the cargo-cicd process-data engine.
//!
//! Each submodule wraps a best-of-breed / bleeding-edge Rust crate and exposes
//! a small, engine-shaped API. The whole tree is gated behind the `advanced`
//! cargo feature so the default build and public CLI surface are unaffected.
//!
//! | module          | crate(s)                       | capability                                   |
//! |-----------------|--------------------------------|----------------------------------------------|
//! | `parallel_scan` | `ignore` + `rayon`             | parallel, gitignore-aware workspace scanning |
//! | `fingerprint`   | `blake3`                       | content-addressed Merkle fingerprinting      |
//! | `observability` | `tracing` + `tracing-subscriber` | structured span instrumentation            |
//! | `diagnostics`   | `miette` + `thiserror`         | rich rendered diagnostics                    |
//! | `cache`         | `moka`                         | concurrent, TTL-aware engine cache           |
//! | `snapshot`      | `bitcode`                      | compact binary engine-state snapshots        |
//! | `dep_graph`     | `petgraph`                     | workspace dependency graph & build order     |
//! | `timeline`      | `jiff`                         | high-precision, zoned process timestamps     |
//! | `histogram`     | `hdrhistogram`                 | latency percentiles for pipeline stages      |
//! | `pattern`       | `aho-corasick`                 | multi-pattern governance/path scanning       |

pub mod cache;
pub mod dep_graph;
pub mod diagnostics;
pub mod fingerprint;
pub mod histogram;
pub mod observability;
pub mod parallel_scan;
pub mod pattern;
pub mod snapshot;
pub mod timeline;
