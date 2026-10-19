# Ingest and reference build performance

Status: in_progress
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-18

## Problem

The full-corpus FTS preview build took 2,139,209 ms (about 36 minutes) for
25,641 records while reusing 260,040 unique inputs across BGE Base and
EmbeddingGemma, with zero new embeddings. This was a fresh workspace with imported
verified checkpoints, not an unchanged-source update or lexical-only backfill.
It demonstrates expensive non-inference work but does not identify its cause.
The build used a debug binary and suppressed routine progress.

Local evidence: `scratch/search-fusion-review/preview/build.json`, associated
workspace and checkpoint-seeding helper. Preserve research vectors and grading
evidence; do not repeat expensive inference to profile unrelated stages.

## Scope

Measure source loading/admission, content and
search preparation, tokenization, checkpoint reads, row/vector publication,
validation, checkpoint pruning and commit/checkpoint time separately. Inspect SQL
query plans and repeated processing where measurements identify bottlenecks.
Compare debug/release performance, fresh cached-vector builds, unchanged updates,
lexical-only upgrades and bounded source changes. Separate inference time from
the cost of loading model assets when every input is reusable.

Also evaluate ordinary inference builds: input preparation and batching,
checkpoint overhead, peak memory, and temporary disk use. Preserve existing
full-corpus research assets and use bounded inference measurements unless a new
full build is necessary to answer a concrete performance question.

## Acceptance

- Reproducible phase timings and resource measurements identify dominant costs.
- Optimizations follow measured bottlenecks and retain validation, saved state,
  exact-input reuse and atomic publication guarantees.
- Representative incremental updates avoid needless full-corpus work where the
  dependency and correctness contracts permit it.
- Progress makes lengthy preparation/publication understandable; do not restart
  an active build merely to improve its progress display.

## Implementation and measurements

The implementation removes duplicate full-batch native token validation, loads
inference models only for uncached inputs, borrows checkpoint and pending-insert
vectors, and reuses each root's validated prepared content. Sequential vector
validation uses an 8 MiB SQLite cache; dirty pages are flushed before validation
without committing the publication transaction. Model inputs, execution policy,
stored vector format and validation guarantees remain unchanged.

Local macOS ARM64 release measurements, with BGE Base and EmbeddingGemma over
25,641 roots and 270,070 attributed semantic units:

| Measurement | Integration baseline | Optimized |
| --- | --- | --- |
| Unchanged build, all vectors reused | 308.85s | 174.78s |
| Maximum resident size, same runs | 7.39 GiB | 4.80 GiB |
| Fresh build with verified checkpoints | Not measured on this exact content version | 221.05s |

These are serial local warm-cache runs, not portable timing guarantees. The
historical unmodified release fresh build took 341.98s with the earlier content
selection version; the original 2,139s build used debug compilation. Keep those
observations separate from the matched unchanged-build comparison.

Exact data/state table digests match across the unchanged builds; fresh snapshot
and checkpoint digests match too. The remaining largest measured phase is shared
passage segmentation/tokenization, about 70s. Preserve its literal input, chunk
and attribution contracts in any future optimization. Evidence, frozen binaries
and research probes are retained under `scratch/ingest-performance/`.
