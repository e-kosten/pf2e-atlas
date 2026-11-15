#!/bin/sh
set -eu

desktop=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(dirname -- "$desktop")
target=aarch64-apple-darwin

case "$(uname -s):$(uname -m)" in
    Darwin:arm64) ;;
    *) printf '%s\n' 'The Atlas macOS bundle requires an Apple Silicon Mac.' >&2; exit 1 ;;
esac

unset CARGO_BUILD_TARGET
export CARGO_TARGET_DIR="$root/target"
cd "$root"
npm --prefix web/atlas-ui run build
node "$desktop/prepare-bootstrap.mjs"
cargo build --locked -p atlas-cli
mkdir -p "$desktop/src-tauri/binaries"
cp "$root/target/debug/atlas" "$desktop/src-tauri/binaries/atlas-$target"
cd "$desktop"
./node_modules/.bin/tauri build --debug --bundles app --no-sign -- --locked
