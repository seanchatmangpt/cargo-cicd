# Handoff Report — Documentation Review & Adversarial Analysis

## 1. Observation

During my review, I examined the proposed documentation updates in `docs/star-toml-refactor/PRD.md`, `docs/star-toml-refactor/ARD.md`, and `docs/star-toml-refactor/REFACTOR.md`, as well as `README.md` and the existing files in the `star-toml` library located at `/Users/sac/star-toml/`.

### Verbatim Quotes from reviewed docs (`docs/star-toml-refactor/`):
* **REFACTOR.md (Lines 53-55)**:
  ```rust
  // Path Sandboxing check
  v.check_path_policy("target.build_dir", &self.target.build_dir, star_toml::PathPolicy::Sandbox);
  ```
* **REFACTOR.md (Lines 105-110)**:
  ```rust
  let loader = TrustedLoader::new()
      .layer_file(&config_path)
      .env_prefix("CICD_")
      .strict_unknown_fields(true); // Fail-fast on unrecognized fields
  ```
* **REFACTOR.md (Lines 153-174)**:
  ```rust
  pub fn verify_publish_gate(
      config: &star_toml::AdmittedConfig<CicdConfig>,
      receipt_path: &str
  ) -> Result<(), anyhow::Error> {
      // 1. Read the latest execution receipt
      let receipt = read_receipt(receipt_path)?;

      // 2. Fetch the cryptographic witness digest from the admitted configuration
      let expected_config_digest = config.witness().hash();

      // 3. Verify they are bound
      if receipt.admitted_config_digest != expected_config_digest {
          return Err(anyhow::anyhow!(
              "Publish Refused: Admitted configuration hash ({}) does not match the execution receipt digest ({})!",
              expected_config_digest,
              receipt.admitted_config_digest
          ));
      }

      Ok(())
  }
  ```
* **REFACTOR.md (Lines 184-186, 203, 213)**:
  ```rust
  #[test]
  fn test_config_invariants() {
      use star_toml_config_test::{assert_schema_admitted, assert_schema_refused};
      ...
      assert_schema_admitted::<CicdConfig>(valid_toml);
      ...
      assert_schema_refused::<CicdConfig>(traversal_toml);
  }
  ```
* **ARD.md (Lines 58-61)**:
  ```text
  5. STANDING LAYER (Final Gate / AdmittedConfig Witness)
  - Calculates final standing bit (q_standing = q_config ∧ q_verdict)
  - Emits signed, BLAKE3 hash-bound execution witness
  - Gates final execution (e.g. publish proceeds only if q_standing = 1)
  ```
* **ARD.md (Lines 81-83)**:
  ```text
  Combines configuration admission standing ($q_{config} = 1$) and process verification standing ($q_{verification} = 1$) to compute the sovereign standing bit ($q_{system}$). Emits the BLAKE3 witness hash representing the absolute state proof.
  ```

### Verbatim Quotes from `star-toml` library:
* **star-toml/src/validation.rs (Lines 860-865)**:
  ```rust
  pub fn check_path_safe(
      &mut self,
      field: &str,
      value: &str,
      source_path: &Path,
      policy: crate::path::PathPolicy,
  ) {
  ```
* **star-toml/src/path.rs (Lines 10-18)**:
  ```rust
  pub enum PathPolicy {
      /// Path must resolve to within `root` (sandbox).
      Sandbox { root: PathBuf },
      /// Path must not escape the directory that contains the source file.
      RelativeOnly,
      /// Path must not begin with a forbidden system prefix.
      BlockForbidden,
  }
  ```
* **star-toml-config-test/src/lib.rs (Lines 53, 67)**:
  ```rust
  pub fn assert_schema_admitted(schema_json: &str) -> Result<(), TestFailure> {
  ...
  pub fn assert_schema_refused(schema_json: &str, expected_code: &str) -> Result<(), TestFailure> {
  ```

---

## 2. Logic Chain

1. **Path Policy Compilation Error**:
   - `REFACTOR.md` (Line 54) uses `star_toml::PathPolicy::Sandbox` as if it were a unit variant and calls `v.check_path_policy` with 3 arguments.
   - However, `star-toml/src/path.rs` (Lines 10-18) defines `PathPolicy::Sandbox` as a struct/tuple variant taking a `root` field (i.e. `Sandbox { root: PathBuf }`), meaning `PathPolicy::Sandbox` alone cannot compile.
   - In addition, `star-toml/src/validation.rs` defines the method as `check_path_safe`, taking 4 arguments (`field`, `value`, `source_path`, `policy`). There is no method named `check_path_policy` on `Validator`.
   - Thus, the code proposed in Step 1 of `REFACTOR.md` will fail to compile.

2. **Loader Configuration Compilation Error**:
   - `REFACTOR.md` (Lines 105-110) attempts to configure `TrustedLoader` via `.strict_unknown_fields(true)`.
   - `star-toml`'s `TrustedLoader` has no method named `strict_unknown_fields` (as shown by a search of `loader.rs`). Instead, `load_admitted()` is strict by default in current versions.
   - Thus, the code proposed in Step 3 of `REFACTOR.md` will fail to compile.

3. **Structural Validation Admission Failure**:
   - `REFACTOR.md` Step 1 defines `CicdConfig` with only `target`, `test`, and `autonomic` fields, and Step 3 uses strict loading mode.
   - However, the real workspace configuration file `cicd.toml` contains `[workspace]`, `[state]`, `[trybuild]`, `[git]`, and `[[events]]` sections.
   - Because strict loading is used, the loader will reject any configuration with unknown fields. Thus, loading the existing `cicd.toml` will immediately fail config admission ($q_{config} = 0$).

4. **Testing Helper Compilation and Logic Errors**:
   - `REFACTOR.md` Step 6 attempts to use `assert_schema_admitted` and `assert_schema_refused` as generic functions taking TOML strings (e.g. `assert_schema_admitted::<CicdConfig>(valid_toml)`).
   - In `star-toml-config-test`, these helper functions are not generic, they take a JSON Schema string (not TOML), and `assert_schema_refused` requires a second argument (`expected_code`).
   - Thus, the tests proposed in `REFACTOR.md` Step 6 will fail to compile and contain logic errors.

5. **Standing Logic and Variable Inconsistency**:
   - `ARD.md` Section 1 defines the final standing bit as `q_standing = q_config ∧ q_verdict` (where `q_verdict` represents the wasm4pm verdict).
   - `ARD.md` Section 2.5 states that the sovereign standing bit is `q_system`, computed from `q_config` and `q_verification`.
   - This introduces confusing inconsistencies in variables ($q_{standing}$ vs $q_{system}$, and $q_{verdict}$ vs $q_{verification}$).

6. **Oracle Verdict Verification Gap**:
   - `ARD.md` Rule 7 states that publish requires `q_config = 1` and that the wasm4pm oracle returns `Accept`.
   - However, the proposed `verify_publish_gate` function in `REFACTOR.md` Step 5 only compares configuration hashes and fails to check the oracle's verdict (`Accept` vs `Refused`).

---

## 3. Caveats

* Due to terminal constraints and timeouts in the execution environment, we were unable to run `cargo test` in this specific session. However, the compile errors and structural mismatches were verified via rigorous static analysis of the source code in `/Users/sac/star-toml/` and `/Users/sac/cargo-cicd/`.
* We assume that `star-toml` is an external dependency that will be added to the project during the refactoring process (as outlined in `REFACTOR.md`), since it is not currently present in `Cargo.toml`.

---

## 4. Conclusion

### Review Summary

**Verdict**: VETOED (equivalent to REQUEST_CHANGES)

The documentation updates in `docs/star-toml-refactor/` are structurally aligned with the requirements but contain several critical technical inaccuracies and compilation errors that would prevent successful implementation of the proposed refactoring.

### Findings

#### [Critical] Finding 1: Path Policy Compilation Error
- **What**: Incorrect validator method and incorrect variant type usage.
- **Where**: `docs/star-toml-refactor/REFACTOR.md` (Line 54).
- **Why**: `Validator` does not have `check_path_policy` (it has `check_path_safe` which takes 4 arguments instead of 3), and `PathPolicy::Sandbox` is a struct variant rather than a unit variant.
- **Suggestion**: Change to:
  ```rust
  let source_path = std::path::Path::new("cicd.toml");
  let sandbox_root = std::path::PathBuf::from(".");
  v.check_path_safe("target.build_dir", &self.target.build_dir, &source_path, star_toml::PathPolicy::Sandbox { root: sandbox_root });
  ```

#### [Critical] Finding 2: `TrustedLoader` Builder Error
- **What**: Call to non-existent builder method `.strict_unknown_fields(true)`.
- **Where**: `docs/star-toml-refactor/REFACTOR.md` (Line 109).
- **Why**: `TrustedLoader` has no `strict_unknown_fields` method. The `load_admitted` method is strict by default.
- **Suggestion**: Remove `.strict_unknown_fields(true)` from the builder chain.

#### [Critical] Finding 3: Incomplete Config Fields & Unknown Fields Rejection
- **What**: `CicdConfig` structure is missing `workspace`, `state`, `trybuild`, `git`, and `events` sections.
- **Where**: `docs/star-toml-refactor/REFACTOR.md` (Lines 20-45).
- **Why**: The existing `cicd.toml` contains these sections. If strict mode is used, loading this configuration will fail due to unrecognized fields.
- **Suggestion**: Declare all existing sections in `CicdConfig` or use exploratory loading during migration.

#### [Critical] Finding 4: Inconsistent Standing Layer Variables
- **What**: Conflicting definitions of the standing variables.
- **Where**: `docs/star-toml-refactor/ARD.md` (Lines 58-61 and 81-83).
- **Why**: Section 1 uses $q_{standing} = q_{config} \wedge q_{verdict}$, while Section 2.5 uses $q_{system}$ computed from $q_{config}$ and $q_{verification}$.
- **Suggestion**: Standardize the equations and terms across both sections (e.g. use $q_{standing} = q_{config} \wedge q_{verdict}$).

#### [Major] Finding 5: Missing Oracle Verdict Verification
- **What**: `verify_publish_gate` does not verify the oracle's verdict.
- **Where**: `docs/star-toml-refactor/REFACTOR.md` (Step 5).
- **Why**: The code only checks the config hash and ignores the verification of the oracle verdict (violating the PRD/ARD rules).
- **Suggestion**: Add a check to verify that the receipt verdict is explicitly `Accept`.

#### [Major] Finding 6: Mismatched Testing Helper Functions
- **What**: Incorrect generic parameters and argument counts in test assertions.
- **Where**: `docs/star-toml-refactor/REFACTOR.md` (Step 6).
- **Why**: `assert_schema_admitted` and `assert_schema_refused` are not generic, take JSON Schema (not TOML), and `assert_schema_refused` requires a second argument.
- **Suggestion**: Replace these assertions with direct calls to `Loader` or write customized test helpers that load the TOML string and assert on the `ValidationErrors` result.

---

## 5. Verification Method

To verify these findings, inspect the following files:
1. **Validation Methods**: Open `/Users/sac/star-toml/src/validation.rs` and inspect the signature of `check_path_safe` (line 860) to confirm it does not match the call in `REFACTOR.md`.
2. **PathPolicy Definition**: Open `/Users/sac/star-toml/src/path.rs` (line 11) to confirm that `Sandbox` has the signature `Sandbox { root: PathBuf }`.
3. **Loader Methods**: Open `/Users/sac/star-toml/src/loader.rs` and search for `strict_unknown_fields` to confirm it is not defined.
4. **Testing Helpers**: Open `/Users/sac/star-toml/star-toml-config-test/src/lib.rs` (lines 53, 67) to confirm the signatures of `assert_schema_admitted` and `assert_schema_refused`.
