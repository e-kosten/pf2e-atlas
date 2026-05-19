#!/bin/sh

set -eu

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/git-hooks/common.sh"

fail() {
  echo "$1" >&2
  exit 1
}

require_clean_worktree() {
  if ! git diff --quiet || ! git diff --cached --quiet; then
    fail "Refusing to land with uncommitted changes in $(repo_root)."
  fi
}

find_main_worktree() {
  git worktree list --porcelain | awk '
    /^worktree / { path = substr($0, 10) }
    /^branch refs\/heads\/main$/ { print path; exit }
  '
}

branch="$(current_branch)"
require_linked_worktree
require_non_main_branch
require_clean_worktree

main_worktree="$(find_main_worktree)"
if [ -z "$main_worktree" ]; then
  fail "Could not find a linked worktree checked out on branch main."
fi

if [ ! -d "$main_worktree" ]; then
  fail "Main worktree path does not exist: $main_worktree"
fi

if ! git -C "$main_worktree" diff --quiet || ! git -C "$main_worktree" diff --cached --quiet; then
  fail "Refusing to land because main has uncommitted changes: $main_worktree"
fi

echo "Rebasing $branch onto main..." >&2
git rebase main

echo "Running required landing checks in worktree before merge..." >&2
run_required_verification

echo "Fast-forwarding main to $branch..." >&2
git -C "$main_worktree" merge --ff-only "$branch"

echo "Running required landing checks on main after merge..." >&2
(
  cd "$main_worktree"
  run_required_verification
)

echo "Landed $branch on main with verified Rust fmt, clippy, tests, and build." >&2
