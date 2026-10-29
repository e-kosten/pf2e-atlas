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
  --plan         explain selection, commands and prerequisites without running
  --status       inspect matching/missing/invalidated local evidence
  --serial       serialize frontend and Rust checks
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
inspect=
serial=0

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
    --plan | --status)
      inspect=$1; shift ;;
    --serial)
      serial=1; shift ;;
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

rust_touched=0
web_touched=0
source_touched=0
release_touched=0
research_touched=0
skill_touched=0
browser_touched=0
web_format_touched=0
dev_touched=0
tooling_touched=0
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
  tooling_touched=1
elif [ ! -s "$paths_file" ]; then
  docs_only=1
else
  classify_validation_paths "$paths_file"
fi

if [ "$list" -eq 1 ]; then
  for entry in "rust:$rust_touched" "web:$web_touched" "source:$source_touched" "release:$release_touched" "research:$research_touched" "skill:$skill_touched" "browser:$browser_touched" "dev:$dev_touched" "tooling:$tooling_touched"; do
    value=false
    [ "${entry#*:}" -eq 0 ] || value=true
    printf '%s=%s\n' "${entry%%:*}" "$value"
  done
  exit 0
fi

if [ "$docs_only" -eq 1 ]; then
  echo "Docs-only change detected; skipping code validation." >&2
  exit 0
fi

selected=
for entry in "rust:$rust_touched" "web:$web_touched" "source:$source_touched" "release:$release_touched" "research:$research_touched" "skill:$skill_touched" "browser:$browser_touched" "web-format:$web_format_touched" "dev:$dev_touched" "tooling:$tooling_touched"; do
  [ "${entry#*:}" -eq 0 ] || selected="$selected ${entry%%:*}"
done
if [ -n "$inspect" ]; then
  printf 'Selection mode: %s %s\n' "$mode" "$range"
  if [ "$full" -eq 1 ]; then
    printf 'Validation scope: full pre-PR gates\n'
  else
    printf 'Validation scope: iteration formatting only (use --full for pre-PR gates)\n'
  fi
  while IFS= read -r path || [ -n "$path" ]; do
    [ -z "$path" ] || printf '%s -> %s\n' "$path" "$(validation_surface_for_path "$path")"
  done <"$paths_file"
fi
rm -rf "$tmp_dir"
trap - EXIT HUP INT TERM
set -- --selected "$selected"
[ "$full" -eq 0 ] || set -- "$@" --full
[ "$verbose" -eq 0 ] || set -- "$@" --verbose
[ -z "$inspect" ] || set -- "$@" "$inspect"
[ "$serial" -eq 0 ] || set -- "$@" --serial
exec sh "$REPO_ROOT/scripts/validation/launch.sh" "$REPO_ROOT/scripts/validation/verify.mjs" "$@"
