#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/verify.sh [--verbose]

Run the full Rust fmt, clippy, nextest, doctest, and build gate.

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

. "$REPO_ROOT/scripts/validation/run-check.sh"

if ! cargo nextest --version >/dev/null 2>&1; then
  echo "The Rust gate requires cargo-nextest. See docs/development/validation.md#local-rust-gate for installation." >&2
  exit 1
fi
cargo nextest show-config version >/dev/null

run_check "cargo fmt" cargo fmt --check
run_check "broad clippy" cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
run_check "strict runtime clippy" cargo clippy --workspace --lib --bins -- -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unimplemented \
  -D clippy::todo \
  -D clippy::unreachable
run_check "workspace tests" cargo nextest run --workspace
run_check "workspace doctests" cargo test --workspace --doc
run_check "workspace build" cargo build --workspace
