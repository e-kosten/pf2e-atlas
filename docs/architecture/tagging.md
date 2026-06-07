# Tagging Architecture

PF2e Atlas tags are authored labels that add search and discovery axes not present in native Pathfinder or Foundry data. They are for predictable retrieval, discovery, encounter prep, and adjacent workflows where semantic search can approximate intent but static labels give users a more reliable filter surface.

This design replaces the old node derived-tag model. The node worktree remains useful as vocabulary inspiration, but its category projections, review queues, and TypeScript facade layout are not the Rust target.

## Product Model

Tags are global concepts with typed applicability predicates. They are not category-local definitions.

A tag definition answers:

- what concept the tag represents
- where it appears in tag-picker presentation
- which records it could mechanically apply to
- what evidence should justify assigning it

Applicability is deliberately narrower than assignment. Applicability says a tag could apply to a record based on mechanical facts such as `record_kind`, optional `foundry_record_type`, traits, source/publication facts, metrics, or other normalized metadata. Assignment decides whether the record actually deserves the tag based on content and evidence.

Search-space tag discovery computes applicable tag options over the current eligible-record relation. If a search can contain spells and equipment, the tag picker should show tags applicable to either kind. Selecting a tag then narrows the result set through normal filter/keyset behavior.

Presentation grouping is navigation only:

```text
display group -> optional display subgroup -> tag
```

Group and subgroup do not affect tag identity, applicability, assignment validity, or filter semantics.

## Authored Data

The catalog and assignment corpus are authored as YAML, not Rust source constants.

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

If a catalog group becomes too large, it may be split by subgroup under that group directory. File layout is for authoring ergonomics only; the parsed catalog is one global tag catalog.

Tag ids use a lowercase dotted namespace with snake-case leaves, such as:

- `setting.darklands`
- `problem.counteract_magic`
- `encounter.skirmisher`

`TagId` should be a validated newtype, not a Rust enum. Group, subgroup, tag kind, operation, facet, applicability field names, and evidence kind should be Rust enums or closed validated vocabularies. Labels, descriptions, guidance, notes, and evidence summaries may be strings.

## Assignment Model

Authored assignments are record-centered:

```rust
pub struct RecordTagAssignments {
    pub record_key: RecordKey,
    pub tags: Vec<TagAssignment>,
}

pub struct TagAssignment {
    pub tag_id: TagId,
    pub evidence: Vec<AssignmentEvidence>,
    pub note: Option<String>,
}
```

The presence of a record assignment entry means the record has been reviewed. An entry with `tags: []` means reviewed and intentionally untagged. The absence of an entry means not reviewed. Do not add a separate review file tree or first-class review state in the baseline.

The baseline should not model excluded tags or formal provenance. Near-miss reasoning belongs in `note` or external review artifacts until a concrete product need justifies first-class behavior.

Evidence should be small and mechanically validatable:

- `content_excerpt`: content path plus quote
- `presentation_section`: section plus summary
- `normalized_fact`: fact field plus value
- `tag_guidance_match`: guidance signal plus explanation
- `source_reference`: related record plus relationship and summary

Free-form reasoning belongs in assignment notes, not in a broad evidence variant.

Example evidence block:

```yaml
record_key: Compendium.pf2e.spells-srd.Item.dispel-magic
tags:
  - tag_id: problem.counteract_magic
    evidence:
      - kind: content_excerpt
        path: description
        quote: "You attempt to counteract a spell or magical effect."
      - kind: presentation_section
        section: description
        summary: "The description frames the spell around removing or suppressing an active magical effect."
      - kind: normalized_fact
        field: record_kind
        value: spell
      - kind: tag_guidance_match
        signal: "Counteracts or dispels an active magical effect."
        explanation: "The record explicitly uses counteract language, matching the tag guidance for problem.counteract_magic."
      - kind: source_reference
        record_key: Compendium.pf2e.spells-srd.Item.dispel-magic
        relationship: remaster_link
        summary: "The legacy/remaster-linked spell preserves the same counteract-magic use case."
    note: "High-confidence assignment; direct counteract wording."
```

## Applicability

Applicability clauses are ORed through `any_of`. Within one clause, fields are ANDed. Multiple values inside one field are ORed.

```rust
pub struct TagApplicability {
    pub any_of: Vec<TagApplicabilityClause>,
}

pub struct TagApplicabilityClause {
    pub record_kinds: BTreeSet<RecordKind>,
    pub foundry_record_types: BTreeSet<FoundryRecordType>,
    pub required_facts: Vec<TagFactPredicate>,
    pub excluded_facts: Vec<TagFactPredicate>,
}
```

`record_kinds` is the primary applicability axis. `foundry_record_types` is an optional refinement for broad record kinds such as equipment or rules. Do not include coarse Foundry document type (`Actor`, `Item`, etc.) in tag applicability or tagging context. Revisit only if a future concrete use case proves it is needed.

## Context Packets

Agent-facing context packets should reuse renderer-neutral record presentation from `atlas-record`, not terminal-rendered CLI output.

Recommended shape:

```rust
pub struct TagAssignmentContextPacket {
    pub record_key: RecordKey,
    pub record: TagRecordContext,
    pub facts: TagRecordFacts,
    pub current_assignment: Option<RecordTagAssignments>,
    pub applicable_tags: Vec<ApplicableTagContext>,
    pub related_records: Vec<TagRelatedRecordContext>,
    pub assignment_guidance: AssignmentRunGuidance,
}
```

The record context should be built from `RecordPresentationDocument`, `PresentationSection`, and `PresentationContent` or a stable projection of those types. If agents need plain text, derive it with a renderer-neutral plain-text function from the presentation/content model and include it beside structured sections. Do not scrape or round-trip terminal output.

Include:

- record key, title, record kind, level, rarity, traits, and publication summary
- optional `foundry_record_type` as a refinement fact
- renderer-neutral presentation sections and content sections
- normalized facts and labelled metrics/mechanics summaries
- compact related-record summaries
- current assignment if present
- applicable tag definitions, guidance, and applicability explanations

Do not expose `AtlasRecord` wholesale as the agent contract. Do not expose raw Foundry JSON, low-level provenance (`source_path`, `raw_json`), coarse Foundry document type, folder ids, visibility/retrieval policy internals, FTS/search projection text, raw reference-policy settings, or variant-detection heuristics/confidence as baseline tagging context. Do not expose raw mechanics or metric rows without labels.

## Agent Workflow

The ordinary assignment workflow is automated and agent-first:

1. Orchestrator requests a worklist.
2. Orchestrator requests context packets for those records.
3. Multiple agents independently propose assignments with evidence.
4. The coordinator compares proposals.
5. If agents disagree, the coordinator sends competing tag sets and evidence back for reconsideration.
6. Automated acceptance requires unanimous agreement after any reconsideration loop.
7. Persistent disagreement or insufficient evidence escalates only the minimal packet to a human.

The coordinator should not make semantic tag decisions by itself. It validates packet shape, detects disagreement, routes evidence, and imports unanimous results.

Agents may suggest new ontology entries as secondary output when the catalog misses a useful, reasonably scoped retrieval concept. Novel tags always require human approval before changing the catalog or assignments. Assignment agents should include the triggering record and rationale; broader exemplar research is a separate follow-up task.

## Crate Ownership

- `atlas-tags` owns tag ontology, YAML parsing, applicability evaluation, assignment validation, ontology-suggestion validation, evidence validation, and agent contract DTOs.
- `atlas-domain` owns shared PF2e Atlas primitives that tags depend on, such as `RecordKind`, `RecordKey`, metadata fields, and filter vocabulary. It should not own tag ontology simply because tags cross crate boundaries.
- `atlas-record` owns tagging record-context projections from normalized records, `RichDocument`, `RecordPresentationDocument`, metrics, traits, references, and normalized facts. It must not know about SQLite tables or agent workflow.
- `atlas-ingest` consumes validated tag catalogs and assignment files during regular index build.
- `atlas-index` owns the `record_tags` table family, physical schema, read/write capabilities, validation, and filter/discovery SQL over authoritative tag rows.
- `atlas-search` owns product-facing tagging services: untagged worklists, applicable tag discovery, assignment context assembly, reconciliation contracts, and search/filter integration.
- `atlas-runtime` owns path/setup policy and construction of tagging service handles.
- `atlas-cli` owns `atlas tags ...` command grammar, JSON/text presentation, progress output, and exit behavior. It does not own durable tag semantics.

## Artifact Contract

Runtime tag filters must use authoritative SQLite rows in `record_tags`, written during regular `atlas index build` from validated YAML catalog and assignment files. `record_tags` is the product-facing table name; do not use legacy `record_derived_tags` for the new model.

The current artifact may continue to reject `metadata.set.derived_tags` until authoritative rows, validation, and query support exist. Once `record_tags` exists, filter discovery and search should compose tag filters through the same authoritative SQL keyset path as other filters.

## Deferred Decisions

- Whether a future faster tag overlay or `atlas tags publish` command is needed. The baseline is regular `atlas index build`.
- Which old node vocabulary families should be retired because record kind, traits, metrics, source axes, or graph retrieval already cover them.
- Whether any future UI needs multi-home presentation for tags. The baseline is one primary group/subgroup path plus related tags.
