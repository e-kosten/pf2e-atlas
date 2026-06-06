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

quiet_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" sh -c ". '$script_dir/common.sh'; run_required_verification" 2>&1)
if printf '%s\n' "$quiet_output" | grep -q 'cargo detail output'; then
  echo "quiet verification surfaced successful cargo detail output" >&2
  exit 1
fi

verbose_output=$(ATLAS_TEST_COMMAND_LOG="$log" PATH="$fake_bin:$PATH" "$repo_root/scripts/verify.sh" --verbose 2>&1)
if ! printf '%s\n' "$verbose_output" | grep -q 'cargo detail output'; then
  echo "verbose verification did not surface cargo detail output" >&2
  exit 1
fi

grep -q 'cargo fmt --check' "$log" || {
  echo "git-hook verification did not run cargo fmt" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro' "$log" || {
  echo "git-hook verification did not run broad clippy validation" >&2
  exit 1
}
grep -q 'cargo clippy --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable' "$log" || {
  echo "git-hook verification did not run strict runtime clippy validation" >&2
  exit 1
}
grep -q 'cargo test --workspace' "$log" || {
  echo "git-hook verification did not run cargo tests" >&2
  exit 1
}
grep -q 'cargo build --workspace' "$log" || {
  echo "git-hook verification did not run cargo build" >&2
  exit 1
}

printf 'git-hook common smoke tests passed\n'
