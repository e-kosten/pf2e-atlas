#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/validation/check-artifact-version-bump.sh --base <git-ref> [--head <git-ref>]

Require version bumps for schema, artifact-contract, and manifest owner changes
between the merge base and candidate. This source-only guard never builds an
artifact.
EOF
}

base=
head=HEAD
while [ "$#" -gt 0 ]; do
  case "$1" in
    --base) base="${2:-}"; shift 2 ;;
    --head) head="${2:-}"; shift 2 ;;
    -h | --help) usage; exit 0 ;;
    *) echo "Unknown artifact-version option: $1" >&2; usage >&2; exit 2 ;;
  esac
done
[ -n "$base" ] || { echo "--base is required" >&2; usage >&2; exit 2; }

repo_root="$(git rev-parse --show-toplevel)"
policy="$repo_root/scripts/validation/artifact-version-owners.txt"
merge_base="$(git merge-base "$base" "$head")"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-artifact-version.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
git diff --name-only --diff-filter=ACDMRTUXB "$merge_base" "$head" >"$tmp_dir/paths"

schema=0
contract=0
manifest=0
while IFS='|' read -r class pattern; do
  case "$class" in ''|'#'*) continue ;; esac
  while IFS= read -r path; do
    case "$path" in
      $pattern) eval "$class=1" ;;
    esac
  done <"$tmp_dir/paths"
done <"$policy"

read_constant() {
  revision="$1"
  constant="$2"
  for source in \
    crates/atlas-index/src/artifact/metadata.rs \
    crates/atlas-index/src/artifact/pair.rs \
    crates/atlas-ingest/src/artifact_manifest.rs
  do
    value="$(git show "$revision:$source" 2>/dev/null | sed -n "s/^\(pub \)\{0,1\}const $constant: &str = \"\([^\"]*\)\";.*/\2/p" | head -n 1)"
    if [ -n "$value" ]; then
      printf '%s\n' "$value"
      return 0
    fi
  done
  echo "unable to read $constant at $revision" >&2
  return 1
}

failed=0
check_bump() {
  class="$1"
  required="$2"
  constant="$3"
  [ "$required" -eq 1 ] || return 0
  before="$(read_constant "$merge_base" "$constant")"
  after="$(read_constant "$head" "$constant")"
  if [ "$before" = "$after" ]; then
    echo "$class owners changed without bumping $constant ($before)" >&2
    failed=1
  else
    echo "$class version bump: $before -> $after" >&2
  fi
}

check_bump schema "$schema" ARTIFACT_SCHEMA_VERSION
check_bump contract "$contract" ARTIFACT_CONTRACT_VERSION
check_bump manifest "$manifest" ARTIFACT_MANIFEST_VERSION

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "Artifact version policy passed for merge base $merge_base." >&2
