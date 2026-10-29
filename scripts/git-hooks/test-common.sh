#!/bin/sh

set -eu

# Fixture timings must not appear as real validation measurements in CI.
unset GITHUB_STEP_SUMMARY

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
repo_root=$(CDPATH='' cd -- "$script_dir/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-git-hooks-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
ATLAS_VALIDATION_OUTPUT_ROOT="$tmp/evidence"
export ATLAS_VALIDATION_OUTPUT_ROOT

fake_bin="$tmp/bin"
log="$tmp/commands.log"
mkdir -p "$fake_bin"

cat > "$fake_bin/cargo" <<'EOF_CARGO'
#!/bin/sh
printf 'cargo %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'cargo detail output for %s\n' "$*"
[ "${ATLAS_TEST_FAIL_CARGO:-0}" -eq 0 ] || exit 19
exit 0
EOF_CARGO
chmod +x "$fake_bin/cargo"

cat > "$fake_bin/npm" <<'EOF_NPM'
#!/bin/sh
printf 'npm %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'npm detail output for %s\n' "$*"
[ "${ATLAS_TEST_FAIL_NPM:-0}" -eq 0 ] || exit 23
exit 0
EOF_NPM
chmod +x "$fake_bin/npm"

# Exercise the actual hook and selector against real Git history. Every compiler
# and frontend command still resolves to the logging stubs above.
fixture="$tmp/repo"
mkdir -p "$fixture/.githooks" "$fixture/scripts/git-hooks" "$fixture/crates/example/src" "$fixture/web/atlas-ui/src"
sh "$repo_root/scripts/git-hooks/test-pre-push.sh"
cp "$repo_root/.githooks/pre-commit" "$repo_root/.githooks/pre-merge-commit" "$fixture/.githooks/"
cp "$repo_root/scripts/git-hooks/common.sh" "$fixture/scripts/git-hooks/common.sh"
cp "$repo_root/scripts/verify-changed.sh" "$repo_root/scripts/verify.sh" "$fixture/scripts/"
mkdir -p "$fixture/scripts/validation" "$fixture/scripts/release"
cp "$repo_root/scripts/validation/run-check.mjs" "$repo_root/scripts/validation/run-process.mjs" "$repo_root/scripts/validation/verify.mjs" "$repo_root/scripts/validation/run-check.sh" "$repo_root/scripts/validation/launch.sh" "$fixture/scripts/validation/"
for skill_check in skill.ts skill.test.ts; do
  printf 'import { appendFileSync } from "node:fs"; appendFileSync(process.env.ATLAS_TEST_COMMAND_LOG, "skill check\\n");\n' >"$fixture/scripts/validation/$skill_check"
done
for stub in validation/research.sh release/validate-release-tooling.sh release/test-release-tools.sh release/test-prepare-release.sh; do
  # shellcheck disable=SC2016 # The fixture expands its own environment.
  printf '#!/bin/sh\nprintf "extra check\\n" >> "$ATLAS_TEST_COMMAND_LOG"\n' >"$fixture/scripts/$stub"
done
git init --quiet "$fixture"
git -C "$fixture" config user.name 'Atlas hook fixture'
git -C "$fixture" config user.email 'atlas-hook-fixture@example.invalid'
git -C "$fixture" config commit.gpgsign false
git -C "$fixture" config core.hooksPath /dev/null
printf 'reference documentation\n' >"$fixture/README.md"
printf 'pub fn example() {}\n' >"$fixture/crates/example/src/lib.rs"
printf 'export const value = 0;\n' >"$fixture/web/atlas-ui/src/example.ts"
git -C "$fixture" add .
git -C "$fixture" commit --quiet -m 'test: initial fixture'

: >"$log"
quiet_output=$(cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$fixture/scripts/verify-changed.sh" --all 2>&1)
if printf '%s\n' "$quiet_output" | grep -Eq '(cargo|npm) detail output'; then
  echo "quiet changed-path verification surfaced successful command detail output" >&2
  exit 1
fi

grep -q 'cargo fmt --check' "$log" || {
  echo "fast changed-path verification did not run cargo fmt" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run format:check' "$log" || {
  echo "fast changed-path verification did not run web format check" >&2
  exit 1
}
if grep -Eq '^cargo (clippy|test|build)|^npm .* run (lint|typecheck|verify)$' "$log"; then
  echo "commit checks unexpectedly ran compilation, lint or tests" >&2
  exit 1
fi
printf 'crates/example/src/lib.rs\nweb/atlas-ui/src/example.ts\n' >"$tmp/application-paths"
: >"$log"
verbose_output=$(cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$fixture/scripts/verify-changed.sh" --paths-file "$tmp/application-paths" --full --verbose 2>&1)
if ! printf '%s\n' "$verbose_output" | grep -Eq '(cargo|npm) detail output'; then
  echo "verbose full changed-path verification did not surface command detail output" >&2
  exit 1
fi

grep -q 'cargo test --workspace' "$log" || {
  echo "full changed-path verification did not run cargo tests" >&2
  exit 1
}
grep -q 'cargo build --workspace' "$log" || {
  echo "full changed-path verification did not run cargo build" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run verify' "$log" || {
  echo "full changed-path verification did not run web verify" >&2
  exit 1
}

case_count=0
check_output="$tmp/check-output"

# The same classifier applies to staged commits and range consumers.
printf 'staged documentation\n' >"$fixture/README.md"
git -C "$fixture" add README.md
for staged_hook in pre-commit pre-merge-commit; do
  : >"$log"
  (
    cd "$fixture/web/atlas-ui"
    ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh "$fixture/.githooks/$staged_hook"
  ) >"$check_output" 2>&1
  [ ! -s "$log" ] || { echo "$staged_hook did not recognize staged docs" >&2; exit 1; }
  case_count=$((case_count + 1))
  printf 'Passed hook fixture: staged docs skip %s\n' "$staged_hook"
done
printf 'bad whitespace  \n' >"$fixture/README.md"
git -C "$fixture" add README.md
for staged_hook in pre-commit pre-merge-commit; do
  if (cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh ".githooks/$staged_hook") >"$check_output" 2>&1; then
    echo "$staged_hook accepted staged whitespace errors" >&2
    exit 1
  fi
done
printf 'valid documentation\n' >"$fixture/README.md"
git -C "$fixture" add README.md
mkdir -p "$fixture/scratch/plans"
printf 'executable fixture\n' >"$fixture/scratch/plans/probe.sh"
git -C "$fixture" add scratch/plans/probe.sh
: >"$log"
(
  cd "$fixture/web/atlas-ui"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh "$fixture/.githooks/pre-commit"
) >"$check_output" 2>&1
grep -q '^cargo fmt --check$' "$log" || { echo 'staged unknown skipped Rust gate' >&2; exit 1; }
grep -q '^npm --prefix web/atlas-ui run format:check$' "$log" || { echo 'staged unknown skipped web gate' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: staged unknown uses both fast gates\n'

if (cd "$fixture"; ATLAS_TEST_FAIL_CARGO=1 ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh .githooks/pre-commit) >"$check_output" 2>&1; then
  echo 'commit accepted a failed Rust formatting check' >&2
  exit 1
fi
if (cd "$fixture"; ATLAS_TEST_FAIL_NPM=1 ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh .githooks/pre-commit) >"$check_output" 2>&1; then
  echo 'commit accepted a failed frontend formatting check' >&2
  exit 1
fi

for entry in 'README.md:0:0' 'crates/example/src/lib.rs:1:0' 'web/atlas-ui/src/example.ts:0:1' 'crates/atlas-app-model/src/filter.rs:1:1' 'unknown.config:1:1'; do
  path=${entry%%:*}
  expected=${entry#*:}
  printf '%s\n' "$path" >"$fixture/paths-input"
  : >"$log"
  (cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input) >"$check_output" 2>&1
  rust_count=$(grep -c '^cargo fmt --check$' "$log" || true)
  web_count=$(grep -c '^npm --prefix web/atlas-ui run format:check$' "$log" || true)
  [ "$rust_count:$web_count" = "$expected" ] || { echo "wrong formatting surface for $path" >&2; exit 1; }
done

# Skill prose runs only its focused check; mixed changes retain both surfaces.
printf 'README.md\nskills/pf2e-atlas-cli/SKILL.md\n' >"$fixture/paths-input"
: >"$log"
(cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input --full) >"$check_output" 2>&1
[ "$(grep -c '^skill check$' "$log")" -eq 2 ]
if grep -Eq '^(cargo|npm) ' "$log"; then echo 'skill-only change ran application checks' >&2; exit 1; fi
cp "$fixture/scripts/validation/skill.ts" "$tmp/valid-skill-check"
printf 'throw new Error("invalid skill");\n' >"$fixture/scripts/validation/skill.ts"
if (cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input --full) >"$check_output" 2>&1; then
  echo 'skill validation failure ignored' >&2; exit 1
fi
cp "$tmp/valid-skill-check" "$fixture/scripts/validation/skill.ts"
printf 'crates/example/src/lib.rs\n' >>"$fixture/paths-input"
(cd "$fixture"; scripts/verify-changed.sh --paths-file paths-input --list) >"$check_output"
grep -q '^rust=true$' "$check_output"
grep -q '^skill=true$' "$check_output"

# Literal path input must not discard a final line without a newline.
printf 'unknown-tooling-file' >"$fixture/paths-input"
: >"$log"
(
  cd "$fixture"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input --full
) >"$check_output" 2>&1
grep -q '^cargo test --workspace$' "$log" || { echo 'unterminated path skipped Rust gate' >&2; exit 1; }
grep -q '^npm --prefix web/atlas-ui run verify$' "$log" || { echo 'unterminated path skipped web gate' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: literal final path without newline is classified\n'

: >"$log"
if (
  cd "$fixture"
  ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --range does-not-exist --full
) >"$check_output" 2>&1; then
  echo 'invalid diff range incorrectly succeeded' >&2
  exit 1
fi
[ ! -s "$log" ] || { echo 'invalid range executed gates' >&2; exit 1; }
case_count=$((case_count + 1))
printf 'Passed hook fixture: invalid diff range fails before gates\n'

printf 'git-hook common smoke tests and %s fixture cases passed\n' "$case_count"

# Explicit surfaces, dependency fan-out, and conservative unknown inputs.
for entry in \
  'README.md:00000000' \
  'web/atlas-ui/src/app.tsx:01000010' \
  'web/atlas-ui/e2e/devices/conditions.spec.ts:00000010' \
  'web/atlas-ui/playwright.devices.config.ts:00000010' \
  'scripts/browser-fixture.mjs:00000010' \
  'crates/atlas-dev/src/main.rs:00000001' \
  'scripts/development/build-preview.mjs:00000001' \
  'scripts/development/build-preview.test.mjs:00000001' \
  'crates/atlas-cli/src/main.rs:10000001' \
  'crates/atlas-cli-support/src/lib.rs:10000001' \
  'crates/atlas-runtime/src/lib.rs:10000010' \
  'docs/research/evaluations/frozen/README.md:00001000' \
  'docs/research/evaluations/frozen/judgments.jsonl:00001000' \
  'docs/research/evaluations/frozen/verify.ts:00001000' \
  'dev-tools/source-contracts/tests/fixture.md:00100000' \
  'dev-tools/release/src/check.ts:00010000' \
  'crates/atlas-foundry-model/src/lib.rs:10100010' \
  'crates/atlas-app-model/bindings/Search.ts:11000010' \
  'Cargo.lock:11110011' \
  'crates/atlas-cli/Cargo.toml:11110011' \
  'THIRD-PARTY-NOTICES.md:00010000' \
  'skills/pf2e-atlas-cli/SKILL.md:00000100' \
  '.cargo/config.toml:11111111' \
  '.github/workflows/ci.yml:11111111' \
  'unknown.config:11111111'; do
  changed_path=${entry%:*}
  expected=${entry##*:}
  printf '%s\n' "$changed_path" >"$fixture/paths-input"
  : >"$log"
  (cd "$fixture"; scripts/verify-changed.sh --paths-file paths-input --list) >"$check_output"
  actual=$(sed -e 's/.*=true/1/' -e 's/.*=false/0/' "$check_output" | tr -d '\n')
  [ "$actual" = "$expected" ] || { echo "wrong selection for $changed_path: $actual" >&2; exit 1; }
done
# Full local dispatch follows ownership: browser-only checks build/type/lint
# their app fixture inputs, while developer-only checks do not invoke npm.
for changed_path in web/atlas-ui/src/app.tsx web/atlas-ui/e2e/devices/conditions.spec.ts crates/atlas-dev/src/main.rs scripts/development/build-preview.mjs crates/example/src/lib.rs; do
  printf '%s\n' "$changed_path" >"$fixture/paths-input"
  : >"$log"
  (cd "$fixture"; ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" scripts/verify-changed.sh --paths-file paths-input --full) >"$check_output" 2>&1
  case "$changed_path" in
    web/atlas-ui/src/*)
      grep -q '^npm --prefix web/atlas-ui run verify$' "$log"
      if grep -q '^cargo ' "$log"; then echo 'frontend-only change ran Cargo' >&2; exit 1; fi
      ;;
    web/atlas-ui/e2e/*)
      grep -q '^npm --prefix web/atlas-ui run verify:bundle$' "$log"
      if grep -q '^cargo ' "$log"; then echo 'frontend-only change ran Cargo' >&2; exit 1; fi
      if grep -q '^npm --prefix web/atlas-ui run verify$' "$log"; then echo 'non-unit surface ran full frontend verify' >&2; exit 1; fi
      ;;
    crates/atlas-dev/* | scripts/development/*)
      grep -q '^cargo test -p atlas-dev -p atlas-cli-support --locked$' "$log"
      grep -q '^cargo build -p atlas-dev --locked$' "$log"
      if grep -Eq '^npm |^cargo .*--workspace' "$log"; then echo 'standalone developer change ran application checks' >&2; exit 1; fi
      ;;
    *)
      grep -q '^npm --prefix web/atlas-ui run build$' "$log"
      grep -q '^cargo test --workspace$' "$log"
      if grep -q '^npm --prefix web/atlas-ui run verify$' "$log"; then echo 'non-unit surface ran full frontend verify' >&2; exit 1; fi
      ;;
  esac
done
printf 'UI, browser, standalone developer and core Rust dispatch regressions passed.\n'

# Validate the real diff path, including deleted source of a rename and file modes.
git -C "$fixture" add .
git -C "$fixture" commit --quiet -m 'test: fixture baseline'
git -C "$fixture" mv crates/example/src/lib.rs docs-only.md
git -C "$fixture" commit --quiet -m 'test: rename source to prose'
(cd "$fixture"; scripts/verify-changed.sh --range HEAD~1...HEAD --list) >"$check_output"
grep -q '^rust=true$' "$check_output"
chmod +x "$fixture/README.md"
git -C "$fixture" add README.md
(cd "$fixture"; scripts/verify-changed.sh --staged --list) >"$check_output"
[ "$(grep -c '=true$' "$check_output")" -eq 8 ]

# The real research runner rejects missing verifiers and propagates failures.
cp "$repo_root/scripts/validation/research.sh" "$fixture/scripts/validation/research.sh"
mkdir -p "$fixture/docs/research/evaluations/frozen"
if (cd "$fixture"; sh scripts/validation/research.sh) >"$check_output" 2>&1; then
  echo 'research dataset without verifier accepted' >&2; exit 1
fi
printf 'throw new Error("invalid archived evidence");\n' >"$fixture/docs/research/evaluations/frozen/verify.ts"
if (cd "$fixture"; sh scripts/validation/research.sh) >"$check_output" 2>&1; then
  echo 'research verifier failure ignored' >&2; exit 1
fi
printf 'console.log("evidence verified");\n' >"$fixture/docs/research/evaluations/frozen/verify.ts"
(cd "$fixture"; sh scripts/validation/research.sh) >"$check_output" 2>&1
grep -q 'evidence verified' "$check_output"
printf 'Validation surface, rename, mode and research failure regressions passed.\n'

# Timed checks retain failures, replay diagnostics and report duration to CI.
summary="$tmp/timing-summary"
set +e
(
  GITHUB_STEP_SUMMARY="$summary"
  export GITHUB_STEP_SUMMARY
  sh "$repo_root/scripts/validation/run-check.sh" 'failure fixture' sh -c 'echo required-diagnostic; exit 23'
) >"$check_output" 2>&1
status=$?
set -e
[ "$status" -eq 23 ] || { echo 'timing wrapper changed failure status' >&2; exit 1; }
grep -q 'required-diagnostic' "$check_output"
grep -Eq 'failure fixture: [0-9]+s \(exit 23\)' "$summary"
printf 'Timing failure propagation and CI summary regression passed.\n'
