# Rust CLI Family Preview Facts

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-05-18

## Problem

Human-readable Atlas search previews show key, name, family, traits, source, and a text excerpt, but the most useful scan facts differ by record family. Creature result previews should expose level and combat stats such as AC or HP. Equipment previews often need price, bulk, category, and activation details. Spell previews already benefit from level and traits but may need action cost, traditions, save, range, and duration more consistently.

Without family-specific facts, users and agents must run follow-up `record get --detail standard --json` calls for records that could have been rejected from the preview list.

## Desired Outcome

Search preview output includes concise, family-owned fact summaries that help users scan result sets without expanding every candidate.

Candidate preview facts:

- Creature: level, rarity, size, AC, HP, saves or a compact defensive summary.
- Equipment: level, price, bulk, category/type, usage or activation when present.
- Spell: level, traditions, cast/action cost, range, save, duration, and targets when present.
- Hazard: level, complexity, stealth/DC, disable summary, and source.

## Constraints

- Keep `summary` compact for table output; richer scan facts belong in `preview` or `description`.
- Preserve structured JSON output as the source of truth for agents needing exact fields.
- Do not duplicate family fact extraction policy in the CLI presentation layer if a runtime record view model should own it.

## Notes

This is a presentation and scanability improvement, not a retrieval-ranking change.
