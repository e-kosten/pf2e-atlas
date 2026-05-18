# Rust Creature Level Filtering

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-05-18

## Problem

The Rust Atlas CLI accepts shared level convenience filters such as `--level`, `--min-level`, and `--max-level`, but creature records currently do not participate correctly in those predicates. A common query such as `atlas search --family creature --max-level 8` can return zero records even though creature level is a core Pathfinder lookup dimension.

This is a product correctness problem rather than an agent instruction issue. Agents and users should be able to trust shared level filters across record families where level is meaningful.

## Desired Outcome

Creature level filters work through the same convenience flags and canonical filter path used by other level-bearing records.

The fix should cover:

- `--level`, `--min-level`, and `--max-level` for creature records.
- Filter discovery metadata that makes creature level visible when inspecting creature fields.
- JSON filter lowering that exposes the intended predicate rather than a misleading empty result path.
- Regression coverage for creature level queries, including at least one bounded range and one max-level query.

## Constraints

- Preserve the shared query/filter contract instead of adding a creature-only CLI flag.
- Do not paper over missing indexed data with presentation-layer fallbacks.
- Keep the level source owned by the ingest/runtime artifact model that owns creature facts.

## Notes

Observed during Atlas CLI skill evaluation: `atlas search --family creature --max-level 8 --detail summary --limit 5 --json` returned an empty result set with a lowered `level <= 8` metadata predicate.
