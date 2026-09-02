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
exhaustive=false' \
  crates/atlas-ingest/src/source/dto/creature_core.rs

assert_route canonical-writer 'lint=true
persistence=true
exhaustive=false' \
  crates/atlas-ingest/src/source/npc_core.rs \
  crates/atlas-index/src/write/sqlite/records.rs

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
exhaustive=true' \
  crates/atlas-ingest/src/source_coverage/parity.rs

assert_route vendor 'lint=false
persistence=false
exhaustive=true' \
  vendor/pf2e/static/system.json

assert_route h12-cutover 'lint=false
persistence=false
exhaustive=true' \
  crates/atlas-ingest/src/validation/source_leaf_cutover.rs

printf 'Source-leaf coverage routing fixtures passed.\n'
