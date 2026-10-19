#!/bin/sh
# The retained tool is deliberately refreshed by dev-tool-build, never by init.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
common=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)
tool=${ATLAS_DEV_BINARY:-"$(dirname -- "$common")/.atlas/dev-baselines/tools/atlas-dev"}
if [ ! -x "$tool" ]; then
  echo "No retained atlas-dev. Run just dev-tool-build once from the tooling checkout." >&2
  exit 1
fi
exec "$tool" baseline "$@"
