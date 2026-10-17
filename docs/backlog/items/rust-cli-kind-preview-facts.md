# Rust CLI Kind Preview Facts

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

Human search results show concise identity, classification and matching evidence. Additional scan facts may help callers reject or choose candidates, but their usefulness differs by record kind. Evaluate the smallest useful additions against actual search tasks.

Without kind-specific facts, users and agents must run follow-up `record get --json` calls for records that could have been rejected from the preview list.

## Desired Outcome

Search preview output includes concise, kind-owned fact summaries that help users scan result sets without expanding every candidate.

Candidate preview facts:

- Creature: level, rarity, size, AC, HP, saves or a compact defensive summary.
- Equipment: level, price, bulk, category/type, usage or activation when present.
- Spell: level, traditions, cast/action cost, range, save, duration, and targets when present.
- Hazard: level, complexity, stealth/DC, disable summary, and source.

## Constraints

- Keep result summaries compact and evaluate any added projection/read cost.
- Preserve structured JSON output as the source of truth for agents needing exact fields.
- Reuse the shared application projection for semantic facts; the CLI owns output layout.

## Notes

This is a presentation and scanability improvement, not a retrieval-ranking change.
