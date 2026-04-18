#!/bin/sh

set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/git-hooks/common.sh"

require_linked_worktree
require_non_main_branch

echo "Preflight OK: linked worktree on branch '$(current_branch)'." >&2

