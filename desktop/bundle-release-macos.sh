#!/bin/sh
set -eu

desktop=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(dirname -- "$desktop")
[ "$#" -eq 4 ] || { echo "Usage: $0 TAG RUN_ID FULL_CLI_ARCHIVE OUTPUT_DMG" >&2; exit 2; }
[ "$(uname -s):$(uname -m)" = Darwin:arm64 ] || { echo 'Apple Silicon macOS required' >&2; exit 1; }

tag=$1
run_id=$2
archive=$(realpath "$3")
output="$(realpath "$(dirname "$4")")/$(basename "$4")"
work=$(mktemp -d)
mount="$work/mount"
cleanup() {
    if [ -d "$mount" ]; then hdiutil detach "$mount" >/dev/null 2>&1 || true; fi
    rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 1' INT TERM

cd "$root"
commit=$(git rev-parse HEAD)
node "$desktop/release-config.mjs" "$tag" "$commit" "$run_id" "$work"
tar -xJf "$archive" -C "$work"
find "$work" -type f -name atlas > "$work/executables"
[ "$(wc -l < "$work/executables" | tr -d ' ')" = 1 ]
cli=$(cat "$work/executables")
sidecar="$desktop/src-tauri/binaries/atlas-aarch64-apple-darwin"
mkdir -p "$(dirname "$sidecar")"
cp "$cli" "$sidecar"
cmp "$cli" "$sidecar"
node "$root/dev-tools/release/dist/src/cli/smoke-packaged-cli.js" "$sidecar" "${tag#v}" full

# Generate notices for the independent desktop workspace as well as the CLI.
node "$root/dev-tools/release/dist/src/cli/generate-notices.js" \
    --output "$desktop/.generated/THIRD-PARTY-NOTICES.md" --desktop
node "$desktop/prepare-bootstrap.mjs"
unset CARGO_BUILD_TARGET
export CARGO_TARGET_DIR="$root/target"
cd "$desktop"
./node_modules/.bin/tauri build --target aarch64-apple-darwin --bundles app \
    --config "$work/config.json" -- --locked

# Create the drag-to-Applications layout without needing a Finder session.
mkdir "$work/image"
ditto "$CARGO_TARGET_DIR/aarch64-apple-darwin/release/bundle/macos/PF2e Atlas.app" \
    "$work/image/PF2e Atlas.app"
ln -s /Applications "$work/image/Applications"
image="$work/atlas.dmg"
hdiutil create -srcfolder "$work/image" -volname 'PF2e Atlas' -format UDZO "$image"
hdiutil verify "$image" >/dev/null
mkdir "$mount"
hdiutil attach "$image" -readonly -nobrowse -mountpoint "$mount" >/dev/null
app="$mount/PF2e Atlas.app"
bundled="$app/Contents/MacOS/atlas"
host="$app/Contents/MacOS/atlas-desktop"
plist="$app/Contents/Info.plist"
[ "$(readlink "$mount/Applications")" = /Applications ]
[ "$(lipo -archs "$host")" = arm64 ]
[ "$(lipo -archs "$bundled")" = arm64 ]
codesign --verify --deep --strict "$app"
codesign --verify --strict "$bundled"
for executable in "$host" "$bundled"; do
    codesign -dvv "$executable" 2>&1 | grep -Fx 'Signature=adhoc'
    # Reject links back into the build machine or to libraries outside the app/system.
    otool -L "$executable" | awk 'NR > 1 && $1 !~ /^(\/usr\/lib\/|\/System\/Library\/)/ { print; bad=1 } END { exit bad }'
done
node "$desktop/verify-release-macos.mjs" "$plist" "$work/config.json" "$host" "$bundled"
test -s "$app/Contents/Resources/THIRD-PARTY-NOTICES.md"
unexpected=$(find "$app" \( -name '*.sqlite*' -o -name '*.onnx' -o -name hf-models -o -path '*/vendor/pf2e' \) -print)
[ -z "$unexpected" ] || { printf 'Unexpected release data:\n%s\n' "$unexpected" >&2; exit 1; }

cp "$cli" "$work/archived-cli"
cp "$bundled" "$work/bundled-cli"
codesign --remove-signature "$work/archived-cli"
codesign --remove-signature "$work/bundled-cli"
node "$desktop/compare-release-cli.mjs" "$work/archived-cli" "$work/bundled-cli"
node "$root/dev-tools/release/dist/src/cli/smoke-packaged-cli.js" "$bundled" "${tag#v}" full
mkdir "$work/home"
HOME="$work/home" "$bundled" desktop-probe | jq -e '.data.version == 1 and .data.status == "needs_setup"'
hdiutil detach "$mount" >/dev/null

cp "$image" "$output"
archive_sha=$(shasum -a 256 "$archive" | cut -d ' ' -f 1)
image_sha=$(shasum -a 256 "$image" | cut -d ' ' -f 1)
jq -n --arg tag "$tag" --arg commit "$commit" --arg archive "$archive_sha" --arg dmg "$image_sha" \
    '{tag: $tag, commit: $commit, archive: $archive, dmg: $dmg}' > "$output.receipt.json"
