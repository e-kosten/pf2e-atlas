#!/bin/sh

set -eu

usage() {
  cat <<'EOF'
Usage:
  scripts/validation/source-leaf-coverage.sh route PATHS_FILE
  scripts/validation/source-leaf-coverage.sh lint
  scripts/validation/source-leaf-coverage.sh persistence

Classify changed paths or run the small A2 source-leaf suites. The lint suite
requires PF2E_SOURCE_REPOSITORY to identify the accepted pinned checkout.
EOF
}

repo_root="$(git rev-parse --show-toplevel)"
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
  exhaustive=false

  if matches '^(contracts/source-leaf-coverage/|crates/atlas-ingest/src/source_coverage/|crates/atlas-ingest/tests/source_leaf_(coverage|persistence)\.rs$|crates/atlas-ingest/tests/fixtures/source-leaf-coverage/|scripts/validation/(source-leaf-coverage|test-source-leaf-coverage-routing)\.sh$)'; then
    lint=true
  fi

  if matches '^(crates/atlas-ingest/src/source/dto/|crates/atlas-ingest/src/source/npc_core(\.rs|_tests\.rs)$|crates/atlas-ingest/src/index_build_input\.rs$|crates/atlas-record/src/(creature|creature_projection|json_projection/creature)\.rs$|crates/atlas-index/src/(artifact/canonical_json|read/records/canonical|write/input|write/sqlite/records)\.rs$|crates/atlas-ingest/tests/source_leaf_persistence\.rs$|crates/atlas-ingest/tests/fixtures/source-leaf-coverage/actor-npc/persistence/)'; then
    lint=true
    persistence=true
  fi

  if matches '^(vendor/pf2e/|crates/atlas-ingest/src/source_coverage/(contract|parity|prevalence|receipt|registry)\.rs$|contracts/source-leaf-coverage/v1/(schema|prevalence|type-registry)\.(json|yaml)$|scripts/validation/exhaustive\.sh$|crates/atlas-ingest/src/validation/)'; then
    exhaustive=true
  fi

  printf 'lint=%s\npersistence=%s\nexhaustive=%s\n' "$lint" "$persistence" "$exhaustive"
}

case "${1:-}" in
  route)
    [ "$#" -eq 2 ] || { usage >&2; exit 2; }
    route "$2"
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
    ;;
  persistence)
    [ "$#" -eq 1 ] || { usage >&2; exit 2; }
    cargo test -p atlas-ingest --test source_leaf_persistence \
      single_actor_npc_no_embedding_persistence_proves_current_mismatch -- --exact
    ;;
  -h | --help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
