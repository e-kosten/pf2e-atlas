# Search research preservation and reuse

The local research is expensive evidence, not disposable build output. Keep its
original bytes, versioned judgments, input bindings, and reproduction tools.
The dated conclusions are in [source-backed evaluation notes](source-backed-search-evaluation.md);
The [synthesis](search-evaluation-synthesis.md) connects the experiments;
current product policy is in ADRs 0050, 0053 and 0059. Historical findings describe the
implementation tested at the time, not necessarily today's defaults.

Paths below are relative to the repository root, not an individual worktree.

| Evidence | Location and entrypoint |
| --- | --- |
| Four embedding models × two representations | `scratch/full-corpus-search/embedding-full/FINDINGS.md` and `REPRODUCE.md` |
| Reviewable semantic judgments and examples | `scratch/full-corpus-search/embedding-full/user-review/README.md` and `all-judgments.json` |
| Semantic timings, model contracts, vector/rank validation | `scratch/full-corpus-search/embedding-full/execution/` and `runtime/` |
| Lexical/fusion final matched comparisons | `scratch/search-fusion-review/projection-fusion-followup/FINDINGS.md`, `PROTOCOL.md`, `AUDIT.md` |
| Revised relevance criteria and adjudication | `scratch/search-fusion-review/relevance-realignment/` |
| Earlier ownership, reranking and input experiments | `scratch/full-corpus-search/{hierarchy,role-evaluation,evidence-evaluation,reranking,embedding-design,retrieval-synthesis}/` |
| Pinned source and original reference artifact | `scratch/full-corpus-search/source/` and `reference.sqlite` |
| Ingest performance measurements | `scratch/ingest-performance/README.md`, logs and digest receipts |
| Configurable Similar matrix, source packets and runtime probes | `scratch/similarity-evaluation/RUN-STATE.json`, `quality-v1/`, `blind-packets/`, `grades-v1/`, `adjudications-v1/`, `performance-v1/` and `product-probes/` under that experiment root |
| Core implementation census, migration and regression evidence | `scratch/core-sections/RUN-STATE.json`, `v22-upgrade/`, `legacy-all-parity/`, `operational/` and `runtime/`; curated coverage in [Core section census](core-section-census.md) |

Keep the whole experiment trees, not only final reports. Rank captures, model
vectors, blind pools, raw reviewer outputs, adjudications, uncertainty analysis,
and known-good nominations support reanalysis without another inference or
grading pass. Some experiments share relative symlinks and inputs across these
directories. Preserve that layout when backing them up. Model downloads remain
in `.cache/hf-models`; experimental model assets also live inside research trees.

Git carries the [curated evaluation dataset](evaluations/2026-10-search/README.md),
its integrity verifier, decision summaries and this map. Raw evidence remains ignored
locally, including copied source and model data; this is **not an off-machine
backup**. Preserve it separately before moving machines or deleting `scratch/`.
Do not add entire SQLite/vector/model archives to Git. The committed subset retains selected reviewer opinions, final judgments,
rankings and metrics, with source pointers/hashes instead of literal source passages.
Full-source interpretation and arbitrary re-ranking still need the local archive.

The [configurable Similar protocol](evaluations/2026-10-similarity/README.md)
separately freezes the stored-vector method experiment's scenarios, exact section
bindings, controlled candidate universe and judging/performance rules. It does
not reuse the older archive's exposed holdouts or judgments.

The Similar study also needs its original `.atlas/workspace.sqlite` at generation
`a9474928b73e8cf89e84c5ed6990a838:6` and the measured executables retained under
`scratch/similarity-evaluation/runtime/`. Keep this schema-16 research workspace
separate from a newer preview workspace; run setup against the preview copy, not
the frozen study. Preserve live SQLite data through a consistent backup, including
pending WAL state. When relocating an archive, retain bound file bytes and record
a separate path map rather than rewriting hashed packets.

The Core implementation also retains `.atlas/core-baseline-v22.sqlite` at
generation 8/schema 17 and `.atlas/core-preview.sqlite` at generation 9/schema 18.
Keep the immutable baseline with its pre-Core executable, the Core executable,
input manifests and raw receipts under `scratch/core-sections/`. The preview is
mutable review state; preserve its committed WAL through SQLite's backup API
before moving or archiving it. Never open the baseline with a migrating Core
runtime. The original generation-7 preview and generation-6 Similar study remain
separate. These are implementation and preservation checks; the broader Core
relevance and decay study has not run.

## Reanalysis rules

- Read the experiment's protocol and reproduction guide first. Different studies
  use different corpora, relevance criteria, representations and rank aggregation.
- Keep query/filter/profile identity, root relevance, and supporting-match quality
  distinct. Do not silently combine old and revised grades.
- Write corrections as new versions and recompute all compared configurations
  uniformly. Preserve original opinions and frozen files; do not patch the past.
- All opened holdouts are exposed. Slicing existing results is exploratory;
  subsequent tuning needs new cases for an independent quality claim.
- Keep inference/build cost separate from quality. Research all-passage scans,
  warm service latency, and fresh CLI startup are different measurements.
- Reuse exact inputs/vectors when valid. Do not rebuild the full corpus merely to
  inspect scores, judgments, or change a ranking-only hypothesis.
