#!/usr/bin/env sh
set -eu

usage() {
  cat <<'USAGE'
Usage: scripts/prepare-release.sh --version <X.Y.Z[-rc.N]> [options]

Creates an annotated release tag and a draft GitHub release after verifying that
the release version and notes were already committed to main.

Options:
  --version <version>      Version without leading v, for example 0.1.0 or 0.1.0-rc.1
  --notes-file <path>      Release notes file, default docs/releases/v<version>.md
  --yes                    Do not prompt before tagging/releasing
  --dry-run                Print planned actions without changing git/GitHub state
  --help                   Show this help

If the default notes file is missing in an interactive terminal, this script
scaffolds docs/releases/v<version>.md and stops so you can edit and commit it.
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

version=""
notes_file=""
yes=0
dry_run=0

while [ "$#" -gt 0 ]; do
  case "$1" in
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

[ -n "$version" ] || die "--version is required"

case "$version" in
  v*) die "pass --version without a leading v" ;;
esac

if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$'; then
  die "version must look like 0.1.0 or 0.1.0-rc.1"
fi

tag="v$version"
[ -n "$notes_file" ] || notes_file="docs/releases/$tag.md"

crate_version=$(awk -F '"' '/^version = / { print $2; exit }' crates/atlas-cli/Cargo.toml)
[ "$crate_version" = "$version" ] || die "crates/atlas-cli version is $crate_version, expected $version"

current_branch=$(git branch --show-current)
if [ "$current_branch" != "main" ]; then
  if [ "$dry_run" -eq 1 ]; then
    warn "real releases must run from main; current branch is $current_branch"
  else
    die "release must run from main, current branch is $current_branch"
  fi
fi

if ! git fetch origin main --tags >/dev/null 2>&1; then
  if [ "$dry_run" -eq 1 ]; then
    warn "failed to fetch origin main and tags; using local refs for dry run"
  else
    die "failed to fetch origin main and tags"
  fi
fi

head_sha=$(git rev-parse HEAD)
origin_main_sha=$(git rev-parse origin/main 2>/dev/null || printf '%s' "$head_sha")
if [ "$head_sha" != "$origin_main_sha" ]; then
  if [ "$dry_run" -eq 1 ]; then
    warn "real releases require HEAD to equal origin/main"
  else
    die "HEAD must equal origin/main before release"
  fi
fi

if [ -n "$(git status --porcelain)" ]; then
  if [ "$dry_run" -eq 1 ]; then
    warn "real releases require a clean working tree"
  else
    die "working tree must be clean before release"
  fi
fi

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

if [ ! -f "$notes_file" ]; then
  if [ -t 0 ] && [ "$dry_run" -eq 0 ]; then
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
    info "Created $notes_file."
    info "Edit and commit that file in a release-preparation PR, then rerun this script from main."
    exit 1
  fi
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
info "  cargo clippy --workspace --all-targets -- -D warnings"
info "  cargo test --workspace"
info "  cargo build --workspace"
info "  dist plan --tag $tag --allow-dirty"

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

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
dist plan --tag "$tag" --allow-dirty

git tag -a "$tag" -m "Release $tag"
git push origin "$tag"

if [ "$is_prerelease" -eq 1 ]; then
  gh release create "$tag" --draft --verify-tag --notes-file "$notes_file" --prerelease --latest=false --title "$tag"
else
  gh release create "$tag" --draft --verify-tag --notes-file "$notes_file" --title "$tag"
fi

info "Created draft release $tag. GitHub Actions will upload assets and publish it after validation."
