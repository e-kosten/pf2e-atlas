#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/verify.sh [--verbose]

Run the full Rust fmt, clippy, test, and build gate.

Options:
  -v, --verbose  stream detailed command output
  -h, --help     show this help
EOF
}

verbose=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    -v | --verbose)
      verbose=1
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown verify option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

REPO_ROOT="$(git rev-parse --show-toplevel)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-verify.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

cd "$REPO_ROOT"

run_check() {
  label="$1"
  shift

  if [ "$verbose" -eq 1 ]; then
    printf 'Running %s...\n' "$label" >&2
    "$@"
    return
  fi

  output="$tmp_dir/$label.log"
  printf 'Running %s...\n' "$label" >&2
  if "$@" >"$output" 2>&1; then
    printf 'Passed %s\n' "$label" >&2
    return
  fi

  printf '\nFailed %s\n' "$label" >&2
  printf 'Command: %s\n\n' "$*" >&2
  cat "$output" >&2
  return 1
}

run_check "cargo fmt" cargo fmt --check
run_check "broad clippy" cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
run_check "strict runtime clippy" cargo clippy --workspace --lib --bins -- -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable
run_check "workspace tests" cargo test --workspace
run_check "workspace build" cargo build --workspace
