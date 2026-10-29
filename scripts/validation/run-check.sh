#!/bin/sh
set -eu
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
exec sh "$script_dir/launch.sh" "$script_dir/run-check.mjs" "$@"
