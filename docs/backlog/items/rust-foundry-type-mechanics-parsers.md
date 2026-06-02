# Rust Foundry Type Mechanics Parsers

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-06-01

## Problem

`SpellMechanics` now models spell-specific Foundry `system` facts as a coherent type-level shape. Other Foundry item and actor types expose comparable type-specific facts mostly through open metrics, such as weapon, armor, shield, speed, sense, and disable values.

Metrics are useful for scalar filtering and comparison, but they are not always the clearest in-memory shape for source facts that naturally belong to a Foundry type. Without peer type-level parsers, the record model can make spells look uniquely structured while other Foundry types remain metric-only even when their source data has a coherent hierarchy.

## Desired Outcome

Evaluate whether common Foundry `type` values deserve peer mechanics parsers and typed record subparts alongside, or before, metric projection.

Candidate shapes should preserve both roles:

- typed mechanics own coherent source facts for a Foundry type;
- metrics remain the generic comparison/filter surface for scalar values;
- shared extraction avoids parsing the same raw JSON field independently for typed mechanics and metric rows.

## Candidate Areas

- weapon mechanics: range increment, reload, damage dice, damage die faces, traits, usage-relevant facts
- armor mechanics: AC bonus, Dexterity cap, Strength requirement, check penalty, speed penalty
- shield mechanics: AC bonus, hardness, HP, broken threshold
- actor movement mechanics: speed types and speed values
- actor senses mechanics: sense names and ranges
- hazard disable mechanics: disable text, skills, DCs, and rank requirements

## Constraints

- Do not convert every metric into typed record data by default.
- Prefer typed mechanics only where the Foundry source data forms a stable, meaningful sub-shape.
- Keep metrics as the durable scalar comparison surface unless a later design explicitly replaces that contract.
- Avoid duplicated raw JSON parsing between typed mechanics and metrics.

## Acceptance Sketch

- Inventory current metric-only mechanics by Foundry document/type.
- Identify which mechanics should remain metrics-only and which deserve typed parsers.
- Propose record-shape changes for accepted peer parsers.
- Update ingest tests to prove typed mechanics and metric rows derive from the same source interpretation.

## Related

- [Rust Side Data And Metric Source Fact Convergence](./rust-side-data-metric-source-fact-convergence.md)
