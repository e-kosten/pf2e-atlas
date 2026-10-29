#!/bin/sh
# Build first; never stop a working preview when compilation fails.
set -eu
mode=${1:?Usage: preview.sh start|refresh CHECKOUT [--profile dev|release] [server options]}
checkout=${2:?Usage: preview.sh start|refresh CHECKOUT [--profile dev|release] [server options]}
shift 2
root=$(git -C "$checkout" rev-parse --show-toplevel)
case "$mode" in
  start | refresh) ;;
  *) echo "Expected start or refresh" >&2; exit 2 ;;
esac
profile=dev
case "${1-}" in
  --profile)
    [ "$#" -ge 2 ] || { echo 'Expected --profile dev or release.' >&2; exit 2; }
    profile=$2
    shift 2
    ;;
  --profile=*) profile=${1#--profile=}; shift ;;
esac
case "${1-}" in
  --profile | --profile=*) echo 'Specify --profile once before server options.' >&2; exit 2 ;;
esac
layout=$(node "$root/scripts/development/build-profile.mjs" "$root" "$profile")
IFS=' ' read -r binary provenance <<EOF
$layout
EOF
node "$root/scripts/development/preflight.mjs" --preview "$root"
node --experimental-strip-types "$root/scripts/development/build-preview.mjs" "$root" --profile "$profile"
if [ "$mode" = start ]; then
  set -- web --managed "$@"
else
  set -- refresh "$@"
fi
exec sh "$root/scripts/development/atlas-dev.sh" "$@" --checkout "$root" \
  --binary "$root/$binary" \
  --frontend-index "$root/web/atlas-ui/dist/index.html" \
  --build-provenance "$root/$provenance"
