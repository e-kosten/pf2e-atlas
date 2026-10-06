#!/bin/sh

set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 SOURCE_REPOSITORY EXPECTED_COMMIT EXPECTED_TREE" >&2
  exit 2
fi

source_repository="$1"
expected_commit="$2"
expected_tree="$3"

if [ -L "$source_repository" ] || [ ! -d "$source_repository" ]; then
  echo "PF2E source must be a direct directory: $source_repository" >&2
  exit 1
fi

physical_source="$(cd "$source_repository" && pwd -P)"
git_root="$(git -C "$physical_source" rev-parse --show-toplevel)"
physical_git_root="$(cd "$git_root" && pwd -P)"
if [ "$physical_git_root" != "$physical_source" ]; then
  echo "PF2E source Git root does not match the configured checkout" >&2
  exit 1
fi

actual_commit="$(git -C "$physical_source" rev-parse HEAD)"
actual_tree="$(git -C "$physical_source" rev-parse 'HEAD^{tree}')"
if [ "$actual_commit" != "$expected_commit" ]; then
  echo "PF2E source commit mismatch: expected $expected_commit, got $actual_commit" >&2
  exit 1
fi
if [ "$actual_tree" != "$expected_tree" ]; then
  echo "PF2E source tree mismatch: expected $expected_tree, got $actual_tree" >&2
  exit 1
fi
if [ -n "$(git -C "$physical_source" status --porcelain --untracked-files=no)" ]; then
  echo "PF2E source checkout has modified tracked files" >&2
  exit 1
fi

printf 'Authenticated PF2E source %s tree %s\n' "$actual_commit" "$actual_tree"
