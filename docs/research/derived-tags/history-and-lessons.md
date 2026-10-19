# Derived-tag history and lessons

This account was consolidated on 2026-10-19. Counts describe the inspected
artifacts, not semantic accuracy or present product coverage. Exact revisions,
snapshots and cleanup limits are in the [source map](sources-and-cleanup.md).

## Node rules and editorial workflows

The early Node application used derived tags to answer questions beyond native
traits: encounter behavior, environment associations, themes and problem-solving
options. It grew a matcher with text/name/description scopes, reference predicates,
POS/lemma analysis, ordered proximity, thresholds, blockers and prerequisite tags.
Rules, translated legacy seeds, explicit assignments and exclusions contributed
to publication; composites expanded iteratively.

The concern that this became overfit has concrete history. Commit `46d13c56`
removed a deception-plus-lie fallback, generalized a gunslinger-specific tracking
blocker and replaced an exact Hampering Web phrase with proximity matching.
`bc963bfa` removed a restraint-equipment prose whitelist and generalized disguise
and ruins wording. These improved particular errors while increasing the machinery
needed to keep semantic boundaries consistent. The user explicitly wants that
semantic rule machinery discarded in a future design.

Regression examples still carry useful knowledge:

- Origin, hometown or a past visit does not establish a current habitat.
- Lake shores do not automatically imply coastal habitat.
- Swimming/cold capability does not establish aquatic/arctic identity.
- A fortress guard is not necessarily a civic official.
- Coming from or leaving Dreamlands differs from characteristically inhabiting it.

At the mature Node archive, the five new `src/tags/rules/*.ts` authored rule arrays
were empty. Runtime publication still combined legacy rules, translated seeds
and a small explicit assignment set. Do not mistake the canonical source layout
or empty new rule files for a complete replacement of legacy derivation.

Editorial tools covered batch assignment, audit, rule/seed migration, exemplars,
discovery and evaluation. Sessions separated record snapshots, decision rows and
review position, with lint/import writeback. Include/exclude assignment decisions,
positive/negative exemplars, confidence, rationale and completion were distinct.
Discovery negatives included family salience, insufficient evidence, mixed family
cues and manual-lore-only cases. Older skills allowed confident LLM assignments
directly into live source; those permissions and statuses do not establish human
review or authorize a future workflow.

## Canonical concepts and category projections

The next Node model separated meaning from placement:

| Historical component | Purpose |
| --- | --- |
| Operational concept | A domain and bounded operation, such as discovering or remediating a problem |
| Descriptive concept | Facets such as setting, theme, role, capability, delivery, effect or mechanism |
| Aggregate concept | A deliberately filterable umbrella, such as reconnaissance or problem resolution |
| Domain concept and relations | Shared semantic anchors and links between concepts |
| Category projection | Browse axis/family, local description, assignment guidance and eligibility |

Live canonical/projection plumbing first appeared partially in `15971b52`,
initially compiled from category-local definitions; `ba9db0cb` moved toward
canonical-first authoring. The mature archive contains independent canonical
trees, while some older editorial docs still describe compiler-derived truth.
The archive implemented four `counteracts` relations connecting curse, disease,
petrification and poison remediation with application concepts.

The [preserved mapping](sources/node-concept-model/derived-tag-concept-mapping.csv.md)
has 515 category projections: creature 157, equipment 124, spell 105, hazard 76,
affliction 53. It proposes 323 descriptive, 165 operational and 26 aggregate rows,
with one dropped concept (`beneficial`); surviving rows refer to 425 distinct
proposed IDs. Its 315 high/200 medium confidence values concern **mapping choices**,
not record assignments. Some canonical definitions remained placeholders such as
“Hazard effect and countermeasure tags”; a rich schema did not guarantee precise
meaning.

Useful vocabulary refinements included `creature_family`, `outbreak_containment`,
`time_critical_resolution`, `observation_driven`, and splitting heterogeneous
equipment function/payload buckets into meaningful operations, capabilities and
delivery facets. Remaining mapping questions centered on spell support/control,
hazard countermeasures/environmental danger and equipment antimagic. These were
historical decisions in that experiment, not frozen requirements for a revival.

The strongest lesson is to distinguish the direction of a claim. A spell that
supplies countermagic and a hazard resolved through countermagic can be related
without having the same filter meaning. Likewise, inflicting a disease, diagnosing
it, treating it and protecting against it answer different questions. A label
can summarize such a claim without requiring a full formal ontology.

Node publication enforced one family per category/tag. The user later preferred
multiple browse homes independent of stable concept identity. Do not revive that
old restriction simply because the archived implementation imposed it.

The mature Node commit `94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5` is an ancestor
of integrated main. Its detailed guidance, projection restrictions, concrete
assignments/exemplars, legacy seed memberships, reviewed negatives, translation
notes and old agent skills remain recoverable from main history after deleting
local branches and worktrees. The [source map](sources-and-cleanup.md) gives exact
paths; the mapping CSV alone does not contain that guidance. Historical claimed
human/LLM sources and expected verdicts require fresh validation before reuse.

Useful cross-category contrasts from the old fixtures are:

| Area | Question worth retaining |
| --- | --- |
| Creature | Does evidence establish habitat, motif or identity, rather than a past location, figurative wording or incidental capability? |
| Equipment | Does an item provide access, disguise, communication, expedition support or a hostile payload, rather than merely mentioning the subject? |
| Spell | Is the effect infiltration, sensory support, transformation, control, remediation or protection; and is the claimed function narrow enough? |
| Hazard | Distinguish trigger/mechanism, encounter function, environment, manifestation, lure/compulsion and the party's resolution path. |
| Affliction | Distinguish impairment, transmission/progression, metaphysical theme and a remedy, rather than treating them as one concept. |
| Native overlap | Does a concept add a useful question beyond existing native traits and typed facts? |

For example, an illusion that changes identity differs from ordinary sensory
support; an item that conceals a weapon differs from one that aids scouting;
a glyph's trigger differs from how a party disables it. These are useful test
questions, not endorsed historical assignments. Exact inputs/assertions remain in
the six test files named in the source map, without copying the old test suite.

## Browser and discovery experiments

The old ontology browser also mixed native search vocabulary with authored tags.
Its historical plans preserve useful interaction distinctions:

- Matching discovery showed nonzero tags; Catalog discovery could include
  zero-count tags, still actionable for searching.
- Cheap summaries and lazy expansion avoided building every value branch just
  to open a small picker.
- An unloaded branch was not an empty branch. Async resolution supplied required
  data, while session-owned materialized state described what the UI had loaded.

These are reusable lessons about counts, completeness and navigation, not a reason
to restore the retired Node TUI or its public tree contracts. Completion of each
old plan is not inferred from its age or presence in the archive.

## Rust model, loader and seed

ADR0028 moved toward global tags, typed applicability, shallow display groups,
YAML authoring, record-centered assignments, reviewed-empty entries, bounded
evidence and an agent-first workflow. It proposed unanimous automated acceptance
after reconsideration and authoritative `record_tags` filtering. The large June
design plan was separately preserved on `preserve/tagging-design`.

Its [complete text is now included here](sources/2026-06-06-rust-tagging-design.md),
so that preservation branch and the orphaned design worktree are unnecessary for
recovering the plan.

The integrated Rust implementation contains `atlas-tags` model types, strict YAML
parsing, corpus merging and structural validation, plus `atlas tags validate`.
It does **not** implement record classification, applicability evaluation,
source-record existence checks, source-grounded evidence verification, worklists,
reconciliation/import, an assessment ledger, runtime tag rows or product filtering.
Evidence may be empty; supplied evidence fields are checked for structural validity,
not semantic truth. A successful validator run is not labeling quality evidence.

A separate local seed branch expanded groups/subgroups into an optional registry.
The current integrated validator has 9 groups/36 subgroups; that experiment has
10 groups/61 subgroups. Its committed seed contains 54 labels in 9 catalog files;
uncommitted work expands to 104 labels in 14 files. The complete working vocabulary
and guidance are [snapshotted](sources-and-cleanup.md#local-authored-snapshots).
They cannot simply be loaded with today's integrated validator vocabulary.

The optional registry externalized labels/descriptions but retained closed Rust
enums: every one of the 10 groups and 61 subgroups had to appear exactly once,
with nonempty labels/descriptions and each subgroup under its one hardcoded
parent. Missing/duplicate entries and wrong placement were rejected. This was
externalized presentation metadata, not extensible or multi-home organization.
The old seed test expected the committed 54 labels; it does not demonstrate
validation or semantic review of the later 104-label working corpus.

| Historical group | Committed seed | Working snapshot |
| --- | ---: | ---: |
| CharacterBuild | 6 | 14 |
| CombatEffect | 6 | 10 |
| EncounterRole | 7 | 7 |
| Exploration | 7 | 11 |
| ProblemResponse | 6 | 35 |
| SceneChallenge | 4 | 9 |
| Setting | 8 | 8 |
| StatusEffect | 6 | 6 |
| Theme | 4 | 4 |
| UsagePattern | 0 | 0 |
| Total | 54 | 104 |

No authored record assignments were found in that seed worktree. UsagePattern
has registry descriptions without actual seed labels. Positive/negative guidance,
source detection versus neutralization, restraint escape, contamination cleanup,
ritual appeasement and personal/party/reactive defense are useful research inputs.

Unresolved faults remain: Curse Removal includes identification/revelation;
Disease Treatment includes diagnosis; source detection mixes supplying detection
with being a scene that requires investigation. Stealth and area/burst damage
appear in multiple groups without a settled meaning distinction. Some themes
exclude RollTables despite no demonstrated semantic reason. Status labels may
duplicate native facts, while browse-shaped ID namespaces pressure identity to
change when organization changes. No seed study measured added value over the
improved search.

## October discussion and current decision

The October discussion reopened YAML, single-home placement, minimal provenance,
negative-state handling and unanimous acceptance. It favored separate ontology
and assessment loops, broad claim-based applicability, human approval of
agent-prepared calibration, evidence-backed explanations, exhaustive eligible
coverage and explicit version/reassessment tracking. Those directions are
preserved in [design if revisited](design-if-revisited.md), with unresolved choices
still open.

On 2026-10-19 the user moved away from proactively building tags. Improved
semantic search may remove the need. Retain the experiments and their lessons;
resume from demonstrated gaps rather than finishing an ontology for its own sake.
