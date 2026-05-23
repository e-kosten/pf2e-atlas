#!/usr/bin/env bash
set -euo pipefail

# Use Ladybug's source-build path for the spike. The default prebuilt archive
# path failed to link cleanly on the first macOS arm64 probe.
export LBUG_RUST_BUILD_FROM_SOURCE="${LBUG_RUST_BUILD_FROM_SOURCE:-1}"

cargo run -p atlas-ladybug-spike -- "$@"
