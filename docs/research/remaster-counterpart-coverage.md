# Available legacy/remaster counterpart coverage

Audit date: 2026-11-06. Atlas candidate: `c70732638349fe231ed9b070b8d4507d1cb77cc9`
plus notice-copy changes that do not affect source data. Foundry PF2e source:
`28d9d156f252d084371d994dc359509b49eadf18`, verified clean. Private development
workspace: schema 30, reference generation 12. The installed workspace was not used.

## Finding

One installed two-record relationship does **not** mean only one available
legacy/remaster comparison. The extractor captures old names, but deliberately
binds records through explicit source references. Old journal operands are usually
plain names. Many have available current legacy bodies whose exact identities
have not yet been reviewed and bound.

There is enough concrete content to justify useful counterpart navigation and
comparison beyond Attack of Opportunity. Search suppression has only one installed
pair today; additional navigation candidates do not automatically grant it more
authority. Complete a bounded mapping review before claiming broad preference
coverage. No mappings or search behavior changed during this audit.

## Method and current coverage

Two independent read-only passes checked the database, pinned source records,
Remaster Changes journal and UUID redirect file. One reproduced catalog counts and
searched all declaration old terms; the other inspected every same-name candidate,
all 83 Bestiaries journal rows and all 1,515 redirect entries. Source record IDs,
types, publication metadata and relevant body fields were inspected. All 225
distinct current source roots checked by the source pass exist in the Atlas catalog.
No selected current source root was excluded by admission.

| Current projection                                      |     Count |
| ------------------------------------------------------- | --------: |
| Admitted root records                                   |    29,894 |
| Change declarations                                     |       612 |
| Old-name aliases / distinct target records              | 499 / 463 |
| Explicit current reference bindings / absent exact keys |   521 / 6 |
| Declarations with distinct current roots on both sides  |         1 |
| Approved search-preference edges                        |         1 |

The one installed edge is the reviewed glossary Attack of Opportunity → Reactive
Strike correspondence. Its selected rules text is identical. The backend's
`prefer_remaster` query/discovery machinery predates this PR; the implementation
changes its relationship authority and consumer consistency and adds the browser's
All versions choice. Existing suggested item variants and their comparison UI also
predate the remaster mapping work.

The initial candidate cohort groups records by normalized name and source type,
requiring both explicit publication eras. It contains **57 groups / 118 records**:
35 creature, 8 hazard, 6 feat, 6 action and 2 effect groups. A reproducible catalog
query is:

```sql
SELECT name_lookup_key, source_type, COUNT(*)
FROM records
WHERE name_state = 'value' AND publication_remaster_state = 'value'
GROUP BY name_lookup_key, source_type
HAVING MIN(publication_remaster) = 0 AND MAX(publication_remaster) = 1;
```

These grouping fields nominate candidates, not identity or replacement authority.

## Same-name cohort

| Disposition                                                  | Groups | Interpretation                                                                                                                                                 |
| ------------------------------------------------------------ | -----: | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Exact source-directed edge connecting current cohort members |      0 | No redirect, retained migration or exact two-root journal statement connects the census members.                                                               |
| Reviewable correspondence candidates                         |     14 | Nine core-book comparisons and five generic templates carried in older adventures. Review exact identity and scope before adding any relationship.             |
| Held                                                         |     43 | Context collisions, scenario variants or insufficient correspondence evidence. Retain independently unless later evidence establishes a specific relationship. |

The nine core-publication candidates are Amalgamite, Barghest, Giant Mantis, Hound
of Tindalos, Kuchisake-Onna, Melody on the Wind, Quatoid, Quelaunt and Sportlebore.
The five lower-priority generic-template candidates are Aeolaeka, Bul-Gae, Giant
Leech, Kun and Severed Head. This is a review queue, not 14 approved replacements.

Concrete reasons to reject broad name/era suppression include:

- **Quelaunt:** review Bestiary → Monster Core specifically. The third current
  Hell's Destiny record adds amphibious/adventure context; its matching native ID
  does not erase its different pack identity.
- **Orc Commander / Orc Scrapper:** the journal's old operands are Orc Warchief /
  Orc Brute. Same-named Troubles in Otari records are not those old operands.
- **Weakened Floor:** a cave fall and a complex toxic-mold hazard have different
  levels and solutions.
- **Tremorsense / Wavesense / Retributive Strike:** familiar, character and creature
  glossary contexts remain distinct even when their names and source types match.

## Additional journal-backed candidates

Searching every declaration's old terms found 47 declarations with another current
exact-name match. Twenty also have a same-source-type legacy candidate and a
current remaster destination. One is the unsafe class-feature/general-action/glossary
Attack of Opportunity collision. The remaining **19** merit scoped review:

| Candidate class          | Count | Cases                                                                                                                                                                                                |
| ------------------------ | ----: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Creature renames         |    12 | Air/Earth/Fire/Water Mephit → respective Scamp; Djinni → Jaathoom; Efreeti → Ifrit; Janni → Jann; Marid → Faydhaan; Shaitan → Jabali; Sea Devil Baron/Brute/Scout → Sedacthy Speaker/Marauder/Scout. |
| Other individual renames |     3 | Liar's Demise → Fraudslayer Oil; Dragging Strike → Sleek Reposition; Green Empathy → Plant Empathy.                                                                                                  |
| Mergers                  |     2 | Ranger's Bramble → Masterful Warden; Faerie Fire → Revealing Light. Preserve collective meaning; do not assume one-to-one search preference.                                                         |
| Class-scoped renames     |     2 | Iron Will → Will Expertise; Alertness → Perception Expertise. Retained old roots do not establish every class context described by the journal.                                                      |

The journal identifies the new roots explicitly and describes the old terms. It
does not supply exact old-root references. The 15 individual rename cases are the
highest-priority review queue, followed by the nine core-book
same-name correspondences. The other cases require distinct scope/disposition
decisions. No sum here is a count of safe suppression edges.

All 83 Bestiaries new references are available. Twelve old-title rows have current
legacy creature bodies; 71 lack a current exact old-title creature. Missing old
titles are not evidence of admission failures, nor a complete source-history audit.
The six exact old spell keys captured from migration 887 are also absent from the
pin; actor references to those keys do not constitute old root documents.

## Redirects

Only five of 1,515 UUID redirects resolve at both exact current root addresses:
one self-redirect, one remaster-to-remaster class-feature consolidation and three
legacy-to-remaster creature redirects. The three are scenario-qualified Dryad,
Boggard Warrior and Dero Stalker records pointing to Monster Core roots. They
establish reference redirection; scenario-specific rules still require review
before treating the targets as interchangeable search replacements.

Most other redirect predecessors are absent current roots. A matching native ID
in another pack or an adventure creature with the same name cannot fill that gap.
General UUID-redirect ingestion remains outside this focused PR.

## Meaningful comparison examples

Both current bodies are available for these journal-supported candidates:

| Candidate                          | Concrete differences in the pinned bodies                                                                   |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Djinni → Jaathoom                  | HP 71 → 55; Whirlwind versus Cloud of Visions, Naturally Invisible, Turbulent Skies and Ominous Dreams.     |
| Sea Devil Baron → Sedacthy Speaker | AC 24 → 23; Blood Frenzy/Bloodletting/Hateful Tide versus a different Speaker action/reaction set.          |
| Dragging Strike → Sleek Reposition | Pull-and-follow Strike effect versus a weapon-constrained Reposition effect with critical-success benefits. |

These are actual current-body differences, not reconstructed snapshots or an
exhaustive journal changelog. They are not yet accessible as linked pairs in the
comparison modal because their exact old endpoints remain unbound.

## Recommended next decision

Retain old-name lookup, compact context and existing variant comparison. Complete
explicit review of the 15 individual journal rename candidates first, recording
exact old/new keys and the journal scope/evidence. Review the nine same-name core
correspondences separately, with honest reviewed origin. Keep mergers, class
scope uncertainty and adventure variants out of automatic suppression unless
their own evidence supports that permission.

This would give comparison and preference concrete use beyond the equal-text AoO
pair without adding a universal name-matching heuristic. If that review is deferred
and only one preference edge remains, defer exposing the new preference choice
rather than presenting its coverage as broad. Publication of additional mappings
and the final search default remain user decisions after this audit.

Implementation is not just a larger seed list. The current reviewed-correspondence
adapter emits Item reference descriptors and stores its rationale as a contextual
qualifier. Extending it to creature roots must preserve Actor identity, and new
reviewed evidence must consolidate with journal-backed notices without repeating
the same old name. Validate those cases before publishing new mappings; preserve
the distinction between authored journal claims and reviewed exact-root bindings.

Local full-census tables, excluded cases, exact keys/paths, scripts and JSON receipts
are retained under `scratch/remaster-implementation/coverage-source-*` and
`coverage-independent-*`. The audit does not cover deleted upstream history,
unadmitted external providers, or every possible semantic correspondence among
the approximately 30,000 current bodies. It ran no ingest, policy mutation or
performance experiment.
