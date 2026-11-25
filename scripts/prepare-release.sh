#!/usr/bin/env sh
set -eu

usage() {
  cat <<'USAGE'
Usage:
  scripts/prepare-release.sh --prepare-pr [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh --open-pr [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh --publish [--version <X.Y.Z[-rc.N]>] [options]
  scripts/prepare-release.sh --recover-draft <tag> <commit> [--yes] [--dry-run]
  scripts/prepare-release.sh

Prepares a release PR branch, opens the release-preparation PR, or creates an
annotated release tag and draft GitHub release after verifying that the release
version and notes were already committed to main.

Options:
  --prepare-pr             Create release-prep/v<version> from clean, current main
  --open-pr                Commit release-prep changes, push the branch, and open a PR
  --publish                Tag the committed release version and create a draft release
  --recover-draft <tag> <commit>
                           Create a missing draft for an already-pushed release tag
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
    *[!0-9rc.-]*) die "invalid release version" ;;
  esac

  if ! printf '%s\n' "$1" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-rc\.(0|[1-9][0-9]*))?$'; then
    die "version must look like 0.1.0 or 0.1.0-rc.1"
  fi
}

set_release_vars() {
  validate_version "$version"
  package_tag="v$version"
  case "$version" in
    *-rc.*) tag="rc/$package_tag" ;;
    *) tag="release/$package_tag" ;;
  esac
  [ -n "$notes_file" ] || notes_file="docs/releases/$package_tag.md"
  release_branch="release-prep/$package_tag"
}

version=""
notes_file=""
yes=0
dry_run=0
prepare_pr=0
open_pr=0
publish=0
recovery_tag=""
recovery_commit=""

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
    --recover-draft)
      [ "$#" -ge 3 ] || die "--recover-draft requires a tag and commit"
      [ -z "$recovery_tag" ] || die "pass --recover-draft only once"
      recovery_tag="$2"
      recovery_commit="$3"
      [ -n "$recovery_tag" ] || die "--recover-draft requires a tag"
      shift 3
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

create_release_draft() {
  if [ "$tag" = "rc/$package_tag" ]; then
    gh release create "$tag" --draft --verify-tag --notes-file "$1" --prerelease --latest=false --title "$tag"
  else
    gh release create "$tag" --draft --verify-tag --notes-file "$1" --title "$tag"
  fi
}

recover_release_draft() {
  [ "$((prepare_pr + open_pr + publish))" -eq 0 ] && [ -z "$version$notes_file" ] ||
    die "--recover-draft cannot be combined with another mode, --version, or --notes-file"
  version="${recovery_tag#*/v}"
  set_release_vars
  [ "$tag" = "$recovery_tag" ] || die "invalid official release tag: $recovery_tag"
  printf '%s\n' "$recovery_commit" | grep -Eq '^[0-9a-f]{40}$' &&
    [ "${#recovery_commit}" -eq 40 ] || die "pass the full release commit SHA"
  ref="refs/tags/$tag"
  refs=$(git ls-remote --exit-code origin "$ref" "$ref^{}") ||
    die "could not resolve remote release tag $tag"
  remote_commit=$(printf '%s\n' "$refs" | awk -v ref="$ref" '
    $2 == ref { direct = $1 }
    $2 == ref "^{}" { peeled = $1 }
    END { print (peeled != "" ? peeled : direct) }
  ')
  [ "$remote_commit" = "$recovery_commit" ] ||
    die "remote tag $tag resolves to $remote_commit, expected $recovery_commit"
  tagged_manifest=$(git show "$recovery_commit:crates/atlas-cli/Cargo.toml") ||
    die "could not read the tagged commit locally; fetch $tag from origin and retry"
  tagged_version=$(printf '%s\n' "$tagged_manifest" | awk -F '"' '/^version = / { print $2; exit }')
  [ "$tagged_version" = "$version" ] || die "tagged crate version $tagged_version does not match $tag"
  notes=$(git show "$recovery_commit:$notes_file") || die "release notes are missing from the tagged commit"
  [ -n "$notes" ] || die "release notes in the tagged commit are empty"

  release_state=$(gh api --paginate 'repos/{owner}/{repo}/releases?per_page=100' \
    --jq ".[] | select(.tag_name == \"$tag\") | .draft") ||
    die "could not determine whether a GitHub release exists for $tag"
  case "$release_state" in
    true) info "Draft release $tag already exists. If its release workflow has failed, rerun the failed jobs."; return ;;
    false) die "release $tag is already published" ;;
    '') ;;
    *) die "unexpected GitHub release state for $tag" ;;
  esac
  info "Recover draft release $tag from $recovery_commit"
  [ "$dry_run" -eq 0 ] || return 0
  if [ "$yes" -ne 1 ]; then
    printf 'Create the missing draft release? [y/N] '
    read answer
    case "$answer" in
      y|Y|yes|YES) ;;
      *) die "draft recovery cancelled" ;;
    esac
  fi
  printf '%s\n' "$notes" | create_release_draft - || die "could not create draft release $tag"
  info "Created draft release $tag. If its release workflow has failed, rerun the failed jobs."
}

if [ -n "$recovery_tag" ]; then
  recover_release_draft
  exit 0
fi

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
  for existing in $(git tag -l "rc/v$base_version-rc.*") docs/releases/v"$base_version"-rc.*.md; do
    case "$existing" in
      docs/releases/*) [ -f "$existing" ] || continue ;;
    esac
    name=${existing##*/}
    rc_number=${name#v"$base_version"-rc.}
    rc_number=${rc_number%.md}
    case "$rc_number" in ''|*[!0-9]*) continue ;; esac
    if [ "$rc_number" -gt "$max_rc" ]; then
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
  ensure_release_identity_has_no_remote
}

ensure_release_identity_has_no_remote() {
  if git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
    if [ "$dry_run" -eq 1 ]; then warn "remote tag $tag already exists"; else die "remote tag $tag already exists"; fi
  else
    [ "$?" -eq 2 ] || die "could not check remote tag $tag"
  fi
  release_match=$(gh api --paginate 'repos/{owner}/{repo}/releases?per_page=100' \
    --jq ".[] | select(.tag_name == \"$tag\") | .tag_name") ||
    die "could not determine whether GitHub release $tag exists"
  if [ -n "$release_match" ]; then
    if [ "$dry_run" -eq 1 ]; then warn "GitHub release $tag already exists"; else die "GitHub release $tag already exists"; fi
  fi
}

prepare_publish_tag() {
  if ! git rev-parse "$tag" >/dev/null 2>&1; then
    return
  fi

  existing_target=$(git rev-parse "$tag^{}" 2>/dev/null || git rev-parse "$tag")
  desired_target=$(git rev-parse HEAD)
  if [ "$dry_run" -eq 1 ]; then
    warn "real publish would ask to delete and recreate local tag $tag"
    warn "local tag target: $existing_target"
    warn "desired target:   $desired_target"
    return
  fi

  info "Local tag $tag already exists."
  info "Current target: $existing_target"
  info "Desired target: $desired_target"
  if ! can_prompt; then
    die "local tag $tag already exists; rerun from a terminal to approve deleting and recreating it"
  fi
  printf 'Delete and recreate local tag %s at %s? [y/N] ' "$tag" "$desired_target"
  read answer
  case "$answer" in
    y|Y|yes|YES) ;;
    *) die "release cancelled; local tag $tag was left unchanged" ;;
  esac
  git tag -d "$tag"
}

write_release_notes_template() {
  mkdir -p "$(dirname "$notes_file")"
  cat > "$notes_file" <<EOF_NOTES
# $package_tag

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

prep_state() {
  node scripts/release/prep-state.mjs "$@"
}

read_main_for_prep() {
  main_ref=$(git ls-remote --exit-code --heads origin refs/heads/main) || die "could not verify main on origin"
  prep_main=${main_ref%%[[:space:]]*}
  if ! git cat-file -e "$prep_main^{commit}" 2>/dev/null; then
    [ "$dry_run" -eq 0 ] || die "fetch origin main before inspecting this dry run"
    git fetch origin main || die "could not fetch main"
    git cat-file -e "$prep_main^{commit}" || die "origin/main moved during inspection; retry"
  fi
}

remote_prep_head() {
  remote_ref=$(git ls-remote --heads origin "refs/heads/$release_branch") || die "could not check release branch on origin"
  printf '%s' "${remote_ref%%[[:space:]]*}"
}

check_prep_publication() {
  remote_head=$(remote_prep_head) || exit 1
  if [ -n "$remote_head" ] && { [ "$kind" != committed ] || [ "$head" != "$remote_head" ]; }; then
    die "origin/$release_branch points to a different release-preparation commit"
  fi
  prs=$(gh pr list --repo "$repo_url" --head "$release_branch" --state all \
    --json url,state,isCrossRepository,baseRefName,headRefName,headRefOid) || die "could not check existing pull requests"
  pr_url=$(printf '%s' "$prs" | prep_state pr "$release_branch" "$head" "$repo_url") || exit 1
  if [ -n "$pr_url" ] && [ -z "$remote_head" ]; then die "existing PR has no matching branch on origin"; fi
}

release_branch_from_current_branch() {
  current_branch=$(git branch --show-current)
  case "$current_branch" in
    release-prep/v*) ;;
    *) die "--open-pr must run from a release-prep/v<version> branch; current branch is $current_branch" ;;
  esac
  branch_version=${current_branch#release-prep/v}
  validate_version "$branch_version"
  if [ -n "$version" ] && [ "$version" != "$branch_version" ]; then
    die "current release branch is for $branch_version, but --version requested $version"
  fi
  version=$branch_version
}

run_release_pr_checks() {
  run_check scripts/release/validate-release-tooling.sh
  run_check npm --prefix web/atlas-ui ci
  run_check npm --prefix web/atlas-ui run build
  run_check scripts/verify.sh
}

run_check() {
  info "Running: $*"
  "$@"
}

if [ "$open_pr" -eq 1 ]; then
  release_branch_from_current_branch
  set_release_vars
  origin_url=$(git remote get-url origin) || die "could not resolve origin"
  repo_url=$(gh repo view "$origin_url" --json url --jq .url) || die "could not identify the GitHub repository for origin"
  [ -n "$repo_url" ] || die "missing GitHub repository URL for origin"
  export GH_REPO="$repo_url"
  ensure_release_identity_is_unused
  read_main_for_prep
  state=$(prep_state candidate "$version" "$notes_file" "$prep_main") || exit 1
  read -r kind head inputs <<EOF_STATE
$state
EOF_STATE
  check_prep_publication
  if [ -n "$pr_url" ]; then info "$pr_url"; exit 0; fi

  info "Release PR: $tag"
  info "Branch:     $release_branch"
  info "Notes:      $notes_file"
  info "Checks:"
  info "  scripts/release/validate-release-tooling.sh"
  info "  npm --prefix web/atlas-ui ci"
  info "  npm --prefix web/atlas-ui run build"
  info "  scripts/verify.sh"

  if [ "$dry_run" -eq 1 ]; then
    [ "$kind" = committed ] || info "Dry run: would commit the validated release-preparation files."
    [ -n "$remote_head" ] || info "Dry run: would push $release_branch."
    info "Dry run: would validate this candidate and open its PR to main."
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
  [ "$(git branch --show-current)" = "$release_branch" ] || die "release branch changed during validation"
  read_main_for_prep
  validated=$(prep_state candidate "$version" "$notes_file" "$prep_main") || exit 1
  [ "$validated" = "$state" ] || die "release-preparation inputs changed during validation"
  if [ "$kind" = pending ]; then
    git add -- Cargo.lock crates/atlas-cli/Cargo.toml "$notes_file"
    git commit --no-verify -m "chore(release): prepare $package_tag" -- Cargo.lock crates/atlas-cli/Cargo.toml "$notes_file"
    head=$(git rev-parse HEAD)
    kind=committed
    [ "$(prep_state candidate "$version" "$notes_file" "$prep_main")" = "committed $head $inputs" ] ||
      die "committed release preparation differs from the validated inputs"
  fi
  check_prep_publication
  if [ -z "$remote_head" ]; then
    git push --no-verify -u origin "$release_branch"
    check_prep_publication
    [ "$remote_head" = "$head" ] || die "could not verify the pushed release branch"
  fi
  if [ -n "$pr_url" ]; then info "$pr_url"; exit 0; fi
  gh pr create --repo "$repo_url" \
    --base main \
    --head "$release_branch" \
    --title "chore(release): prepare $package_tag" \
    --body "Prepare $package_tag for release."
  exit 0
fi

# A retry on this exact branch may complete a partially written pair of
# version files. Only the original and intended bytes are eligible for recovery.
if [ "$prepare_pr" -eq 1 ]; then
  current=$(git branch --show-current)
  case "$current" in
    release-prep/v*)
      if [ -z "$version" ] || [ "$version" = "${current#release-prep/v}" ]; then
        release_branch_from_current_branch
        set_release_vars
        ensure_release_identity_is_unused
        read_main_for_prep
        if [ "$dry_run" -eq 1 ]; then
          prep_state resume-dry-run "$version" "$notes_file" "$prep_main"
          info "Dry run: would resume preparation of $release_branch."
          exit 0
        fi
        prep_state resume "$version" "$notes_file" "$prep_main"
        if [ ! -f "$notes_file" ]; then write_release_notes_template; fi
        open_release_notes_editor
        info "Resumed release-preparation branch $release_branch."
        exit 0
      fi ;;
  esac
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
if [ "$publish" -eq 1 ]; then
  ensure_release_identity_has_no_remote
else
  ensure_release_identity_is_unused
fi

if [ "$prepare_pr" -eq 1 ]; then
  if git rev-parse --verify "$release_branch" >/dev/null 2>&1; then
    if [ "$dry_run" -eq 1 ]; then
      warn "local branch $release_branch already exists"
    else
      die "local branch $release_branch already exists"
    fi
  fi
  if git ls-remote --exit-code --heads origin "refs/heads/$release_branch" >/dev/null; then
    if [ "$dry_run" -eq 1 ]; then
      warn "remote branch $release_branch already exists"
    else
      die "remote branch $release_branch already exists"
    fi
  else
    [ "$?" -eq 2 ] || die "could not check release branch on origin"
  fi

  current_crate_version=$(read_crate_version)
  info "Release prep: $tag"
  info "Base:         $(git rev-parse HEAD)"
  info "Branch:       $release_branch"
  info "Notes:        $notes_file"
  info "Crate:        $current_crate_version -> $version"

  prep_state check "$version" "$notes_file"
  if [ "$dry_run" -eq 1 ]; then
    info "Dry run: would create $release_branch, update crates/atlas-cli/Cargo.toml, and ensure $notes_file exists."
    exit 0
  fi

  git switch -c "$release_branch"
  prep_state write "$version" "$notes_file" || die "version preparation was interrupted; rerun --prepare-pr --version $version from $release_branch"
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
info "  npm --prefix web/atlas-ui ci"
info "  npm --prefix web/atlas-ui run build"
info "  scripts/verify.sh"
info "  dist plan --verbose error --tag $package_tag --allow-dirty"
prepare_publish_tag

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

run_check npm --prefix web/atlas-ui ci
run_check npm --prefix web/atlas-ui run build
run_check scripts/verify.sh
run_check dist plan --verbose error --tag "$package_tag" --allow-dirty
ensure_clean_worktree

git tag -a "$tag" -m "Release $tag"
git push --no-verify origin "$tag"

if ! create_release_draft "$notes_file"; then
  die "tag $tag was pushed but draft creation failed; retry with --recover-draft $tag $head_sha"
fi

info "Created draft release $tag. GitHub Actions will upload assets and publish it after validation."
