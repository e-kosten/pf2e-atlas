# Historical snapshot: derived-tag-ontology-shape-primer.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/derived-tag-ontology/derived-tag-ontology-shape-primer.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Derived-Tag Ontology Shape Primer

## What This Note Is

This is the short reminder version of the ontology reshaping work.

Use this when you want a quick refresher on:

- what future ontology shape we were pushing toward
- why we were doing it
- what the major concept types mean
- how real current tags would map into the future model

For the deeper source material, also see:

- [Derived Tag Canonical Concept Model](./derived-tag-concept-model.md)
- [Derived Tag Concept Mapping Summary](./derived-tag-concept-mapping-summary.md)
- [Derived Tag Concept Mapping CSV](./derived-tag-concept-mapping.csv)

## What We Were Trying To Fix

The goal was to improve the ontology shape before going deeper on assignments.

This work was about:

- making sure the ontology itself is retrieval-oriented
- making sure we can express the Pathfinder search/use cases we care about
- aligning semantically identical concepts across categories
- separating browse structure from semantic meaning

This was not primarily about:

- whether current assignments are balanced
- whether every content area is tagged yet
- whether legacy naming is cleaned up

The intended sequence was:

1. fix the ontology shape
2. finish ontology review on the remaining mixed families
3. then implement the source-of-truth model in `src/tags/`
4. then resume large-scale assignment work on top of that cleaner shape

## Future Shape In One Sentence

Keep the current user-facing category tag surface, but put a canonical semantic concept layer underneath it, then project those concepts into each category's browse tree.

## Main Concept Types

### `DomainConcept`

The noun/problem anchor used mostly by operational concepts.

Examples:

- `poison`
- `disease`
- `hidden_source`
- `parasite`
- `ritual_grievance`

These let related operations share one semantic target, such as:

- `poison_application`
- `poison_remediation`

### `OperationalConcept`

"Do X to Y" concepts.

These are for answer paths, problem application, discovery, containment, cleanup, and similar semantic actions.

Examples:

- `poison_remediation`
- `source_discovery`
- `parasite_removal`
- `ritual_appeasement`
- `outbreak_containment`

Typical bounded operations:

- `discover`
- `remediate`
- `remove`
- `contain`
- `clean_up`
- `counteract`
- `expel`
- `appease`
- `disarm`
- `bypass`

### `DescriptiveConcept`

Non-operational meaning, usually classified by facets rather than by many separate top-level classes.

Examples:

- `forest_setting`
- `body_horror`
- `guardian_npc`
- `brute_combatant`
- `inhaled_exposure`
- `delayed_onset`
- `sinspawn_family`

Approved descriptive facet kinds included:

- `setting`
- `theme`
- `creature_family`
- `role`
- `delivery`
- `progression`
- `behavior_override`
- `response_demand`
- `challenge_structure`
- `capability`
- `effect`
- `function`
- `mechanism`
- `pathogenesis`

### `AggregateConcept`

Meaningful umbrella concepts that users may still want to filter on directly.

Examples:

- `communication`
- `infiltration`
- `reconnaissance`
- `problem_discovery`
- `problem_resolution`

These are not fake browse headers. If an umbrella is exposed as a real filterable tag, it should have canonical backing.

### `ConceptRelation`

The semantic edge between concepts.

Examples:

- `poison_remediation` `counteracts` `poison_application`
- `source_discovery` `enables` `source_cleanup`
- `organized_undead_society_setting` `evokes` `courtly_undeath`

### `CategoryProjection`

How a canonical concept shows up inside a category.

A projection owns:

- category
- `browseAxis`
- `family`
- description
- assignment guidance

A projection does not own naming. The canonical concept owns the shared label.

## Browse Structure vs Semantic Structure

One of the biggest shifts was separating browse structure from semantic meaning.

### Browse Structure

These are user-facing navigation tools:

- `browseAxis`
- `family`

Examples of `browseAxis`:

- `encounter`
- `npc_role`
- `disease_model`
- `behavior`
- `support`
- `setting`

Examples of `family`:

- `npc_role / scene_role`
- `encounter / combat_role`
- `disease_model / delivery_profile`

These are projection-local. They help users find things, but they do not define canonical meaning.

### Semantic Structure

This is the canonical layer:

- domain concepts
- canonical concepts
- relations between concepts

This is where cross-category consistency comes from.

## The Main Naming Rule

One canonical concept owns one shared label.

That means:

- projections do not rename concepts
- cross-category awkwardness should force us to improve the canonical label
- legacy compatibility belongs in `src/tags/legacy-*`, not in the canonical model

## Today vs Future Examples

### Example: Poison

Today:

- `creature/threat_profile/poison_threat`
- `spell/support/anti_poison`
- `equipment/function/anti_poison`

Future:

- `DomainConcept: poison`
- `OperationalConcept: poison_application`
- `OperationalConcept: poison_remediation`
- relation: `poison_remediation counteracts poison_application`

Then each category gets projections of those shared concepts.

Meaning:

- creature-side poison is the problem/application side
- spell-side and equipment-side poison help are the remediation side
- they no longer need separate category-local semantic tags

### Example: Source Discovery

Today:

- `affliction/response_profile/source_tracing`
- `spell/revelation/source_revelation`

Future:

- `DomainConcept: hidden_source`
- `OperationalConcept: source_discovery`

Meaning:

- different category-local wording normalizes into one shared discovery concept

### Example: Curse Discovery and Cure

Today:

- `spell/revelation/curse_revelation`
- `spell/resolution/curse_removal`

Future:

- `DomainConcept: curse`
- `OperationalConcept: curse_discovery`
- `OperationalConcept: curse_remediation`

Meaning:

- discovery-side naming becomes systematic
- answer-side keeps a bounded but specific operation noun instead of flattening everything into `resolution`

### Example: Outbreak Management

Today:

- `affliction/response_profile/outbreak_management`
- `spell/resolution/quarantine_containment`

Future:

- `DomainConcept: outbreak`
- `OperationalConcept: outbreak_containment`

Meaning:

- we dropped the vaguer `outbreak_response` direction
- the clearer operational concept is containment

### Example: Regional Setting

Today:

- `creature/regional_setting/organized_undead_society_setting`

Future:

- `DescriptiveConcept: organized_undead_society_setting`
- primary facet: `setting`

Meaning:

- this stays setting-first even when it carries strong thematic weight

### Example: Creature Family

Today:

- `creature/ontology_cluster/sinspawn_family`
- `creature/bound_object/possessed_object`

Future:

- `DescriptiveConcept: sinspawn_family`
- `DescriptiveConcept: possessed_object`
- primary facet: `creature_family`

Meaning:

- these moved out of awkward placeholder buckets
- they now live in a real semantic creature-family space

### Example: Aggregate Umbrellas

Today:

- `spell/communication`
- `spell/revelation`

Future:

- `AggregateConcept: communication`
- `AggregateConcept: problem_discovery`

Meaning:

- meaningful umbrella tags stay real
- they are not just local browse headers

## Key Resolved Design Decisions

These had already mostly settled in the review:

- `regional_setting` stays as a setting-first concept space
- `bound_object` and `ontology_cluster` map into `creature_family`
- `outbreak_response` was dropped in favor of `outbreak_containment`
- `epidemiological_profile` is a heterogeneous browse family under `disease_model`
- exposed composite tags are canonical `AggregateConcept`s
- `beneficial` should be dropped
- `equipment/function` should split across remediation/support/capability
- `ammunition_payload` should split across existing facet kinds
- `creature/threat_profile` should stop overusing `*_application`

## The Remaining Review Areas

At the last planning pass, the main remaining ontology-review families were:

- `spell/support`
- `spell/control`
- `hazard/countermeasure_profile`
- `hazard/environmental_danger`
- `equipment/anti_magic`

Those were the areas still needing another semantic validation pass before the model could be treated as fully settled.

## The Long-Term Implementation Target

The long-term target was not just a better scratch mapping.

It was:

1. define real tracked source-of-truth schema for canonical concepts and projections
2. move that ownership into `src/tags/`
3. surface canonical metadata in the TUI and editorial tooling
4. keep published runtime tags flattened if helpful, but compile them from the canonical model

So the real architectural shift was:

- from flat category-local tag definitions
- to canonical concepts plus category projections

## Best Existing Deeper References

If you want the canonical source docs rather than this refresher:

- [Derived Tag Canonical Concept Model](./derived-tag-concept-model.md)
  This is the main design note.

- [Derived Tag Concept Mapping Summary](./derived-tag-concept-mapping-summary.md)
  This shows the planned schema split and sample canonical renames.

- [Derived Tag Concept Mapping CSV](./derived-tag-concept-mapping.csv)
  This is the detailed row-by-row planning artifact.
````
