# Search research preservation and reuse

The local research is expensive evidence, not disposable build output. Keep its
original bytes, versioned judgments, input bindings, and reproduction tools.
The dated conclusions are in [source-backed evaluation notes](source-backed-search-evaluation.md);
The [synthesis](search-evaluation-synthesis.md) connects the experiments;
current product policy is in ADRs 0050, 0053, 0059 and 0060. Historical findings describe the
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

For new experiments, Git carries concise findings and decisions; raw datasets,
rankings, opinions, timing samples and experimental replay scripts stay in ignored
`scratch/`. Repository tests use small synthetic fixtures rather than historical
research artifacts. The older [curated search archive](evaluations/2026-10-search/README.md)
remains an existing historical exception, not the template for new studies.
Local evidence is **not an off-machine backup**; preserve it separately before
moving machines or deleting retained research directories.

The [Similar evaluation summary](similarity-evaluation.md) records the durable
findings and limitations. Its complete frozen protocol, inputs, result package
and replay tools now live in `scratch/similarity-evaluation/study-2026-10/`.
Original bytes and hashes are preserved. Read its `LOCAL-REPLAY.md` for relocated
commands; older recorded commands retain their original paths. The move receipt
is `scratch/similarity-evaluation/repository-cleanup-receipt.json`.
The role census is `scratch/similarity-evaluation/role-census-export-2026-10-27.json.gz`;
the Core census is `scratch/core-sections/census-export-2026-10-27.json.gz`.

The Similar study requires its original generation-6/schema-16 artifact and matching
measured executables under `scratch/similarity-evaluation/runtime/`. Core and main
integration evidence is retained separately under `scratch/core-sections/`.
These artifacts use several historical migration/preparation versions; match the
recorded binary and generation, and never migrate a frozen research database.
Use a consistent SQLite snapshot when relocating live data. Record path changes
separately rather than rewriting frozen inputs or relabeling old measurements as
current evidence. Detailed task-only migration receipts remain local under
`scratch/core-sections/main-sync/`; they are not product compatibility policy.
Broader Core relevance/identity/decay evaluation has not run.

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
