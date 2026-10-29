#!/bin/sh
# Build first; never stop a working preview when compilation fails.
set -eu
mode=${1:?Usage: preview.sh start|refresh CHECKOUT [server options]}
checkout=${2:?Usage: preview.sh start|refresh CHECKOUT [server options]}
shift 2
root=$(git -C "$checkout" rev-parse --show-toplevel)
case "$mode" in
  start | refresh) ;;
  *) echo "Expected start or refresh" >&2; exit 2 ;;
esac
node --experimental-strip-types "$root/scripts/development/build-preview.mjs" "$root"
if [ "$mode" = start ]; then
  set -- web --managed "$@"
else
  set -- refresh "$@"
fi
exec sh "$root/scripts/development/atlas-dev.sh" "$@" --checkout "$root" \
  --binary "$root/target/debug/atlas" \
  --frontend-index "$root/web/atlas-ui/dist/index.html" \
  --build-provenance "$root/.atlas/preview-build.json"
