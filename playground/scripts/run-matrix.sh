#!/usr/bin/env bash
set -euo pipefail

PASS=0; FAIL=0; BLOCKED=0

run_scenario() {
  local name=$1; shift
  local dir; dir=$(mktemp -d)
  # shellcheck disable=SC2064
  trap "rm -rf $dir" EXIT
  cd "$dir"
  mkdir src && printf 'fn main(){}' > src/main.rs
  printf '[package]\nname="smoke"\nversion="0.1.0"\nedition="2021"\n' > Cargo.toml
  git init -q && git add -A && git commit -q -m "init"
  if cargo-cicd "$@" 2>&1 | head -3; then
    PASS=$((PASS+1)); echo "  [PASS] $name"
  else
    FAIL=$((FAIL+1)); echo "  [FAIL] $name"
  fi
  cd - > /dev/null
}

run_scenario "status_show"      status show
run_scenario "target_show"      target show
run_scenario "workspace_doctor" workspace doctor
run_scenario "publish_run"      publish run
run_scenario "test_changed"     test changed
run_scenario "trybuild_changed" trybuild changed
run_scenario "git_close"        git close

echo ""
echo "PASS=$PASS FAIL=$FAIL BLOCKED=$BLOCKED"
[ $FAIL -eq 0 ]
