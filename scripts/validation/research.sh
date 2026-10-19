#!/bin/sh
# Each frozen dataset owns its verifier. No inference, downloads or source corpus.
set -eu
root=$(git rev-parse --show-toplevel)
found=0
for dataset in "$root"/docs/research/evaluations/*; do
  [ -d "$dataset" ] || continue
  if [ ! -f "$dataset/verify.ts" ]; then
    echo "Research dataset has no offline verifier: $dataset" >&2
    exit 1
  fi
  node --experimental-strip-types "$dataset/verify.ts"
  found=1
done
if [ "$found" -eq 0 ]; then echo 'No checked-in evaluation datasets.'; fi
