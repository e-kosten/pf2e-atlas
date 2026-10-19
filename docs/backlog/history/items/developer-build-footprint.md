# Active worktree build footprint

Status: done

Ordinary development and tests use limited debug information and enable
incremental compilation except for the large generated `atlas-foundry-model`
crate. Clean CI jobs disable incremental compilation globally. Full variable/type
debugging uses `--profile debug-full`. Release settings, assertions, overflow
checks, features, crate boundaries and test coverage remain intact.

[CONTRIBUTING.md](../../../../CONTRIBUTING.md) and AGENTS.md define worktree
retirement, preview preservation and disk investigation thresholds. A preview's
frozen binary and saved state can be retained independently of compiler output.
Active targets remain useful caches; routine cleanup/rebuild cycles are discouraged.

New-branch pre-push checks use known destination-remote ancestry, combine all
pushed refs and reject a code tip different from the checkout's HEAD. Unknown paths
or uncertain ancestry require full checks. Real-Git fixture tests protect the
selector and run in CI without invoking compilers.

[Measured alternatives](../../../research/build-footprint.md) record the pinned
comparison and limitations. Test consolidation, inference feature splitting and
compiler caching were evaluated as alternatives; none had enough demonstrated
benefit to include in this change.
