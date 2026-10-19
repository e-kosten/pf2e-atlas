# Historical snapshot: derived-tag-concept-mapping-summary.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/derived-tag-ontology/derived-tag-concept-mapping-summary.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Derived Tag Concept Mapping Summary

This artifact was generated from the current ontology and contains a row for every covered derived tag.

- Rows mapped: 515
- Output table: [derived-tag-concept-mapping.csv](./derived-tag-concept-mapping.csv)

## Proposed Schema Split

- `aggregate`: 26
- `descriptive`: 323
- `drop`: 1
- `operational`: 165

## Primary Facet Counts

- `behavior_override`: 4
- `capability`: 97
- `challenge_structure`: 4
- `creature_family`: 5
- `delivery`: 14
- `effect`: 24
- `function`: 13
- `mechanism`: 6
- `pathogenesis`: 6
- `progression`: 4
- `response_demand`: 2
- `role`: 35
- `setting`: 60
- `theme`: 49

## Approved Extended Facet Kinds Used By The Mapping

- `capability`: creature/casting_profile/arcane_spellcaster, creature/casting_profile/divine_spellcaster, creature/casting_profile/dragon_spellcaster, creature/casting_profile/occult_spellcaster, creature/casting_profile/primal_spellcaster, creature/casting_profile/ritualist_creature
- `creature_family`: creature/bound_object/animated_object, creature/bound_object/animated_statue, creature/bound_object/possessed_object, creature/ontology_cluster/sinspawn_family, creature/ontology_cluster/undead_adjacent
- `effect`: affliction/impact/action_denial, affliction/impact/cognitive_impairment, affliction/impact/healing_suppression, affliction/impact/mental_impairment, affliction/impact/mobility_impairment, affliction/impact/physical_debilitation
- `function`: hazard/function/alarm, hazard/function/ambush_burst, hazard/function/area_denial, hazard/function/attrition_pressure, hazard/function/barrier_lockdown, hazard/function/forced_separation
- `mechanism`: hazard/mechanism/control_interface, hazard/mechanism/planar_breach, hazard/mechanism/pressure_trigger, hazard/mechanism/threshold_lockdown, hazard/mechanism/tripwire_trigger, hazard/mechanism/ward_trigger
- `pathogenesis`: affliction/pathogenesis/bestial_transformation, affliction/pathogenesis/blood_rot, affliction/pathogenesis/fungal_growth, affliction/pathogenesis/infestation_implant, affliction/pathogenesis/petrifying_corruption, affliction/pathogenesis/rot_decay

## Sample Canonical Renames

- `affliction/antidote_resolution` -> `poison_remediation`
- `affliction/countermagic_resolution` -> `active_magic_counteraction`
- `affliction/cursebreaking_resolution` -> `curse_remediation`
- `affliction/exorcism_resolution` -> `hostile_presence_expulsion`
- `affliction/quarantine_containment_resolution` -> `outbreak_containment`
- `affliction/ritual_appeasement_resolution` -> `ritual_appeasement`
- `affliction/source_cleanup_resolution` -> `source_cleanup`
- `affliction/surgical_extraction_resolution` -> `parasite_removal`
- `affliction/cure_clock_urgency` -> `time_critical_resolution`
- `affliction/outbreak_management` -> `outbreak_containment`
- `affliction/quarantine_risk` -> `outbreak_containment`
- `affliction/source_tracing` -> `source_discovery`
- `creature/ambush_grabber` -> `ambush_grab`
- `creature/curse_threat` -> `curse_application`
- `creature/death_burst_threat` -> `death_burst`
- `creature/disease_vector` -> `disease_application`
- `creature/infiltration_threat` -> `infiltration`
- `creature/life_drain_threat` -> `life_drain_application`
- `creature/petrification_threat` -> `petrification_application`
- `creature/poison_threat` -> `poison_application`

## Notes

- The table intentionally mixes stable mappings with explicit uncertainty notes where the canonical proposal may still need additional refinement.
- Composite umbrella tags are now treated as approved canonical aggregate concepts when they remain exposed as filterable tags.
- The strongest approved cross-category renames in this pass are remediation/discovery/outbreak-containment alignments such as `anti_poison -> poison_remediation`, `revelation/*_revelation/*_reveal/*_detection -> *_discovery`, `source_revelation/source_tracing -> source_discovery`, and `quarantine_containment -> outbreak_containment`.
````
