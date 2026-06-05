# Rust CLI Content Output Formats

Status: deferred

## Context

The Rust CLI now exposes structured presentation-content blocks derived from canonical `RichDocument` values and renders terminal output directly from that DTO. Future consumers may still need additional output formats tailored to automation, source auditing, or web presentation.

There are plausible future consumers for more than one content representation:

- structured `RichDocument` JSON for source-preserving audit/debug use
- structured presentation-content JSON for user-facing record output
- original Foundry rich text or source document fragments
- plain text projections for simple terminal and shell use

## Desired Outcome

Decide which non-default content formats the Rust CLI should expose, how callers select them, and whether optional output should represent canonical `RichDocument` structure, presentation content, plain text, or Foundry-origin format where available.

The default record output should remain stable while adding any optional content format.
