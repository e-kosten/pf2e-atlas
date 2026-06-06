default:
    @just --list --unsorted

[doc('Install tracked hooks and verify this linked worktree is ready for development')]
[group('development')]
dev-setup:
    scripts/install-git-hooks.sh
    scripts/preflight.sh

[doc('Verify this checkout is a linked worktree on a non-main branch')]
[group('development')]
preflight:
    scripts/preflight.sh

[doc('Run the full Rust fmt, clippy, test, and build gate; pass --verbose for detailed output')]
[group('validation')]
verify *args:
    scripts/verify.sh {{args}}

[doc('Install the local atlas CLI from this checkout')]
[group('development')]
install:
    cargo install --path crates/atlas-cli --locked

[doc('Rebase, verify, fast-forward main, and verify main again')]
[group('development')]
land-worktree:
    scripts/land-worktree.sh

[doc('Open the interactive release workflow picker')]
[group('release')]
release *args:
    scripts/prepare-release.sh {{args}}

[doc('Create the release-preparation branch, version bump, notices, and notes')]
[group('release')]
release-prepare *args:
    scripts/prepare-release.sh --prepare-pr {{args}}

[doc('Validate, commit, push, and open the release-preparation PR')]
[group('release')]
release-open-pr *args:
    scripts/prepare-release.sh --open-pr {{args}}

[doc('Dry-run release publication from the committed crate version')]
[group('release')]
release-publish-dry *args:
    scripts/prepare-release.sh --publish --dry-run {{args}}

[doc('Tag the release and create the draft GitHub release')]
[group('release')]
release-publish *args:
    scripts/prepare-release.sh --publish {{args}}

[doc('Install a published Atlas release/RC; optional leading version defaults to the crate version')]
[group('release')]
release-install *args:
    #!/usr/bin/env sh
    set -eu
    set -- {{args}}
    case "${1:-}" in
      v[0-9]*|[0-9]*)
        version="$1"
        shift
        ;;
      *)
        version=$(awk -F '"' '/^version = / { print $2; exit }' crates/atlas-cli/Cargo.toml)
        ;;
    esac
    case "$version" in
      v*) tag="$version" ;;
      *) tag="v$version" ;;
    esac
    curl --proto '=https' --tlsv1.2 -LsSf \
      "https://github.com/e-kosten/pf2e-atlas/releases/download/$tag/atlas-installer.sh" |
      sh -s -- --version "$tag" "$@"

[doc('Run static release script, manifest, notices, and workflow checks')]
[group('release')]
release-check-tools:
    scripts/release/validate-release-tooling.sh
