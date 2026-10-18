#!/bin/sh
# Check committed dependency notices without fetching packages or compiling Rust.
set -eu
cd "$(git rev-parse --show-toplevel)"

relevant_paths() {
  grep -E '(^|/)Cargo\.(toml|lock)$|^THIRD-PARTY-NOTICES\.md$|^dev-tools/release/|^\.cargo/|^rust-toolchain' >/dev/null
}

required=false
head=$(git rev-parse HEAD)
while read -r local_ref local_oid remote_ref remote_oid; do
  case "$local_oid" in *[!0]*) ;; *) continue ;; esac
  candidate=$(git rev-parse --verify "$local_oid^{commit}" 2>/dev/null) || continue
  if git cat-file -e "$remote_oid^{commit}" 2>/dev/null; then
    paths=$(git -c core.quotePath=true diff --no-renames --name-only "$remote_oid" "$candidate")
  else
    # New refs (or unavailable remote objects) conservatively check the tree.
    paths=$(git -c core.quotePath=true ls-tree -r --name-only "$candidate")
  fi
  if printf '%s\n' "$paths" | relevant_paths; then
    if [ "$candidate" != "$head" ]; then
      echo "Notices check: push $local_ref from a checkout at $candidate so its inputs can be checked." >&2
      exit 1
    fi
    required=true
  fi
done
[ "$required" = true ] || exit 0

# A locally regenerated file must not mask stale committed notices.
changed=$(git -c core.quotePath=true diff --no-renames --name-only HEAD)
untracked=$(git -c core.quotePath=true ls-files --others --exclude-standard)
if printf '%s\n%s\n' "$changed" "$untracked" | relevant_paths; then
  echo 'Notices check: commit or stash changes to dependency/notices inputs before pushing.' >&2
  exit 1
fi
for tool in node npm cargo; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Notices check requires $tool on PATH." >&2
    exit 1
  fi
done
if [ ! -f dev-tools/release/node_modules/typescript/bin/tsc ]; then
  echo 'Notices check requires release tooling dependencies. Run: npm --prefix dev-tools/release ci --ignore-scripts' >&2
  exit 1
fi
echo 'Checking third-party notices (offline)...' >&2
if ! CARGO_NET_OFFLINE=true npm --prefix dev-tools/release run notices -- --check; then
  echo 'Notices check failed. If dependencies are missing, fetch them explicitly; if notices are stale, run npm --prefix dev-tools/release run notices and commit the result.' >&2
  exit 1
fi
