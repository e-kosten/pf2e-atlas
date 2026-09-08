#!/bin/sh

set -eu

repo_root="$(git rev-parse --show-toplevel)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/atlas-version-guard-fixture.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT HUP INT TERM

new_fixture() {
  name="$1"
  case_dir="$fixture/$name"
  mkdir -p "$case_dir"
  git -C "$case_dir" init -q
  git -C "$case_dir" config user.name "Atlas Fixture"
  git -C "$case_dir" config user.email "atlas-fixture@example.invalid"
  mkdir -p "$case_dir/scripts/validation"
  cp "$repo_root/scripts/validation/check-artifact-version-bump.sh" "$case_dir/scripts/validation/"
  cp "$repo_root/scripts/validation/artifact-version-owners.txt" "$case_dir/scripts/validation/"
  mkdir -p "$case_dir/crates/atlas-index/src/artifact"
  printf '%s\n' \
    'pub const ARTIFACT_CONTRACT_VERSION: &str = "fixture-contract/v1";' \
    'pub const ARTIFACT_SCHEMA_VERSION: &str = "1";' \
    'pub const ARTIFACT_MANIFEST_VERSION: &str = "fixture-manifest/v1";' \
    >"$case_dir/crates/atlas-index/src/artifact/metadata.rs"
  for path in \
    crates/atlas-index/src/artifact/inventory/tables/canonical.rs \
    crates/atlas-index/src/artifact/canonical_json.rs \
    crates/atlas-index/src/artifact/pair.rs \
    crates/atlas-index/src/artifact/storage.rs \
    crates/atlas-index/src/read/records/canonical.rs \
    crates/atlas-index/src/write/input.rs \
    crates/atlas-index/src/write/sqlite/records.rs \
    crates/atlas-index/migrations/0001_fixture/up.sql \
    crates/atlas-record/src/content/search_projection.rs \
    crates/atlas-record/src/creature.rs \
    crates/atlas-record/src/hazard.rs \
    crates/atlas-record/src/mechanics/projection.rs \
    crates/atlas-record/src/metrics/model.rs \
    crates/atlas-record/src/reference_policy.rs \
    crates/atlas-record/src/retrieved_record.rs \
    crates/atlas-ingest/src/source/dto/npc.rs \
    crates/atlas-ingest/src/source/dto/hazard.rs \
    crates/atlas-ingest/src/source/loader.rs \
    crates/atlas-ingest/src/source/normalize.rs \
    crates/atlas-ingest/src/source/normalize/content.rs \
    crates/atlas-ingest/src/source/npc_core.rs \
    crates/atlas-ingest/src/source/npc_entities.rs \
    crates/atlas-ingest/src/source/hazard_core.rs \
    crates/atlas-ingest/src/source/hazard_entities.rs \
    crates/atlas-ingest/src/source/owned_content.rs \
    crates/atlas-ingest/src/build.rs \
    crates/atlas-ingest/src/diagnostics.rs \
    crates/atlas-ingest/src/embeddings.rs \
    crates/atlas-ingest/src/embeddings/generation.rs \
    crates/atlas-ingest/src/records/aliases.rs \
    crates/atlas-ingest/src/records/loaded.rs \
    crates/atlas-ingest/src/records/metrics/emit.rs \
    crates/atlas-ingest/src/records/references.rs \
    crates/atlas-ingest/src/records/taxonomy.rs \
    crates/atlas-ingest/src/records/variants.rs \
    crates/atlas-ingest/src/generated/afflictions/records.rs \
    crates/atlas-ingest/src/index_build_input.rs \
    crates/atlas-ingest/src/source_pipeline.rs \
    crates/atlas-ingest/src/artifact_manifest.rs
  do
    mkdir -p "$case_dir/$(dirname "$path")"
    printf 'base\n' >"$case_dir/$path"
  done
  for path in \
    crates/atlas-index/src/artifact/validation/canonical.rs \
    crates/atlas-record/src/json_projection.rs \
    crates/atlas-record/src/mechanics_view.rs \
    crates/atlas-record/src/presentation.rs \
    crates/atlas-record/src/presentation_recipe_tests.rs \
    crates/atlas-record/src/metrics/tests.rs \
    crates/atlas-ingest/src/source/dto/tests.rs \
    crates/atlas-ingest/src/source/normalize/content_tests.rs \
    crates/atlas-ingest/src/source/npc_core_tests.rs \
    crates/atlas-ingest/src/records/aliases/tests.rs \
    crates/atlas-ingest/src/records/metrics/tests.rs \
    README.md
  do
    mkdir -p "$case_dir/$(dirname "$path")"
    printf 'unrelated\n' >"$case_dir/$path"
  done
  git -C "$case_dir" add .
  git -C "$case_dir" commit -qm "test: base fixture"
  case_base="$(git -C "$case_dir" rev-parse HEAD)"
}

commit_case() {
  git -C "$case_dir" add -A
  git -C "$case_dir" commit -qm "$1"
}

replace_version() {
  before="$1"
  after="$2"
  sed -i.bak "s#$before#$after#" "$case_dir/crates/atlas-index/src/artifact/metadata.rs"
  rm "$case_dir/crates/atlas-index/src/artifact/metadata.rs.bak"
}

expect_guard_failure() {
  if (cd "$case_dir" && scripts/validation/check-artifact-version-bump.sh --base "$case_base") >/dev/null 2>&1; then
    echo "$1" >&2
    exit 1
  fi
}

expect_guard_success() {
  if ! (cd "$case_dir" && scripts/validation/check-artifact-version-bump.sh --base "$case_base") >/dev/null 2>&1; then
    echo "$1" >&2
    exit 1
  fi
}

expect_guard_failure_head() {
  message="$1"
  candidate_head="$2"
  if (cd "$case_dir" && scripts/validation/check-artifact-version-bump.sh --base "$case_base" --head "$candidate_head") >/dev/null 2>&1; then
    echo "$message" >&2
    exit 1
  fi
}

expect_guard_success_head() {
  message="$1"
  candidate_head="$2"
  if ! (cd "$case_dir" && scripts/validation/check-artifact-version-bump.sh --base "$case_base" --head "$candidate_head") >/dev/null 2>&1; then
    echo "$message" >&2
    exit 1
  fi
}

# Keep one positive fixture for each compatibility-owner family. The seven paths
# called out by the independent review are included literally in this inventory.
for owner in \
  crates/atlas-index/src/artifact/metadata.rs \
  crates/atlas-index/src/artifact/inventory/tables/canonical.rs \
  crates/atlas-index/src/artifact/storage.rs \
  crates/atlas-index/src/read/records/canonical.rs \
  crates/atlas-index/src/write/input.rs \
  crates/atlas-index/src/write/sqlite/records.rs \
  crates/atlas-record/src/content/search_projection.rs \
  crates/atlas-record/src/creature.rs \
  crates/atlas-record/src/hazard.rs \
  crates/atlas-record/src/mechanics/projection.rs \
  crates/atlas-record/src/metrics/model.rs \
  crates/atlas-record/src/reference_policy.rs \
  crates/atlas-record/src/retrieved_record.rs \
  crates/atlas-ingest/src/source/dto/npc.rs \
  crates/atlas-ingest/src/source/dto/hazard.rs \
  crates/atlas-ingest/src/source/loader.rs \
  crates/atlas-ingest/src/source/normalize.rs \
  crates/atlas-ingest/src/source/normalize/content.rs \
  crates/atlas-ingest/src/source/npc_core.rs \
  crates/atlas-ingest/src/source/npc_entities.rs \
  crates/atlas-ingest/src/source/hazard_core.rs \
  crates/atlas-ingest/src/source/hazard_entities.rs \
  crates/atlas-ingest/src/source/owned_content.rs \
  crates/atlas-ingest/src/diagnostics.rs \
  crates/atlas-ingest/src/embeddings.rs \
  crates/atlas-ingest/src/embeddings/generation.rs \
  crates/atlas-ingest/src/records/aliases.rs \
  crates/atlas-ingest/src/records/loaded.rs \
  crates/atlas-ingest/src/records/metrics/emit.rs \
  crates/atlas-ingest/src/records/references.rs \
  crates/atlas-ingest/src/records/taxonomy.rs \
  crates/atlas-ingest/src/records/variants.rs \
  crates/atlas-ingest/src/generated/afflictions/records.rs \
  crates/atlas-ingest/src/index_build_input.rs \
  crates/atlas-ingest/src/source_pipeline.rs
do
  fixture_name="$(printf '%s' "$owner" | tr '/.' '--')"
  new_fixture "contract-owner-$fixture_name"
  printf 'changed\n' >>"$case_dir/$owner"
  commit_case "test: change contract owner"
  expect_guard_failure "guard missed contract owner $owner"
done

new_fixture schema-owner
printf 'changed schema\n' >>"$case_dir/crates/atlas-index/migrations/0001_fixture/up.sql"
commit_case "test: change schema owner"
expect_guard_failure "guard accepted a schema owner without a schema bump"

new_fixture manifest-owner
printf 'changed manifest\n' >>"$case_dir/crates/atlas-ingest/src/artifact_manifest.rs"
commit_case "test: change manifest owner"
expect_guard_failure "guard accepted a manifest owner without a manifest bump"

new_fixture build-contract-and-manifest-owner
printf 'changed final build projection\n' >>"$case_dir/crates/atlas-ingest/src/build.rs"
commit_case "test: change final build projection"
expect_guard_failure "guard accepted the final build projection without contract and manifest bumps"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
replace_version 'fixture-manifest/v1' 'fixture-manifest/v2'
commit_case "test: bump build projection contract and manifest versions"
expect_guard_success "guard rejected exact contract and manifest bumps for the final build projection"

for unrelated in \
  crates/atlas-index/src/artifact/validation/canonical.rs \
  crates/atlas-record/src/json_projection.rs \
  crates/atlas-record/src/mechanics_view.rs \
  crates/atlas-record/src/presentation.rs \
  crates/atlas-record/src/presentation_recipe_tests.rs \
  crates/atlas-record/src/metrics/tests.rs \
  crates/atlas-ingest/src/source/dto/tests.rs \
  crates/atlas-ingest/src/source/normalize/content_tests.rs \
  crates/atlas-ingest/src/source/npc_core_tests.rs \
  crates/atlas-ingest/src/records/aliases/tests.rs \
  crates/atlas-ingest/src/records/metrics/tests.rs \
  README.md
do
  fixture_name="$(printf '%s' "$unrelated" | tr '/.' '--')"
  new_fixture "unrelated-$fixture_name"
  printf 'changed unrelated\n' >>"$case_dir/$unrelated"
  commit_case "test: change unrelated path"
  expect_guard_success "guard rejected unrelated path $unrelated"
done

new_fixture version-declaration-only
replace_version 'fixture-manifest/v1' 'fixture-manifest/v2'
commit_case "test: change only a version declaration"
expect_guard_success "guard treated a version declaration as contract metadata semantics"

new_fixture downgrade
printf 'changed pair\n' >>"$case_dir/crates/atlas-index/src/artifact/pair.rs"
replace_version 'fixture-contract/v1' 'fixture-contract/v0'
replace_version 'fixture-manifest/v1' 'fixture-manifest/v0'
commit_case "test: downgrade versions"
expect_guard_failure "guard accepted version downgrades"

new_fixture skipped-version
printf 'changed records\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
replace_version 'fixture-contract/v1' 'fixture-contract/v3'
commit_case "test: skip contract version"
expect_guard_failure "guard accepted a non-sequential contract version"

new_fixture corrected-owner
printf 'changed records\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: bump contract version"
expect_guard_success "guard rejected the exact next contract version"

new_fixture sequential-history
printf 'first owner change\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
commit_case "test: change first contract owner"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: advance contract to version two"
printf 'second owner change\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
commit_case "test: change second contract owner"
replace_version 'fixture-contract/v2' 'fixture-contract/v3'
commit_case "test: advance contract to version three"
expect_guard_success "guard rejected sequential accumulated contract history"

new_fixture sequential-skip
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: advance contract to version two"
replace_version 'fixture-contract/v2' 'fixture-contract/v4'
commit_case "test: skip contract version three"
expect_guard_failure "guard accepted a skipped transition inside accumulated history"

new_fixture sequential-regression
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: advance contract to version two"
replace_version 'fixture-contract/v2' 'fixture-contract/v1'
commit_case "test: regress contract to version one"
expect_guard_failure "guard accepted a regression inside accumulated history"

new_fixture malformed-version
replace_version 'fixture-contract/v1' 'fixture-contract/vx'
commit_case "test: make contract declaration malformed"
expect_guard_failure "guard accepted a malformed version declaration"

new_fixture missing-version
sed -i.bak '/ARTIFACT_CONTRACT_VERSION/d' "$case_dir/crates/atlas-index/src/artifact/metadata.rs"
rm "$case_dir/crates/atlas-index/src/artifact/metadata.rs.bak"
commit_case "test: remove contract declaration"
expect_guard_failure "guard accepted a missing version declaration"

new_fixture explicit-candidate-head
printf 'candidate owner change\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: advance candidate contract"
candidate_head="$(git -C "$case_dir" rev-parse HEAD)"
git -C "$case_dir" switch -qc synthetic-merge "$case_base"
printf 'synthetic side\n' >>"$case_dir/README.md"
commit_case "test: add synthetic side commit"
git -C "$case_dir" merge -q --no-ff -m "test: synthetic pull request merge" "$candidate_head"
synthetic_head="$(git -C "$case_dir" rev-parse HEAD)"
expect_guard_success_head "synthetic checkout changed explicit candidate evaluation" "$candidate_head"
expect_guard_failure_head "guard silently linearized a nonlinear synthetic head" "$synthetic_head"

new_fixture multi-version
printf 'changed pair\n' >>"$case_dir/crates/atlas-index/src/artifact/pair.rs"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: bump only contract version"
expect_guard_failure "guard accepted an overlapping owner with only one required bump"
replace_version 'fixture-manifest/v1' 'fixture-manifest/v2'
commit_case "test: bump manifest version too"
expect_guard_success "guard rejected exact contract and manifest bumps"

new_fixture deleted-owner
rm "$case_dir/crates/atlas-ingest/src/source/normalize/content.rs"
commit_case "test: delete contract owner"
expect_guard_failure "guard missed a deleted contract owner"

new_fixture renamed-owner
mkdir -p "$case_dir/docs"
git -C "$case_dir" mv crates/atlas-index/src/artifact/canonical_json.rs docs/retired-canonical-json.rs
commit_case "test: rename contract owner"
expect_guard_failure "guard missed a renamed contract owner"

echo "Artifact version guard fixture passed."
