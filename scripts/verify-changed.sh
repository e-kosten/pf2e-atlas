#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/verify-changed.sh (--staged | --range <git-range> | --paths-file <file> | --all) [--full] [--verbose]

Run path-sensitive formatting checks, or full validation with --full.

Modes:
  --staged       validate staged paths for pre-commit
  --range RANGE  validate paths changed in a git diff range
  --paths-file FILE  validate a newline-separated list of paths
  --all          validate Rust and web UI gates regardless of paths

Options:
  --full         run full integration gates for touched surfaces
  -v, --verbose  stream detailed command output
  -h, --help     show this help
EOF
}

mode=
range=
input_paths=
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
    --paths-file)
      mode=paths
      input_paths="${2:-}"
      if [ -z "$input_paths" ]; then
        echo "--paths-file requires a file" >&2
        exit 2
      fi
      shift 2
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
. "$REPO_ROOT/scripts/git-hooks/common.sh"
# Resolve caller-relative files before normalizing the working directory.
if [ "$mode" = paths ]; then
  case "$input_paths" in
    /*) ;;
    *) input_paths="$PWD/$input_paths" ;;
  esac
fi
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-verify-changed.XXXXXX")"
paths_file="$tmp_dir/paths"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

cd "$REPO_ROOT"

case "$mode" in
  staged)
    collect_changed_paths "$paths_file" --cached --diff-filter=ACDMRTUXB
    ;;
  range)
    collect_changed_paths "$paths_file" "$range"
    ;;
  paths)
    cat "$input_paths" >"$paths_file"
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
  classify_validation_paths "$paths_file"
fi

if [ "$docs_only" -eq 1 ]; then
  echo "Docs-only change detected; skipping code validation." >&2
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
  fi
fi

if [ "$web_touched" -eq 1 ]; then
  if [ "$full" -eq 1 ]; then
    run_check "web UI verify" npm --prefix web/atlas-ui run verify
  else
    run_check "web UI format" npm --prefix web/atlas-ui run format:check
  fi
fi
