# ADR 0060 Core sections and search scope

Status: accepted.

## Context

Owned abilities, spells and equipment can dominate a record's descriptive
similarity. A small, explicit selection by record type provides another useful
representation to test. Restricting destination content is a separate choice
from selecting the source. Neither choice establishes a better default.

## Decision

Expose independent source presets All sections, Core sections and Custom in
content-based Similar, and an All/Core result scope for text search and Similar.
Keep the existing methods, correspondence, profiles, decay, filters and sorting
independent. Each destination uses its own typed recipe. Defaults remain All/All,
record Passage matches/Any/0.5 and trait Trait definition.

`atlas-record` owns `core-sections/v1`, using checked source family, exact owner
chain, typed category, field and existing audience admission. All structural
sections and chunks in an allowed field qualify. The initial prose recipes are:

- NPC: root public notes and blurb.
- Hazard: root description and routine, plus direct intrinsic action and melee
  descriptions; not Stealth, Disable or Reset.
- Spell/ritual, rule, equipment and character-option documents: their own main
  descriptions, without prerequisites, GM notes or nested grants/contents.
- Vehicle: root description and direct actions.
- Army: root description and direct army-tactic/army-war-action campaign features.
- Character: permitted appearance/backstory and direct ancestry, heritage,
  background and class main descriptions.
- Familiar: direct actions categorized as familiar abilities.

Journals, roll tables and other unmodelled families have no Core prose recipe.
Record Core always includes one existing root identity component, whether prose
is present or absent. No-prose/unmodelled records remain identity-only under the
selected method, with an explicit reason. Required missing vectors are readiness
errors. Trait Core retains its existing definition or label-only representation,
without an additional record identity.

Identity and Section are distinct typed components. Identity has no invented
prose field, byte range, ordinal or role. Passage matches treats identity as one
logical contribution under the same sorted geometric decay; Content average
gives it one logical centroid contribution. Corresponding compares identity with
identity and retains existing prose-role rules. Any permits cross-component
matches. Core or explicit-identity sources also admit destination identity in All
scope; legacy All/prose-only comparisons keep their original eligibility and
scores. Identity counts toward the existing resource ceilings. Baseline methods
accept destination scope but reject explicit source presets.

`atlas-index` materializes exact Core membership over existing parts, lexical
units and semantic locations. Retain root/alias lexical rows and their existing
vocabulary. Owned names qualify independently of prose presence when their exact
owner is admitted; heading/definition labels require their exact Core section.
Do not add prose terms, lexical rows, a separate FTS index or scope-specific BM25
statistics. Intersect eligibility before FTS/KNN limits, aggregation, centroids,
witnesses, counts and edition preference. Ordinary semantic search keeps its
strongest-unit-per-root aggregation.

Migration 18 adds metadata tables. Offline publication reconstructs membership
from retained checked snapshots/content, validates it and advances reference
generation atomically. The separate Core context version is excluded from
preparation/native-input identity. Existing lexical postings, native inputs,
vectors and saved state are preserved. Query paths read membership without
decoding source bodies.

App-service projects component inventory, exact navigation, effective scope,
selection and recipe acknowledgements. CLI and web transport these choices
without inferring field policy or calculating scores. The shared Search settings
dialog remembers one result scope across text/Similar and All/Core source presets
by projected family. Custom selectors remain seed/generation-specific. Explicit
legacy links retain All semantics; opening a link does not overwrite saved
preferences. Browse and exact membership/reference operations retain their
existing contracts.

## Consequences and validation

Core is an opt-in restriction and representation, not a relevance winner.
Compare identity-only, prose-only and combined representations, equipment's own
versus intrinsic spell content, and multiple decay values in a separate frozen
evaluation. Preserve useful associated-content queries in that later study.
No broader study or default change is implied by implementing these controls.

Validation must prove exact typed membership, privacy/category distinctions,
pre-limit masking, identity weighting/correspondence, missing-vector failures,
legacy All parity and offline lexical/native/saved-state preservation. Record
coverage and metadata cost separately from semantic usefulness. See
[ADR0053](0053-field-aware-search-fusion.md) and
[ADR0059](0059-configurable-similarity.md) for the unchanged lexical and scoring
owners.
