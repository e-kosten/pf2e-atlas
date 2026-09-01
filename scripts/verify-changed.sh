#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/verify-changed.sh (--staged | --range <git-range> | --all) [--full] [--verbose]

Run path-sensitive validation.

Modes:
  --staged       validate staged paths for pre-commit
  --range RANGE  validate paths changed in a git diff range
  --all          validate Rust and web UI gates regardless of paths

Options:
  --full         run full integration gates for touched surfaces
  -v, --verbose  stream detailed command output
  -h, --help     show this help
EOF
}

mode=
range=
full=0
verbose=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --staged)
      mode=staged
      shift
      ;;
    --range)
      mode=range
      range="${2:-}"
      if [ -z "$range" ]; then
        echo "--range requires a git range" >&2
        exit 2
      fi
      shift 2
      ;;
    --all)
      mode=all
      shift
      ;;
    --full)
      full=1
      shift
      ;;
    -v | --verbose)
      verbose=1
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown verify-changed option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$mode" ]; then
  usage >&2
  exit 2
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-verify-changed.XXXXXX")"
paths_file="$tmp_dir/paths"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

cd "$REPO_ROOT"

case "$mode" in
  staged)
    git diff --cached --name-only --relative --diff-filter=ACDMRTUXB >"$paths_file"
    ;;
  range)
    git diff --name-only --relative "$range" >"$paths_file"
    ;;
  all)
    : >"$paths_file"
    ;;
esac

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

path_matches() {
  pattern="$1"
  grep -Eq "$pattern" "$paths_file"
}

rust_touched=0
web_touched=0
docs_only=0

if [ "$mode" = all ]; then
  rust_touched=1
  web_touched=1
elif [ ! -s "$paths_file" ]; then
  echo "No changed paths detected; skipping validation." >&2
  exit 0
else
  if path_matches '(^Cargo\.toml$|^Cargo\.lock$|^crates/|^src/|^tests/|^build\.rs$|^scripts/.*\.sh$|^\.githooks/)'; then
    rust_touched=1
  fi
  if path_matches '(^web/atlas-ui/|^crates/atlas-app-model/bindings/|^crates/atlas-app-model/src/)'; then
    web_touched=1
  fi
  if ! grep -Ev '(^scratch/plans/|\.md$)' "$paths_file" >/dev/null; then
    docs_only=1
  fi
fi

if [ "$docs_only" -eq 1 ]; then
  echo "Docs-only change detected; skipping code validation." >&2
  exit 0
fi

if [ "$rust_touched" -eq 0 ] && [ "$web_touched" -eq 0 ]; then
  echo "No Rust or web UI validation required for changed paths." >&2
  exit 0
fi

if [ "$rust_touched" -eq 1 ]; then
  if [ "$full" -eq 1 ]; then
    if [ "$verbose" -eq 1 ]; then
      run_check "rust full verify" "$REPO_ROOT/scripts/verify.sh" --verbose
    else
      run_check "rust full verify" "$REPO_ROOT/scripts/verify.sh"
    fi
  else
    run_check "cargo fmt" cargo fmt --check
    run_check "workspace clippy" cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
  fi
fi

if [ "$web_touched" -eq 1 ]; then
  if [ "$full" -eq 1 ]; then
    run_check "web UI verify" npm --prefix web/atlas-ui run verify
  else
    run_check "web UI format" npm --prefix web/atlas-ui run format:check
    run_check "web UI lint" npm --prefix web/atlas-ui run lint
    run_check "web UI typecheck" npm --prefix web/atlas-ui run typecheck
  fi
fi
