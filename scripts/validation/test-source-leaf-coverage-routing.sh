#!/bin/sh

set -eu

repo_root="$(git rev-parse --show-toplevel)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/atlas-source-leaf-routing.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT HUP INT TERM

assert_route() {
  name="$1"
  expected="$2"
  shift 2
  paths="$fixture/$name.paths"
  printf '%s\n' "$@" >"$paths"
  assert_route_file "$name" "$expected" "$paths"
}

assert_route_file() {
  name="$1"
  expected="$2"
  paths="$3"
  actual="$($repo_root/scripts/validation/source-leaf-coverage.sh route "$paths")"
  [ "$actual" = "$expected" ] || {
    printf '%s route mismatch\nexpected:\n%s\nactual:\n%s\n' "$name" "$expected" "$actual" >&2
    exit 1
  }
}

assert_route actor-ledger 'lint=true
persistence=false
exhaustive=false' \
  contracts/source-leaf-coverage/v1/actor-npc.yaml

assert_route parser-owner 'lint=true
persistence=true
exhaustive=true' \
  crates/atlas-ingest/src/source/dto/creature_core.rs

assert_route canonical-writer 'lint=true
persistence=true
exhaustive=true' \
  crates/atlas-ingest/src/source/npc_core.rs \
  crates/atlas-index/src/write/sqlite/records.rs

for owner in \
  crates/atlas-index/src/write/sqlite/canonical.rs \
  crates/atlas-index/src/write/sqlite.rs \
  crates/atlas-index/src/read/records.rs \
  crates/atlas-index/src/read/records/canonical.rs \
  crates/atlas-index/src/write.rs
do
  fixture_name="$(printf '%s' "$owner" | tr '/.' '--')"
  assert_route "persistence-owner-$fixture_name" 'lint=true
persistence=true
exhaustive=true' "$owner"
done

assert_route persistence-facade 'lint=true
persistence=true
exhaustive=false' \
  crates/atlas-index/src/sqlite/mod.rs

assert_route authoritative-registry 'lint=true
persistence=false
exhaustive=false' \
  contracts/pf2e-type-registry.yaml

assert_route nonexistent-source-leaf-registry 'lint=true
persistence=false
exhaustive=false' \
  contracts/source-leaf-coverage/v1/type-registry.yaml

assert_route ui-only 'lint=false
persistence=false
exhaustive=false' \
  web/atlas-ui/src/record/CreatureSheet.tsx

assert_route unrelated 'lint=false
persistence=false
exhaustive=false' \
  README.md

assert_route shared-engine 'lint=true
persistence=false
exhaustive=false' \
  crates/atlas-ingest/src/source_coverage/parity.rs

assert_route vendor 'lint=false
persistence=false
exhaustive=true' \
  vendor/pf2e/static/system.json

assert_route final-gate-runner 'lint=false
persistence=false
exhaustive=true' \
  scripts/validation/exhaustive.sh

assert_route exhaustive-engine 'lint=false
persistence=false
exhaustive=true' \
  crates/atlas-ingest/src/validation/mod.rs

assert_route exhaustive-diagnostic-engine 'lint=false
persistence=false
exhaustive=false' \
  crates/atlas-ingest/src/validation/record_round_trip_diagnostic.rs

assert_route validation-test-only 'lint=false
persistence=false
exhaustive=false' \
  crates/atlas-ingest/src/validation/c2pr_tests.rs

assert_route artifact-schema-owner 'lint=false
persistence=false
exhaustive=true' \
  crates/atlas-index/migrations/00000000000002_atomic_canonical_records/up.sql

assert_route document-embedding-producer 'lint=false
persistence=false
exhaustive=true' \
  crates/atlas-embedding/src/document_units/builder.rs

for owner in \
  crates/atlas-index/src/read/validation.rs \
  crates/atlas-index/src/read/search/vector.rs \
  crates/atlas-index/src/read/search/vector/extension.rs \
  crates/atlas-sqlite-vec/src/lib.rs
do
  fixture_name="$(printf '%s' "$owner" | tr '/.' '--')"
  assert_route "production-validation-owner-$fixture_name" 'lint=false
persistence=false
exhaustive=true' "$owner"
done

assert_route query-embedding-only 'lint=false
persistence=false
exhaustive=false' \
  crates/atlas-embedding/src/vector_math.rs

assert_route search-only 'lint=false
persistence=false
exhaustive=false' \
  crates/atlas-search/src/search.rs \
  crates/atlas-index/src/read/search/fts/ranking.rs

new_git_fixture() {
  name="$1"
  owner="$2"
  case_root="$fixture/$name"
  mkdir -p "$case_root/$(dirname "$owner")"
  git -C "$case_root" init -q
  git -C "$case_root" config user.name "Atlas Routing Fixture"
  git -C "$case_root" config user.email "atlas-routing@example.invalid"
  printf 'owned\n' >"$case_root/$owner"
  git -C "$case_root" add .
  git -C "$case_root" commit -qm "test: add routed owner"
  fixture_base="$(git -C "$case_root" rev-parse HEAD)"
}

collect_fixture_paths() {
  case_root="$1"
  base="$2"
  output="$3"
  head="$(git -C "$case_root" rev-parse HEAD)"
  SOURCE_LEAF_GIT_REPOSITORY="$case_root" \
    "$repo_root/scripts/validation/source-leaf-coverage.sh" diff-paths "$base" "$head" \
    >"$output"
}

rename_owner='crates/atlas-index/src/write/sqlite.rs'
new_git_fixture rename-owner "$rename_owner"
mkdir -p "$case_root/docs"
git -C "$case_root" mv "$rename_owner" docs/retired-writer.rs
git -C "$case_root" commit -qm "test: rename routed owner"
rename_paths="$fixture/rename-owner.paths"
collect_fixture_paths "$case_root" "$fixture_base" "$rename_paths"
grep -Fxq "$rename_owner" "$rename_paths"
grep -Fxq 'docs/retired-writer.rs' "$rename_paths"
assert_route_file rename-owner 'lint=true
persistence=true
exhaustive=true' "$rename_paths"

delete_owner='crates/atlas-index/src/read/records.rs'
new_git_fixture delete-owner "$delete_owner"
git -C "$case_root" rm -q "$delete_owner"
git -C "$case_root" commit -qm "test: delete routed owner"
delete_paths="$fixture/delete-owner.paths"
collect_fixture_paths "$case_root" "$fixture_base" "$delete_paths"
grep -Fxq "$delete_owner" "$delete_paths"
assert_route_file delete-owner 'lint=true
persistence=true
exhaustive=true' "$delete_paths"

shared_owner='crates/atlas-ingest/src/source_coverage/receipt.rs'
new_git_fixture rename-shared-engine "$shared_owner"
mkdir -p "$case_root/docs"
git -C "$case_root" mv "$shared_owner" docs/retired-receipt.rs
git -C "$case_root" commit -qm "test: rename shared engine owner"
shared_paths="$fixture/rename-shared-engine.paths"
collect_fixture_paths "$case_root" "$fixture_base" "$shared_paths"
grep -Fxq "$shared_owner" "$shared_paths"
assert_route_file rename-shared-engine 'lint=true
persistence=false
exhaustive=false' "$shared_paths"

printf 'Source-leaf coverage routing fixtures passed.\n'
