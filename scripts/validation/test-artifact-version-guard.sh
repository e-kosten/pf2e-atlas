#!/bin/sh

set -eu

repo_root="$(git rev-parse --show-toplevel)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/atlas-version-guard-fixture.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT HUP INT TERM

git -C "$fixture" init -q
git -C "$fixture" config user.name "Atlas Fixture"
git -C "$fixture" config user.email "atlas-fixture@example.invalid"
mkdir -p \
  "$fixture/scripts/validation" \
  "$fixture/crates/atlas-index/src/artifact" \
  "$fixture/crates/atlas-index/src/read" \
  "$fixture/crates/atlas-index/migrations"
cp "$repo_root/scripts/validation/check-artifact-version-bump.sh" "$fixture/scripts/validation/"
cp "$repo_root/scripts/validation/artifact-version-owners.txt" "$fixture/scripts/validation/"

printf '%s\n' \
  'pub const ARTIFACT_CONTRACT_VERSION: &str = "fixture-contract/v1";' \
  'pub const ARTIFACT_SCHEMA_VERSION: &str = "1";' \
  'pub const ARTIFACT_MANIFEST_VERSION: &str = "fixture-manifest/v1";' \
  >"$fixture/crates/atlas-index/src/artifact/metadata.rs"
printf 'base pair\n' >"$fixture/crates/atlas-index/src/artifact/pair.rs"
printf 'base schema\n' >"$fixture/crates/atlas-index/src/artifact/schema.rs"
git -C "$fixture" add .
git -C "$fixture" commit -qm "test: base fixture"
base="$(git -C "$fixture" rev-parse HEAD)"

printf 'changed pair\n' >"$fixture/crates/atlas-index/src/artifact/pair.rs"
git -C "$fixture" add .
git -C "$fixture" commit -qm "test: change envelope and contract owner"
if (cd "$fixture" && scripts/validation/check-artifact-version-bump.sh --base "$base") >/dev/null 2>&1; then
  echo "guard accepted owner changes without contract and manifest bumps" >&2
  exit 1
fi

sed -i.bak \
  -e 's/fixture-contract\/v1/fixture-contract\/v2/' \
  -e 's/fixture-manifest\/v1/fixture-manifest\/v2/' \
  "$fixture/crates/atlas-index/src/artifact/metadata.rs"
rm "$fixture/crates/atlas-index/src/artifact/metadata.rs.bak"
git -C "$fixture" add .
git -C "$fixture" commit -qm "test: bump overlapping versions"
(cd "$fixture" && scripts/validation/check-artifact-version-bump.sh --base "$base") >/dev/null

printf 'changed schema\n' >"$fixture/crates/atlas-index/src/artifact/schema.rs"
git -C "$fixture" add .
git -C "$fixture" commit -qm "test: change schema owner"
if (cd "$fixture" && scripts/validation/check-artifact-version-bump.sh --base "$base") >/dev/null 2>&1; then
  echo "guard accepted schema owner changes without a schema bump" >&2
  exit 1
fi

sed -i.bak 's/ARTIFACT_SCHEMA_VERSION: &str = "1"/ARTIFACT_SCHEMA_VERSION: \&str = "2"/' \
  "$fixture/crates/atlas-index/src/artifact/metadata.rs"
rm "$fixture/crates/atlas-index/src/artifact/metadata.rs.bak"
git -C "$fixture" add .
git -C "$fixture" commit -qm "test: bump schema version"
(cd "$fixture" && scripts/validation/check-artifact-version-bump.sh --base "$base") >/dev/null

echo "Artifact version guard fixture passed."
