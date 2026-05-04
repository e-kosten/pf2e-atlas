# Structured Editor Continuation Model Convergence

Status: done
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-04

## Problem

This item is complete. It remains here as durable context for the structured search-editor continuation model that landed on 2026-05-04.

Before this work, the live structured search editor routed similar user intents through multiple continuation and mutation pathways.

Examples that should feel like one interaction family but do not consistently share one host-owned path include:

- `edit clause` on shared-explorer-backed field rows
- `add here` continuation inside an already-active structural block
- grouped field continuation after mixing field families
- returning from explorer-backed flows to the tree
- reopening the structured editor after prior live edits

That split keeps surfacing regressions where one path preserves the intended flat/group-local editor model while another path rewraps the tree, duplicates siblings, or restores focus through a different projection heuristic.

## Desired Outcome

The landed outcome makes the structured search editor use one explicit search-host-owned continuation model for child editing flows while keeping canonical query state live in `SearchRequest`.

That end state should:

- route structured-editor child flows through one search-host continuation seam
- keep canonical mutation and focus restoration search-owned
- preserve one consistent “return to surrounding tree with live edits kept” behavior
- remove duplicated and inconsistent explorer callback handling from feature-local paths
- stop relying on multiple parallel mutation/writeback models for similar continuation intents

## Constraints

- Do not introduce a second long-lived editor or query state model.
- Do not push search-specific tree/block mutation logic down into `src/tui/filter-explorer/`.
- Do not collapse ontology browse and structured search editing into one host/session abstraction.
- Prompt-local draft state remains acceptable for incomplete numeric/text value entry.
- Treat grouped-field and single-node writeback as distinct mutation kinds even if they resume through one host continuation seam.

## Related

- [TUI architecture](../../../architecture/tui.md)
- [ADR 0013: TUI canonical search state and derived editor projections](../../../architecture/decisions/0013-tui-canonical-search-state-and-derived-editor-projections.md)
- [ADR 0015: shared explorer host contract and live group editing](../../../architecture/decisions/0015-shared-explorer-host-contract-and-live-group-editing.md)
- [Structured draft shared explorer session cleanup](./structured-draft-shared-explorer-session-cleanup.md)
