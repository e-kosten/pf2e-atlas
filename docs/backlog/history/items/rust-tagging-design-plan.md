# Rust Tagging Design Plan

## Summary

Design the Rust replacement for derived tags as a typed tagging subsystem for PF2e Atlas records. The goal is not to port the old node implementation. The node worktree is useful for vocabulary mining, old workflow intent, and cautionary examples, but the Rust design should start from current crate ownership, current artifact contracts, record kinds, typed filter discovery, and agent-friendly batch assignment.

This plan starts from the existing backlog item [Rust Derived-Tag Runtime And Editorial Redesign](../../docs/backlog/items/rust-derived-tag-redesign.md). It narrows that broad item into a first execution path for:

- a Rust-owned tag vocabulary model
- persisted applied tag assignments
- CLI endpoints that let agents fetch untagged records and applicable tags
- an automated multi-agent assignment loop with human escalation only for disagreement or insufficient evidence
- search/filter/record presentation integration after the model is accepted

## Architecture Context

Current source-of-truth docs read for this plan:

- `docs/architecture/overview.md`
- `docs/architecture/runtime.md`
- `docs/architecture/artifact-contract.md`
- `docs/architecture/decisions/README.md`
- `docs/architecture/decisions/0026-rust-cli-product-surface.md`
- `docs/backlog/items/rust-derived-tag-redesign.md`

Relevant current constraints:

- `atlas-cli` must remain a presentation/command surface. It can expose tagging commands, but durable tag selection, assignment semantics, artifact reads, and artifact writes belong below the CLI.
- `atlas-domain` can own stable shared DTO vocabulary, but it must not own SQLite schema, raw source parsing, or CLI presentation.
- `atlas-record` owns storage-agnostic normalized record/content/presentation models. It is the right source for the record context agents need, not raw JSON parsing.
- `atlas-index` owns artifact schema, migrations, read/write access, validation, filter discovery SQL, and future `record_tags` rows.
- `atlas-search` owns product-facing retrieval/filter-discovery orchestration and should own tag-assignment worklist retrieval as a product service over index capabilities.
- `atlas-runtime` owns path/setup policy and construction of handles for CLI and future surfaces.
- `MetadataSetField::DerivedTags` already exists in `atlas-domain`, but `atlas-index` deliberately reports it as unsupported because no authoritative derived-tag row model exists yet.
- The artifact contract explicitly says tag rows are deferred until the derived-tag redesign.
- ADR 0026 says future derived-tag work must be a Rust-owned redesign against record kinds, explicit metadata axes, typed discovery, and artifact ownership.

The old node worktree shows useful concepts, but not a target layout:

- useful vocabulary shape: descriptive concepts, operational concepts, aggregate concepts, category projections, concept relations, assignment guidance, exemplars, and applicability constraints
- useful seed families: creature setting/role/specialization, spell support/control/utility/effect, hazard problem/resolution/setting/mechanism, equipment utility/effect/party role, affliction disease/effect/response
- useful workflow ideas: untagged cohort discovery, semantic candidates, evidence analysis, reviewed records, exemplars, and assignment memory
- problematic shape to avoid: sprawling TypeScript folders with runtime, editorial, UI, migration, review queues, and writeback tightly coupled around broad facades and human-in-the-loop review sessions

## Resolved Planning Decisions

- Tags are global concepts with typed applicability predicates. They are not category-local definitions.
- Do not recreate node-style category projections as a semantic model. The Rust product no longer has the same category boundary, and search should naturally span record kinds such as spells and equipment when the user wants a mixed result set.
- Presentation grouping is separate from applicability. A UI can group applicable tags by concept area such as setting, problem solved, encounter role, mechanism, or utility, but those groups do not change tag identity.
- Applicable tags for a search should be computed from the current search space. If the current filter scope can contain spells and equipment, the tag picker should show tags applicable to either kind, then selecting one narrows the result set through the normal filter/keyset path.
- A tag may be applicable to one record kind, several record kinds, optional Foundry record type refinements, or another small set of normalized record facts. Applicability should stay intentionally small and mechanical; evidence-sensitive "does this record deserve the tag?" belongs to assignment, not applicability.
- The first implementation must pin down struct, enum, trait, and file layout before any specific tag vocabulary lands beyond minimal fixtures.
- A reviewed record can have zero assigned tags. This is distinct from a missing review and is needed because not every record has enough information or product value to justify tags.
- The baseline should not model excluded tags or formal provenance. Near-miss reasoning can live in notes, comments, or agent-run artifacts until there is evidence that it needs first-class product behavior.
- Tag identifiers have no backwards-compatibility promise during early catalog development. Renames, splits, and merges are allowed until the catalog becomes part of a published artifact contract.
- Assignment evidence is useful and should be recorded where available, but it remains review/audit support rather than a user-facing search axis.
- Presentation should support shallow nesting: a broad display group plus an optional subgroup. The node taxonomy effectively had `category -> axis -> family -> tag`; Rust should drop category, keep no semantic projection layer, and allow `display_group -> display_subgroup -> tag` for navigation only.
- Initial presentation groups are `SettingAndPlace`, `EncounterAndRole`, `ProblemSolving`, `ExplorationAndUtility`, `CombatAndRulesEffect`, `ThemeAndMotif`, `HazardAndObstacle`, `BuildAndEquipmentSupport`, and `ConditionAndAffliction`. These are navigational groupings, not semantic ownership.
- Agent assignment should primarily choose from existing applicable tags, but agents may include secondary ontology-expansion suggestions when the existing catalog misses a reasonably scoped retrieval concept. Novel tag suggestions must always require human approval before they affect the catalog or assignments.
- The coordinator should not decide disputed assignments by itself. When agents disagree, it should send the competing tag sets and evidence back to the agents for reconsideration until they reach quorum or produce an escalation packet.
- Record context packets should prefer completeness over premature minimization. Agents should receive the parsed record/presentation context broadly enough to see valuable signals in non-obvious fields.
- Curated structural vocabularies should use Rust enums or validated newtypes rather than free strings. Free strings are acceptable for user-facing labels, descriptions, notes, and evidence summaries, but not for group/subgroup identity, tag kind, operation, facet, or applicability field names.
- The catalog and assignment corpus should be authored as parsed data files, not Rust source constants. Rust still owns the typed representation, parsing, validation, and ingest/runtime consumption.
- Ontology suggestions from assignment agents should not require broad example-record research. Assignment agents can provide the triggering record and rationale; a separate follow-up/research pass can find more examples before human approval.

## Target Model

The accepted Rust model should use typed entities rather than ad hoc strings:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId(String);

pub struct TagDefinition {
    pub id: TagId,
    pub label: String,
    pub kind: TagKind,
    pub presentation: TagPresentation,
    pub applicability: TagApplicability,
    pub guidance: TagGuidance,
    pub relations: Vec<TagRelation>,
    pub lifecycle: TagLifecycle,
}

pub enum TagKind {
    Descriptive { facet: TagFacet },
    Operational { operation: TagOperation, domain: TagDomain },
    Aggregate,
}

pub enum TagFacet {
    Setting,
    Theme,
    CreatureFamily,
    EncounterRole,
    Capability,
    Effect,
    Mechanism,
    Utility,
    Delivery,
    Progression,
    ChallengeStructure,
}

pub struct TagPresentation {
    pub group: TagDisplayGroup,
    pub subgroup: Option<TagDisplaySubgroup>,
    pub short_label: Option<String>,
    pub sort_key: String,
}

pub enum TagDisplayGroup {
    SettingAndPlace,
    EncounterAndRole,
    ProblemSolving,
    ExplorationAndUtility,
    CombatAndRulesEffect,
    ThemeAndMotif,
    HazardAndObstacle,
    BuildAndEquipmentSupport,
    ConditionAndAffliction,
}

pub enum TagDisplaySubgroup {
    Habitat,
    Planar,
    Regional,
    Site,
    CombatRole,
    SceneRole,
    SocialRole,
    ThreatProfile,
    Countermeasure,
    AccessAndBarriers,
    Investigation,
    Communication,
    TravelAndMovement,
    Reconnaissance,
    BattlefieldControl,
    Impact,
    Support,
    Summoning,
    Transformation,
    Motif,
    CreatureFamily,
    Corruption,
    Mechanism,
    EnvironmentalDanger,
    Haunt,
    ItemMechanical,
    PartyRole,
    PlayPattern,
    AfflictionProgression,
    AfflictionResponse,
    PhysiologyOverride,
}

pub struct TagApplicability {
    pub any_of: Vec<TagApplicabilityClause>,
}

pub struct TagApplicabilityClause {
    pub record_kinds: BTreeSet<RecordKind>,
    pub foundry_record_types: BTreeSet<FoundryRecordType>,
    pub required_facts: Vec<TagFactPredicate>,
    pub excluded_facts: Vec<TagFactPredicate>,
}

pub enum TagFactPredicate {
    HasTrait(String),
    HasPublicationFamily(PublicationFamily),
    HasMetric(MetricKey),
    HasMetadataSetValue { field: MetadataSetField, value: String },
    HasMetadataEnumValue { field: MetadataEnumStringField, value: String },
}

pub struct RecordTagAssignments {
    pub record_key: RecordKey,
    pub tags: Vec<TagAssignment>,
}

pub struct TagAssignment {
    pub tag_id: TagId,
    pub evidence: Vec<AssignmentEvidence>,
    pub note: Option<String>,
}

pub enum AssignmentEvidence {
    ContentExcerpt {
        path: RecordContentPath,
        quote: String,
    },
    PresentationSection {
        section: RecordPresentationSection,
        summary: String,
    },
    NormalizedFact {
        field: TagFactField,
        value: String,
    },
    TagGuidanceMatch {
        signal: String,
        explanation: String,
    },
    SourceReference {
        record_key: RecordKey,
        relationship: String,
        summary: String,
    },
}

pub struct TagAssignmentProposal {
    pub record_key: RecordKey,
    pub proposed_tags: Vec<TagAssignment>,
    pub reviewed_empty: bool,
    pub confidence: AssignmentConfidence,
    pub note: Option<String>,
    pub ontology_suggestions: Vec<OntologySuggestion>,
}

pub struct OntologySuggestion {
    pub proposed_id: Option<String>,
    pub label: String,
    pub group: TagDisplayGroup,
    pub subgroup: Option<TagDisplaySubgroup>,
    pub applicability: TagApplicability,
    pub rationale: String,
    pub triggering_record_key: RecordKey,
    pub follow_up_research_needed: bool,
}
```

Design rules:

- A tag definition is a durable global concept. Record-kind-specific presentation may exist only as derived UI state, not as tag identity or category projection.
- Operational concepts model what a record helps do or what problem it creates: `remediate disease`, `discover hidden source`, `counteract active magic`, `bypass barrier`, `control battlefield`.
- Descriptive concepts model what the record is, where it appears, or how it behaves: setting, theme, creature family, role, mechanism, effect, capability, delivery, progression, challenge structure.
- Aggregate tags are allowed only when they are meaningful filter targets with declared member relations; they are not a shortcut for loose grouping.
- Applicability constraints should be machine-readable, small, and explainable. They should generally be limited to record kind, optional Foundry record type refinements, traits, source/publication axes, and existing normalized metadata. Avoid broad semantic applicability rules that duplicate assignment reasoning.
- The normal applicability surface should start with `record_kinds`, then optionally refine with `foundry_record_types` when the Rust kind intentionally groups multiple source types. For example, equipment tags may distinguish `weapon`, `armor`, `consumable`, and `treasure`; rule tags may distinguish `action`, `condition`, and `effect`.
- Search-space tag discovery should use applicability predicates over the current eligible-record relation. It should answer "which tags could apply to at least one record in this search space?" and return presentation groups and subgroups for UI display.
- Assignments should be stored as one reviewed assignment set per record, with a `Vec<TagAssignment>` that may be empty. This cleanly distinguishes reviewed-with-no-tags from no reviewed assignment entry.
- Assignment evidence should be lightweight and mechanically validatable enough not to become a provenance system. The baseline evidence enum should cover content excerpts, presentation-section summaries, normalized facts, tag-guidance matches, and source/reference relationships. Free-form reasoning belongs in `note`, not in a broad evidence variant.
- Near-miss or disagreement notes should not become first-class excluded tags in the baseline.
- Assignment proposals should separate primary assignment output from secondary ontology suggestions. Existing applicable tags are the normal path; ontology suggestions are never auto-applied.
- A proposal may be `reviewed_empty` only when the agent finds no high-signal existing tags and has no assignment-worthy ontology suggestion for the record.
- Disagreement reconciliation belongs in the agent loop. The coordinator gathers proposals, detects differences, and returns disagreeing evidence to agents; it does not adjudicate by overriding agent judgment.
- Automated acceptance should require unanimous agreement in the baseline. Relaxing to majority or confidence-weighted quorum should require evidence from real usage that unanimity creates too much avoidable human escalation.
- Group and subgroup values should be enum-like in Rust and validated at catalog-load time. Do not let arbitrary strings create parallel groups such as `problem-solving`, `problem_solving`, and `Problem Solving`.
- Keep tag ids as validated newtypes rather than enum variants because the catalog will grow and change often during authoring. The controlled vocabularies around tags should be typed more tightly than the tag ids themselves.

## Presentation Grouping Notes

The node implementation used old category-scoped axes and families. Useful examples:

- `setting` axis with habitat, planar, regional, and site families
- `utility` axis with access bypass, communication, expedition, infiltration, movement/traversal, reconnaissance, resolution, revelation, security, teleportation, and wayfinding
- `effect` axis with impact, environmental danger, forced position, perception control, delivery, offensive profile, consumable role, and physiology override
- `encounter` and `npc_role` axes with combat role, cohort role, threat profile, scene role, and social role
- `item_mechanical` and `party_role` axes for gear/build support
- `disease_model`, `response`, `metaphysical`, and `behavior` axes for affliction and condition concepts
- `presentation` axis for genre, story, visual, and object motifs

Rust should not preserve those as category projections. The useful shape is a UI browse path:

```text
display group -> display subgroup -> tag
```

Recommended initial group mapping:

- `SettingAndPlace`: habitat, planar, regional, site, settlement, route, environment
- `EncounterAndRole`: combat role, cohort role, social role, scene role, threat profile, party/build role when used for encounter prep
- `ProblemSolving`: resolution, countermeasure, access bypass, breaching, anti-magic, containment, cleanup, security, investigation
- `ExplorationAndUtility`: communication, consultation, expedition, infiltration, movement/traversal, reconnaissance, revelation, teleportation, wayfinding, sensory support
- `CombatAndRulesEffect`: battlefield control, impact, influence, support, summoning, transformation, delivery, offensive profile, defenses
- `ThemeAndMotif`: genre motif, story motif, visual motif, corruption, metaphysical profile, creature family/ontology clusters when used as flavor retrieval
- `HazardAndObstacle`: mechanism, problem shape, environmental danger, forced position, haunt manifestation, perception control, attack vector
- `BuildAndEquipmentSupport`: item mechanical, access system, ammunition payload, defense profile, party role, play pattern, carry logistics
- `ConditionAndAffliction`: pathogenesis, progression profile, response profile, resolution profile, physiology override, behavioral override, epidemiological profile

Nesting guidance:

- Two levels are useful because old `utility` and `effect` were too broad to scan as flat groups.
- Do not add a third level until a UI actually proves that two levels are overloaded.
- Subgroups are presentation metadata only. They must not affect tag ids, applicability, assignment validity, or filter semantics.
- A tag can appear under one primary subgroup for browsing. If a tag naturally belongs in multiple places, prefer related/adjacent tags over multi-home presentation until there is a clear product need.

## Catalog And Assignment Authoring Shape

There are two different shapes to keep separate:

- Rust representation: typed structs/enums/newtypes used for parsing, validation, service contracts, artifact writing, and runtime search.
- Authored data: repo-tracked catalog and assignment files parsed by Rust during validation/ingest/publish workflows.

The catalog should not be handwritten Rust constants. It should be authored as data, then parsed into the typed Rust model. The assignments definitely should be data files because the corpus may grow to thousands of record entries and agents need to propose/edit batches without touching source code.

Recommended initial layout:

```text
data/tags/
  catalog/
    setting-and-place.yaml
    encounter-and-role.yaml
    problem-solving.yaml
    exploration-and-utility.yaml
    combat-and-rules-effect.yaml
    theme-and-motif.yaml
    hazard-and-obstacle.yaml
    build-and-equipment-support.yaml
    condition-and-affliction.yaml
  assignments/
    <record-kind>/
      <pack-name>.yaml
```

If a group file becomes too large, split within that group by subgroup:

```text
data/tags/catalog/exploration-and-utility/
  communication.yaml
  reconnaissance.yaml
  travel-and-movement.yaml
```

File layout rules:

- Group/subgroup file organization is for authoring ergonomics only. The parsed catalog is one global tag catalog.
- Group and subgroup identities should parse into Rust enums or closed validated vocabularies.
- Tag ids should parse into a `TagId` newtype with validation for format and uniqueness.
- Applicability should start with record kind and optional Foundry record type refinements. Add trait/metadata predicates only when real catalog examples prove they are needed.
- Labels, descriptions, guidance text, assignment notes, and evidence summaries remain strings.
- Authored assignment files should be record-centered: one record key maps to zero or more assigned tags plus optional notes/evidence.
- Do not introduce a separate review file tree in the baseline. Review state is represented by assignment-file presence: a record entry with an empty `tags` vector means reviewed and intentionally untagged; absence of a record entry means not reviewed.
- Runtime artifact rows can still be normalized row-per-tag for search/filtering even though authored assignment files are record-centered.

Required tag-definition fields:

- `id`
- `label`
- `description`
- `display.group`
- optional `display.subgroup`
- `kind`
- `applicability`
- `guidance.applies_when`
- `guidance.does_not_apply_when`
- optional `guidance.positive_signals`
- optional `guidance.negative_signals`
- optional `related_tags`

Ontology suggestion fields from assignment agents:

- proposed label and optional proposed id
- proposed display group/subgroup
- proposed applicability
- rationale explaining the retrieval use case and why existing tags are insufficient
- triggering record key
- whether follow-up research is needed

Do not require assignment agents to find a broad exemplar set while suggesting a new tag. That research should be a separate follow-up task so assignment-agent context stays focused.

## YAML Schema Sketches

Use lowercase dotted tag ids with a namespace and snake-case leaf:

- `setting.darklands`
- `problem.counteract_magic`
- `encounter.skirmisher`

Applicability semantics:

- Values inside one `any_of` clause are ANDed.
- Multiple values inside one field are ORed.
- Separate `any_of` clauses are ORed.

For example, `record_kinds: [equipment]` plus
`foundry_record_types: [weapon, armor]` means equipment records whose Foundry
record type is weapon OR armor. A second clause can make the same tag applicable
to a different record kind.

### Tag Definition Examples

Catalog files are grouped for authoring ergonomics only. This example could live
in `data/tags/catalog/problem-solving.yaml`:

```yaml
tags:
  - id: problem.counteract_magic
    label: Counteract Magic
    description: Helps suppress, dispel, counteract, or remove active magical effects.
    kind:
      operational:
        operation: remediate
        domain: magic
    display:
      group: ProblemSolving
      subgroup: Countermeasure
      sort_key: counteract_magic
    applicability:
      any_of:
        - record_kinds: [spell, equipment]
    guidance:
      applies_when:
        - The record explicitly dispels, counteracts, suppresses, or removes magic.
        - The record is useful against ongoing magical effects, not merely magical creatures.
      does_not_apply_when:
        - The record only deals magical damage.
        - The record only has the magical trait without solving an active magic problem.
      positive_signals: [dispel, counteract, suppress, remove magic]
      negative_signals: [magical damage only, spell attack only]

  - id: setting.darklands
    label: Darklands
    description: Closely associated with the Darklands, subterranean societies, or deep underground environments.
    kind:
      descriptive:
        facet: Setting
    display:
      group: SettingAndPlace
      subgroup: Regional
      sort_key: darklands
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record text, source context, or creature ecology closely ties it to the Darklands.
      does_not_apply_when:
        - The record merely can appear underground without specific Darklands association.

  - id: equipment.party_support.defense
    label: Defensive Party Support
    description: Equipment that protects allies, improves group defense, or helps the party absorb danger.
    kind:
      operational:
        operation: support
        domain: defense
    display:
      group: BuildAndEquipmentSupport
      subgroup: PartyRole
      sort_key: defense_support
    applicability:
      any_of:
        - record_kinds: [equipment]
          foundry_record_types: [armor, shield, consumable, equipment]
    guidance:
      applies_when:
        - The item primarily improves durability, defense, mitigation, or ally protection.
      does_not_apply_when:
        - The item is only personally offensive or has a defensive trait with no meaningful protective use.
```

### Assignment Set Examples

Assignment files are record-centered. This example could live in
`data/tags/assignments/spell/spells.yaml`:

```yaml
records:
  - record_key: Compendium.pf2e.spells-srd.Item.dispel-magic
    tags:
      - tag_id: problem.counteract_magic
        evidence:
          - kind: content_excerpt
            path: description
            quote: Attempts to counteract a spell effect or magic item.
          - kind: normalized_fact
            field: record_kind
            value: spell
        note: Core use is removing active magic.

  - record_key: Compendium.pf2e.spells-srd.Item.water-breathing
    tags:
      - tag_id: exploration.environment.aquatic
        evidence:
          - kind: content_excerpt
            path: description
            quote: Allows creatures to breathe underwater.
          - kind: tag_guidance_match
            signal: Enables underwater exploration.
            explanation: The spell directly solves a travel/environment problem.
        note: Enables aquatic exploration rather than combat advantage alone.

  - record_key: Compendium.pf2e.equipment-srd.Item.spyglass
    tags:
      - tag_id: exploration.reconnaissance.long_range
        evidence:
          - kind: content_excerpt
            path: description
            quote: Helps view distant objects.
          - kind: normalized_fact
            field: foundry_record_type
            value: equipment
          - kind: presentation_section
            section: description
            summary: The presented description is about distant visual inspection.
```

### Reviewed-Empty Examples

An entry with `tags: []` means the record was reviewed and no high-signal tag was
justified. There is no separate review file:

```yaml
records:
  - record_key: Compendium.pf2e.spells-srd.Item.generic-flavor-cantrip
    tags: []
    note: Short record text did not establish a useful retrieval concept.

  - record_key: Compendium.pf2e.equipment-srd.Item.plain-clothing
    tags: []
    note: Generic item; no high-value setting, utility, or build-support tag.

  - record_key: Compendium.pf2e.bestiary.Actor.minimal-placeholder
    tags: []
    note: Parsed record lacks enough descriptive content for a repeatable assignment.
```

### Ontology Suggestion Examples

Ontology suggestions are secondary outputs from assignment agents. They never
apply a novel tag directly:

```yaml
suggestions:
  - proposed_id: exploration.environment.aquatic
    label: Aquatic Environment Support
    display:
      group: ExplorationAndUtility
      subgroup: TravelAndMovement
    applicability:
      any_of:
        - record_kinds: [spell, equipment]
    rationale: Several records solve underwater breathing or underwater movement, which is a predictable exploration-prep search axis not covered by native traits alone.
    triggering_record_key: Compendium.pf2e.spells-srd.Item.water-breathing
    follow_up_research_needed: true

  - proposed_id: encounter.summoner_support
    label: Summoner Support
    display:
      group: EncounterAndRole
      subgroup: SceneRole
    applicability:
      any_of:
        - record_kinds: [creature, spell, equipment]
    rationale: The triggering record exists mainly to create, enhance, or coordinate summoned allies; existing encounter-role tags do not distinguish this prep need.
    triggering_record_key: Compendium.pf2e.spells-srd.Item.summoners-precaution
    follow_up_research_needed: true

  - proposed_id: hazard.countermeasure.noise
    label: Noise Countermeasure
    display:
      group: HazardAndObstacle
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [hazard, spell, equipment]
    rationale: The triggering hazard is specifically detected, bypassed, or suppressed through sound/noise interaction, which may be a useful obstacle-prep axis.
    triggering_record_key: Compendium.pf2e.hazards.Actor.resonant-alarm
    follow_up_research_needed: true
```

## Trait Shape

The core Rust shape should make ownership explicit before vocabulary work begins:

```rust
pub trait TagCatalog {
    fn definitions(&self) -> &[TagDefinition];
    fn get(&self, id: &TagId) -> Option<&TagDefinition>;
    fn applicable_to_record(&self, facts: &TagRecordFacts) -> Vec<&TagDefinition>;
    fn applicable_to_search_space(
        &self,
        facts: &SearchSpaceFacts,
    ) -> Vec<SearchSpaceTagOption>;
}

pub trait TagAssignmentReadIndex {
    fn assignments_for_record(
        &self,
        key: &RecordKey,
    ) -> Result<Option<RecordTagAssignments>, TagReadError>;

    fn untagged_records(
        &self,
        request: UntaggedRecordsRequest,
    ) -> Result<UntaggedRecordsPage, TagReadError>;
}

pub trait TagAssignmentWriteIndex {
    fn write_record_assignments(
        &mut self,
        assignments: &[RecordTagAssignments],
    ) -> Result<(), TagWriteError>;
}

pub struct TagRecordFacts {
    pub record_key: RecordKey,
    pub record_kind: RecordKind,
    pub foundry_record_type: Option<FoundryRecordType>,
    pub traits: BTreeSet<String>,
    pub metadata_sets: BTreeMap<MetadataSetField, BTreeSet<String>>,
    pub metadata_enums: BTreeMap<MetadataEnumStringField, String>,
    pub metric_keys: BTreeSet<MetricKey>,
}

pub struct SearchSpaceFacts {
    pub possible_record_kinds: BTreeSet<RecordKind>,
    pub possible_foundry_record_types: BTreeSet<FoundryRecordType>,
    pub possible_traits: BTreeSet<String>,
    pub possible_metadata_values: BTreeMap<MetadataSetField, BTreeSet<String>>,
}

pub struct SearchSpaceTagOption {
    pub tag: TagDefinitionSummary,
    pub group: TagDisplayGroup,
    pub applies_to: SearchSpaceApplicabilitySummary,
}

pub struct TagAssignmentContextPacket {
    pub record_key: RecordKey,
    pub record: TagRecordContext,
    pub facts: TagRecordFacts,
    pub current_assignment: Option<RecordTagAssignments>,
    pub applicable_tags: Vec<ApplicableTagContext>,
    pub related_records: Vec<TagRelatedRecordContext>,
    pub assignment_guidance: AssignmentRunGuidance,
}

pub struct TagRecordContext {
    pub presentation: RecordPresentationDocument,
    pub content_sections: Vec<RecordContentSection>,
    pub traits: Vec<String>,
    pub metrics: Vec<RecordMetricSummary>,
    pub publication: RecordPublicationSummary,
}

pub struct ApplicableTagContext {
    pub tag: TagDefinitionSummary,
    pub guidance: TagGuidance,
    pub applicability_explanation: String,
    pub related_tags: Vec<TagId>,
}
```

Trait ownership guidance:

- `TagCatalog` should be pure and storage-agnostic. It evaluates static tag definitions against typed fact summaries.
- `TagAssignmentReadIndex` and `TagAssignmentWriteIndex` belong at the index/artifact boundary, not in CLI code.
- Read APIs should preserve the difference between `None` for not reviewed and `Some(RecordTagAssignments { tags: vec![] })` for reviewed with no justified tags.
- `TagRecordFacts` should be projected from normalized records and indexed side tables. It must not parse `raw_json`.
- `SearchSpaceFacts` should be derived from the same eligible-record/filter discovery machinery used by search. It should not rely on category assumptions or UI-selected buckets.
- The first prototype can keep write support out of the runtime query path, but the trait shape should reserve a clear write boundary for later import/publish work.

## Proposed Rust File Layout

Before adding real tags, land a small model-only slice with this intended layout:

```text
crates/atlas-tags/src/lib.rs
crates/atlas-tags/src/
  id.rs
  model.rs
  applicability.rs
  assignment.rs
  proposal.rs
  yaml.rs
  validation.rs
  catalog.rs
  agent_contract.rs

crates/atlas-record/src/tagging.rs
crates/atlas-record/src/tagging/
  facts.rs
  context.rs

crates/atlas-search/src/tags.rs
crates/atlas-search/src/tags/
  catalog.rs
  applicability.rs
  worklist.rs
  context.rs
  reconciliation.rs

crates/atlas-index/src/read/tags.rs
crates/atlas-index/src/write/sqlite/tags.rs
crates/atlas-index/src/artifact/validation/tags.rs

crates/atlas-cli/src/commands/tags.rs
crates/atlas-cli/src/cli/tags.rs
```

Layout rules:

- `atlas-tags` owns tag definitions, tag ids, presentation-group enums, applicability predicates, assignment file parsing, ontology-suggestion parsing, validation, and agent packet DTOs.
- `atlas-tags` may depend on stable domain/record vocabulary such as `RecordKind`, `RecordKey`, and `FoundryRecordType`, but it should not own raw source normalization, SQLite storage, runtime path policy, or CLI presentation.
- `atlas-record/src/tagging/` owns fact/context projections from normalized records and presentation documents.
- `atlas-search/src/tags/` owns product services: applicable tag discovery for a search space, worklist assembly, context packet assembly, and reconciliation contracts.
- `atlas-index` owns persisted rows and validation once assignments become artifact data.
- `atlas-ingest` consumes validated tag catalogs and assignment files during regular index build to write authoritative artifact rows.
- `atlas-cli` only maps command arguments to service requests and formats JSON/text output.

## Proposed Crate Ownership

The likely end-state ownership is:

- `atlas-tags`: tag ontology, authored YAML parsing, applicability evaluation, assignment validation, ontology-suggestion validation, and agent contract DTOs.
- `atlas-domain`: stable shared PF2e Atlas vocabulary that tags depend on, such as `RecordKind`, `RecordKey`, metadata fields, and filter primitives. It should not own tag ontology simply because tags cross crate boundaries.
- `atlas-record`: record-context projections for tagging, derived from `AtlasRecord`, `RichDocument`, `RecordPresentationDocument`, metrics, traits, references, and normalized facts. This should not know about SQLite tables or agent workflow.
- `atlas-ingest`: validated tag catalog and assignment consumption during regular index build.
- `atlas-index`: physical tables and read/write capabilities for tag definitions, record assignment sets, evidence summaries, and artifact validation. It should also make `metadata.set.derived_tags` compile only after authoritative rows exist.
- `atlas-search`: product-facing tagging services, including untagged worklists, applicable tag discovery, assignment context assembly, and search/filter integration.
- `atlas-runtime`: construction of tagging service handles and path policy for any writable assignment store or rebuilt artifact target.
- `atlas-cli`: `atlas tags ...` command grammar, JSON/text presentation, progress output, and exit behavior.

## Storage Strategy

Split durable authored/editable state from runtime query rows:

- Authored tag catalog: versioned repo-tracked YAML files under `data/tags/catalog/`, grouped first by display group and later by subgroup only if group files become too large.
- Authored assignments: repo-tracked YAML files under `data/tags/assignments/<record-kind>/<pack-name>.yaml` for merge hygiene. Each reviewed record should have one assignment set with a vector of assigned tags; an empty vector means reviewed and no high-signal tags justified. The old pack-grouped assignment idea is worth preserving, but the file format should be data, not TypeScript.
- Runtime artifact rows: `record_tags` written into SQLite during regular index build. These rows are the authoritative filter/search surface.
- Assignment notes/evidence: keep only enough durable evidence outside the query row to review surprising decisions. Do not introduce a formal provenance system in the baseline.

The baseline publication path is regular `atlas index build`. A faster tag overlay or separate `atlas tags publish` command should be deferred until agent assignment iteration proves that full rebuilds are too slow.

## CLI And Agent Workflow

The CLI should be designed for automation first:

- `atlas tags worklist`: returns records needing tag assignment, scoped by record kind, publication, pack, tag family, missing tag coverage, or previous disagreement status.
- `atlas tags context <record-key...>`: returns all context needed to tag the records, including record presentation, content sections, normalized facts, references, existing assignment set if reviewed, applicable tag definitions, applicability explanations, adjacent tags, exemplars, and assignment guidance.
- `atlas tags applicable <record-key...>`: returns applicable tag definitions and constraints without full record content for cheaper planning.
- `atlas tags propose`: accepts machine-authored assignment proposals with tags, evidence, confidence, and notes. It validates tag ids, applicability, duplicate decisions, and required evidence shape.
- `atlas tags reconcile`: combines multiple proposals for the same record, emits accepted assignments when the configured quorum agrees, or emits reconsideration/escalation packets otherwise.
- `atlas tags import`: writes accepted assignment batches into the authored assignment store.
- `atlas tags coverage`: reports not-reviewed counts, reviewed-with-tags counts, reviewed-without-tags counts, disagreement queues, and family-level gaps.

The desired automated loop:

1. Orchestrator calls `atlas tags worklist --json --limit N`.
2. Orchestrator calls `atlas tags context --json` for the returned records.
3. Orchestrator delegates each record or small coherent batch to multiple agents.
4. Each agent returns a proposal packet with assigned tags, evidence, notes, and confidence.
5. Orchestrator calls `atlas tags reconcile --json` or applies the same typed reconciliation contract locally.
6. If every assigned agent returns the same assignment set after any reconsideration loop, the accepted assignment is imported without human review.
7. If agents disagree materially or evidence is insufficient, only the minimal escalation packet goes to a human.

The CLI JSON should make the agent contract explicit. It should not rely on agents scraping terminal prose.

Recommended context packet shape:

- Include one `TagAssignmentContextPacket` per record.
- Reuse the existing renderer-neutral record presentation path from `atlas-record` as the primary human-readable record surface. The tagging packet should be built from `RecordPresentationDocument`, `PresentationSection`, and `PresentationContent` or a stable projection of those types.
- Do not depend on the CLI terminal renderer as the agent contract. `atlas-cli` text output adds terminal styling and command-specific formatting; it is a consumer of the presentation model, not the durable tagging surface.
- If agents need a simple text field, derive it with a renderer-neutral plain-text function from the presentation/content model and include it beside the structured sections. Do not scrape or round-trip terminal output.
- Include normalized facts that affect applicability or repeatable assignment: `record_kind`, optional `foundry_record_type`, traits, publication/source metadata, metrics, references, and existing assignment state.
- Include applicable tag definitions with guidance and a short applicability explanation, not the entire global catalog.
- Include related records only as compact reference summaries; agents should not need broad graph traversal in the default assignment loop.
- Exclude raw Foundry JSON from the baseline. Add it only if repeated assignment gaps show that normalized/presentation context loses important evidence.
- Do not expose `AtlasRecord` wholesale as the agent contract. Project only the fields that are useful for assignment and stable enough to validate.
- Do not expose low-level provenance (`source_path`, `raw_json`), coarse Foundry document type, folder ids, visibility/retrieval policy internals, FTS/search projection text, raw reference-policy settings, or variant-detection heuristics/confidence as baseline tagging context.
- Do not expose raw mechanics or raw metric rows without labels. Use compact, labelled summaries for mechanics and metrics that are useful evidence.

## Vocabulary Seed Strategy

Start with a small high-value seed, not the full old taxonomy:

- creature setting tags: habitat, planar, and selected Golarion-flavored regional settings where the old node taxonomy has strong guidance
- spell problem-solver tags: affliction cleanup, anti-disease, anti-poison, anti-curse, fear remediation, counter magic, barrier bypass/breaking, reconnaissance, wayfinding, communication, mobility, battlefield control
- hazard problem and resolution tags: detection pressure, disarm/bypass/disable paths, environmental danger, forced movement, perception control, guard/threshold mechanisms
- equipment utility tags: reconnaissance, traversal, concealment, communication, party-role support, anti-magic, recovery, item payload/delivery

Use the old node concepts as a candidate pool, but each retained tag must pass Rust design review:

- Is it not already covered by record kind, traits, metadata, metrics, or graph context?
- Is it useful as a search/filter/recommendation axis?
- Can agents apply it from available record context with repeatable evidence?
- Does it have clear applicability constraints?
- Does it need category-specific projection, or is it one shared semantic concept?

## Implementation Slices

### Slice 1: Model Shape And File Layout

Pin down the Rust struct, enum, trait, and module shape before implementing real vocabulary.

Deliverables:

- `docs/architecture/tagging.md` with the accepted model shape and file layout
- a small model-only Rust skeleton or doc-backed pseudo-code for the core DTOs and traits
- explicit decision that node-style category projections are not part of the Rust semantic model
- explicit separation between global tag identity, applicability predicates, and UI presentation groups
- examples of search-space applicability for mixed result sets, such as spells plus equipment

Validation:

- docs agree with `overview.md`, `runtime.md`, `artifact-contract.md`, and ADR 0026
- no tag seed file or assignment workflow is implemented before the shape is accepted
- no code or docs claim `metadata.set.derived_tags` works before rows exist

### Slice 2: Design ADR

Add an ADR for the accepted model once Slice 1 is reviewed.

Deliverables:

- durable decision on global tags plus applicability predicates
- durable decision on search-space tag discovery
- durable decision on storage boundary and `atlas-tags` crate ownership
- explicit migration stance for old node vocabulary

Validation:

- ADR and `docs/architecture/tagging.md` agree
- no active architecture doc continues to describe derived tags as category-projection based

### Slice 3: Catalog Schema Prototype

Create the tag-definition and projection schema in data form with a tiny seed catalog.

Deliverables:

- schema file and validation code
- sample fixture definitions for 5-10 tags across creature/spell/hazard/equipment; these are fixtures for shape validation, not a committed taxonomy push
- tests for duplicate ids, invalid applicability constraints, aggregate relations, presentation groups/subgroups, and invalid references

Likely owners:

- `atlas-tags` for schema structs, YAML parsing, validation, applicability, and assignment model tests
- `atlas-domain` only for already-shared PF2e Atlas primitives such as record keys, record kinds, metadata fields, and filter vocabulary

Validation:

- `cargo test -p <owning-crate> tags`
- schema fixture round-trips through serde

### Slice 4: Search-Space Applicability

Implement applicable-tag discovery over a current search space before record assignment.

Deliverables:

- `SearchSpaceFacts` derived from eligible records and existing filter discovery machinery
- `SearchSpaceTagOption` results grouped by `TagDisplayGroup`
- tests proving mixed scopes such as spell plus equipment return the union of applicable tags and selecting a tag narrows through normal filters

Likely owners:

- `atlas-search` for product-facing service
- `atlas-index` only for fact/discovery reads needed to summarize the eligible relation

Validation:

- no category parameter is required
- no category-specific presentation model appears in the semantic DTOs

### Slice 5: Record Tagging Context

Build a storage-agnostic record context packet for agents.

Deliverables:

- `TaggingRecordContext` derived from `AtlasRecord`, full parsed/presentation record context, traits, metrics, references, source/publication, current assignment set, and applicable tags
- enough evidence snippets for agents to justify assigned tags or a reviewed-empty decision without reading raw JSON
- context-size controls suitable for batch CLI JSON

Likely owners:

- `atlas-record` for record-context projection primitives
- `atlas-search` for assembling current assignments/applicable tags with records

Validation:

- focused tests over minimal records with rich content, traits, metrics, and references
- JSON snapshots only where stable enough to protect the agent contract

### Slice 6: Artifact Rows And Filter Support

Add authoritative persisted tag rows.

Deliverables:

- migration for `record_tags`
- index writer support from authored assignments
- read capability for record assignments
- artifact validation coverage
- `MetadataSetField::DerivedTags` compiles through the same authoritative SQL keyset path as other filters

Likely owners:

- `atlas-index` for schema/read/write/validation
- `atlas-ingest` for build-input handoff if assignments are consumed during artifact build

Validation:

- `cargo test -p atlas-index filters derived_tags`
- row coverage validation for assignments referencing existing records and known tags
- search/filter integration tests prove no post-filter-only approximation

### Slice 7: Agent-Facing CLI

Expose the automated worklist/context/proposal workflow.

Deliverables:

- `atlas tags worklist --json`
- `atlas tags context --json`
- `atlas tags applicable --json`
- proposal/reconsideration/escalation/import command skeletons if assignment writes are in scope
- first-party skill updates explaining the workflow

Likely owners:

- `atlas-search` for worklist/context/applicable services
- `atlas-runtime` for handle construction
- `atlas-cli` for command grammar and output
- `skills/pf2e-atlas-cli` for agent guidance

Validation:

- CLI JSON contract tests
- installed/source smoke commands using real artifact paths where practical
- skill examples use installed `atlas`, not contributor-only `cargo run`

### Slice 8: Assignment Orchestration

Implement the quorum-oriented assignment loop outside core retrieval first.

Deliverables:

- proposal packet schema
- reconsideration packet schema carrying disagreeing tag sets and evidence back to agents
- accepted/disagreed/escalated output packets
- unanimous baseline quorum rule
- ontology suggestion packet schema for human-approved catalog expansion
- optional follow-up research queue for ontology suggestions that need more examples
- import/writeback path into authored assignments
- coverage reports

Design preference:

- keep LLM orchestration outside Rust core unless a Rust command can validate deterministic packets
- Rust should validate packet shape, applicability, known tag ids, reviewed-empty state, and ontology-suggestion shape
- the coordinator should reconcile by asking agents to reconsider with each other's evidence; it should not make the semantic decision itself
- accepted assignments require unanimous agreement in the baseline; any persistent disagreement escalates rather than being majority-voted
- ontology suggestions are secondary outputs and always require human approval before catalog or assignment changes

Validation:

- tests for quorum agreement, reconsideration loops, conflicting assigned-tag sets, reviewed-empty decisions, insufficient evidence, invalid tag ids, stale record/tag versions, and ontology suggestions
- tests that ontology suggestions do not require assignment agents to provide broad example-record research
- dry-run import mode

## Docs And ADR Updates

Required follow-through once the model is accepted:

- Add `docs/architecture/tagging.md`.
- Add an ADR under `docs/architecture/decisions/` for Rust tag model ownership and assignment workflow.
- Update `docs/architecture/overview.md` and `docs/architecture/runtime.md` to replace “future derived tags” placeholders with the accepted shape.
- Update `docs/architecture/artifact-contract.md` when tag rows are introduced.
- Update `docs/backlog/items/rust-derived-tag-redesign.md` as slices land.
- Update `skills/pf2e-atlas-cli/SKILL.md` only when working CLI commands exist.

## Validation Plan

Planning validation:

- Re-read architecture docs and confirm no ownership conflict.
- Compare the plan against the old node worktree for useful vocabulary and workflow ideas without importing layout.
- Confirm current Rust code still treats derived tags as unsupported before implementation.
- Confirm the plan does not preserve node-style category projections as a Rust target model.

Implementation validation:

- focused crate tests for each slice
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace`
- `just verify` before any commit that changes executable code

End-state checks:

- no compatibility shim around node tag files
- no CLI-owned tagging semantics
- no semantic category projection model or category-required tag picker
- no raw JSON parsing for normal assignment context
- no first-class excluded-tag or provenance system in the baseline
- no `metadata.set.derived_tags` support unless authoritative rows and validation exist
- docs and artifact contract agree with implemented storage
- agent JSON contracts are explicit and stable enough for delegated assignment loops

## Landing Workflow

This planning work is isolated in worktree:

```text
.worktrees/tagging-design
branch: docs/tagging-design
```

Keep planning edits uncommitted until the user asks for a commit. If implementation begins from this plan, prefer staying in this worktree for the first docs/modeling slice and using additional worktrees only for parallel implementation agents with disjoint write scopes.

## Assumptions, Blockers, And Open Questions

Assumptions:

- The first durable target is an agent-first CLI workflow, not a TUI review screen.
- Global tags plus applicability predicates replace node-style common tags plus category projections.
- Search-space tag discovery should work for mixed record-kind result sets.
- Authored assignments should remain reviewable in repo-tracked data files even if the ordinary assignment loop is automated.
- A reviewed assignment entry with an empty tag vector means reviewed and intentionally no tags; absence of an entry means not reviewed.
- Agents may suggest new tags, but those suggestions are secondary outputs and require human approval.
- Ontology suggestions should include the triggering record and rationale, not a full exemplar research set.
- Disagreements should loop back through agents with competing evidence rather than being resolved directly by the coordinator.
- The Rust artifact must eventually expose tag filters through authoritative SQL rows, not post-filtering.

Open questions:

- Which old node vocabulary families should be retired immediately because current Rust record kind, traits, metrics, source axes, or graph retrieval already cover them?
