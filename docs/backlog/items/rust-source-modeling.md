# Rust source types and parsers

Status: in_progress

Complete the pinned serialized source model before expanding family product/UI
modeling. See [ADR 0038](../../architecture/decisions/0038-parser-only-source-modeling.md).

The common Item envelope and ItemSystemSource parser are implemented independently
of the existing ingest pipeline. They share source types across root and
actor-embedded occurrences. Typed common fields cover descriptions, traits,
publication, migration, metadata, ownership and grants. Family-specific systems,
rule elements and effects retain pending source payloads unless refined below.

The shared PhysicalSystemSource parser is also implemented for all eight
physical discriminators, with typed price/coins, HP/bulk, equipment state,
identification, material, size, usage, apex and activations. Physical subitems
recursively share the common, physical and family parsers. Equipment, backpack,
book and treasure have typed system refinements, including container bulk,
book contents/category, treasure groups and observed legacy equipment payloads.
Declared forbidden fields require absence; defaultable source fields preserve
missing/null. Traits and usage retain broad string vocabularies. Rules and
effects remain pending, so these refinements do not complete whole documents.

Remaining work:

- Model armor, consumable, shield and weapon refinements, including specialized
  traits/usage/flags, runes, remaining family admission restrictions and full
  consumable spell children.
- Model ABC (ancestry/background/class), abstract effects, ability/feature schemas,
  other item systems and spell patches. Compare and reuse the existing spell
  payload types where source semantics and fidelity match.
- Model the 42 registered built-in rule schemas and reachable Foundry components;
  preserve unknown custom rule/flag extensions without executing game rules.
- Model common actor components and all eight actor families, including declared
  zero-count loot and party bodies.
- Model JournalEntry/pages, RollTable/results and script/chat Macro sources.
- Resolve observed legacy/derived-looking source fields without silently losing
  them; compare typed shapes with declarations and observed schema/value reports.
- Exercise the pinned corpus through the source-only API and supplement it with
  declaration-based and adversarial fixtures, including absent/embedded-only
  families. Raw retention or a source round trip alone is insufficient evidence.

Known item discriminators are action, affliction, ancestry, armor, background,
backpack, book, campaignFeature, class, condition, consumable, deity, effect,
equipment, feat, heritage, kit, lore, melee, shield, spell, spellcastingEntry,
treasure and weapon. Actor discriminators are army, character, familiar, hazard,
loot, npc, party and vehicle.

Canonical conversion, pipeline adoption/old-parser retirement, database/storage,
metrics, indexing, embeddings and UI remain deferred to the later integration
phase. Source-model completion is relative to PF2e 6.12.4 at
`4cbdaa37d6c33e9519561bae2c59a23e0288cbce`; it does not certify those later phases.
