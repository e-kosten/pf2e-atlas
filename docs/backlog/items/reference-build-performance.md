# Reference build performance without inference

Status: planned
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

After the FTS comparison slice, measure source loading/admission, content and
search preparation, tokenization, checkpoint reads, row/vector publication,
validation, checkpoint pruning and commit/checkpoint time separately. Inspect SQL
query plans and repeated processing where measurements identify bottlenecks.
Compare debug/release performance, fresh cached-vector builds, unchanged updates,
lexical-only upgrades and bounded source changes. Separate inference time from
the cost of loading model assets when every input is reusable.

## Acceptance

- Reproducible phase timings and resource measurements identify dominant costs.
- Optimizations follow measured bottlenecks and retain validation, saved state,
  exact-input reuse and atomic publication guarantees.
- Representative incremental updates avoid needless full-corpus work where the
  dependency and correctness contracts permit it.
- Progress makes lengthy preparation/publication understandable; do not restart
  an active build merely to improve its progress display.
