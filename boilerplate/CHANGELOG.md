# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

---

## [0.1.0] - YYYY-MM-DD

### Added

- Initial release of the project
- Core workspace functionality
- CLI entry point with noun-verb grammar
- `status show` — workspace health snapshot
- `target show` and `target prune` — target directory analysis and cleanup
- `test changed` — selective test execution for changed files
- `git status`, `git close`, `git phase` — git phase tracking
- `workspace doctor` — workspace-wide diagnostics
- `publish run` — artifact publishing gate
- `evidence doctor` and `evidence audit` — process evidence emission and adjudication
- `pipeline run` — sequential CI/CD activity execution
- Terminal UI design system (`src/ui/`) with zero external dependencies
- Feature flags: `process-data`, `autonomic`, `wasm4pm`, `advanced`
- Process evidence emission in XES + JSONL format
- wasm4pm oracle integration for verdict adjudication

---

<!-- Link references for diffs between versions -->
<!-- [Unreleased]: https://github.com/your-org/your-repo/compare/v0.1.0...HEAD -->
<!-- [0.1.0]: https://github.com/your-org/your-repo/releases/tag/v0.1.0 -->
