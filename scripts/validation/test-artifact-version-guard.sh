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
  mkdir -p \
    "$case_dir/scripts/validation" \
    "$case_dir/crates/atlas-index/src/artifact" \
    "$case_dir/crates/atlas-index/src/read/records" \
    "$case_dir/crates/atlas-index/src/write/sqlite" \
    "$case_dir/crates/atlas-index/migrations/0001_fixture" \
    "$case_dir/crates/atlas-ingest/src/source/normalize" \
    "$case_dir/crates/atlas-ingest/src"
  cp "$repo_root/scripts/validation/check-artifact-version-bump.sh" "$case_dir/scripts/validation/"
  cp "$repo_root/scripts/validation/artifact-version-owners.txt" "$case_dir/scripts/validation/"
  printf '%s\n' \
    'pub const ARTIFACT_CONTRACT_VERSION: &str = "fixture-contract/v1";' \
    'pub const ARTIFACT_SCHEMA_VERSION: &str = "1";' \
    'pub const ARTIFACT_MANIFEST_VERSION: &str = "fixture-manifest/v1";' \
    >"$case_dir/crates/atlas-index/src/artifact/metadata.rs"
  for path in \
    crates/atlas-index/src/artifact/canonical_json.rs \
    crates/atlas-index/src/artifact/pair.rs \
    crates/atlas-index/src/write/sqlite/records.rs \
    crates/atlas-index/migrations/0001_fixture/up.sql \
    crates/atlas-ingest/src/source/normalize/content.rs \
    crates/atlas-ingest/src/artifact_manifest.rs
  do
    printf 'base\n' >"$case_dir/$path"
  done
  printf 'unrelated\n' >"$case_dir/README.md"
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

for owner in \
  crates/atlas-ingest/src/source/normalize/content.rs \
  crates/atlas-index/src/write/sqlite/records.rs \
  crates/atlas-index/src/artifact/metadata.rs
do
  new_fixture "missed-owner-$(basename "$owner" .rs)"
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

new_fixture unrelated
printf 'changed unrelated\n' >>"$case_dir/README.md"
commit_case "test: change unrelated path"
expect_guard_success "guard rejected an unrelated path"

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
