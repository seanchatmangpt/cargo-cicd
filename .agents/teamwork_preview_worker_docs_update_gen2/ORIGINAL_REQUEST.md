## 2026-06-28T21:45:04-07:00
You are teamwork_preview_worker. Your working directory is /Users/sac/cargo-cicd/.agents/teamwork_preview_worker_docs_update_gen2/. Your identity is teamwork_preview_worker_docs_update_gen2.

Your mission is to correct the documentation of cargo-cicd to address all findings from the Reviewers.

Target Files to Update/Modify:
1. `docs/star-toml-refactor/PRD.md` (ensure it exists and is clean).
2. `docs/star-toml-refactor/ARD.md` (unify standing variables to use $q_{standing} = q_{config} \wedge q_{verification}$ consistently).
3. `docs/star-toml-refactor/REFACTOR.md` (apply star-toml API fixes).
4. `README.md` (replace broken docs/dx/ links with valid links).
5. `docs/INDEX.md` (correct dead and mismatched links).

Detailed Corrections to apply:

### 1. REFACTOR.md Changes
- In **Step 1**, implement the full `CicdConfig` schema matching all fields in a real `cicd.toml` file (including `workspace`, `state`, `target`, `test.changed`, `trybuild.changed`, `git.phase`, `autonomic`, and `events`).
- The `Validate` implementation for `CicdConfig` must use the correct star-toml API:
  - Use `check_path_safe` for path verification. Signature: `v.check_path_safe(field: &str, value: &str, source_path: &std::path::Path, policy: star_toml::path::PathPolicy)`.
  - Use `star_toml::path::PathPolicy::Sandbox { root: std::path::PathBuf::from(".") }` instead of a unit variant.
  - Use `v.check_one_of("autonomic.mode", &self.autonomic.mode, &["suggest", "enforce"])` instead of `check_enum`.
- In **Step 3**, remove the non-existent `.strict_unknown_fields(true)` builder call. Document that `load_admitted()` is strict by default in `star-toml` v26.6.29.
- In **Step 5**, update the `verify_publish_gate` snippet to verify both the config digest AND that the receipt's verdict is explicitly equal to `"Accept"`.
- In **Step 6**, replace `assert_schema_admitted` and `assert_schema_refused` generic calls in the test snippet with standard `TrustedLoader` setup using `.layer_str(toml_str, "label")` and `load_admitted()`.

### 2. ARD.md Changes
- Section 1 (Standing Formula):
  ```
  q_standing = q_config ∧ q_verification
  ```
  Unify all sections to use $q_{standing} = q_{config} \wedge q_{verification}$ consistently, including Section 2.5 (change $q_{system}$ to $q_{standing}$).

### 3. README.md Changes
- Under `## Documentation` table, remove the three broken links:
  - `docs/dx/ONBOARDING.md`
  - `docs/dx/CHEATSHEET.md`
  - `docs/dx/ECOSYSTEM_MAP.md`
- Replace them with:
  - `docs/contributing/README.md` | Contributor Guide: onboarding, setup, and workflow
  - `docs/DX_GUIDE.md` | Developer Experience Guide: quick reference and aliases

### 4. docs/INDEX.md Changes
- Line 76: Remove `how-to/use-all-features.md` from the table since it does not exist.
- Line 114 and 248: Change link target `reference/commands.md` to uppercase `reference/COMMANDS.md` to match the actual file name.
- Line 119: Remove `reference/capabilities.md` from the table since it does not exist.
- Line 169: Change `explanation/combinatorial-maximalism-rationale.md` to `explanation/combinatorial-maximalism.md`.

Verify that all changes compile structurally, have correct formatting, and all markdown files compile without broken links. Report your completion back to me.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
