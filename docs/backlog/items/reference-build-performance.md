# Ingest and reference build performance

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-10-29

## Remaining work

PR74 completed the measured optimization below. Further work should target shared
passage segmentation/tokenization, broader hardware measurements, and bounded
inference/batching profiles where an actual bottleneck warrants it. Do not repeat
the original full build or imply that its debug timing measures release behavior.
New optimization requires a measured hypothesis and unchanged output/state checks.
PR103 removes duplicate native-vector storage and sparse index capacity; its
measurement is below. Shared segmentation/tokenization remains separate follow-up.

### Deep artifact validation during maintenance

A schema19-to22 shared-seed refresh on macOS ARM64 used 22s for private staging,
594s for controlled setup/upgrade and 336s for snapshot publication (952s total).
This used a dev executable and reused the published corpus/vectors, with no corpus
embedding regeneration. Publication still ran query inference for its per-profile
smoke tests. Sampling during upgrade placed the main thread in
reference validation. These observations motivate profiling, not a release timing
claim or evidence that every validation pass is redundant.

Measure deep-validation components and SQL plans on the same snapshot with
matched dev/release binaries before changing algorithms. Investigate repeated
record decoding, field/embedding attribution checks and row-at-a-time access
where measurements support it. Keep atomic migration and reference-projection
upgrade validation, validation of the exact copied publication snapshot, saved
state and per-profile query probes.
Use corruption/rollback cases and unchanged data digests to verify any optimization.
This work is separate from preview admission, model verification reuse and frozen
maintenance orchestration in the development workflow.

### Trait validation corpus loading

Trait association validation now accumulates compact evidence during the existing
checked-record identity pass. It no longer reconstructs and retains whole-corpus
build inputs. Required cached-row metadata and admission diagnostics are validated
per record, including unpublished fields; exact trait evidence and whole-field
rendering-success checks remain.

Serial macOS ARM64 measurements, Rust1.95.0, schema22, the same29,894-record
two-profile corpus (BGE Base and EmbeddingGemma), baseline965f8123 and candidate
a3b191f2:

| Deep validation           |   Baseline | Candidate |
| ------------------------- | ---------: | --------: |
| Dev elapsed               |    273.26s |   260.09s |
| Dev observed peak RSS     | 1,573.7MiB |  362.8MiB |
| Release elapsed           |     49.99s |    48.11s |
| Release observed peak RSS | 1,619.9MiB |  404.4MiB |

Memory was sampled once per second; these are local observations, not portable
limits. The measured memory reduction is75–77%. Elapsed differences are small
enough that these runs do not establish a dependable speedup. The full16-minute
maintenance operation was not rerun. All validation reports/counts/source
fingerprints matched, and the database, including saved state, remained
byte-identical. No corpus embedding regeneration was used.

## Original observation

The full-corpus FTS preview build took 2,139,209 ms (about 36 minutes) for
25,641 records while reusing 260,040 unique inputs across BGE Base and
EmbeddingGemma, with zero new embeddings. This was a fresh workspace with imported
verified checkpoints, not an unchanged-source update or lexical-only backfill.
It demonstrates expensive non-inference work but does not identify its cause.
The build used a debug binary and suppressed routine progress.

Local evidence: `scratch/search-fusion-review/preview/build.json`, associated
workspace and checkpoint-seeding helper. Retain small measurement summaries; use existing compatible vectors rather than
repeating expensive inference to profile unrelated stages. Repository development
workspaces and compiler outputs follow the disposable-state guidance.

## Scope

Measure source loading/admission, content and
search preparation, tokenization, checkpoint reads, row/vector publication,
validation, checkpoint pruning and commit/checkpoint time separately. Inspect SQL
query plans and repeated processing where measurements identify bottlenecks.
Compare debug/release performance, fresh cached-vector builds, unchanged updates,
lexical-only upgrades and bounded source changes. Separate inference time from
the cost of loading model assets when every input is reusable.

Also evaluate ordinary inference builds: input preparation and batching,
pending-checkpoint overhead, peak memory, and temporary disk use. Use bounded
inference measurements and at most one disposable full-corpus snapshot for an
actual corpus question; remove it and its temporary migration backup afterward.

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

| Measurement                           | Integration baseline                       | Optimized |
| ------------------------------------- | ------------------------------------------ | --------- |
| Unchanged build, all vectors reused   | 308.85s                                    | 174.78s   |
| Maximum resident size, same runs      | 7.39 GiB                                   | 4.80 GiB  |
| Fresh build with verified checkpoints | Not measured on this exact content version | 221.05s   |

These are serial local warm-cache runs, not portable timing guarantees. The
historical unmodified release fresh build took 341.98s with the earlier content
selection version; the original 2,139s build used debug compilation. Keep those
observations separate from the matched unchanged-build comparison.

Exact data/state table digests match across the unchanged builds; fresh snapshot
and checkpoint digests match too. The remaining largest measured phase is shared
passage segmentation/tokenization, about 70s. Preserve its literal input, chunk
and attribution contracts in any future optimization. Evidence, frozen binaries
and research probes are retained under `scratch/ingest-performance/`.

## Native embedding storage (PR103)

The measured PF2e 8.6.1 corpus (revision `28d9d156f252d084371d994dc359509b49eadf18`)
contains 29,894 roots and 161,731 semantic occurrences per profile, with BGE Base
and EmbeddingGemma both retained. Workspace migration 10 reuses published vec0
values directly, retains pending computations only until successful publication,
normalizes model/input metadata, shares passage addresses and stores trait native
values/centroids as binary f32. It preserves one KNN entry per source occurrence.

Local macOS ARM64, Rust 1.95.0 dev-profile probe on implementation `a93d0423`:

| Measurement                                   |              Schema 9 |  Schema 10 after explicit reclamation |
| --------------------------------------------- | --------------------: | ------------------------------------: |
| Workspace bytes                               |         4,315,258,880 |                         1,612,591,104 |
| Workspace GiB                                 |                 4.019 |                                 1.502 |
| Vector blocks, each profile                   |                   268 |                                   158 |
| Allocated vector slots, each profile          |               274,432 |                               161,792 |
| Reusable native inputs, both profiles         | 323,016 cached inputs | 323,016 supplied by published storage |
| Separate checkpoint vectors after publication |               323,016 |                                     0 |

The file shrank by 2,702,667,776 bytes (2.517 GiB, 62.6%). This excludes the normal
single pre-upgrade backup, which remains part of installed-workspace upgrade
safety. No embedding inference, model change, quantization or content reduction
was used. Every vector byte, occurrence address and saved-domain table digest
matched. Six filtered KNN comparisons (both models at k=1,25,4096) retained exact
distance sequences; tied witness IDs are deliberately not required to match.
Every native input was reusable, and the migrated workspace passed deep validation.

The one-time debug migration, including packing, took 374.63s; full reuse reads
were 51.18s/52.43s per profile; deep validation took 243.75s. Peak resident size for
the entire probe was 1,700,954,112 bytes (1.584 GiB). These are diagnostic timings,
with other development validation running, not a matched release benchmark or a
portable latency guarantee. Sample filtered queries took 0.25–0.45s afterward;
that observation does not establish a general search-speed improvement. Temporary
space must accommodate the snapshot, normal migration backup, WAL and staging for
one profile. No precise peak temporary-disk measurement was captured.

The ignored [`embedding_storage` integration test](../../../crates/atlas-dev/tests/embedding_storage.rs)
reproduces the checks using `ATLAS_STORAGE_PROBE_SOURCE`; it snapshots rather than
migrates its source and deletes all probe copies/backups afterward. Small owning
fixtures cover failed migration/publication rollback, interrupted forced builds,
trait-only/document value transitions, input corruption and compaction rollback.
Bounded native-model tests cover zero inference on unchanged input, one changed
passage, forced inference, profile changes and preservation after a failed build.
No retired-task database/archive is required to retain this evidence.

### Published passage-layout reuse experiment

A macOS ARM64 release preparation experiment over 29,894 records compared fresh
splitting with replay of already-published byte ranges. Both paths reused existing
checked source/prepared HTML and ran native validation for BGE Base and
EmbeddingGemma. No inference or publication occurred in this comparison.

| Case                                      | Preparation, including reading prior inputs | Native profile validation | Combined |
| ----------------------------------------- | ------------------------------------------- | ------------------------- | -------- |
| Fresh splitting, passage layouts disabled | 114.20s                                     | 40.87s                    | 155.07s  |
| Unchanged source, published layouts       | 57.95s                                      | 41.02s                    | 98.97s   |
| One private Heal description update       | 59.89s                                      | 40.74s                    | 100.63s  |

All 161,735 document inputs and 793 trait inputs were checked for both profiles.
Unchanged preparation matched fresh text, addresses, counts and execution
identities. The bounded update matched an independent fresh preparation of the
changed root; every other root matched the original. It reused 123,582 sections,
versus 123,583 for unchanged input, from 131,922 published passage layouts.

The first case measures forced fresh splitting, not recipe admission. Separate
regressions check passage-policy, segmentation-tokenizer and input-recipe misses;
they fall back to the same fresh path. These timings used the frozen prototype
before additional model/kind integrity checks and earlier release of per-record
layout metadata. They are preparation measurements, not end-to-end setup claims.
The existing locations/input hashes supply reuse without a new schema, persisted
cache or duplicate text corpus. Native token validation remains unchanged.

The earlier candidate (`e02cb2af`), which allowed layout reuse during forced
setup, completed a full rebuild against a private controlled seed in
**315.68s (5m 16s)**,
including vector reuse, publication and validation. Both profiles retained 161,735
semantic units; 323,022 unique inputs were reused and zero inferred. Ordinary
unchanged setup took **3.33s**, reporting ready with `build_index` skipped. Saved
state and record identities were unchanged across 23 tables. Compilation and
asset downloads are excluded. The earlier forced release measurement was 369.77s;
that is a historical reference, not a new paired control run. Current
`setup --force` deliberately bypasses passage-layout reuse and reruns splitting
while keeping compatible vector reuse. The 315.68s result is not a current
forced-setup timing. Normal source updates can still use layout replay; the
matched preparation comparison above measures that optimization.

The seed snapshot was 1,738,186,752 bytes and the rebuilt private database was
1,741,590,528 bytes. There is no new persisted layout payload; physical SQLite
allocation changed during publication. Peak-memory collection was unavailable:
macOS `time -l` failed its post-run `kern.clockrate` query in the sandbox, after
recording wall time. Atlas reported success and the subsequent ordinary setup
exited successfully. No full debug-profile rebuild was repeated. The force-bypass
follow-up uses a small native setup regression instead of repeating the
full-corpus timing.

The ignored `published_layout_reuse_full_corpus_benchmark` in
[`atlas-ingest`](../../../crates/atlas-ingest/src/passage_benchmark.rs) reproduces
the preparation comparison with explicit index, schema, model-cache and evidence
paths. Its editable source must be a private checkout: the small-update case
changes one description and restores it. The measured run used a separate Git
clone under the task checkout's `.atlas/source-fixture`, leaving shared pinned
source and model assets intact.
