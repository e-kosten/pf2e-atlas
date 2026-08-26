#!/bin/sh

set -eu

if [ "$#" -ne 0 ]; then
  echo "validate-focused takes no arguments" >&2
  exit 2
fi

unset PF2E_SOURCE_ROOT PF2E_COMMIT CANDIDATE_COMMIT CANDIDATE_TREE \
  ATLAS_VALIDATION_SNAPSHOT_ROOT ATLAS_VALIDATION_FORCE_REPRODUCTION

repo_root=$(git rev-parse --show-toplevel)
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/atlas-validate-focused.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT HUP INT TERM
cd "$repo_root"

run() {
  label=$1
  shift
  log="$work_dir/$label.log"
  printf 'Running %s...\n' "$label" >&2
  if "$@" >"$log" 2>&1; then
    printf 'Passed %s\n' "$label" >&2
  else
    printf 'Failed %s\n' "$label" >&2
    tail -n 300 "$log" >&2
    return 1
  fi
}

run atlas-record cargo test -p atlas-record
run atlas-ingest cargo test -p atlas-ingest
run atlas-index cargo test -p atlas-index
