# ADR 0043: Shared Foundry authored models and typed snapshots

Status: Accepted

## Context

Generated source models were ingest-owned and serialized only for developer
inspection. Application readers need those same authored types without linking
the build pipeline or repeating Foundry admission. A parallel Atlas family
schema would introduce separately maintained authority for the same mechanics.
Existing untagged unions and the ordered raw reader did not form a lossless
typed persistence contract.

## Decision

`atlas-foundry-model` owns the existing generated family structures, ordered
source values/maps, missing/null/value/invalid states, strict parsers, field
admission and pure document-family dispatch. Its dependencies are serde,
serde_json, serde_stacker and ryu-js; the last preserves JavaScript numeric-key semantics.
Discovery, files, original bytes/hashes, source provenance, quarantine, reports,
normalization, content interpretation and enrichment execution stay in ingest.
Ingest calls the shared bytes-based document admission once. There is no old
ingest model implementation or compatibility re-export.

Source-shaped authored types remain the future mechanics authority. Focused
Atlas identities/content/relationships and justified query projections can be
added by their existing owners. Shared source types do not become the frontend
API automatically and do not implement Foundry runtime preparation.

The codec stores one typed `FoundryDocumentSource` in a JSON envelope with a
format version and generated upstream declaration digest. Generated value
unions carry `$variant` and `$value`; deserialization does not choose a union
arm by structural resemblance. Ordered source objects/maps remain pair lists,
including duplicates. SourceValue uses ordinary tagged Serde representation for
snapshots. `parse_source_value` separately reads authored JSON with the existing
ordered visitor. No serde_json::Value conversion lies on the codec path.

`decode_snapshot` checks the envelope identities before decoding its borrowed
raw model fragment. Unsupported versions or source identities require a
rebuild; corrupt snapshots fail explicitly. There is no raw-source fallback,
admission retry, defaulting, coercion or repair. Skipped parser retry state is
initialized empty; durable invalid-field payloads and contextual diagnostics
remain intact. Exact original source bytes/provenance and any diagnostic index
remain outside the model snapshot; a complete SourceAdmission/raw tree is not
duplicated in it.

Snapshot-only decoding disables serde_json's default nesting limit and uses
serde_stacker on encode/decode. Ordinary source admission retains its existing
128-container JSON limit. Ordered object pair lists and union/presence tags
expand source nesting, so that source limit cannot also be used for snapshots.
A string/escape-aware guard caps snapshots at 1,024 JSON containers before
decoding; this leaves conservative space for the current generated/ordered
representation while bounding recursive Drop/Debug outside Serde. Serde remains
the JSON grammar owner. Near-limit retained array/object and typed recursive
predicate fixtures verify the expansion; over-limit snapshots fail explicitly.

Increment SNAPSHOT_VERSION when encoding or generated value-shape policies
change. Upstream source changes alter SOURCE_CONTRACT_ID through generation.
The pair identifies compatibility for immutable-artifact rebuilding, not a
general historical migration framework.

## Consequences

Developer model serialization now exposes explicit union discriminants. The
source fidelity probes compare authored payloads through their independent
declaration oracle; typed snapshot tests separately assert exact Rust equality,
including variants. Corpus outcomes and admission policy remain unchanged.
Rust runtime consumers can decode/traverse without ingest, indexing, records or
embedding dependencies. Compilation and application execution need no Node.

The current product build, artifact schema, metrics, retrieval and UI are
unchanged. Source-backed record enrichment and artifact/consumer replacement
remain subsequent coherent work. Snapshot sizes and performance are evidence
for that design, not a commitment to a particular database body table.
