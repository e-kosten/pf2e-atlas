#!/bin/sh

set -eu

usage() {
  echo "Usage: $0 --source PATH --candidate-head SHA --snapshot-root PATH --report PATH [--embedding-cache-path PATH]" >&2
}

source_root=
candidate_head=
snapshot_root=
report=
embedding_cache=${PF2E_EMBEDDING_CACHE_ROOT:-}
force=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --source) source_root=${2:-}; shift 2 ;;
    --candidate-head) candidate_head=${2:-}; shift 2 ;;
    --snapshot-root) snapshot_root=${2:-}; shift 2 ;;
    --report) report=${2:-}; shift 2 ;;
    --embedding-cache-path) embedding_cache=${2:-}; shift 2 ;;
    --force-reproduction) force=--force-reproduction; shift ;;
    *) usage; exit 2 ;;
  esac
done

if [ -z "$source_root" ] || [ -z "$candidate_head" ] || [ -z "$snapshot_root" ] || [ -z "$report" ] || [ -z "$embedding_cache" ]; then
  usage
  echo "PF2E_EMBEDDING_CACHE_ROOT or --embedding-cache-path is required" >&2
  exit 2
fi

printf 'exhaustive phase=preflight status=started\n' >&2
cargo run -p atlas-cli -- --progress always index validate-corpus \
  --source "$source_root" \
  --candidate-head "$candidate_head" \
  --snapshot-root "$snapshot_root" \
  --report "$report" \
  --embedding-cache-path "$embedding_cache" \
  $force
printf 'exhaustive phase=complete status=passed\n' >&2
