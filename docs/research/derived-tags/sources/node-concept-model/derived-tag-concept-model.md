# Historical snapshot: derived-tag-concept-model.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `scratch/derived-tag-ontology/derived-tag-concept-model.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Derived Tag Canonical Concept Model

## Summary

This note captures the current preferred long-term shape for derived tags:

- keep the current user-facing tag surface as category-local tags
- introduce a canonical semantic layer underneath those tags
- split today's flat tag definitions into:
  - canonical concepts
  - category projections
- keep category browse structure, but make it explicitly user-facing rather than semantic

The key refinement from earlier drafts is that the canonical model should stay small:

- `OperationalConcept`
- `DescriptiveConcept`
- `AggregateConcept`
- `DomainConcept`
- `ConceptRelation`
- `CategoryProjection`

Rather than inventing many top-level concept kinds, the non-operational space is represented through structured facets on `DescriptiveConcept`.

## Core Model

### Canonical Layers

#### DomainConcept

Shared noun/problem anchors used primarily by `OperationalConcept`s.

Examples:

- `poison`
- `disease`
- `hidden_source`
- `parasite`
- `ritual_grievance`

These are useful when multiple operations need explicit semantic linkage, such as:

- `poison_application`
- `poison_remediation`

#### CanonicalConcept

Canonical concept nodes that represent reusable ontology meaning.

Preferred concept kinds:

- `OperationalConcept`
  - best for "do X to Y"
  - usually references a `DomainConcept`
  - examples: `poison_remediation`, `source_discovery`, `parasite_removal`, `ritual_appeasement`
- `DescriptiveConcept`
  - best for non-operational concepts
  - uses structured facets rather than many separate top-level kinds
  - examples:
    - `forest_setting`
    - `body_horror`
    - `guardian_npc`
    - `brute_combatant`
    - `inhaled_exposure`
    - `delayed_onset`
- `AggregateConcept`
  - best for meaningful filterable umbrella concepts that group narrower canonical concepts
  - examples: `communication`, `infiltration`, `reconnaissance`, `problem_discovery`, `problem_resolution`

#### ConceptFacet

Structured semantic classifiers used primarily by `DescriptiveConcept`, and optionally by `OperationalConcept` where useful.

Current approved facet kinds:

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

The important design point is that these are semantic facets, not browse organization.

#### ConceptRelation

Explicit semantic edges between canonical nodes.

Examples:

- `poison_remediation` `counteracts` `poison_application`
- `source_discovery` `enables` `source_cleanup`
- `organized_undead_society_setting` `evokes` `courtly_undeath`
- `magic_blight_wasteland_setting` `evokes` `arcane_devastation`

### User-Facing Layer

#### CategoryProjection

Category-local authored tag surfaces derived from canonical concepts.

A projection owns:

- category
- `browseAxis`
- family placement
- category-local description
- assignment behavior
- category-local guidance such as `appliesWhen`, `doesNotApplyWhen`, and `adjacentTags`

This means the projection is the long-term replacement for today's flat per-category tag object.
The projection does not own naming. Canonical concepts own the shared label, and that label applies universally across projections.
Composite umbrella tags should only exist when they are meaningful filter targets. In those cases they should be canonical aggregate concepts, not projection-only pseudo-tags.

#### BrowseAxis

`browseAxis` is the user-facing top-level retrieval lens inside a category.

Examples:

- `encounter`
- `npc_role`
- `disease_model`
- `behavior`
- `support`
- `setting`

`browseAxis` is retained because it answers a different question than canonical semantics:

- canonical layer: "what is this concept?"
- browse layer: "where does a user look for it in this category?"

#### ProjectionFamily

`family` remains the finer user-facing grouping under a `browseAxis`.

Examples:

- `npc_role / scene_role`
- `npc_role / social_role`
- `encounter / combat_role`
- `disease_model / delivery_profile`
- `disease_model / progression_profile`

Families are projection-local. They do not define canonical meaning.

### Published Runtime Layer

The runtime and TUI can still consume flattened published tag records. The canonical model changes the authored source model, not the browsing surface.

Pure browse organization should live in `browseAxis` and `family`, not in standalone tag records. If a composite tag is exposed as a real filterable tag, it should have canonical backing.

Compile flow:

1. Author canonical concepts and relations.
2. Author category projections.
3. Compile projections into the same flat published tag records the runtime already expects.

This preserves the existing browsing and assignment workflow while improving semantic consistency underneath.

## Type Shape

```ts
type CanonicalConcept =
  | OperationalConcept
  | DescriptiveConcept
  | AggregateConcept;

type ConceptText = {
  definition: string;
  examples?: string[];
  nonExamples?: string[];
  pathfinderContext?: string;
};

type DomainConcept = {
  id: string;
  domainClass: string;
  text: ConceptText;
};

type CanonicalConceptBase = {
  id: string;
  label: string;
  text: ConceptText;
  projections: CategoryProjection[];
};

type OperationalConcept = CanonicalConceptBase & {
  schemaKind: "operational";
  domainId: string;
  operation: string;
  facets?: ConceptFacet[];
};

type DescriptiveConcept = CanonicalConceptBase & {
  schemaKind: "descriptive";
  primaryFacet: ConceptFacet;
  facets: ConceptFacet[];
};

type AggregateConcept = CanonicalConceptBase & {
  schemaKind: "aggregate";
  memberConceptIds: string[];
};

type ConceptFacet =
  | { kind: "setting"; settingType: "habitat" | "site" | "regional" | "planar" }
  | { kind: "theme"; themeType: string }
  | { kind: "creature_family"; familyType: string }
  | { kind: "role"; roleType: "scene" | "social" | "institutional" | "tactical" }
  | { kind: "delivery"; deliveryType: string }
  | { kind: "progression"; progressionType: string }
  | { kind: "behavior_override"; overrideType: string }
  | { kind: "response_demand"; demandType: string }
  | { kind: "challenge_structure"; structureType: string }
  | { kind: "capability"; capabilityType: string }
  | { kind: "effect"; effectType: string }
  | { kind: "function"; functionType: string }
  | { kind: "mechanism"; mechanismType: string }
  | { kind: "pathogenesis"; pathogenesisType: string };

type CategoryProjection = {
  conceptId: string;
  category: SearchCategory;
  browseAxis: string;
  family: string;
  description: string;
  assignmentMode: DerivedTagAssignmentMode;
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  adjacentTags?: string[];
};

type ConceptRelation = {
  fromConceptId: string;
  relation:
    | "counteracts"
    | "applies"
    | "requires"
    | "enables"
    | "evokes"
    | "overlaps_with"
    | "specializes";
  toConceptId: string;
};
```

## Why Facets Improve The Model

Earlier drafts were heading toward a larger set of top-level kinds:

- `OperationalConcept`
- `SettingConcept`
- `ThemeConcept`
- `ActivityConcept`
- `MechanismConcept`

That structure was becoming unstable because some splits felt crisp and some did not.

The facet refactor gives us:

- a small, stable top-level concept model
- clearer separation between canonical semantics and browse organization
- support for multi-dimensional descriptive concepts
- less pressure to invent new top-level concept kinds whenever the ontology expands

Examples:

- `magic_blight_wasteland_setting`
  - `schemaKind: "descriptive"`
  - primary facet: `setting`
  - secondary facet: `theme`
- `guardian_npc`
  - `schemaKind: "descriptive"`
  - primary facet: `role`
  - `roleType: "scene"`
- `brute_combatant`
  - `schemaKind: "descriptive"`
  - primary facet: `role`
  - `roleType: "tactical"`
- `inhaled_exposure`
  - `schemaKind: "descriptive"`
  - primary facet: `delivery`
- `delayed_onset`
  - `schemaKind: "descriptive"`
  - primary facet: `progression`
- `compulsion`
  - `schemaKind: "descriptive"`
  - primary facet: `behavior_override`
- `community_outbreak`
  - `schemaKind: "descriptive"`
  - primary facet: `response_demand`
- `sinspawn_family`
  - `schemaKind: "descriptive"`
  - primary facet: `creature_family`
- `observation_driven`
  - `schemaKind: "descriptive"`
  - primary facet: `challenge_structure`

The strong conceptual split is:

- `OperationalConcept`
- `DescriptiveConcept`

The finer distinctions then live in facets.

## Naming Rules

- Canonical concept ids should prefer the normalized long-form names we actually want to standardize on.
- Canonical concepts own the shared user-facing label.
- Projections do not override labels. If a label is wrong or awkward in one category, that is evidence the canonical label itself needs improvement.
- This means concepts like `poison_remediation` should replace category-local semantic duplicates like `anti_poison`.
- The approved direction is to prefer one shared user-facing name where the same concept appears across categories. For example, `outbreak_containment` should replace category-local wording variants rather than preserving both `outbreak` and `quarantine` language for the same concept.
- A canonical label may differ from the canonical id, but only at the canonical concept level. If that happens, the same canonical label still applies across every projection.
- Legacy compatibility naming should live only in `src/tags/legacy-*`, not in the canonical ontology model.

## Operation Vocabulary

- Operational canonical ids should be derived from `(domain, operation)` rather than manually named one-off.
- Discovery-side concepts should normalize to `*_discovery` rather than mixing `revelation`, `reveal`, and `detection` nouns for the same semantic pattern.
- Answer-side concepts should choose from a bounded set of canonical operations rather than collapsing everything into `*_resolution`.
- If two concepts use the same operation, they should use the same derived noun form.
- If a concept needs a new operation, adding it should be explicit and justified.

Current approved starter operation set:

- `discover` -> `*_discovery`
- `remediate` -> `*_remediation`
- `remove` -> `*_removal`
- `contain` -> `*_containment`
- `clean_up` -> `*_cleanup`
- `counteract` -> `*_counteraction`
- `expel` -> `*_expulsion`
- `appease` -> `*_appeasement`
- `sanctify` -> `*_sanctification`
- `resolve` -> `*_resolution`
- `disarm` -> `*_disarm`
- `bypass` -> `*_bypass`

This is intentionally a bounded vocabulary, not a universal suffix rule. The goal is to constrain synonymous drift while still preserving meaningful distinctions between different answer paths.

## Where Current Browse Structures Land

### Rule of Thumb

- `schemaKind` describes the broad canonical shape.
- `facet.kind` describes the semantic dimension of the concept.
- `browseAxis` is the top-level retrieval lens inside a category.
- `family` is the finer browse grouping inside that axis.

`browseAxis` and `family` are retained, but projection-local.

### Creature / encounter / *

Current families under `creature / encounter` split across descriptive facet kinds:

- `combat_role`
  - mostly `DescriptiveConcept` with `role`
  - `roleType: "tactical"`
  - examples: `brute_combatant`, `controller_combatant`, `artillery_combatant`
- `cohort_role`
  - mixed `role`, `theme`, and sometimes `challenge_structure`
  - examples:
    - `patrol_member`: role-oriented
    - `warband_member`: theme + role
    - `cult_member`: theme + role
    - `pack_hunter`: tactical or challenge-structure flavored

Recommendation:

- keep `encounter` as a `browseAxis`
- keep `combat_role` and `cohort_role` as families
- classify the canonical concepts underneath with facets rather than assuming the family itself is a canonical kind
- treat `threat_profile` as a heterogeneous operational family
- use `*_application` only for threats that actually inflict recurring states/effects on victims
- use more specific operations where needed, such as `ambush_grab`, `prey_control`, `terrain_control`, `spawn_creation`, `regeneration`, `reinforcement`, and `infiltration`

### Creature / npc_role / *

These are mostly `DescriptiveConcept`s with `role` facets.

- `scene_role`
  - examples: `guardian_npc`, `watcher_npc`, `escort_npc`, `captive_npc`
- `social_role`
  - examples: `profession_npc`, `authority_npc`, `religious_npc`, `criminal_npc`, `merchant_npc`

Recommendation:

- retain `npc_role` as a `browseAxis`
- retain `scene_role` and `social_role` as families
- model most of these tags canonically as `DescriptiveConcept + role facet`

### Creature / specialization / *

These families remain mixed, but the old placeholder treatment for `ontology_cluster` and `bound_object` is no longer preferred.

- `ontology_cluster`
  - `DescriptiveConcept` with `creature_family` facets
  - examples: `undead_family`, `sinspawn_family`
  - these are retrieval-real creature buckets even when native PF2E taxonomy does not express them cleanly
- `bound_object`
  - `DescriptiveConcept` with `creature_family` facets
  - examples: `animated_object`, `animated_statue`, `possessed_object`
  - these are embodied/bound-form creature families rather than freeform themes
- `casting_profile`
  - mixed:
    - `arcane_spellcaster`, `divine_spellcaster`: likely `role`- or theme-adjacent descriptive concepts
    - `ritualist_creature`: role or theme depending on the final mapping
    - `dragon_spellcaster`: multi-facet descriptive concept
- `corruption_profile`
  - mostly `theme`-heavy descriptive concepts, sometimes adjacent to operational domains
  - examples: `blight_tainted`, `fungal_infested`, `plaguebearing`

Recommendation:

- retain `specialization` as a `browseAxis`
- retain these families as projection organization
- treat `ontology_cluster` and `bound_object` as projections into a shared `creature_family` semantic space
- expect the rest of these families to contain multiple facet types

### Affliction / behavior / *

These are mostly `DescriptiveConcept`s with `behavior_override` facets.

Examples:

- `compulsion`
- `violence_compulsion`
- `truth_compulsion`
- `self_destructive_impulse`

Recommendation:

- retain `behavior` as a `browseAxis`
- keep `behavioral_override` as a family
- model most tags here as descriptive concepts with `behavior_override` facets

### Affliction / disease_model / *

These families split across several descriptive facet types:

- `pathogenesis`
  - mostly `theme`
  - examples: `rot_decay`, `blood_rot`, `fungal_growth`, `bestial_transformation`
- `epidemiological_profile`
  - intentionally heterogeneous across existing descriptive facet kinds
  - examples:
    - `community_outbreak`
    - `carrier_vector`
    - `epidemic_pestilence`
- `response_profile`
  - mixed `response_demand` and operational demand-side projections
  - examples:
    - `community_outbreak`
    - `time_critical_resolution`
    - `outbreak_containment`
    - `source_discovery`
- `delivery_profile`
  - mostly `delivery`
  - examples: `inhaled_exposure`, `contact_exposure`, `waterborne_exposure`
- `progression_profile`
  - mostly `progression`
  - examples: `delayed_onset`, `recurrent_flare`, `cumulative_transformation`

Recommendation:

- retain `disease_model` as a `browseAxis`
- retain the current families
- treat `epidemiological_profile` as a heterogeneous browse family rather than a single semantic kind
- classify tags canonically by descriptive facets rather than by axis/family names, with approved operational exceptions such as `source_discovery` and `outbreak_containment`

### Equipment / effect and item_mechanical / *

These browse families also remain useful for navigation even when their canonical semantics split across existing kinds.

- `function`
  - heterogeneous across remediation operations, support operations, and capability concepts
  - `beneficial` should be dropped rather than normalized because it is too vague to remain a stable retrieval concept
- `ammunition_payload`
  - heterogeneous across `delivery` payload concepts and capability-style targeting specialization
  - examples:
    - `elemental_payload`, `explosive_payload`, `spell_payload` -> `delivery`
    - `creature_bane` -> `capability`

Recommendation:

- retain the current browse families for now
- split canonical meaning row-by-row across existing facet kinds and operational concepts
- revisit the browse family layout later if these heterogeneous buckets remain awkward in the TUI

### Approved Problem-Space Refactor

The old exploratory `problem_shape` bucket is no longer the preferred structure.

Approved replacement:

- `behavior_override`
  - examples: `compulsion`, `violence_compulsion`, `truth_compulsion`, `self_destructive_impulse`
- `response_demand`
  - examples: `community_outbreak`, `time_critical_resolution`
- `challenge_structure`
  - examples: `observation_driven`, `timing_window`, `multi_stage_resolution`, `endurance_pressure`
- `creature_family`
  - examples: `sinspawn_family`, `undead_family`, `animated_object`
- operational exceptions:
  - `source_discovery`
  - `outbreak_containment`

Approved renames in this subspace:

- `outbreak_management` -> `outbreak_containment`
- `cure_clock_urgency` -> `time_critical_resolution`
- `observation_first` -> `observation_driven`
- `layered_resolution` -> `multi_stage_resolution`
- `quarantine_containment` -> `outbreak_containment`

## Assignment Model

Assignment behavior should stay on projections, not concepts.

Reason:

- the same concept can have different assignment behavior in different categories
- the current runtime already expects category-local assignment metadata

Projection-owned fields should include:

- `assignmentMode`
- `appliesWhen`
- `doesNotApplyWhen`
- `adjacentTags`
- category-local exemplars if we continue to attach them at tag level

Example:

- `poison_remediation` concept
  - spell projection: `assignmentMode = "hybrid"`
  - equipment projection: `assignmentMode = "deterministic"`

## TUI Impact

The TUI should remain projection-first.

The user still browses:

- category
- `browseAxis`
- family
- tag

The detail view can then show canonical context:

- `schemaKind`
- canonical definition
- primary and secondary facets
- related concepts
- sibling projections in other categories

That gives us semantic richness without making the TUI abstract or harder to use.

## Mapping Pass Next

The next follow-up artifact should be a tag-by-tag mapping table from the current ontology to the proposed model, including:

- current category / `browseAxis` / family / tag
- proposed canonical concept kind
- proposed canonical concept id
- proposed canonical concept label
- `DomainConcept`, if any
- primary facet
- secondary facets, if any
- notes on coverage or gaps

That pass should validate whether the proposal covers the current ontology cleanly and identify any tags that need:

- concept renaming
- concept splitting
- concept merging
- facet refinement
````
