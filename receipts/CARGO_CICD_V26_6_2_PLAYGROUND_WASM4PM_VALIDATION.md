# wasm4pm Validation Receipt — cargo-cicd v26.6.2

**date:** 2026-06-03
**wpm path:** /Users/sac/wasm4pm/target/release/wpm

## wpm doctor Output

```
wpm found: /Users/sac/wasm4pm/target/release/wpm

Running wpm doctor...
  [PASS] rustc: rustc 1.95.0 (59807616e 2026-04-14)
  [PASS] wasm-pack: wasm-pack 0.14.0
  [PASS] Cargo.toml found
  [PASS] src/ directory found
  [WARN] .wasm4pm directory not found

Summary:
All checks passed! Your environment is healthy.
  [PASS] wpm doctor returned 0

VERDICT=PASS
```

## Refusal Gate Output

```
error: Failed to load event log into WASM state
  [PASS] wpm refused: empty file

error: Failed to read event log: binary-garbage.jsonl
Caused by: stream did not contain valid UTF-8
  [PASS] wpm refused: binary garbage

error: Failed to load event log into WASM state
  [PASS] wpm refused: truncated json

error: Failed to load event log into WASM state
  [PASS] wpm refused: missing required fields

PASS=4 FAIL=0 BLOCKED=0
```

## Notes

- wpm is not on system PATH; validation scripts require PATH override
- `.wasm4pm` directory absence produces WARN but not failure
- All 4 mutation cases correctly refused by wpm audit
