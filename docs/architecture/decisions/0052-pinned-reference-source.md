# ADR 0052: Reference data and model share an immutable upstream pin

Status: Accepted
Date: 2026-10-18

## Decision

`dev-tools/source-contracts/source-pin.json` is the canonical tested PF2e revision
for both declaration generation and default reference data. Generation emits
repository, commit and system version constants beside the model contract digest.
Runtime uses those constants; ordinary Rust builds require neither Node nor an
upstream checkout. A pin update is a coordinated model/data support change.

Runtime owns a commit-keyed reference cache: `.cache/pf2e/<commit>` in repo mode,
or `pf2e-atlas/pf2e/<commit>` under the platform cache directory in global mode.
Setup fetches only the immutable commit, checks out detached HEAD in a temporary
sibling directory, validates revision and cleanliness, then publishes the cache.
Warm caches are validated without fetching. Wrong revisions or local changes are
errors, not permission to reset, pull, delete or repair a directory. Check and
offline modes do not acquire missing data.

Explicit source directories and previously persisted source selections remain
user-managed. Setup never clones into, pulls, or checks out those directories.
Selecting the known managed cache path explicitly opts back into managed acquisition.
Existing saved selections are not silently retargeted when default paths change.

Ingest validates a PF2e system manifest's version before product preparation,
including direct builds, and checks the manifest against the captured source
inventory. Runtime performs the same preflight for setup. Incompatibility reports
expected version/commit and detected version before inference or reference
publication. Developer loading/admission diagnostics remain available for
unsupported inputs. A custom module's own version is not a PF2e version; modules
continue through ordinary checked admission without claiming verified upstream
version compatibility. Supplied sources can be edited, exported, or custom;
matching a version label is not a proof of byte identity. The exact immutable
commit and clean checkout guarantee applies to the managed source cache.

## Consequences

Default data can no longer drift with upstream's default branch. Existing
`vendor/pf2e` directories are preserved and are not default managed caches.
For a workspace that retains an old source selection, explicitly select the new
managed path through `--source`; do not recreate the workspace or delete saved data.
Changing the requested source does not replace the published reference until a
successful build commits. Ordinary retrieval continues to work without a checkout.

A newer pin requires generation, admission and product-consumer validation plus
a state-preserving reference rebuild across the model contract change. Multiple
upstream model versions and embedded-document failure isolation are separate
product/architecture decisions; this policy introduces neither adapters nor
relaxed union admission. See ADR0034, ADR0041 and ADR0051.
