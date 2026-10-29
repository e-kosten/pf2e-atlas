#!/bin/sh

set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/git-hooks/common.sh"

require_linked_worktree
require_non_main_branch

if [ "$#" -gt 0 ]; then
  exec node "$SCRIPT_DIR/development/preflight.mjs" "$@" "$(repo_root)"
fi

echo "Preflight OK: linked worktree on branch '$(current_branch)'." >&2

