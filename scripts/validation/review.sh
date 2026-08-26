#!/bin/sh

set -eu

source_root=
candidate_head=
snapshot_root=
review_root=
report=
embedding_cache=${PF2E_EMBEDDING_CACHE_ROOT:-}
while [ "$#" -gt 0 ]; do
  case "$1" in
    --source) source_root=${2:-}; shift 2 ;;
    --candidate-head) candidate_head=${2:-}; shift 2 ;;
    --snapshot-root) snapshot_root=${2:-}; shift 2 ;;
    --review-root) review_root=${2:-}; shift 2 ;;
    --report) report=${2:-}; shift 2 ;;
    --embedding-cache-path) embedding_cache=${2:-}; shift 2 ;;
    --force-reproduce | --force-reproduction | --refuse-author-snapshots) shift ;;
    *) echo "Unknown validate-exhaustive-review option: $1" >&2; exit 2 ;;
  esac
done

if [ -n "$review_root" ]; then
  if [ -n "$snapshot_root" ]; then
    echo "Pass only one of --review-root or --snapshot-root" >&2
    exit 2
  fi
  snapshot_root=$review_root/reviewer-snapshot
fi

exec scripts/validation/exhaustive.sh \
  --force-reproduction \
  --source "$source_root" \
  --candidate-head "$candidate_head" \
  --snapshot-root "$snapshot_root" \
  --report "$report" \
  --embedding-cache-path "$embedding_cache"
