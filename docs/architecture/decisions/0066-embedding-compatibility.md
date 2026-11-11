# ADR0066. Embedding compatibility across dependency upgrades

Status: accepted

[Selectable search profiles](0050-selectable-search-profiles.md) bind each
publication to a complete profile. That manifest includes both passage layout
and numerical execution. Routine dependency upgrades need independent evidence
of compatibility to avoid expensive document inference for existing workspaces.

Use the complete profile for build inputs, physical storage, and publication
identity. Compare numerical contracts by requiring agreement on every field
apart from the shared segmentation digest, unit policy and baseline input
recipe. Atlas checks the exact prepared input digest and native token count
before reusing a vector across a layout change. Different model assets,
tokenization, prefixes or execution identities still require a separate
numerical contract.

Bind a query to the stored profile it was prepared against. Runtime admits
compatible execution, and the consuming snapshot checks the complete stored
profile again. Readers can continue to query a coherent publication while
maintenance prepares updated passage locations.

A layout change can select an authenticated vector from the prior publication
for the same profile. Verify its shape and checksum and stage the selected bytes
in bounded windows for installation into the new profile's table. The update
owner still publishes atomically. Exact-profile reuse keeps its existing
checksum-only selection path, and matching pending results remain resumable
under their recorded profile. See [owned publication](0065-input-owned-reference-publication.md).

Retain the existing policy IDs as opaque compatibility labels for the current
execution. Keep the resolved library set in the separate review receipt.
Changes to that set require reviewed evidence using both supported models,
including candidate queries against a predecessor artifact. Bump the execution
ID when behavior is incompatible. Explicit `atlas setup` is the migration
window for rebuilding an affected profile and may interrupt use of the app.
See [dependency maintenance](../../development/dependencies.md).

## Precomputed corpus embeddings

Pin and publish corpus data independently for each model. For managed English
setup, supply a verified asset after local reuse misses and before requiring
explicit local inference. Release validation uses a clean workspace and the
packaged candidate to prove that both profiles have complete coverage. Retain
the predecessor compatibility probe for changes to numerical behavior. See the
[embedding asset contract](../artifact-contract.md#published-corpus-embedding-assets)
and [release procedure](../../development/releases.md#refresh-precomputed-corpus-embeddings).
