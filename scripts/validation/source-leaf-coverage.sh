#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage:
  scripts/validation/source-leaf-coverage.sh route PATHS_FILE [BASE HEAD]
  scripts/validation/source-leaf-coverage.sh diff-paths BASE HEAD
  scripts/validation/source-leaf-coverage.sh lint
  scripts/validation/source-leaf-coverage.sh persistence

Classify changed paths for focused source-leaf and final artifact-integration
checks, or run the small A2 source-leaf suites. The lint suite requires
PF2E_SOURCE_REPOSITORY to identify the accepted pinned checkout.
EOF
}

repo_root="${SOURCE_LEAF_GIT_REPOSITORY:-$(git rev-parse --show-toplevel)}"
cd "$repo_root"
route_tmp_dir=
trap '[ -z "$route_tmp_dir" ] || rm -rf "$route_tmp_dir"' EXIT

matches() {
  pattern="$1"
  grep -Eq "$pattern" "$paths_file"
}

matches_artifact_owner() {
  policy="$repo_root/scripts/validation/artifact-version-owners.txt"
  while IFS= read -r path; do
    while IFS='|' read -r class pattern scope; do
      case "$class" in ''|'#'*) continue ;; esac
      # The policy intentionally supplies shell globs such as migrations/*.
      # shellcheck disable=SC2254
      case "$path" in
        $pattern)
          if [ "$scope" = test-tail ] \
            && [ -n "$route_base" ] \
            && [ -n "$route_head" ] \
            && test_tail_production_unchanged "$path" "$route_base" "$route_head"
          then
            continue
          fi
          if [ -n "$route_base" ] \
            && [ -n "$route_head" ] \
            && ! git cat-file -e "$route_base:$path" 2>/dev/null \
            && git cat-file -e "$route_head:$path" 2>/dev/null \
            && transfer_source="$(git show "$route_head:scripts/validation/artifact-owner-transfers.txt" 2>/dev/null \
              | awk -F '|' -v class="$class" -v target="$path" \
                '$1 == class && $3 == target { print $2 }')" \
            && [ -n "$transfer_source" ] \
            && "$repo_root/scripts/validation/verify-artifact-owner-transfer.py" \
              --class "$class" \
              --source "$transfer_source" \
              --target "$path" \
              --before "$route_base" \
              --after "$route_head" >/dev/null 2>&1
          then
            continue
          fi
          return 0
          ;;
      esac
    done <"$policy"
  done <"$paths_file"
  return 1
}

test_tail_production_unchanged() {
  path="$1"
  before="$2"
  after="$3"
  if [ -z "$route_tmp_dir" ]; then
    route_tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-source-leaf-route.XXXXXX")"
  fi
  before_source="$route_tmp_dir/owner-before"
  after_source="$route_tmp_dir/owner-after"
  before_production="$route_tmp_dir/owner-before-production"
  after_production="$route_tmp_dir/owner-after-production"
  git show "$before:$path" >"$before_source" 2>/dev/null || return 1
  git show "$after:$path" >"$after_source" 2>/dev/null || return 1
  before_markers="$(grep -c '^#\[cfg(test)\]$' "$before_source" || true)"
  after_markers="$(grep -c '^#\[cfg(test)\]$' "$after_source" || true)"
  [ "$before_markers" = 1 ] && [ "$after_markers" = 1 ] || return 1
  sed '/^#\[cfg(test)\]$/,$d' "$before_source" >"$before_production"
  sed '/^#\[cfg(test)\]$/,$d' "$after_source" >"$after_production"
  cmp -s "$before_production" "$after_production"
}

matches_pair_integration_owner() {
  matches '^(crates/atlas-index/src/artifact/(pair|pair_integration_tests|publication)\.rs$|crates/atlas-index/src/sqlite/reader\.rs$)'
}

route() {
  paths_file="$1"
  route_base="${2:-}"
  route_head="${3:-}"
  [ -f "$paths_file" ] || {
    echo "route requires a readable changed-path file" >&2
    exit 2
  }

  lint=false
  persistence=false
  pair_integration=false
  exhaustive=false

  if matches '^(contracts/pf2e-type-registry\.yaml$|contracts/source-leaf-coverage/|crates/atlas-ingest/src/source_coverage/|crates/atlas-ingest/src/source/(hazard_core|hazard_entities)(\.rs|_tests\.rs)$|crates/atlas-record/src/(hazard|hazard_projection)\.rs$|crates/atlas-ingest/tests/source_leaf_(coverage|persistence)\.rs$|crates/atlas-ingest/tests/fixtures/(source-leaf-coverage/|hazards/pinned/)|scripts/validation/(authenticate-pf2e-source|source-leaf-coverage|test-source-leaf-coverage-routing)\.sh$)'; then
    lint=true
  fi

  if matches '^(crates/atlas-ingest/src/source/dto/|crates/atlas-ingest/src/source/npc_core(\.rs|_tests\.rs)$|crates/atlas-ingest/src/index_build_input\.rs$|crates/atlas-record/src/(creature|creature_projection|json_projection/creature)\.rs$|crates/atlas-index/src/(artifact/canonical_json|read/records(\.rs|/canonical\.rs)|sqlite/mod\.rs|write\.rs|write/input|write/sqlite(\.rs|/(canonical|records)\.rs))$|crates/atlas-ingest/tests/source_leaf_persistence\.rs$|crates/atlas-ingest/tests/fixtures/source-leaf-coverage/actor-npc/persistence/)'; then
    lint=true
    persistence=true
  fi

  if matches_pair_integration_owner; then
    pair_integration=true
  fi

  # The embedded production build is the final cross-crate compatibility gate.
  # Route only concrete artifact constructors/codecs plus the production
  # validation path and document-embedding producers. Focused source-coverage
  # and search-only changes have cheaper direct tests and stay out of this gate.
  if matches_artifact_owner || matches '^(vendor/pf2e/|crates/atlas-embedding/src/(catalog|document_input|document_renderer|document_units|document_units/(builder|generation|model)|document_units/token_budget/(children|diagnostics|mod|telemetry)|minilm|model_cache|text|tokenization|unit_kind)\.rs$|crates/atlas-ingest/src/validation/mod\.rs$|crates/atlas-index/src/artifact/validation(\.rs|/)|crates/atlas-index/src/read/(validation\.rs|search/vector(\.rs|/(extension|validation)\.rs))$|crates/atlas-sqlite-vec/src/lib\.rs$|crates/atlas-cli/src/commands/index(\.rs|/args\.rs)$|scripts/validation/exhaustive\.sh$)'; then
    exhaustive=true
  fi

  printf 'lint=%s\npersistence=%s\npair_integration=%s\nexhaustive=%s\n' \
    "$lint" "$persistence" "$pair_integration" "$exhaustive"
}

diff_paths() {
  base="$1"
  head="$2"
  merge_base="$(git merge-base "$base" "$head")"
  git diff --no-renames --name-only --relative --diff-filter=ACDMRTUXB \
    "$merge_base" "$head"
}

case "${1:-}" in
  route)
    { [ "$#" -eq 2 ] || [ "$#" -eq 4 ]; } || { usage >&2; exit 2; }
    route "$2" "${3:-}" "${4:-}"
    ;;
  diff-paths)
    [ "$#" -eq 3 ] || { usage >&2; exit 2; }
    diff_paths "$2" "$3"
    ;;
  lint)
    [ "$#" -eq 1 ] || { usage >&2; exit 2; }
    source_repository="${PF2E_SOURCE_REPOSITORY:-}"
    # Git hooks export repository-local GIT_* variables. They must not redirect
    # the source-grounding subprocess from the independent PF2e checkout.
    unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR \
      GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
    if [ -z "$source_repository" ] || ! git -C "$source_repository" rev-parse --git-dir >/dev/null 2>&1; then
      echo "PF2E_SOURCE_REPOSITORY must name the accepted pinned Git checkout" >&2
      exit 2
    fi
    PF2E_SOURCE_REPOSITORY="$source_repository" \
      cargo test -p atlas-ingest --test source_leaf_coverage
    PF2E_SOURCE_REPOSITORY="$source_repository" \
      cargo test -p atlas-ingest --lib \
        source_coverage::receipt::tests::actor_adversaries_keep_exact_authored_null_identity_distinct_from_optional_null \
        -- --exact
    PF2E_SOURCE_REPOSITORY="$source_repository" \
      cargo test -p atlas-ingest --lib \
        source_coverage::receipt::tests::exact_intimidate_alias_is_indispensable_through_real_sqlite \
        -- --exact
    ;;
  persistence)
    [ "$#" -eq 1 ] || { usage >&2; exit 2; }
    cargo test -p atlas-ingest --test source_leaf_persistence \
      single_b1_no_embedding_sqlite_build_proves_ability_and_skill_ownership -- --exact
    ;;
  -h | --help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
