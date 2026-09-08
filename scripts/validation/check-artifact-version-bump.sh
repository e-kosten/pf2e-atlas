#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage: scripts/validation/check-artifact-version-bump.sh --base <git-ref> [--head <git-ref>]

Require sequential version bumps for schema, artifact-contract, and manifest
owner changes along the merge-base-to-candidate ancestry. This source-only
guard never builds an artifact.
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
if ! git merge-base --is-ancestor "$merge_base" "$head"; then
  echo "artifact-version head $head does not descend from merge base $merge_base" >&2
  exit 1
fi
git rev-list --ancestry-path --reverse "$merge_base..$head" >"$tmp_dir/commits"
if git rev-list --ancestry-path --min-parents=2 "$merge_base..$head" | grep -q .; then
  echo "artifact-version history from $merge_base to $head is nonlinear; pass the explicit candidate head or use a linear candidate ancestry" >&2
  exit 1
fi

owner_change_requires_bump() {
  owner_class="$1"
  owner_path="$2"
  owner_before="$3"
  owner_after="$4"
  if [ "$owner_class" = contract ] && [ "$owner_path" = crates/atlas-index/src/artifact/metadata.rs ]; then
    git diff --unified=0 "$owner_before" "$owner_after" -- "$owner_path" \
      | sed -n '/^[+-][^+-]/p' \
      | grep -Ev '^[+-](pub )?const ARTIFACT_(CONTRACT|SCHEMA|MANIFEST)_VERSION: &str = "[^"]*";$' \
      | grep -q .
    return
  fi
  return 0
}

commit_changes_class() {
  wanted_class="$1"
  change_before="$2"
  change_after="$3"
  git diff --name-only --no-renames --diff-filter=ACDMRTUXB "$change_before" "$change_after" \
    | LC_ALL=C sort -u >"$tmp_dir/paths"
  while IFS='|' read -r policy_class policy_pattern; do
    case "$policy_class" in ''|'#'*) continue ;; esac
    [ "$policy_class" = "$wanted_class" ] || continue
    while IFS= read -r changed_path; do
      # The policy intentionally supplies shell globs such as migrations/*.
      # shellcheck disable=SC2254
      case "$changed_path" in
        $policy_pattern)
          if owner_change_requires_bump "$policy_class" "$changed_path" "$change_before" "$change_after"; then
            return 0
          fi
          ;;
      esac
    done <"$tmp_dir/paths"
  done <"$policy"
  return 1
}

read_constant() {
  read_revision="$1"
  read_constant_name="$2"
  for read_source in \
    crates/atlas-index/src/artifact/metadata.rs \
    crates/atlas-index/src/artifact/pair.rs \
    crates/atlas-ingest/src/artifact_manifest.rs
  do
    read_value="$(git show "$read_revision:$read_source" 2>/dev/null | sed -n "s/^\(pub \)\{0,1\}const $read_constant_name: &str = \"\([^\"]*\)\";.*/\2/p" | head -n 1)"
    if [ -n "$read_value" ]; then
      printf '%s\n' "$read_value"
      return 0
    fi
  done
  echo "unable to read $read_constant_name at $read_revision" >&2
  return 1
}

expected_next_version() {
  version_class="$1"
  version_before="$2"
  case "$version_class" in
    schema)
      version_number="$version_before"
      version_prefix=
      ;;
    contract | manifest)
      case "$version_before" in
        */v[0-9]*)
          version_number="${version_before##*/v}"
          version_prefix="${version_before%/v*}/v"
          ;;
        *) return 1 ;;
      esac
      ;;
    *) return 1 ;;
  esac
  case "$version_number" in
    '' | *[!0-9]*) return 1 ;;
  esac
  printf '%s%s\n' "$version_prefix" "$((version_number + 1))"
}

validate_history() {
  history_class="$1"
  history_constant="$2"
  before_revision="$merge_base"
  if ! before_version="$(read_constant "$before_revision" "$history_constant")"; then
    return 1
  fi
  if ! expected_next_version "$history_class" "$before_version" >/dev/null; then
    echo "unable to parse $history_class version from $history_constant=$before_version at $before_revision" >&2
    return 1
  fi
  owner_change_seen=0
  version_bump_seen=0

  while IFS= read -r revision; do
    if commit_changes_class "$history_class" "$before_revision" "$revision"; then
      owner_change_seen=1
    fi
    if ! current_version="$(read_constant "$revision" "$history_constant")"; then
      return 1
    fi
    if ! expected_next_version "$history_class" "$current_version" >/dev/null; then
      echo "unable to parse $history_class version from $history_constant=$current_version at $revision" >&2
      return 1
    fi
    if [ "$current_version" != "$before_version" ]; then
      if ! expected="$(expected_next_version "$history_class" "$before_version")"; then
        echo "unable to derive the next $history_class version from $history_constant=$before_version at $before_revision" >&2
        return 1
      fi
      if [ "$current_version" != "$expected" ]; then
        echo "$history_class version transition at $revision must advance exactly $before_version -> $expected (found $current_version)" >&2
        return 1
      fi
      echo "$history_class version bump at $revision: $before_version -> $current_version" >&2
      version_bump_seen=1
    fi
    before_revision="$revision"
    before_version="$current_version"
  done <"$tmp_dir/commits"

  if [ "$owner_change_seen" -ne 0 ] && [ "$version_bump_seen" -eq 0 ]; then
    echo "$history_class owners changed without a qualifying $history_constant bump (head $head remains $before_version)" >&2
    return 1
  fi
}

failed=0
validate_history schema ARTIFACT_SCHEMA_VERSION || failed=1
validate_history contract ARTIFACT_CONTRACT_VERSION || failed=1
validate_history manifest ARTIFACT_MANIFEST_VERSION || failed=1
if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "Artifact version policy passed for linear history $merge_base..$head." >&2
