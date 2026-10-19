# October 2026 search evaluation archive

This is a frozen analytical dataset, not a production test oracle or a new search
implementation. It preserves the eight-configuration semantic study and final
lexical/fusion comparison. Read the [synthesis](../../search-evaluation-synthesis.md)
for conclusions and [local archive map](../../search-evaluation-handoff.md) for
full source packets, vectors, full-corpus rankings and earlier experiments.

## Contents and provenance

- `semantic/`: 48 concepts (24 development, 24 then-fresh), 91 observations,
  eight model/representation configurations, 2,372 final query/root judgments,
  individual selected opinions and final decisions, top-ten
  rankings and independently nominated reference ranks. `method/` preserves the
  rubric, reviewer tasks, expanded-review amendment, model contracts, corpus/input
  bindings, review reliability, uncertainty results, and separate cost reports.
- `lexical/`: development and holdout query/filter contexts across two semantic
  profiles and five lexical/fusion configurations. The pool includes the 14 new
  pairs from the final matrix extension. Individual opinions, final utilities,
  adjudications, disagreements and unresolved cases remain distinguishable.
  Mean/low/high metrics and contrasts use the same expanded pool.
- `manifest.json`: original repository-relative paths, original file hashes, and
  exported file hashes. These bind the transformation to the untouched local
  archive; they do not prove that the original judgments were correct.

The curated subset is about 12 MiB. JSONL keeps one judgment or ranking per line.
Tabular JSON arrays keep each row on one line. The standalone TypeScript verifier
is an archive integrity tool, not a product command.

Literal source evidence quotations are replaced by `quoteSha256`; their field
pointers and source-file hashes remain. Analyst rationales and record names remain
readable. Paired double-quoted spans inside rationales are marked as omitted,
with `originalRationaleSha256` retaining the original binding; this includes
quotations embedded in analyst prose, not only explicit evidence fields. Absolute checkout prefixes become repository-relative paths. This avoids
republishing full source passages while retaining bindings for local source review.
The unchanged original packets still contain the quotations. Reinstating a source
quote requires inspecting the pinned source and matching its digest, not assuming
that today's Foundry text is identical.

Rankings retain top ten and known-reference ranks, not every candidate. They can
replay the published point metrics and support alternative slices of the judged
pool, but cannot evaluate an arbitrary new ranking function or establish corpus
recall. Use the full local captures for that. Provisional reviewer submissions, superseded
corrections and intermediate reports remain only in the local archive. Selected
opinions and final decisions remain here; semantic `finalOpinion` and
`primaryOpinion` index into each judgment’s deduplicated `opinions` array.
Independent second reviews use `opinionIndex`; third source audits retain their
original structure. Query intent and criteria live in the query sets, rather
than being repeated for every judged record. Blind IDs are scoped to their
experiment and cohort, not global record identities.

All opened holdouts are now exposed. The lexical matrix extension and rubric
realignment are exploratory, not a fresh confirmatory trial. Semantic scope pairs
are averaged within concepts; lexical contexts within query clusters. They are
not independent extra queries. Missing/unknown judgments do not mean irrelevant.
The lexical `hold-41` cluster is excluded consistently because a source judgment
remains unresolved; low/high grade scenarios are not confidence intervals.

## Verify the frozen selection

With the repository’s Node 22 toolchain (or newer) (no npm packages, database, source checkout or model needed):

```sh
node --experimental-strip-types docs/research/evaluations/2026-10-search/verify.ts
```

The verifier checks exported hashes, query/ranking/judgment joins and recomputes
22,533 summary metric and judgment-utility values, including semantic overall
averages and all three lexical judgment scenarios. Per-query metrics are computed
from rankings and judgments rather than saved again in reports. Bootstrap intervals
and source interpretations are retained evidence, not independently rerun here.

The session-specific exporter and selection scripts live with the ignored local
archive under `scratch/release-handoff/archive-selection/`. They are not maintained
repository tooling. The manifest retains the original source-file bindings,
including inputs now represented only through selected final evidence. Correction
work must create a new version rather than silently replacing this frozen dataset.

Protocol documents are historical copies: relative paths in those documents refer
to their original archive locations recorded in the manifest. Their statements
about sealed queries, execution approval, or pending product adoption describe
that stage of the experiment, not current authorization or product behavior.
