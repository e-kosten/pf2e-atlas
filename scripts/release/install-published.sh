#!/usr/bin/env sh
set -eu
case "${1:-}" in
  release/*|rc/*|v[0-9]*|[0-9]*)
    version="$1"
    shift
    ;;
  *)
    version=$(awk -F '"' '/^version = / { print $2; exit }' crates/atlas-cli/Cargo.toml)
    ;;
esac
case "$version" in
  release/*|rc/*) tag="$version" ;;
  *-rc.*) tag="rc/v${version#v}" ;;
  *) tag="release/v${version#v}" ;;
esac
installer=$(mktemp)
trap 'rm -f "$installer"' EXIT
trap 'exit 1' HUP INT TERM
curl --proto '=https' --tlsv1.2 -LsSf \
  "https://github.com/e-kosten/pf2e-atlas/releases/download/$tag/atlas-installer.sh" -o "$installer"
if [ ! -s "$installer" ]; then
  echo "Downloaded Atlas installer is empty" >&2
  exit 1
fi
sh "$installer" --version "$tag" "$@"
