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

# Keep the path policy shared by commit hooks and changed-path verification.
validation_surface_for_path() {
  case "$1" in
    scripts/development/*) printf 'development\n' ;;
    # Package fixtures and executable research precede the prose exemption.
    dev-tools/source-contracts/*) printf 'source\n' ;;
    dev-tools/release/* | scripts/release/* | scripts/prepare-release.sh | scripts/install/* | THIRD-PARTY-NOTICES.md) printf 'release\n' ;;
    skills/pf2e-atlas-cli/*) printf 'skill\n' ;;
    docs/research/evaluations/*) printf 'research\n' ;;
    AGENTS.md | CONTRIBUTING.md | README.md | docs/*.md | scratch/plans/*.md | .codex/skills/*.md | skills/*.md | crates/*/README.md | dev-tools/README.md | vendor/README.md | web/atlas-ui/AGENTS.md | web/atlas-ui/README.md | web/atlas-ui/docs/*.md)
      printf 'docs\n' ;;
    Cargo.toml | Cargo.lock | crates/*/Cargo.toml) printf 'rust web source release\n' ;;
    crates/atlas-app-model/*) printf 'rust web\n' ;;
    crates/atlas-foundry-model/*) printf 'rust source\n' ;;
    crates/* | src/* | tests/* | build.rs) printf 'rust\n' ;;
    web/atlas-ui/*) printf 'web\n' ;;
    *) printf 'rust web source release research skill development\n' ;;
  esac
}

classify_validation_paths() {
  rust_touched=0
  web_touched=0
  source_touched=0
  release_touched=0
  research_touched=0
  skill_touched=0
  development_touched=0
  docs_only=0
  classification_saw_path=0
  while IFS= read -r path || [ -n "$path" ]; do
    [ -n "$path" ] || continue
    classification_saw_path=1
    for surface in $(validation_surface_for_path "$path"); do
      case "$surface" in
        rust) rust_touched=1 ;;
        web) web_touched=1 ;;
        source) source_touched=1 ;;
        release) release_touched=1 ;;
        research) research_touched=1 ;;
        skill) skill_touched=1 ;;
        development) development_touched=1 ;;
      esac
    done
  done <"$1"
  if [ "$classification_saw_path" -eq 1 ] && [ "$rust_touched$web_touched$source_touched$release_touched$research_touched$skill_touched$development_touched" = 0000000 ]; then
    docs_only=1
  fi
}

collect_changed_paths() {
  collected_paths="$1"
  shift
  # Run from the root, include deleted rename sources, and check Git failures.
  # Quoted unusual filenames conservatively fall into the unknown-path gate.
  if ! git -C "$(repo_root)" -c core.quotePath=true diff --no-renames --name-only "$@" >"$collected_paths"; then
    return 1
  fi
  if ! git -C "$(repo_root)" diff --no-renames --raw "$@" >"$collected_paths.modes"; then
    return 1
  fi
  # Executable files, symlinks and submodules cannot use a documentation exemption.
  awk '($1 != ":000000" && $1 != ":100644") || ($2 != "000000" && $2 != "100644") { print "__atlas_special_file_mode__" }' \
    "$collected_paths.modes" >>"$collected_paths"
}

staged_changes_are_docs_only() {
  staged_paths_dir="$(mktemp -d "${TMPDIR:-/tmp}/atlas-staged-paths.XXXXXX")"
  if ! collect_changed_paths "$staged_paths_dir/paths" --cached --diff-filter=ACDMRTUXB; then
    rm -rf "$staged_paths_dir"
    return 1
  fi
  classify_validation_paths "$staged_paths_dir/paths"
  rm -rf "$staged_paths_dir"
  [ "$docs_only" -eq 1 ]
}

run_required_verification() {
  "$(repo_root)/scripts/verify-changed.sh" --staged
}

require_linked_worktree() {
  git_dir="$(absolute_git_dir)"
  git_common_dir="$(absolute_git_common_dir)"

  if [ "$git_dir" = "$git_common_dir" ]; then
    echo "Refusing to proceed from the primary checkout." >&2
    echo "Use a dedicated linked worktree for this worktree-specific command." >&2
    return 1
  fi
}

require_non_main_branch() {
  branch="$(current_branch)"

  if [ "$branch" = "main" ]; then
    echo "Refusing to proceed on branch 'main'." >&2
    echo "Use a non-main branch for this worktree-specific command." >&2
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
