#!/bin/sh

set -eu

repo_root() {
  git rev-parse --show-toplevel
}

current_branch() {
  git symbolic-ref --quiet --short HEAD 2>/dev/null || true
}

absolute_git_dir() {
  cd "$(git rev-parse --git-dir)" && pwd -P
}

absolute_git_common_dir() {
  cd "$(git rev-parse --git-common-dir)" && pwd -P
}

is_zero_oid() {
  case "$1" in
    0000000000000000000000000000000000000000) return 0 ;;
    *) return 1 ;;
  esac
}

is_docs_only_path() {
  case "$1" in
    scratch/plans/* | *.md) return 0 ;;
    *) return 1 ;;
  esac
}

paths_are_docs_only() {
  saw_path=0

  while IFS= read -r path; do
    [ -n "$path" ] || continue
    saw_path=1
    if ! is_docs_only_path "$path"; then
      return 1
    fi
  done

  [ "$saw_path" -eq 1 ]
}

staged_changes_are_docs_only() {
  git diff --cached --name-only --relative --diff-filter=ACDMRTUXB | paths_are_docs_only
}

push_range_is_docs_only() {
  local_oid="$1"
  remote_oid="$2"

  if is_zero_oid "$local_oid" || is_zero_oid "$remote_oid"; then
    return 1
  fi

  git diff --name-only --relative "$remote_oid" "$local_oid" | paths_are_docs_only
}

require_linked_worktree() {
  git_dir="$(absolute_git_dir)"
  git_common_dir="$(absolute_git_common_dir)"

  if [ "$git_dir" = "$git_common_dir" ]; then
    echo "Refusing to proceed from the primary checkout." >&2
    echo "Create a dedicated linked worktree before committing." >&2
    return 1
  fi
}

require_non_main_branch() {
  branch="$(current_branch)"

  if [ "$branch" = "main" ]; then
    echo "Refusing to proceed on branch 'main'." >&2
    echo "Create a task branch in a linked worktree first." >&2
    return 1
  fi
}

validate_commit_message() {
  message_file="$1"

  subject="$(awk 'NF { print; exit }' "$message_file")"
  if [ -z "$subject" ]; then
    echo "Commit message must include a Conventional Commit subject line." >&2
    return 1
  fi

  if ! printf '%s\n' "$subject" | grep -Eq '^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([A-Za-z0-9._/-]+\))?!?: .+$'; then
    echo "Commit subject must use Conventional Commits." >&2
    echo "Example: feat(tags): add corruption profile retrieval tags" >&2
    return 1
  fi

  body_first_line="$(awk '
    BEGIN { saw_subject = 0 }
    /^[[:space:]]*#/ { next }
    !saw_subject && NF { saw_subject = 1; next }
    saw_subject && NF { print NR; exit }
  ' "$message_file")"

  if [ -n "$body_first_line" ]; then
    has_blank_separator="$(awk -v body_line="$body_first_line" '
      BEGIN { saw_subject = 0; saw_blank = 0 }
      NR >= body_line { print saw_blank; exit }
      /^[[:space:]]*#/ { next }
      !saw_subject && NF { saw_subject = 1; next }
      saw_subject && !NF { saw_blank = 1 }
    ' "$message_file")"

    if [ "$has_blank_separator" -ne 1 ]; then
      echo "Commit message bodies must start after a blank line." >&2
      return 1
    fi
  fi
}
