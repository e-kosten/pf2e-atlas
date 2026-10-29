#!/bin/sh
set -eu
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
exec sh "$script_dir/validation/launch.sh" "$script_dir/validation/verify.mjs" --rust "$@"
