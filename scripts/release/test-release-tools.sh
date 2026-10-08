#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d "${TMPDIR:-/tmp}/atlas-release-test.XXXXXX")
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT INT TERM

dist="$tmp/dist"
mkdir -p "$dist"

make_unix_archive_in() {
  dest="$1"
  target="$2"
  body="$3"
  work="$tmp/work/$dest/$target"
  mkdir -p "$work/$target"
  printf '%s\n' "$body" > "$work/$target/atlas"
  chmod +x "$work/$target/atlas"
  cp "$repo_root/LICENSE" "$work/$target/LICENSE"
  cp "$repo_root/README.md" "$work/$target/README.md"
  cp "$repo_root/THIRD-PARTY-NOTICES.md" "$work/$target/THIRD-PARTY-NOTICES.md"
  tar -cJf "$dest/atlas-cli-$target.tar.xz" -C "$work" "$target"
}

make_unix_archive() {
  target="$1"
  make_unix_archive_in "$dist" "$target" "#!/bin/sh
printf 'atlas 9.9.9\\n'"
}

make_windows_archive() {
  target="$1"
  archive="$dist/atlas-cli-$target.zip"
  node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" windows-archive "$archive" "$repo_root"
}

for target in \
  aarch64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  make_unix_archive "$target"
done

for target in \
  x86_64-pc-windows-msvc
do
  make_windows_archive "$target"
done

cp "$repo_root/scripts/install/atlas-installer.sh" "$dist/atlas-installer.sh"
cp "$repo_root/scripts/install/atlas-installer.ps1" "$dist/atlas-installer.ps1"
cp "$repo_root/THIRD-PARTY-NOTICES.md" "$dist/THIRD-PARTY-NOTICES.md"
printf '{}\n' > "$dist/dist-manifest.json"

node "$repo_root/dev-tools/release/dist/src/cli/generate-release-manifest.js" v9.9.9 "$dist"
node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" dist-manifest "$dist/dist-manifest.json"
node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$dist"
node "$repo_root/dev-tools/release/dist/src/cli/generate-notices.js" --check >/dev/null
grep -q 'ONNX Runtime: MIT' "$repo_root/THIRD-PARTY-NOTICES.md" || {
  echo "generated notices are missing ONNX Runtime coverage" >&2
  exit 1
}

bad_manifest_dist="$tmp/bad-manifest-dist"
cp -R "$dist" "$bad_manifest_dist"
node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" remove-target "$bad_manifest_dist/atlas-release-manifest.json"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_manifest_dist" >/dev/null 2>&1; then
  echo "asset validation accepted a manifest missing a required target" >&2
  exit 1
fi

bad_checksum_dist="$tmp/bad-checksum-dist"
cp -R "$dist" "$bad_checksum_dist"
sed 's/^[0-9a-f][0-9a-f]*/0000000000000000000000000000000000000000000000000000000000000000/' "$bad_checksum_dist/SHA256SUMS" > "$bad_checksum_dist/SHA256SUMS.new"
mv "$bad_checksum_dist/SHA256SUMS.new" "$bad_checksum_dist/SHA256SUMS"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_checksum_dist" >/dev/null 2>&1; then
  echo "asset validation accepted a checksum mismatch" >&2
  exit 1
fi

bad_manifest_checksum_dist="$tmp/bad-manifest-checksum-dist"
cp -R "$dist" "$bad_manifest_checksum_dist"
node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" bad-manifest-checksum "$bad_manifest_checksum_dist/atlas-release-manifest.json"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_manifest_checksum_dist" >/dev/null 2>&1; then
  echo "asset validation accepted a manifest checksum mismatch" >&2
  exit 1
fi

bad_dist_manifest_dist="$tmp/bad-dist-manifest-dist"
cp -R "$dist" "$bad_dist_manifest_dist"
node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" missing-artifact "$bad_dist_manifest_dist/dist-manifest.json"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_dist_manifest_dist" >/dev/null 2>&1; then
  echo "asset validation accepted a dist manifest with a missing artifact" >&2
  exit 1
fi

case "$(uname -s):$(uname -m)" in
  Darwin:arm64) host_target="aarch64-apple-darwin" ;;
  Darwin:x86_64) host_target="" ;;
  Linux:x86_64|Linux:amd64) host_target="x86_64-unknown-linux-gnu" ;;
  Linux:aarch64|Linux:arm64) host_target="aarch64-unknown-linux-gnu" ;;
  *) host_target="" ;;
esac

if [ -n "$host_target" ]; then
  fake_bin="$tmp/bin"
  install_dir="$tmp/install"
  mkdir -p "$fake_bin" "$install_dir"
  cat > "$fake_bin/curl" <<'EOF_CURL'
#!/bin/sh
dest=
url=
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      dest="$2"
      shift 2
      ;;
    http://*|https://*)
      url="$1"
      shift
      ;;
    *)
      shift
      ;;
  esac
done
[ -n "$dest" ] || exit 2
[ -n "$url" ] || exit 2
cp "$ATLAS_FAKE_RELEASE_DIR/$(basename "$url")" "$dest"
EOF_CURL
  chmod +x "$fake_bin/curl"

  dry_run_dir="$tmp/dry-run-install"
  PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$dry_run_dir" --version v9.9.9 --dry-run >/dev/null
  [ ! -e "$dry_run_dir/atlas" ] || {
    echo "installer dry-run wrote an atlas binary" >&2
    exit 1
  }

  first_output=$(PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$install_dir" --version v9.9.9 --yes)
  printf '%s\n' "$first_output" | grep -q 'is not on PATH' || {
    echo "installer did not print PATH guidance" >&2
    exit 1
  }
  installed_version=$("$install_dir/atlas" --version)
  [ "$installed_version" = "atlas 9.9.9" ] || {
    echo "unexpected installed version: $installed_version" >&2
    exit 1
  }

  PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$install_dir" --version v9.9.9 --yes >/dev/null

  if PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$install_dir" --version v9.9.9 </dev/null >/dev/null 2>&1
  then
    echo "installer replaced existing binary without confirmation" >&2
    exit 1
  fi

  rollback_dist="$tmp/rollback-dist"
  cp -R "$dist" "$rollback_dist"
  make_unix_archive_in "$rollback_dist" "$host_target" "#!/bin/sh
exit 42"
  node "$repo_root/dev-tools/release/dist/src/cli/generate-release-manifest.js" v9.9.9 "$rollback_dist" >/dev/null
  if PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$rollback_dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$install_dir" --version v9.9.9 --yes >/dev/null 2>&1
  then
    echo "installer accepted a binary that failed version validation" >&2
    exit 1
  fi
  installed_version=$("$install_dir/atlas" --version)
  [ "$installed_version" = "atlas 9.9.9" ] || {
    echo "installer did not restore previous binary after validation failure" >&2
    exit 1
  }

  bad_dist="$tmp/bad-dist"
  cp -R "$dist" "$bad_dist"
  first_sum=$(awk -v target="$host_target" '$2 ~ target { print $1; exit }' "$bad_dist/SHA256SUMS")
  [ -n "$first_sum" ] || {
    echo "missing checksum for $host_target" >&2
    exit 1
  }
  replacement="0000000000000000000000000000000000000000000000000000000000000000"
  sed "s/$first_sum/$replacement/" "$bad_dist/SHA256SUMS" > "$bad_dist/SHA256SUMS.new"
  mv "$bad_dist/SHA256SUMS.new" "$bad_dist/SHA256SUMS"
  if PATH="$fake_bin:$PATH" ATLAS_FAKE_RELEASE_DIR="$bad_dist" \
    "$repo_root/scripts/install/atlas-installer.sh" \
    --install-dir "$install_dir" --version v9.9.9 --yes >/dev/null 2>&1
  then
    echo "installer accepted a checksum mismatch" >&2
    exit 1
  fi
fi

bad_archive_dist="$tmp/bad-archive-dist"
cp -R "$dist" "$bad_archive_dist"
bad_work="$tmp/bad-work"
mkdir -p "$bad_work/atlas-cli-aarch64-apple-darwin/vendor/pf2e"
printf '{}\n' > "$bad_work/atlas-cli-aarch64-apple-darwin/vendor/pf2e/source.json"
tar -cJf "$bad_archive_dist/atlas-cli-aarch64-apple-darwin.tar.xz" -C "$bad_work" atlas-cli-aarch64-apple-darwin
node "$repo_root/dev-tools/release/dist/src/cli/generate-release-manifest.js" v9.9.9 "$bad_archive_dist"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_archive_dist" >/dev/null 2>&1; then
  echo "archive validation accepted bundled runtime data" >&2
  exit 1
fi

bad_zip_dist="$tmp/bad-zip-dist"
cp -R "$dist" "$bad_zip_dist"
node "$repo_root/dev-tools/release/dist/tests/release-fixtures.js" bad-zip "$bad_zip_dist/atlas-cli-x86_64-pc-windows-msvc.zip"
node "$repo_root/dev-tools/release/dist/src/cli/generate-release-manifest.js" v9.9.9 "$bad_zip_dist"
if node "$repo_root/dev-tools/release/dist/src/cli/validate-release-assets.js" v9.9.9 "$bad_zip_dist" >/dev/null 2>&1; then
  echo "archive validation accepted bundled runtime data in a zip" >&2
  exit 1
fi

printf 'release tool smoke tests passed\n'
