#!/usr/bin/env sh
set -eu

usage() {
  cat <<'USAGE'
Usage:
  scripts/prepare-release.sh --prepare-pr [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh --open-pr [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh --publish [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh

Prepares a release PR branch, opens the release-preparation PR, or creates an
annotated release tag and draft GitHub release after verifying that the release
version and notes were already committed to main.

Options:
  --prepare-pr             Create release/v<version> from clean, current main
  --open-pr                Commit release-prep changes, push the branch, and open a PR
  --publish                Tag the committed release version and create a draft release
  --version <version>      Version without leading v; optional guard for open-pr/publish
  --notes-file <path>      Release notes file, default docs/releases/v<version>.md
  --yes                    Do not prompt before tagging/releasing
  --dry-run                Print planned actions without changing git/GitHub state
  --help                   Show this help

When --prepare-pr omits --version, the script prompts with semver bump choices
based on the current atlas-cli crate version and existing release tags.
Interactive prepare-pr opens the release notes in VISUAL, EDITOR, or git
core.editor after creating or confirming the notes file.
When open-pr or publish mode omits --version, the script uses the committed
atlas-cli crate version.
When no mode flag is passed, the script prompts for the mode. Interactive
prompts use fzf when it is installed, with numbered prompts as the fallback.
USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

warn() {
  printf 'warning: %s\n' "$*" >&2
}

info() {
  printf '%s\n' "$*"
}

can_prompt() {
  [ -t 0 ] || [ "${ATLAS_RELEASE_TEST_PROMPTS:-0}" = 1 ]
}

read_crate_version() {
  awk -F '"' '/^version = / { print $2; exit }' crates/atlas-cli/Cargo.toml
}

validate_version() {
  case "$1" in
    v*) die "pass --version without a leading v" ;;
  esac

  if ! printf '%s\n' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'; then
    die "version must look like 0.1.0 or 0.1.0-rc.1"
  fi
}

set_release_vars() {
  validate_version "$version"
  tag="v$version"
  [ -n "$notes_file" ] || notes_file="docs/releases/$tag.md"
  release_branch="release/$tag"
}

version=""
notes_file=""
yes=0
dry_run=0
prepare_pr=0
open_pr=0
publish=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --prepare-pr)
      prepare_pr=1
      shift
      ;;
    --open-pr)
      open_pr=1
      shift
      ;;
    --publish)
      publish=1
      shift
      ;;
    --version)
      [ "$#" -ge 2 ] || die "--version requires a value"
      version="$2"
      shift 2
      ;;
    --notes-file)
      [ "$#" -ge 2 ] || die "--notes-file requires a value"
      notes_file="$2"
      shift 2
      ;;
    --yes)
      yes=1
      shift
      ;;
    --dry-run)
      dry_run=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

selected_modes=$((prepare_pr + open_pr + publish))
if [ "$selected_modes" -gt 1 ]; then
  die "choose only one of --prepare-pr, --open-pr, or --publish"
fi

choose_mode() {
  if ! can_prompt; then
    die "no release workflow step was provided and stdin is not a terminal; rerun with --prepare-pr, --open-pr, or --publish"
  fi
  if [ "${ATLAS_RELEASE_TEST_DISABLE_FZF:-0}" != 1 ] && command -v fzf >/dev/null 2>&1; then
    mode_choice=$(
      printf '%s\n' \
        "prepare-pr  Prepare release PR branch" \
        "open-pr     Commit, push, and open release-preparation PR" \
        "publish     Publish tag and draft release" |
        fzf --prompt 'Release step> ' --height=~40% --layout=reverse
    ) || die "release workflow step selection cancelled"
    set -- $mode_choice
    case "$1" in
      prepare-pr) prepare_pr=1 ;;
      open-pr) open_pr=1 ;;
      publish) publish=1 ;;
      *) die "invalid selection: $mode_choice" ;;
    esac
    return
  fi

  info "Choose release workflow step:"
  info "  1) prepare PR branch"
  info "  2) open release-preparation PR"
  info "  3) publish tag and draft release"
  printf 'Selection: '
  if ! read mode_selection; then
    die "choose --prepare-pr, --open-pr, or --publish when the script cannot read a selection"
  fi
  case "$mode_selection" in
    1) prepare_pr=1 ;;
    2) open_pr=1 ;;
    3) publish=1 ;;
    *) die "invalid selection: $mode_selection" ;;
  esac
}

if [ "$prepare_pr" -eq 0 ] && [ "$open_pr" -eq 0 ] && [ "$publish" -eq 0 ]; then
  choose_mode
fi

ensure_clean_worktree() {
  if [ -n "$(git status --porcelain)" ]; then
    if [ "$dry_run" -eq 1 ]; then
      warn "real release work requires a clean working tree"
    else
      die "working tree must be clean before release work"
    fi
  fi
}

ensure_current_main_checkout() {
  ensure_clean_worktree
  if ! git fetch origin main --tags >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "failed to fetch origin main and tags; using local refs for dry run"
    else
      die "failed to fetch origin main and tags"
    fi
  fi

  current_branch=$(git branch --show-current)
  if [ "$current_branch" != "main" ]; then
    if [ "$dry_run" -eq 1 ]; then
      warn "real release work would switch from $current_branch to main"
    else
      git switch main
    fi
  fi

  if [ "$dry_run" -eq 1 ]; then
    head_sha=$(git rev-parse HEAD)
    origin_main_sha=$(git rev-parse origin/main 2>/dev/null || printf '%s' "$head_sha")
  else
    git pull --ff-only origin main
    head_sha=$(git rev-parse HEAD)
    origin_main_sha=$(git rev-parse origin/main)
  fi

  if [ "$head_sha" != "$origin_main_sha" ]; then
    if [ "$dry_run" -eq 1 ]; then
      warn "real release work requires main to equal origin/main after fast-forward"
    else
      die "main must equal origin/main before release work"
    fi
  fi
}

next_rc_version() {
  base_version="$1"
  max_rc=0
  for existing_tag in $(git tag -l "v$base_version-rc.*"); do
    rc_number=$(printf '%s\n' "$existing_tag" | sed -n "s/^v$base_version-rc\\.\\([0-9][0-9]*\\)$/\\1/p")
    if [ -n "$rc_number" ] && [ "$rc_number" -gt "$max_rc" ]; then
      max_rc="$rc_number"
    fi
  done
  printf '%s-rc.%s\n' "$base_version" "$((max_rc + 1))"
}

choose_prepare_version() {
  if ! can_prompt; then
    die "--version is required when --prepare-pr is run without a terminal"
  fi

  current_crate_version=$(read_crate_version)
  base_version=${current_crate_version%%-*}
  old_ifs=$IFS
  IFS=.
  set -- $base_version
  IFS=$old_ifs
  major=$1
  minor=$2
  patch=$3
  rc_candidate=$(next_rc_version "$base_version")
  patch_candidate="$major.$minor.$((patch + 1))"
  minor_candidate="$major.$((minor + 1)).0"
  major_candidate="$((major + 1)).0.0"

  info "Current atlas-cli version: $current_crate_version"

  if [ "${ATLAS_RELEASE_TEST_DISABLE_FZF:-0}" != 1 ] && command -v fzf >/dev/null 2>&1; then
    version_choice=$(
      printf '%s\n' \
        "$rc_candidate  Release candidate for $base_version" \
        "$patch_candidate  Patch" \
        "$minor_candidate  Minor" \
        "$major_candidate  Major" \
        "custom  Custom version" |
        fzf --prompt 'Release version> ' --height=~40% --layout=reverse
    ) || die "release version selection cancelled"
    set -- $version_choice
    case "$1" in
      custom)
        printf 'Version: '
        read version
        ;;
      *) version=$1 ;;
    esac
    return
  fi

  info "Choose the release-prep version:"
  info "  1) $rc_candidate (release candidate for $base_version)"
  info "  2) $patch_candidate (patch)"
  info "  3) $minor_candidate (minor)"
  info "  4) $major_candidate (major)"
  info "  5) custom"

  printf 'Selection [1]: '
  if ! read selection; then
    die "--version is required when --prepare-pr cannot read a selection"
  fi
  [ -n "$selection" ] || selection=1
  case "$selection" in
    1) version=$rc_candidate ;;
    2) version=$patch_candidate ;;
    3) version=$minor_candidate ;;
    4) version=$major_candidate ;;
    5)
      printf 'Version: '
      read version
      ;;
    *) die "invalid selection: $selection" ;;
  esac
}

ensure_release_identity_is_unused() {
  if git rev-parse "$tag" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "local tag $tag already exists"
    else
      die "local tag $tag already exists"
    fi
  fi

  if git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "remote tag $tag already exists"
    else
      die "remote tag $tag already exists"
    fi
  fi

  if gh release view "$tag" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "GitHub release $tag already exists"
    else
      die "GitHub release $tag already exists"
    fi
  fi
}

write_release_notes_template() {
  mkdir -p "$(dirname "$notes_file")"
  cat > "$notes_file" <<EOF_NOTES
# $tag

## Summary

Describe the user-facing release in a short paragraph or bullets.

## Install Notes

Mention install, update, setup, or compatibility notes.

## Known Issues

- None known.
EOF_NOTES
}

open_release_notes_editor() {
  if ! can_prompt || [ "${ATLAS_RELEASE_TEST_SKIP_EDITOR:-0}" = 1 ]; then
    return
  fi

  editor=${VISUAL:-${EDITOR:-}}
  if [ -z "$editor" ]; then
    editor=$(git config --get core.editor 2>/dev/null || true)
  fi
  if [ -z "$editor" ]; then
    warn "no editor configured; set VISUAL, EDITOR, or git config core.editor to edit $notes_file automatically"
    return
  fi

  info "Opening release notes in editor: $notes_file"
  # Intentionally split configured editor commands so values like "code --wait"
  # work while the notes path stays one argument.
  # shellcheck disable=SC2086
  set -- $editor
  "$@" "$notes_file"
}

update_crate_version() {
  tmp_file="crates/atlas-cli/Cargo.toml.release-tmp"
  awk -v version="$version" '
    BEGIN { replaced = 0 }
    /^version = / && replaced == 0 {
      print "version = \"" version "\""
      replaced = 1
      next
    }
    { print }
    END {
      if (replaced == 0) {
        exit 1
      }
    }
  ' crates/atlas-cli/Cargo.toml > "$tmp_file" || {
    rm -f "$tmp_file"
    die "failed to update crates/atlas-cli version"
  }
  mv "$tmp_file" crates/atlas-cli/Cargo.toml
}

update_lockfile() {
  cargo check -p atlas-cli
}

update_third_party_notices() {
  scripts/release/generate-notices.py
}

release_branch_from_current_branch() {
  current_branch=$(git branch --show-current)
  case "$current_branch" in
    release/v*) ;;
    *) die "--open-pr must run from a release/v<version> branch; current branch is $current_branch" ;;
  esac
  branch_version=${current_branch#release/v}
  validate_version "$branch_version"
  if [ -n "$version" ] && [ "$version" != "$branch_version" ]; then
    die "current release branch is for $branch_version, but --version requested $version"
  fi
  version=$branch_version
}

ensure_release_notes_are_edited() {
  [ -s "$notes_file" ] || die "release notes file is missing or empty: $notes_file"
  if grep -Eq 'Describe the user-facing release|Mention install, update, setup, or compatibility notes' "$notes_file"; then
    die "release notes still contain template placeholder text: $notes_file"
  fi
}

ensure_release_prep_scope() {
  status_output=$(git status --porcelain)
  [ -n "$status_output" ] || return 0
  printf '%s\n' "$status_output" | while IFS= read -r status_line; do
    path=${status_line#???}
    case "$path" in
      Cargo.lock|THIRD-PARTY-NOTICES.md|crates/atlas-cli/Cargo.toml|"$notes_file") ;;
      *) die "unexpected release-prep change: $path" ;;
    esac
  done
}

run_release_pr_checks() {
  run_check scripts/release/validate-release-tooling.sh
  run_check scripts/release/test-prepare-release.sh
  run_check cargo fmt --check
  run_check cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro
  run_strict_runtime_clippy
  run_check cargo test --workspace
  run_check cargo build --workspace
}

run_strict_runtime_clippy() {
  run_check cargo clippy --workspace --lib --bins -- -D warnings \
    -D clippy::unwrap_used \
    -D clippy::expect_used \
    -D clippy::panic \
    -D clippy::unimplemented \
    -D clippy::todo \
    -D clippy::unreachable
}

run_check() {
  info "Running: $*"
  "$@"
}

if [ "$open_pr" -eq 1 ]; then
  release_branch_from_current_branch
  set_release_vars
  crate_version=$(read_crate_version)
  [ "$crate_version" = "$version" ] || die "crates/atlas-cli version is $crate_version, expected $version"
  ensure_release_identity_is_unused
  ensure_release_notes_are_edited
  ensure_release_prep_scope
  if ! gh auth status >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "gh is not authenticated; real PR creation requires gh auth login"
    else
      die "gh is not authenticated; run gh auth login"
    fi
  fi
  if gh pr view "$release_branch" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "GitHub PR already exists for $release_branch"
    else
      die "GitHub PR already exists for $release_branch"
    fi
  fi

  info "Release PR: $tag"
  info "Branch:     $release_branch"
  info "Notes:      $notes_file"
  info "Checks:"
  info "  scripts/release/validate-release-tooling.sh"
  info "  scripts/release/test-prepare-release.sh"
  info "  cargo fmt --check"
  info "  cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro"
  info "  cargo clippy --workspace --lib --bins -- -D warnings -D clippy::{unwrap_used,expect_used,panic,unimplemented,todo,unreachable}"
  info "  cargo test --workspace"
  info "  cargo build --workspace"

  if [ "$dry_run" -eq 1 ]; then
    info "Dry run: would run checks, commit release-prep files, push $release_branch, and create a PR to main."
    exit 0
  fi

  if [ "$yes" -ne 1 ]; then
    printf 'Run checks, commit release prep, push %s, and open PR? [y/N] ' "$release_branch"
    read answer
    case "$answer" in
      y|Y|yes|YES) ;;
      *) die "open-pr cancelled" ;;
    esac
  fi

  run_release_pr_checks
  git add Cargo.lock THIRD-PARTY-NOTICES.md crates/atlas-cli/Cargo.toml "$notes_file"
  git commit -m "chore(release): prepare $tag"
  git push -u origin "$release_branch"
  gh pr create \
    --base main \
    --head "$release_branch" \
    --title "chore(release): prepare $tag" \
    --body "Prepare $tag for release."
  exit 0
fi

ensure_current_main_checkout

if [ -z "$version" ]; then
  if [ "$prepare_pr" -eq 1 ]; then
    choose_prepare_version
  else
    version=$(read_crate_version)
  fi
fi
set_release_vars
ensure_release_identity_is_unused

if [ "$prepare_pr" -eq 1 ]; then
  if git rev-parse --verify "$release_branch" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "local branch $release_branch already exists"
    else
      die "local branch $release_branch already exists"
    fi
  fi
  if git ls-remote --exit-code --heads origin "$release_branch" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "remote branch $release_branch already exists"
    else
      die "remote branch $release_branch already exists"
    fi
  fi

  current_crate_version=$(read_crate_version)
  info "Release prep: $tag"
  info "Base:         $(git rev-parse HEAD)"
  info "Branch:       $release_branch"
  info "Notes:        $notes_file"
  info "Crate:        $current_crate_version -> $version"

  if [ "$dry_run" -eq 1 ]; then
    info "Dry run: would create $release_branch, update crates/atlas-cli/Cargo.toml, and ensure $notes_file exists."
    exit 0
  fi

  git switch -c "$release_branch"
  update_crate_version
  update_lockfile
  update_third_party_notices
  if [ ! -f "$notes_file" ]; then
    write_release_notes_template
  fi
  open_release_notes_editor
  info "Created release-preparation branch $release_branch."
  info "Edit $notes_file, validate, commit, and open a PR to main."
  exit 0
fi

crate_version=$(read_crate_version)
[ "$crate_version" = "$version" ] || die "crates/atlas-cli version is $crate_version, expected $version"

if [ ! -f "$notes_file" ]; then
  die "release notes file is missing: $notes_file"
fi

if ! git ls-files --error-unmatch "$notes_file" >/dev/null 2>&1; then
  if [ "$dry_run" -eq 1 ]; then
    warn "real releases require tracked release notes: $notes_file"
  else
    die "release notes file must be tracked: $notes_file"
  fi
fi
if ! git diff --quiet -- "$notes_file"; then
  if [ "$dry_run" -eq 1 ]; then
    warn "real releases require committed release notes: $notes_file"
  else
    die "release notes file has uncommitted changes: $notes_file"
  fi
fi

if ! gh auth status >/dev/null 2>&1; then
  if [ "$dry_run" -eq 1 ]; then
    warn "gh is not authenticated; real releases require gh auth login"
  else
    die "gh is not authenticated; run gh auth login"
  fi
fi

is_prerelease=0
case "$version" in
  *-*) is_prerelease=1 ;;
esac

info "Release: $tag"
info "Commit:  $head_sha"
info "Notes:   $notes_file"
if [ "$is_prerelease" -eq 1 ]; then
  info "Type:    prerelease draft"
else
  info "Type:    final draft"
fi

info "Local checks:"
info "  cargo fmt --check"
info "  cargo clippy --workspace --all-targets -- -D warnings -D clippy::dbg_macro"
info "  cargo clippy --workspace --lib --bins -- -D warnings -D clippy::{unwrap_used,expect_used,panic,unimplemented,todo,unreachable}"
info "  cargo test --workspace"
info "  cargo build --workspace"
info "  dist plan --verbose error --tag $tag --allow-dirty"

if [ "$dry_run" -eq 1 ]; then
  info "Dry run: would run checks, create annotated tag $tag, push it, and create draft GitHub release."
  exit 0
fi

if [ "$yes" -ne 1 ]; then
  printf 'Create and publish tag %s and draft release? [y/N] ' "$tag"
  read answer
  case "$answer" in
    y|Y|yes|YES) ;;
    *) die "release cancelled" ;;
  esac
fi

run_check cargo fmt --check
run_check cargo clippy --workspace --all-targets -- -D warnings
run_strict_runtime_clippy
run_check cargo test --workspace
run_check cargo build --workspace
run_check dist plan --verbose error --tag "$tag" --allow-dirty
ensure_clean_worktree

git tag -a "$tag" -m "Release $tag"
git push origin "$tag"

if [ "$is_prerelease" -eq 1 ]; then
  gh release create "$tag" --draft --verify-tag --notes-file "$notes_file" --prerelease --latest=false --title "$tag"
else
  gh release create "$tag" --draft --verify-tag --notes-file "$notes_file" --title "$tag"
fi

info "Created draft release $tag. GitHub Actions will upload assets and publish it after validation."
