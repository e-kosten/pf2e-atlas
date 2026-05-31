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

[doc('Run the full Rust fmt, clippy, test, and build gate')]
[group('validation')]
verify:
    scripts/verify.sh

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

[doc('Run static release script, manifest, notices, and workflow checks')]
[group('release')]
release-check-tools:
    scripts/release/validate-release-tooling.sh
