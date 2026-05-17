# Rust CLI Content Output Formats

Status: deferred

## Context

Phase 5 of the Rust CLI runtime migration projects `ContentDocument` blocks to markdown in record JSON output. That keeps the first durable record DTO simple while preserving the canonical rich content model internally.

There are plausible future consumers for more than one content representation:

- structured `ContentDocument` JSON owned by the Rust content model
- original Foundry rich text or source document fragments
- plain text projections for simple terminal and shell use

## Desired Outcome

Decide which non-markdown content formats the Rust CLI should expose, how callers select them, and whether the output should represent internal `ContentDocument` structure or preserve Foundry-origin format where available.

The default record output should remain stable while adding any optional content format.
