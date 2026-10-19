# Historical snapshot: derived-tag-ontology-glossary.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/derived-tag-ontology/derived-tag-ontology-glossary.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Derived-Tag Ontology Glossary

## What This Note Is

This is a compact reference for the approved ontology vocabulary.

Use it when you want to quickly remember:

- what each concept type means
- what the descriptive facet kinds are for
- what the approved operational vocabulary is doing
- what the common relation types mean

For the fuller design context, also see:

- [Derived Tag Ontology Shape Primer](./derived-tag-ontology-shape-primer.md)
- [Derived Tag Canonical Concept Model](./derived-tag-concept-model.md)

## Core Concept Types

### `DomainConcept`

The noun/problem anchor that operational concepts act on.

Examples:

- `poison`
- `disease`
- `hidden_source`
- `parasite`
- `outbreak`

### `OperationalConcept`

A concept shaped like "do X to Y."

Examples:

- `poison_remediation`
- `source_discovery`
- `parasite_removal`
- `outbreak_containment`
- `hostile_presence_expulsion`

### `DescriptiveConcept`

A non-operational concept described through one or more semantic facets.

Examples:

- `forest_setting`
- `body_horror`
- `sinspawn_family`
- `guardian_npc`
- `inhaled_exposure`

### `AggregateConcept`

A meaningful umbrella concept users may filter on directly.

Examples:

- `communication`
- `infiltration`
- `reconnaissance`
- `problem_discovery`
- `problem_resolution`

### `CategoryProjection`

The category-local authored projection of a canonical concept.

A projection owns:

- category
- `browseAxis`
- `family`
- description
- assignment guidance

A projection does not own the semantic label.

### `ConceptRelation`

A semantic edge between canonical concepts.

Examples:

- `counteracts`
- `enables`
- `specializes`
- `evokes`

## Descriptive Facet Kinds

These are semantic classifiers, not browse buckets.

### `setting`

Use for place, environment, or region concepts.

Examples:

- `forest_setting`
- `organized_undead_society_setting`
- `alien_technology_wasteland_setting`

### `theme`

Use for motif, tone, or thematic framing.

Examples:

- `body_horror`
- `folk_horror`
- `courtly_undeath`
- `arcane_devastation`

### `creature_family`

Use for retrieval-real creature buckets that matter even when native PF2E taxonomy is missing or fragmented.

Examples:

- `sinspawn_family`
- `undead_family`
- `possessed_object`
- `animated_statue`

### `role`

Use for scene, social, institutional, or tactical role.

Examples:

- `guardian_npc`
- `priest_npc`
- `brute_combatant`
- `artillery_combatant`

### `delivery`

Use for how an effect, exposure, or payload reaches its target.

Examples:

- `inhaled_exposure`
- `contact_exposure`
- `carrier_vector`
- `explosive_payload`

### `progression`

Use for how a problem unfolds or escalates over time.

Examples:

- `delayed_onset`
- `escalating_progression`

### `behavior_override`

Use for affliction-driven or condition-driven behavior changes.

Examples:

- `violence_compulsion`
- `flight_response`
- `social_withdrawal`

### `response_demand`

Use for descriptive pressure about how burdensome or urgent the response is.

Examples:

- `time_critical_resolution`
- `community_outbreak`

### `challenge_structure`

Use for how a problem is structured from a solving perspective.

Examples:

- `observation_driven`
- `multi_stage_resolution`

### `capability`

Use for broad ability or support value rather than a specific answer-path action.

Examples:

- `translation_support`
- `tracking`
- `forgery_support`
- `magic_protection` if it lands on the capability side

### `effect`

Use for a meaningful effect-state or imposed condition profile.

Examples:

- `mental_impairment`
- `mobility_impairment`
- `action_denial`

### `function`

Use for what a thing functionally does in a scenario, especially on hazards or similar problem-side records.

Examples:

- `alarm`
- `area_denial`
- `barrier_lockdown`
- `guard_post`

### `mechanism`

Use for the operating mechanism or trigger logic.

Examples:

- `tripwire_trigger`
- `pressure_trigger`
- `control_interface`
- `ward_trigger`

### `pathogenesis`

Use for disease or corruption model identity.

Examples:

- `blood_rot`
- `fungal_growth`
- `infestation_implant`
- `petrifying_corruption`

## Operational Vocabulary

These are the bounded action patterns that operational concepts should draw from.

### `discover`

Find, expose, reveal, or identify a hidden problem.

Examples:

- `problem_discovery`
- `source_discovery`
- `curse_discovery`
- `hazard_discovery`

### `remediate`

Treat or neutralize an ongoing harmful condition.

Examples:

- `poison_remediation`
- `curse_remediation`
- `disease_remediation`

### `remove`

Remove a concrete attached or embedded problem.

Examples:

- `parasite_removal`
- `restraint_removal`

### `contain`

Limit spread, isolate a danger, or keep escalation bounded.

Examples:

- `outbreak_containment`

### `clean_up`

Purge, cleanse, or clear harmful residue or source-state.

Examples:

- `source_cleanup`
- `contamination_cleanup`

### `counteract`

Directly oppose an active magical or similar effect.

Examples:

- `active_magic_counteraction`

### `expel`

Force out a possessing or hostile presence.

Examples:

- `hostile_presence_expulsion`

### `appease`

Resolve a problem by ritual or social-spiritual calming.

Examples:

- `ritual_appeasement`

### `sanctify`

Purify or consecrate a tainted place or target.

Examples:

- `sacred_taint_sanctification`

### `disarm`

Disable a hazard or mechanism directly.

Examples:

- `physical_disarm`

### `bypass`

Circumvent rather than disable.

Examples:

- `procedural_bypass`
- `lock_bypass`

## Common Operational Shapes

These are useful recurring patterns, even when the exact operation differs.

### Problem-Side Application

A threat inflicts or applies something harmful.

Examples:

- `poison_application`
- `curse_application`
- `disease_application`
- `petrification_application`

### Discovery-Side Answer

A record helps identify the hidden problem.

Examples:

- `problem_discovery`
- `source_discovery`

### Resolution-Side Answer

A record helps fix, contain, or remove the problem.

Examples:

- `poison_remediation`
- `outbreak_containment`
- `source_cleanup`
- `parasite_removal`

## Relation Vocabulary

### `counteracts`

Use when one concept is the answer to another harmful concept.

Example:

- `poison_remediation counteracts poison_application`

### `enables`

Use when one concept helps make another possible or effective.

Example:

- `source_discovery enables source_cleanup`

### `specializes`

Use when one concept is a narrower member of a broader concept.

Examples:

- `animated_statue specializes possessed_object_family`
- `curse_discovery specializes problem_discovery`

### `evokes`

Use when a setting or descriptive concept strongly carries a thematic association.

Examples:

- `organized_undead_society_setting evokes courtly_undeath`
- `magic_blight_wasteland_setting evokes arcane_devastation`

### `overlaps_with`

Use when two concepts are related but neither cleanly contains the other.

Example:

- a thematic overlap between a setting-heavy concept and a theme-heavy concept

### `applies`

Use when an operational concept inflicts or imposes a harmful state.

Example:

- `poison_application applies poison`

### `requires`

Use when one solving path depends on another prerequisite concept.

Example:

- a procedural answer that requires prior source discovery

## Browse Vocabulary

### `browseAxis`

The top-level user-facing retrieval lens inside a category.

Examples:

- `setting`
- `support`
- `npc_role`
- `disease_model`
- `encounter`

### `family`

The finer browse grouping under a browse axis.

Examples:

- `setting / regional_setting`
- `support / remediation`
- `disease_model / epidemiological_profile`

These are navigation structures, not canonical semantic classes.

## Short Rules Of Thumb

- If it reads like "do X to Y," it is probably operational.
- If it reads like "what kind of thing is this," it is probably descriptive.
- If it is a broad umbrella that users may still want to filter on directly, it may be aggregate.
- If it only helps users browse, it belongs in `browseAxis` or `family`, not as a semantic concept by itself.
- If two categories mean the same thing, they should project the same canonical concept instead of owning separate names.
````
