#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/validation/fast.sh [--base <git-ref>] [--verbose]

Run the focused, path-sensitive formatting and lint tier. When --base is
provided, also enforce artifact version bumps against the merge base.
EOF
}

base=
verbose=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --base)
      base="${2:-}"
      [ -n "$base" ] || { echo "--base requires a git ref" >&2; exit 2; }
      shift 2
      ;;
    -v | --verbose)
      verbose=--verbose
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown validate-fast option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

unset PF2E_SOURCE_ROOT PF2E_COMMIT CANDIDATE_COMMIT CANDIDATE_TREE \
  ATLAS_VALIDATION_SNAPSHOT_ROOT ATLAS_VALIDATION_FORCE_REPRODUCTION

repo_root="$(git rev-parse --show-toplevel)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-validate-fast.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
cd "$repo_root"

run_check() {
  label="$1"
  shift
  if [ -n "$verbose" ]; then
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
  printf '\nFailed %s\nCommand: %s\n\n' "$label" "$*" >&2
  cat "$output" >&2
  return 1
}

if [ -n "$base" ]; then
  merge_base="$(git merge-base "$base" HEAD)"
  scripts/validation/check-artifact-version-bump.sh --base "$base"
  scripts/verify-changed.sh --range "$merge_base...HEAD" ${verbose:+"$verbose"}
else
  run_check "cargo fmt" cargo fmt --check
  run_check "runtime clippy" cargo clippy --workspace --lib --bins -- \
    -D warnings -D clippy::dbg_macro -D clippy::unwrap_used -D clippy::expect_used \
    -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable
  run_check "test clippy" cargo clippy --workspace --tests --benches --examples -- \
    -D warnings -D clippy::dbg_macro
fi
