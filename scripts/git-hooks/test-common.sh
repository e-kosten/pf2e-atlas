#!/bin/sh

set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
repo_root=$(CDPATH='' cd -- "$script_dir/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-git-hooks-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

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

: >"$log"
quiet_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/verify-changed.sh" --all 2>&1)
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
: >"$log"
verbose_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/verify-changed.sh" --all --full --verbose 2>&1)
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

# Exercise the actual hook and selector against real Git history. Every compiler
# and frontend command still resolves to the logging stubs above.
fixture="$tmp/repo"
mkdir -p "$fixture/.githooks" "$fixture/scripts/git-hooks" "$fixture/crates/example/src" "$fixture/web/atlas-ui/src"
[ ! -e "$repo_root/.githooks/pre-push" ] || { echo "automatic pre-push gate was restored" >&2; exit 1; }
cp "$repo_root/.githooks/pre-commit" "$repo_root/.githooks/pre-merge-commit" "$fixture/.githooks/"
cp "$repo_root/scripts/git-hooks/common.sh" "$fixture/scripts/git-hooks/common.sh"
cp "$repo_root/scripts/verify-changed.sh" "$repo_root/scripts/verify.sh" "$fixture/scripts/"
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
