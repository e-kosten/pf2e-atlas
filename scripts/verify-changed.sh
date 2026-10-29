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
  --all          select every validation surface regardless of paths

Options:
  --list         print selected surfaces as key=true/false; run no checks
  --full         run full integration gates for touched surfaces
  -v, --verbose  stream detailed command output
  -h, --help     show this help
EOF
}

mode=
range=
input_paths=
full=0
list=0
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
    --list)
      list=1
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

. "$REPO_ROOT/scripts/validation/run-check.sh"

rust_touched=0
web_touched=0
source_touched=0
release_touched=0
research_touched=0
skill_touched=0
browser_touched=0
web_format_touched=0
dev_touched=0
docs_only=0

if [ "$mode" = all ]; then
  rust_touched=1
  web_touched=1
  source_touched=1
  release_touched=1
  research_touched=1
  skill_touched=1
  browser_touched=1
  dev_touched=1
elif [ ! -s "$paths_file" ]; then
  docs_only=1
else
  classify_validation_paths "$paths_file"
fi

if [ "$list" -eq 1 ]; then
  for entry in "rust:$rust_touched" "web:$web_touched" "source:$source_touched" "release:$release_touched" "research:$research_touched" "skill:$skill_touched" "browser:$browser_touched" "dev:$dev_touched"; do
    value=false
    [ "${entry#*:}" -eq 0 ] || value=true
    printf '%s=%s\n' "${entry%%:*}" "$value"
  done
  exit 0
fi

atlas_validation_kind=final
[ "$full" -eq 1 ] || atlas_validation_kind=iteration

if [ "$docs_only" -eq 1 ]; then
  echo "Docs-only change detected; skipping code validation." >&2
  exit 0
fi

if [ "$web_touched" -eq 1 ] || [ "$web_format_touched" -eq 1 ] || { [ "$full" -eq 1 ] && [ "$browser_touched" -eq 1 ] && [ "$rust_touched" -eq 0 ]; }; then
  if [ "$full" -eq 1 ] && [ "$web_touched" -eq 1 ]; then
    run_check "web UI verify" npm --prefix web/atlas-ui run verify
  elif [ "$full" -eq 1 ]; then
    run_check "browser checks and bundle" npm --prefix web/atlas-ui run verify:bundle
  else
    run_check "web UI format" npm --prefix web/atlas-ui run format:check
  fi
fi

if [ "$full" -eq 1 ] && [ "$rust_touched" -eq 1 ] && [ "$web_touched" -eq 0 ]; then
  run_check "web bundle for Rust" npm --prefix web/atlas-ui run build
fi

if [ "$rust_touched" -eq 1 ]; then
  if [ "$full" -eq 1 ]; then
    if [ "$verbose" -eq 1 ]; then
      run_composite "$REPO_ROOT/scripts/verify.sh" --verbose
    else
      run_composite "$REPO_ROOT/scripts/verify.sh"
    fi
  else
    run_check "cargo fmt" cargo fmt --check
  fi
fi

if [ "$dev_touched" -eq 1 ] && [ "$rust_touched" -eq 0 ]; then
  if [ "$full" -eq 1 ]; then
    run_check "developer formatting" cargo fmt --check
    run_check "developer clippy" cargo clippy -p atlas-dev -p atlas-cli-support --all-targets -- -D warnings -D clippy::dbg_macro
    run_check "developer runtime clippy" cargo clippy -p atlas-dev -p atlas-cli-support --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unimplemented -D clippy::todo -D clippy::unreachable
    run_check "developer tests" cargo test -p atlas-dev -p atlas-cli-support --locked
    run_check "developer build" cargo build -p atlas-dev --locked
  else
    run_check "cargo fmt" cargo fmt --check
  fi
fi

# Contributor packages have no formatting gate; substantive checks are explicit.
if [ "$full" -eq 1 ]; then
  if [ "$skill_touched" -eq 1 ]; then
    run_check "bundled skill" node --test scripts/validation/skill.test.ts
    run_check "skill package" node scripts/validation/skill.ts
  fi
  if [ "$source_touched" -eq 1 ]; then
    run_check "source contract fixtures" npm --prefix dev-tools/source-contracts run verify
    run_check "pinned source generation" npm --prefix dev-tools/source-contracts run verify-generated
  fi
  if [ "$release_touched" -eq 1 ]; then
    run_check "release tooling and static checks" sh scripts/release/validate-release-tooling.sh
    run_check "release smoke tests" sh scripts/release/test-release-tools.sh
    run_check "release preparation tests" sh scripts/release/test-prepare-release.sh
  fi
  if [ "$research_touched" -eq 1 ]; then
    run_check "research evidence" sh scripts/validation/research.sh
  fi
fi
