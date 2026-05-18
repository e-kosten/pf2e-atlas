# Rust Graph Context Deeper Local Graph

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-18

## Problem

The first Rust graph context retrieval slice is intentionally limited to a bounded one-hop neighborhood around known record keys. That gives agents a trustworthy local context bundle with seed records, direct outgoing references, opt-in backlinks, and edge evidence.

That V1 shape does not answer deeper local graph questions that are likely useful once the baseline graph command exists:

- Which secondary records are worth reading because multiple direct neighbors point at them?
- Which backlinks are more relevant because they also connect to other local nodes?
- Which nearby records form a dense local cluster around the seed?
- Which generic high-degree records should be suppressed because they add little context?

Obsidian's local graph is a useful reference point: the value is not the global graph, but a focused neighborhood around the active note. PF2E Atlas should eventually support a similar local-structure view for records, but the first graph command should not ship with unvalidated depth and ranking behavior.

## Desired Outcome

Extend graph context retrieval beyond one-hop adjacency while preserving clear provenance and deterministic behavior.

Potential capabilities:

- `--depth 2` or an equivalent explicit secondary-neighbor mode.
- Secondary-neighbor groups that stay separate from direct outgoing/backlink evidence.
- Shared-neighbor scoring for records connected through multiple local paths.
- Mutual-link boosts for records that link back to the seed or to multiple direct neighbors.
- Degree-aware curation that suppresses generic hubs unless directly linked or explicitly requested.
- Local-cluster summaries that explain "also connected through" relationships without synthesizing rules answers.
- A future Rust TUI local graph/navigation pane powered by the same graph context service.

## Constraints

- Do not mix direct edge evidence with inferred secondary relevance in the same output group.
- Do not add opaque graph scores without explanations.
- Do not use naive global degree or centrality ranking as the main relevance signal; common PF2E hubs such as traits, conditions, and generic actions can dominate otherwise.
- Cap secondary retrieval aggressively and expose truncation by depth and direction.
- Preserve the V1 graph command's role as retrieval only. Deeper graph context should not become answer synthesis.
- Keep search relationship filters separate from graph context retrieval. `atlas search --references` and `--referenced-by` remain result-set filters, not local graph commands.

## Acceptance Sketch

- Secondary retrieval is opt-in and cannot appear in default `atlas graph get` output.
- JSON output distinguishes direct neighbors from secondary/derived local graph results.
- Every secondary result includes a deterministic explanation, such as "referenced by two direct neighbors" or "links back to the seed."
- Tests cover at least one high-degree hub case where naive expansion would over-include noisy records.
- Architecture docs describe how deeper graph retrieval builds on the V1 graph context service without bypassing `atlas-index` and `AtlasRetrievalService`.
