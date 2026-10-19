#!/bin/sh
# Compile once, retain outside linked worktrees. Runtime commands never use target/.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
common=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)
destination="$(dirname -- "$common")/.atlas/dev-baselines/tools"
cd "$root"
cargo build --locked -p atlas-dev --target-dir "$root/target"
mkdir -p "$destination"
staged=$(mktemp "$destination/.atlas-dev.XXXXXX")
trap 'rm -f "$staged"' EXIT HUP INT TERM
cp "$root/target/debug/atlas-dev" "$staged"
chmod 755 "$staged"
mv -f "$staged" "$destination/atlas-dev"
echo "Retained atlas-dev: $destination/atlas-dev"
