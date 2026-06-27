# justfile
# Maps desired DX commands to the corresponding cargo or cargo-cicd targets

# List all available just commands
list:
	@just --list

# Lint and type-check without building
check:
	cargo check --all-targets --all-features

# Run all test suites
test:
	cargo test --workspace

# Run clippy on all targets
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Launch the terminal UI dashboard
dx:
	cargo run -q --bin cargo-cicd -- ui dashboard

# Run workspace doctor diagnostics
doctor:
	cargo run -q --bin cargo-cicd -- workspace doctor

# Run evidence gate tests (requires wpm binary)
gate:
	cargo test --test wasm4pm_evidence_gate --features wasm4pm -- --nocapture

# Audit the emitted OCEL evidence
ocel-replay:
	cargo run -q --bin cargo-cicd -- evidence audit

# Verify the cryptographic receipts
receipt-verify:
	wpm receipt doctor --format json --strict receipts/*.json

# Install git hooks
hooks-install:
	cargo run -q --bin cargo-cicd -- hooks install
