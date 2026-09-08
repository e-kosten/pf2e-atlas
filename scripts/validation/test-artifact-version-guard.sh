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
  cp "$repo_root/scripts/validation/verify-artifact-owner-transfer.py" "$case_dir/scripts/validation/"
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
    crates/atlas-index/src/artifact/pair_manifest.rs \
    crates/atlas-index/src/artifact/publication.rs \
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

new_transfer_fixture() {
  name="$1"
  new_fixture "$name"
  rm "$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
  cat >"$case_dir/crates/atlas-index/src/artifact/pair.rs" <<'EOF'
const MANIFEST_VERSION: &str = "v3";

fn read_manifest() -> &'static str {
    MANIFEST_VERSION
}

fn lifecycle() {}
EOF
  sed -i.bak \
    's#manifest|crates/atlas-index/src/artifact/pair_manifest.rs#manifest|crates/atlas-index/src/artifact/pair.rs#' \
    "$case_dir/scripts/validation/artifact-version-owners.txt"
  rm "$case_dir/scripts/validation/artifact-version-owners.txt.bak"
  git -C "$case_dir" add -A
  git -C "$case_dir" commit -qm "test: establish mixed manifest owner"
  case_base="$(git -C "$case_dir" rev-parse HEAD)"
}

write_transfer_files() {
  mkdir -p "$case_dir/scripts/validation/artifact-owner-transfers"
  cat >"$case_dir/scripts/validation/artifact-owner-transfers.txt" <<'EOF'
manifest|crates/atlas-index/src/artifact/pair.rs|crates/atlas-index/src/artifact/pair_manifest.rs|scripts/validation/artifact-owner-transfers/pair.items
EOF
  cat >"$case_dir/scripts/validation/artifact-owner-transfers/pair.items" <<'EOF'
const MANIFEST_VERSION
fn read_manifest
EOF
  cat >"$case_dir/crates/atlas-index/src/artifact/pair.rs" <<'EOF'
fn lifecycle() {}
EOF
cat >"$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs" <<'EOF'
pub(super) const MANIFEST_VERSION: &str = "v3";

pub(super) fn read_manifest() -> &'static str {
    MANIFEST_VERSION
}
EOF
  sed -i.bak \
    '\#^[^|]*|crates/atlas-index/src/artifact/pair.rs$#d' \
    "$case_dir/scripts/validation/artifact-version-owners.txt"
  rm "$case_dir/scripts/validation/artifact-version-owners.txt.bak"
  printf 'manifest|crates/atlas-index/src/artifact/pair_manifest.rs\n' \
    >>"$case_dir/scripts/validation/artifact-version-owners.txt"
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
printf 'changed manifest parser\n' >>"$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
commit_case "test: change manifest owner"
expect_guard_failure "guard accepted a manifest owner without a manifest bump"
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: bump only contract version for manifest change"
expect_guard_failure "guard accepted a contract bump for a manifest-only change"
replace_version 'fixture-manifest/v1' 'fixture-manifest/v2'
commit_case "test: bump manifest version for manifest change"
expect_guard_success "guard rejected the exact manifest bump after a manifest change"

new_transfer_fixture verified-owner-transfer
write_transfer_files
commit_case "test: transfer unchanged manifest owner"
expect_guard_success "guard rejected a mechanically identical owner transfer"
printf '\nfn changed_manifest_semantics() {}\n' \
  >>"$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
commit_case "test: change transferred manifest owner"
expect_guard_failure "guard treated a post-transfer manifest mutation as version-neutral"

new_transfer_fixture changed-owner-transfer
write_transfer_files
sed -i.bak 's/MANIFEST_VERSION$/"changed"/' \
  "$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
rm "$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs.bak"
commit_case "test: change semantics during owner transfer"
expect_guard_failure "guard accepted changed semantics during owner transfer"

new_transfer_fixture retained-source-transfer
write_transfer_files
cat >>"$case_dir/crates/atlas-index/src/artifact/pair.rs" <<'EOF'
const MANIFEST_VERSION: &str = "v3";

fn read_manifest() -> &'static str {
    MANIFEST_VERSION
}
EOF
commit_case "test: retain manifest semantics in prior owner"
expect_guard_failure "guard accepted duplicated semantics after owner transfer"

new_transfer_fixture extra-target-item-transfer
write_transfer_files
printf '\nfn undeclared_manifest_semantics() {}\n' \
  >>"$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
commit_case "test: add undeclared target semantics"
expect_guard_failure "guard accepted undeclared target semantics during transfer"

new_transfer_fixture wrong-class-transfer
sed -i.bak \
  's#manifest|crates/atlas-index/src/artifact/pair.rs#contract|crates/atlas-index/src/artifact/pair.rs#' \
  "$case_dir/scripts/validation/artifact-version-owners.txt"
rm "$case_dir/scripts/validation/artifact-version-owners.txt.bak"
git -C "$case_dir" add -A
git -C "$case_dir" commit -qm "test: register prior owner in wrong class"
case_base="$(git -C "$case_dir" rev-parse HEAD)"
write_transfer_files
commit_case "test: transfer from wrong owner class"
expect_guard_failure "guard accepted transfer from a wrong-class owner"

new_transfer_fixture missing-source-transfer
write_transfer_files
rm "$case_dir/crates/atlas-index/src/artifact/pair.rs"
commit_case "test: delete prior owner during transfer"
expect_guard_failure "guard accepted transfer with a missing prior owner"

new_transfer_fixture duplicate-transfer-declaration
write_transfer_files
cat >>"$case_dir/scripts/validation/artifact-owner-transfers.txt" <<'EOF'
manifest|crates/atlas-index/src/artifact/pair.rs|crates/atlas-index/src/artifact/pair_manifest.rs|scripts/validation/artifact-owner-transfers/pair.items
EOF
commit_case "test: duplicate owner transfer declaration"
expect_guard_failure "guard accepted duplicate owner transfer declarations"

new_transfer_fixture renamed-target-after-transfer
write_transfer_files
commit_case "test: transfer unchanged manifest owner"
git -C "$case_dir" mv \
  crates/atlas-index/src/artifact/pair_manifest.rs \
  crates/atlas-index/src/artifact/renamed_manifest.rs
sed -i.bak \
  's#artifact/pair_manifest.rs#artifact/renamed_manifest.rs#' \
  "$case_dir/scripts/validation/artifact-version-owners.txt"
rm "$case_dir/scripts/validation/artifact-version-owners.txt.bak"
commit_case "test: rename transferred owner"
expect_guard_failure "guard accepted a later transferred-owner rename"

new_transfer_fixture deleted-target-after-transfer
write_transfer_files
commit_case "test: transfer unchanged manifest owner"
rm "$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
commit_case "test: delete transferred owner"
expect_guard_failure "guard accepted a later transferred-owner deletion"

new_fixture deleted-manifest-owner
rm "$case_dir/crates/atlas-index/src/artifact/pair_manifest.rs"
commit_case "test: delete manifest owner"
expect_guard_failure "guard missed a deleted manifest owner"

new_fixture lifecycle-non-owner
printf 'changed generation lifecycle\n' >>"$case_dir/crates/atlas-index/src/artifact/pair.rs"
printf 'changed publication lifecycle\n' >>"$case_dir/crates/atlas-index/src/artifact/publication.rs"
commit_case "test: change version-neutral artifact lifecycle"
expect_guard_success "guard treated internal artifact lifecycle as a persisted contract"

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
printf 'changed final package\n' >>"$case_dir/crates/atlas-ingest/src/build.rs"
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

new_fixture owner-after-last-bump
replace_version 'fixture-contract/v1' 'fixture-contract/v2'
commit_case "test: advance contract to version two"
printf 'uncovered owner change\n' >>"$case_dir/crates/atlas-index/src/write/sqlite/records.rs"
commit_case "test: change owner after the final bump"
expect_guard_failure "guard accepted an owner change after the last qualifying bump"

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
printf 'changed final package\n' >>"$case_dir/crates/atlas-ingest/src/build.rs"
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
