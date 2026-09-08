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
persistence=false' \
  contracts/source-leaf-coverage/v1/actor-npc.yaml

assert_route parser-owner 'lint=true
persistence=true' \
  crates/atlas-ingest/src/source/dto/creature_core.rs

assert_route canonical-writer 'lint=true
persistence=true' \
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
persistence=true' "$owner"
done

assert_route persistence-facade 'lint=true
persistence=true' \
  crates/atlas-index/src/sqlite/mod.rs

assert_route authoritative-registry 'lint=true
persistence=false' \
  contracts/pf2e-type-registry.yaml

assert_route nonexistent-source-leaf-registry 'lint=true
persistence=false' \
  contracts/source-leaf-coverage/v1/type-registry.yaml

assert_route ui-only 'lint=false
persistence=false' \
  web/atlas-ui/src/record/CreatureSheet.tsx

assert_route unrelated 'lint=false
persistence=false' \
  README.md

assert_route shared-engine 'lint=true
persistence=false' \
  crates/atlas-ingest/src/source_coverage/parity.rs

assert_route source-authentication 'lint=true
persistence=false' \
  scripts/validation/authenticate-pf2e-source.sh

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
persistence=true' "$rename_paths"

delete_owner='crates/atlas-index/src/read/records.rs'
new_git_fixture delete-owner "$delete_owner"
git -C "$case_root" rm -q "$delete_owner"
git -C "$case_root" commit -qm "test: delete routed owner"
delete_paths="$fixture/delete-owner.paths"
collect_fixture_paths "$case_root" "$fixture_base" "$delete_paths"
grep -Fxq "$delete_owner" "$delete_paths"
assert_route_file delete-owner 'lint=true
persistence=true' "$delete_paths"

shared_owner='crates/atlas-ingest/src/source_coverage/receipt.rs'
new_git_fixture rename-shared-engine "$shared_owner"
mkdir -p "$case_root/docs"
git -C "$case_root" mv "$shared_owner" docs/retired-receipt.rs
git -C "$case_root" commit -qm "test: rename shared engine owner"
shared_paths="$fixture/rename-shared-engine.paths"
collect_fixture_paths "$case_root" "$fixture_base" "$shared_paths"
grep -Fxq "$shared_owner" "$shared_paths"
assert_route_file rename-shared-engine 'lint=true
persistence=false' "$shared_paths"

printf 'Source-leaf coverage routing fixtures passed.\n'
