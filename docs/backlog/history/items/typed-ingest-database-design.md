# Source-backed artifact and ingest cutover

Status: done
Priority: current
Owner: Codex
Last reviewed: 2026-10-10

## Completed implementation

[PR 49](https://github.com/e-kosten/pf2e-atlas/pull/49) contains the coordinated
replacement against `integration/source-modeling`. The full pinned corpus,
actual-model sample, product/browser workflows, independent reviews and complete
hosted implementation CI pass; see the
[validation report](../../../research/source-artifact-cutover-validation.md).
This records implementation completion, not a merge into main.
Family-specific presentation modeling remains a
[separate review](../../items/rust-web-ui-architecture-review.md).

## Approved direction

Replace the reference artifact, ingest pipeline and affected consumers together.
One checked generated Foundry DTO remains the authored authority for each root;
embedded documents remain inside that snapshot. Named typed query projections,
selected prepared HTML/control bundles, attributed relationships and independently
selected lexical/semantic units are rebuildable outputs. Canonical keys remain
Foundry pack plus source ID. There is no old-artifact compatibility path.

The approved design retires generic metric EAV, persisted RichDocument,
heuristic taxonomy and synthetic canonical affliction records. Actual embedded
and prose affliction definitions remain searchable and directly navigable.
RollTables are product records; Macros are developer-only. Verified remaster
preference applies only when both versions match a request. Related variants
remain a bounded, transparent derived view.

Filters use a discoverable catalog with authored values and explicit availability
states. Root and immediate embedded Item scopes share extraction. CLI CEL and a
structured Ant query builder independently lower to one typed predicate. The
initial catalog includes shared traits, rarity, publication and applicable
level/rank, Actor saves/IWR/defenses/movement and representative Item-family facts.
Skills, nuanced gameplay totals and affliction-specific filters remain deferred.

Precision FTS uses identity, verified aliases, typed vocabulary and named
definitions. Semantic search includes selected root and owned explanatory prose.
Pinned BGE inference uses FastEmbed; tokenizer-aware segmentation uses
text-splitter with exact final-input budget and attributable byte coverage.
Eligible roots constrain both lanes before ranking. Candidate windows are bounded
and reported honestly.

Content preparation includes GM prose and DCs. Index locale defaults to English
and can be overridden; changing search locale requires rebuilding. Display uses
the artifact locale. The CLI formats prepared HTML at read time; the UI handles
narrow supported controls without executing Foundry scripts.

## Completion gate

Land this as one complete PR against the dedicated integration branch, with
cohesive commits after validation. Replace writer, readers, retrieval, runtime,
app contracts, CLI and web consumers together; remove old authorities and callers.
Saved lists and encounters remain separate durable local state. Preserve saved
keys/snapshots, HP edit intent and explicit overrides. New HP uses a known integral
effective typed maximum; missing capacity remains unknown.

Require checked snapshot fidelity for the complete pinned corpus, projection
oracles and query truth tables, cache/relationship/vector corruption rejection,
atomic publication and bounded reader evidence. Confirm real-model judged
retrieval, actual CLI output, browser detail/owned/passage routes and local-state
workflows. Run the Rust and frontend gates, independent plan/completion and
regression/evidence reviews, and update current architecture documentation.
Owner tests alone do not establish completion.

## References

- [Current artifact contract](../../../architecture/artifact-contract.md)
- [ADR 0046: source-backed artifact](../../../architecture/decisions/0046-source-backed-reference-artifact.md)
- [Completed source contract work](./rust-source-contract-generation.md)
- [Query projection research](../../../research/source-record-query-projections.md)
- [Source enrichment evidence](../../../research/source-record-enrichment.md)
- [Search quality follow-up](../../items/rust-search-quality-tuning.md)
