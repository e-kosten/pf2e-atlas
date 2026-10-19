# Derived-tag sources and cleanup boundaries

Inventory date: 2026-10-19. Integration baseline:
`21e71ed3c519fcf0937d8a7a315c8d0c3ea5ec9f`, locally available `origin/main`.
The [closure audit](deletion-readiness.md) subsequently verified that the mature
Node commit is an ancestor of integrated main. Its authored corpus and fixtures
need no separate preservation branch. Local-only knowledge and the full June
Rust plan are stored directly in this package. Obsolete executable machinery and
generated session bulk are deliberately retired, not unresolved backup work.

## Git history anchors

Use exact commits rather than a worktree's current path or branch name. The Node
example below works from main's history; the Rust plan is now also included as a
[complete snapshot](sources/2026-06-06-rust-tagging-design.md).

```sh
git show 94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5:src/tags/canonical/relations.ts
```

| Generation | Exact revision and important paths | Retention observed |
| --- | --- | --- |
| Early scored rules | `e1039dfbeb0d5ef9c1a6f975af49ce7f4428716b`, `fe72a4f272c524fc1b96cba0dc53db4c9e7d5d76`; `src/derived-tags.ts`, `tests/derived-tags.test.ts` | Node history |
| Overfit rule repairs | `46d13c56dcdb82a54c93ad6618ef6d60a5757e1c`, `bc963bfa4de3877b7c55ff134ba0f1e1c09b477b`; `src/tags/index.ts`, `tests/tags/derived-tags.test.ts` | Node history |
| Canonical transition | `15971b52906d334c7a0c89f9d38240ae227e413c`, then `ba9db0cb88df97bc1eea744ddb76c4af842bdae8` | Node history |
| Mature Node subsystem | `94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5` | Verified ancestor of main; archive refs are optional convenience names |
| Node runtime assembly | `20cb368d1943cf599a3f103874220a554ec3e3fd`; `src/tags/runtime/derivation/runtime-builder.ts` | Old runtime-boundary worktree; also inspect mature Node archive |
| Family ordering | `0326d1105cdbd07600f420730216a87ba527593d`, later fix `93c926d559542caf6f444f55be0329a8d614c324`; `src/app/ontology/search-semantics/derived-tag-builder.ts` | Old family-ordering branches/worktree; its dirty file matches the later commit |
| Accepted Rust design | `c17c6c648ad33708145d0b9603aabfee0222f5db`; tagging architecture and ADR0028 | Integrated ancestry |
| Rust model | `af1f47b64bfd19dce098d5897240c1266f8f7dbe`; `crates/atlas-tags/` | Integrated ancestry |
| Corpus loader | `953766afd36b1c603acf9082a66d5f2c6dc5de81`; `crates/atlas-tags/src/corpus.rs` | Integrated ancestry |
| Validation CLI | `f1fa08f3f9af929c43015477dc684b727c820f29`; `crates/atlas-cli/src/commands/tags.rs` and `commands/tags/args.rs` | Integrated ancestry |
| Full Rust design plan | `7160b9c8642dd61970aa504612c1af1b5c735896`; `docs/backlog/history/items/rust-tagging-design-plan.md` | Complete payload snapshotted here; no retaining branch required |
| Local ontology/registry experiment | `b05582fc6516bf861c81d80742d5591dc5f23ca6`, `7a17cec721b4df418eae65231ebb2aec26735c72`, tip `aa04bf804039f7d46411a3cf2296ebc36aa22d0c` | Local-only `feat/tag-corpus-seed`; working authored knowledge snapshotted below |
| Source-backed contract adjustment | `7cf3c5fbb1a5a3d5c34aa250022d5fa4573eb8f8`; tag DTO dependencies and retired metric predicates | Integrated ancestry |

At the mature Node anchor, useful paths are:

- `src/tags/canonical/concepts/`, `canonical/projections/`, `canonical/relations.ts`
  and `src/domain/derived-tag-concept-types.ts` for semantics.
- `src/tags/runtime/derivation/runtime-builder.ts`, `runtime/publication/catalog.ts`,
  `runtime/matcher/engine.ts` and `legacy-rules/` for the retired machinery.
- `src/tags/editorial/`, `cli/editorial/`, `discovery/`, `evaluation/`,
  `assignments/packs/`, `exemplars/` and `reviews/` for workflow/data examples.
- `.codex/skills/pf2e-tag-{audit,batch,rules,seeds,exemplars,legacy-migration}/SKILL.md`
  for the earlier agent workflow. Historical instructions are not current authority.
- `tests/tags/derived-tags-creature.test.ts` for habitat/identity boundaries.
- `tests/tags/derived-tags-{affliction,equipment,hazard,spell}.test.ts` and
  `tests/tags/native-ontology-guard.test.ts` for the other category contrasts and
  native-overlap checks. Historical test assertions are not calibrated truth.
- `src/tags/legacy-seed-migrations/{creature,hazard,spell}.ts` and
  `src/tags/reviews/discovery-reviewed-records.ts` for candidate memberships and
  reviewed-negative classes. Other assignment-memory/assignment-review/exemplar-
  review registries were empty at this revision.
- `src/tags/translations/{base-mappings,family-defaults,tag-overrides}.ts` for
  mapping rationale and authored overrides.
- `docs/backlog/items/derived-tag-{assignments-layout,concept-model-implementation,index-service-layer,manifest-tooling-metadata,ontology-next-review-questions}.md`,
  `docs/backlog/items/tagging-tooling-reorg.md` and `docs/architecture/editorial.md`
  for the old storage, rollout and editorial contracts.
- `77e4741a4685a49a3a4e0dc7d1b181b44d911174:docs/backlog/items/derived-tag-ontology-next-review-questions.md`
  for unresolved Node mapping families.

The June Rust plan includes schema examples, reviewed-empty and suggestion packets,
proposed traits, layout, owners, storage, agent CLI operations, eight rollout slices
and validation assumptions. The [history](history-and-lessons.md) and
[future design](design-if-revisited.md) retain its useful intent and later changes.
Its exact full text is now included in the tracked tree. The separate preservation
branch is not needed to retain it after this PR merges.

The Node source uses a small declarative DSL. `projectionFamily` pairs a family
descriptor with per-tag seeds; each key gives `currentTag`, default concept is
`seed.concept ?? currentTag`, default ID is `category:tag`, default label is
`seed.label ?? conceptId`, and default translation status is mapped. Facet helpers
attach kind/value; assignment-map keys are record identities and `tag()` packages
the authored decision. These details make the old definitions readable without
recreating its build toolchain. They do not endorse category ownership or the
old matcher for a revival.

## Local authored snapshots

These were copied from local files or Git objects, including uncommitted work,
into inert Markdown payloads. Original UTF-8 content is enclosed in sufficiently
long code fences; indexed archives contain several payloads identified by the manifest.
Links and instructions inside are historical text. [The manifest](sources/manifest.md)
records each original path, payload index where needed, byte count and SHA-256.
Extract the payload, preserve
its original final-newline state and verify the hash to recover the bytes.
These archives are not inputs to normal builds, corpus validation or product search.

| Material | Durable snapshot |
| --- | --- |
| Node canonical model | [Concept model](sources/node-concept-model/derived-tag-concept-model.md) |
| Complete 515-row proposed mapping | [CSV payload](sources/node-concept-model/derived-tag-concept-mapping.csv.md) |
| Mapping counts/renames | [Mapping summary](sources/node-concept-model/derived-tag-concept-mapping-summary.md) |
| Terminology and ontology dimensions | [Glossary](sources/node-concept-model/derived-tag-ontology-glossary.md), [shape primer](sources/node-concept-model/derived-tag-ontology-shape-primer.md) |
| October ontology/assessment discussion | [October 9 plan, captured October 13 revision](sources/2026-10-09-design-discussion.md) |
| Full June Rust design, schema sketches and rollout | [Complete plan](sources/2026-06-06-rust-tagging-design.md) |
| Earlier local batch/applicability planning | [Tag-specific scratch excerpts](sources/earlier-local-planning.md) |
| Later Rust candidate enumeration | [Tag ontology](sources/rust-seed/tag-ontology.md) |
| Working seed catalog | [Catalog snapshots](sources/rust-seed/catalog/) and [registry snapshots](sources/rust-seed/registry/); all 24 files indexed in the manifest |
| Node Matching/Catalog semantics | [Mode semantics plan](sources/node-browser-plans/2026-04-26-ontology-derived-tag-mode-semantics-plan.md) |
| Lazy browser bootstrap | [Cold-open plan](sources/node-browser-plans/ontology-browser-search-semantics-cold-open-plan-2026-04-20.md) |
| Unloaded versus empty branches | [Child resolution plan](sources/node-browser-plans/2026-05-06-ontology-child-resolution-contract-plan.md) |

The Node mapping/notes had no identical blob found in reachable Git history during
the audit. The local mapping generator, `generate-derived-tag-concept-mapping.mjs`
(47,803 bytes), imports the retired `src/tags/ontology/*.ts` layout. Its output and
conceptual notes are preserved; its executable implementation is deliberately
omitted. No additional unmaterialized semantic knowledge was found in it. Its
implementation is intentionally disposable; the archived CSV is the preserved
result and is not claimed to regenerate against current code.

The Rust archive captures the entire working catalog and registry, not just its
54-label committed baseline. It includes uncommitted expansion to 104 labels and
renames: `condition_cleanup` → `condition_removal`, `source_tracing` →
`source_detection`, `aquatic_support` → `aquatic_travel`, `stealth_support` →
`stealth`, and scene IDs moving from `hazard.challenge/spatial` to
`scene.resolution/spatial`. Source detection is distinguished from neutralization.
These are recorded transformations, not aliases supported by current runtime.

The dirty seed architecture note also described candidate enumeration: mine prior
work and real queries, merge/normalize, drop native-data duplicates, split only
useful retrieval concepts, author positive/negative guidance and validate/review
coherent groups. Its proposed order was ProblemResponse, Exploration, CombatEffect,
CharacterBuild, EncounterRole, StatusEffect, SceneChallenge, Setting, UsagePattern,
Theme. The sequence is historical process guidance, not a resumed work queue.

## Generated sessions and review claims

[The session inventory and examples](sources/session-inventory-and-examples.md)
preserve original file hashes, migration manifests, all nonpending/noted decision
rows, representative rationale templates and a translation format example. It is
a curated extract, not an exact backup of every original payload.

- Migration: 38 sessions/190 files, 77,759,336 bytes; 70,029 decision rows,
  of which 70,024 say `needs_review` and five `complete`. There are 2,571 nonempty
  decision-row occurrences, many repeated/generated. The seven distinct rationale
  strings consist of legacy-seed review prompts, an urban-setting exemplar prompt
  and explicit temporary CLI fixture prompts. Seven distinct nonpending/noted rows
  are retained with their session/line occurrences.
- All 6,223 nested decisions name `llm` as source; 6,216 are pending, four approved
  and three rejected. These labels are historical UI state, not proof of who
  approved them or that the verdict is correct. Even complete/approved rows need
  source review before reuse as calibration.
- Translation: 55 sessions, 2,112,606 bytes and 2,593 row snapshots. Every session
  has `imported:false`; no draft override differs from its current override. No
  pending authored translation delta was found. Generated snapshots are not an
  undiscovered body of edits.

Repeated queue snapshots, record-body copies, cursor state and pending generated
decisions are deliberately discarded as transient workflow bulk. The meaningful
decisions and review limitations are retained. Exact session replay is not part
of the useful-knowledge preservation goal and requires no additional backup for
this workstream.

## Cleanup disposition

This task removes no old artifacts. A later cleanup can use the following map:

| Local artifact | What is preserved; what to check before removal |
| --- | --- |
| `scratch/derived-tag-ontology/` | All five notes/mapping artifacts are byte-recoverable snapshots. Obsolete generator code is omitted deliberately. |
| `scratch/plans/2026-10-09-concept-labeling-and-ontology-design.md` | Full current text is snapshotted; readable discussion and current deferral are in this package. Historical authorization clauses remain inert. |
| Three archived Node browser plans | Full text snapshotted; useful interaction lessons summarized. Other inspected generic Node search/refactor plans add no unique labeling knowledge. |
| `.worktrees/tag-corpus-loader` | All working catalog/registry YAML and ontology proposal preserved; registry tradeoffs summarized. Local-only registry implementation is intentionally retired. |
| `.worktrees/tagging-design` | All 149 non-build files checked: 148 match the preservation commit; its extra plan is now snapshotted here. No unique tag-specific content remains outside this package/history. |
| `.worktrees/pathfinder-mcp-derived-tag-order` | Dirty builder matches `93c926d5`; its broad-before-specific ordering is summarized. No unique authored ontology data differs from the integrated Node revision. |
| Private derived-runtime worktree | Clean at `20cb368d`; no unique authored ontology data differs from integrated Node. Runtime wiring is intentionally retired. |
| Old Node reference worktree | Already removed; cleanup identity recorded clean `94ae1ef5`. Authored data, workflows, fixtures and code are retained in main history. |
| `scratch/migration-sessions/`, `scratch/translation-sessions/` | Useful state and inventory retained; repeated queues, copied source bodies and UI cursor state intentionally discarded. |
| Old derived-tag branches/remote preservation refs | No separate ref is required: Node authored sources are in main ancestry; June Rust plan and local-only knowledge are snapshotted. Non-main runtime/registry experiments are intentionally retired. |

After the PR merges, the listed derived-tag state is no longer required to retain
the useful knowledge. This is a knowledge-preservation conclusion, not a command
to delete unrelated work or later edits. [The closure audit](deletion-readiness.md)
records verification and scope. Old commits outside main are optional forensic
context; none is the only remaining source of an important design idea.
