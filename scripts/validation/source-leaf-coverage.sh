#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage:
  scripts/validation/source-leaf-coverage.sh route PATHS_FILE
  scripts/validation/source-leaf-coverage.sh diff-paths BASE HEAD
  scripts/validation/source-leaf-coverage.sh lint
  scripts/validation/source-leaf-coverage.sh persistence

Classify changed paths for focused source-leaf checks, or run the small A2
source-leaf suites. The lint suite requires
PF2E_SOURCE_REPOSITORY to identify the accepted pinned checkout.
EOF
}

repo_root="${SOURCE_LEAF_GIT_REPOSITORY:-$(git rev-parse --show-toplevel)}"
cd "$repo_root"

matches() {
  pattern="$1"
  grep -Eq "$pattern" "$paths_file"
}

route() {
  paths_file="$1"
  [ -f "$paths_file" ] || {
    echo "route requires a readable changed-path file" >&2
    exit 2
  }

  lint=false
  persistence=false

  if matches '^(contracts/pf2e-type-registry\.yaml$|contracts/source-leaf-coverage/|crates/atlas-ingest/src/source_coverage/|crates/atlas-ingest/src/source/(hazard_core|hazard_entities)(\.rs|_tests\.rs)$|crates/atlas-record/src/(hazard|hazard_projection)\.rs$|crates/atlas-ingest/tests/source_leaf_(coverage|persistence)\.rs$|crates/atlas-ingest/tests/fixtures/(source-leaf-coverage/|hazards/pinned/)|scripts/validation/(authenticate-pf2e-source|source-leaf-coverage|test-source-leaf-coverage-routing)\.sh$)'; then
    lint=true
  fi

  if matches '^(crates/atlas-ingest/src/source/dto/|crates/atlas-ingest/src/source/npc_core(\.rs|_tests\.rs)$|crates/atlas-ingest/src/index_build_input\.rs$|crates/atlas-record/src/(creature|creature_projection|json_projection/creature)\.rs$|crates/atlas-index/src/(artifact/canonical_json|read/records(\.rs|/canonical\.rs)|sqlite/mod\.rs|write\.rs|write/input|write/sqlite(\.rs|/(canonical|records)\.rs))$|crates/atlas-ingest/tests/source_leaf_persistence\.rs$|crates/atlas-ingest/tests/fixtures/source-leaf-coverage/actor-npc/persistence/)'; then
    lint=true
    persistence=true
  fi

  printf 'lint=%s\npersistence=%s\n' "$lint" "$persistence"
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
    [ "$#" -eq 2 ] || { usage >&2; exit 2; }
    route "$2"
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
