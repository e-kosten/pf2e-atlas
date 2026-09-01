#!/bin/sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd -P)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-git-hooks-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

fake_bin="$tmp/bin"
log="$tmp/commands.log"
mkdir -p "$fake_bin"

cat > "$fake_bin/cargo" <<'EOF_CARGO'
#!/bin/sh
printf 'cargo %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'cargo detail output for %s\n' "$*"
exit 0
EOF_CARGO
chmod +x "$fake_bin/cargo"

cat > "$fake_bin/npm" <<'EOF_NPM'
#!/bin/sh
printf 'npm %s\n' "$*" >> "$ATLAS_TEST_COMMAND_LOG"
printf 'npm detail output for %s\n' "$*"
exit 0
EOF_NPM
chmod +x "$fake_bin/npm"

: >"$log"
fast_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/validation/fast.sh" 2>&1)
if printf '%s\n' "$fast_output" | grep -Eq '(cargo|npm) detail output'; then
  echo "quiet fast validation surfaced successful command detail output" >&2
  exit 1
fi
grep -q 'cargo fmt --check' "$log" || {
  echo "fast validation did not run cargo fmt" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro' "$log" || {
  echo "fast validation did not run one workspace clippy pass" >&2
  exit 1
}
if grep -Eq 'cargo (test|build)' "$log"; then
  echo "fast validation unexpectedly ran workspace tests or build" >&2
  exit 1
fi

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
grep -q 'cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro' "$log" || {
  echo "fast changed-path verification did not run workspace clippy validation" >&2
  exit 1
}
if [ "$(grep -c '^cargo clippy ' "$log")" -ne 1 ]; then
  echo "fast changed-path verification did not run exactly one workspace clippy pass" >&2
  exit 1
fi
grep -q 'npm --prefix web/atlas-ui run format:check' "$log" || {
  echo "fast changed-path verification did not run web format check" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run lint' "$log" || {
  echo "fast changed-path verification did not run web lint" >&2
  exit 1
}
grep -q 'npm --prefix web/atlas-ui run typecheck' "$log" || {
  echo "fast changed-path verification did not run web typecheck" >&2
  exit 1
}
if grep -q 'cargo test --workspace' "$log" || grep -q 'cargo build --workspace' "$log"; then
  echo "fast changed-path verification unexpectedly ran full Rust integration checks" >&2
  exit 1
fi
if grep -q 'npm --prefix web/atlas-ui run verify' "$log"; then
  echo "fast changed-path verification unexpectedly ran full web verify" >&2
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

printf 'git-hook common smoke tests passed\n'
