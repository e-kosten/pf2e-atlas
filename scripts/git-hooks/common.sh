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
    .github/workflows/ci.yml | .github/workflows/macos-desktop.yml | scripts/git-hooks/common.sh | scripts/verify-changed.sh | scripts/validation/ci-results.mjs | scripts/validation/windows-selection.py | scripts/validation/embedding-stack.py | __atlas_special_file_mode__)
      printf 'rust web source release research skill browser corpus desktop dev tooling audit fanout\n' ;;
    desktop/README.md) printf 'docs\n' ;;
    desktop/bundle-release-macos.sh | desktop/compare-release-cli.mjs | desktop/verify-release-macos.mjs | desktop/smoke-macos.mjs) printf 'release\n' ;;
    desktop/release-config*) printf 'desktop release\n' ;;
    desktop/*) printf 'desktop\n' ;;
    .cargo/* | rust-toolchain.toml) printf 'rust web source release research skill browser corpus desktop dev tooling audit\n' ;;
    # Package fixtures and executable research precede the prose exemption.
    dev-tools/source-contracts/*) printf 'source\n' ;;
    dev-tools/release/* | scripts/release/* | scripts/prepare-release.sh | scripts/install/*) printf 'release\n' ;;
    skills/pf2e-atlas-cli/*) printf 'skill\n' ;;
    docs/research/evaluations/*) printf 'research\n' ;;
    scripts/validation/dependency-audit.py) printf 'audit tooling\n' ;;
    scripts/validation/skill*) printf 'skill tooling\n' ;;
    scripts/validation/research.sh) printf 'research tooling\n' ;;
    AGENTS.md | CONTRIBUTING.md | README.md | docs/*.md | scratch/plans/*.md | .codex/skills/*.md | skills/*.md | crates/*/README.md | dev-tools/README.md | vendor/README.md | web/atlas-ui/AGENTS.md | web/atlas-ui/README.md | web/atlas-ui/docs/*.md)
      printf 'docs\n' ;;
    Cargo.toml | Cargo.lock | crates/*/Cargo.toml) printf 'rust browser dev audit desktop\n' ;;
    crates/atlas-dev/*) printf 'dev\n' ;;
    scripts/development/* | scripts/validation/* | scripts/verify*.sh | scripts/preflight.sh | scripts/git-hooks/* | .githooks/*) printf 'tooling\n' ;;
    crates/atlas-cli/*) printf 'rust desktop\n' ;;
    crates/atlas-cli-support/*) printf 'rust dev desktop\n' ;;
    crates/atlas-app-model/* | crates/atlas-web/api-contract.json | crates/atlas-web/openapi.json | dev-tools/openapi/* | web/atlas-ui/src/generated/http.ts) printf 'rust web browser desktop\n' ;;
    crates/atlas-foundry-model/*) printf 'rust source browser desktop\n' ;;
    crates/* | src/* | tests/* | build.rs) printf 'rust browser desktop\n' ;;
    web/atlas-ui/e2e/readingUrl.ts) printf 'browser corpus web-format\n' ;;
    web/atlas-ui/e2e/devices/* | web/atlas-ui/playwright.devices.config.ts) printf 'browser web-format\n' ;;
    web/atlas-ui/e2e/* | web/atlas-ui/playwright.config.ts | web/atlas-ui/scripts/list-corpus-tests.mjs) printf 'corpus web-format\n' ;;
    web/atlas-ui/*.test.* | web/atlas-ui/*.spec.* | web/atlas-ui/vitest.config.* | web/atlas-ui/scripts/test/*) printf 'web browser\n' ;;
    scripts/browser-fixture*.mjs | scripts/validate-browser*.mjs) printf 'browser tooling\n' ;;
    web/atlas-ui/*) printf 'web browser desktop\n' ;;
    *) printf 'rust web source release research skill browser corpus desktop dev tooling audit\n' ;;
  esac
}

classify_validation_paths() {
  rust_touched=0
  web_touched=0
  source_touched=0
  release_touched=0
  research_touched=0
  skill_touched=0
  corpus_touched=0
  desktop_touched=0
  fanout_touched=0
  browser_touched=0
  web_format_touched=0
  dev_touched=0
  tooling_touched=0
  audit_touched=0
  docs_only=0
  classification_saw_path=0
  while IFS= read -r path || [ -n "$path" ]; do
    [ -n "$path" ] || continue
    classification_saw_path=1
    case "$path" in
      Cargo.toml | Cargo.lock | */Cargo.toml | */Cargo.lock | package.json | package-lock.json | */package.json | */package-lock.json) audit_touched=1 ;;
    esac
    for surface in $(validation_surface_for_path "$path"); do
      case "$surface" in
        rust) rust_touched=1 ;;
        web) web_touched=1 ;;
        source) source_touched=1 ;;
        release) release_touched=1 ;;
        research) research_touched=1 ;;
        skill) skill_touched=1 ;;
        corpus) corpus_touched=1 ;;
        desktop) desktop_touched=1 ;;
        fanout) fanout_touched=1 ;;
        browser) browser_touched=1 ;;
        web-format) web_format_touched=1 ;;
        dev) dev_touched=1 ;;
        tooling) tooling_touched=1 ;;
        audit) audit_touched=1 ;;
      esac
    done
  done <"$1"
  if [ "$classification_saw_path" -eq 1 ] && [ "$rust_touched$web_touched$source_touched$release_touched$research_touched$skill_touched$browser_touched$dev_touched$web_format_touched$tooling_touched$audit_touched$corpus_touched$desktop_touched$fanout_touched" = 00000000000000 ]; then
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
  awk -F '\t' '
    {
      split($1, modes, " ")
      old = substr(modes[1], 2)
      new = modes[2]
      if (old == "100755" && new == old &&
          $2 ~ /^(scripts\/|\.githooks\/|desktop\/)/ && $2 !~ /\.md$/) next
      if ((old != "000000" && old != "100644") ||
          (new != "000000" && new != "100644")) print "__atlas_special_file_mode__"
    }' "$collected_paths.modes" >>"$collected_paths"
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
