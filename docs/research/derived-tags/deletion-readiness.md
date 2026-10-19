# Deletion-readiness audit

Audited: 2026-10-19. Acceptance: once PR82 is merged, close the chat and delete
the local derived-tag scratch documents, plans, worktrees and branches without
losing the useful ontology, implementation lessons or future design context.
This is knowledge preservation, not a promise to reproduce the obsolete app or
replay every generated review session.

## Retained independently of local state

| Knowledge | Durable home |
| --- | --- |
| Why the feature is deferred and what would justify reopening | [Entry point](README.md), ADR0054 and deferred backlog |
| History, useful vocabulary, semantic pitfalls and why the brittle matcher should not return | [History and lessons](history-and-lessons.md) |
| Stable meaning versus browse groups; native overlap; ontology/assessment loops; applicability, coverage, versions, negatives, evidence, UI/coordinator feedback and open choices | [Design if revisited](design-if-revisited.md), full October plan snapshot |
| Annotation/reviewer research, limitations and proposed evaluation | [Research and evaluation](research-and-evaluation.md) |
| Local concept model, terminology and full 515-row mapping | Five original-byte snapshots |
| Working 104-label Rust catalog, registry and candidate ontology | All 24 YAML files plus ontology document snapshotted; schema incompatibility and registry limitations summarized |
| Full June Rust plan, including schema examples and rollout assumptions | [Complete snapshot](sources/2026-06-06-rust-tagging-design.md); no preservation branch required |
| Earlier batch/applicability concerns | [Local planning excerpts](sources/earlier-local-planning.md) |
| Node concepts/projections, detailed guidance, concrete assignments, exemplars, seeds, reviewed negatives, translations, old skills and boundary fixtures | Exact paths at Node commit `94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5`, verified in main ancestry; see [source map](sources-and-cleanup.md) |
| Session format, meaningful decisions and evidence limits | Session inventory/extract, with fixtures and generated pending state identified |

The Node history check matters: a SHA in a document alone would not preserve an
unreachable commit. `git merge-base --is-ancestor` confirmed the Node revision is
already an ancestor of integrated main. It therefore survives removing the old
Node worktree and its local/archive branch names. A full-history checkout can use:

```sh
git merge-base --is-ancestor 94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5 HEAD
git show 94ae1ef5ba9a7f9cbabcfbc847cf467db17f5ab5:src/tags/canonical/relations.ts
```

The June Rust plan is not an ancestor of main; its full payload was added to
the tracked tree for that reason. Local-only registry/runtime experiments are
not being retained as implementation requirements.

## Local audit coverage

Independent Node and Rust inspections covered the primary scratch artifacts,
archived tag/browser plans, migration and translation sessions, the orphaned
tagging-design tree, the Rust seed worktree, the private Node runtime worktree,
the family-ordering worktree and their relevant branch tips.

- All five concept-mapping/notes artifacts and 25 Rust seed/ontology files match
  their snapshot hashes. The June plan is 51,389 bytes and matches its source.
- The orphaned tagging-design tree has 149 non-build files: 148 match its
  preservation revision; the extra scratch plan matches the now-retained June
  snapshot. No other unique tag-specific knowledge was found there.
- Rust seed dirt consists of the captured vocabulary/docs; no dirty code or
  authored record assignments were found. Its registry required all closed
  enums and single-parent placement; that limitation is documented.
- The non-main runtime-boundary and family-ordering tips contain the same authored
  ontology/editorial data as the integrated Node commit. Their runtime/publication
  differences are retired implementation detail. Earlier Node planning revisions
  are retained in the integrated history. The one dirty ordering file matches a known
  historical fix, whose behavior is summarized.
- The old Node reference checkout was already removed; its recorded cleanup
  identity was clean at the retained Node revision. Saved generic refactor
  notes do not contain another unique tag corpus.
- The database-design/record-envelope scratch copies contain integrated or
  earlier integrated atlas-tags sources, not unique concepts or assessments.
- Mixed future-planning notes add tag batching/applicability concerns now
  preserved as excerpts; their personal backup duplicates those ideas.

## Intentionally disposable

The Node matcher, NLP adapter, accumulated text/proximity/name blockers, old
runtime and UI wiring, local-only registry code and obsolete mapping generator
are not required for future work. Their useful rationale, limitations and outputs
are retained; integrated Node code remains available in history if needed.

Repeated generated pending review queues, source-body copies and UI cursor state
are transient bulk. Every translation draft matched its current override; no
unimported authored translation delta was found. Meaningful session decisions
and the warning about CLI test fixtures are retained. Historical status fields
and test assertions are not a human-approved calibration set.

No additional backup of these intentionally retired artifacts is needed to meet
this workstream's preservation goal. Removing existing runtime code from the
current product remains separate work.

## Verification

Result: passed. An export of the proposed Git tree validated all 37 payloads
(382,950 original bytes), the 515 mapping rows/425 surviving proposed IDs,
104 working Rust labels and 248 navigable local links. That check read no
original scratch files or old worktree contents. Node/main ancestry and the
named historical source paths were checked separately. Independent Node and
Rust closure reviews passed.

The closure check uses only the proposed tracked tree and ordinary main history:

1. Extract every manifest payload, respecting its index and final-newline flag;
   compare its byte count and SHA-256. There are 37 payloads, including the two
   selected local-planning excerpts. Original local files are not needed to
   perform this check.
2. Check navigable Markdown links outside historical code fences and confirm all
   archive paths are in the proposed Git tree.
3. Verify Node ancestry and the exact source paths named for its authored data,
   skills, planning and fixtures. No matcher, old application or classification
   job is executed.
4. Independently review the Node/Rust coverage against the useful-knowledge bar,
   rather than requiring a byte backup of discarded machinery.

This audit covers the inspected derived-tag state. Later edits or unrelated
workstreams are not authorized for deletion by this document. The merge is the
durability milestone; this PR does not itself remove local artifacts.
