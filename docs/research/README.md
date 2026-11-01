# Research Notes

This directory preserves durable findings from spikes and exploratory branches.

For search conclusions and unresolved tradeoffs, start with the
[research synthesis](search-evaluation-synthesis.md). The
[curated evaluation archive](evaluations/2026-10-search/README.md) preserves grading
and ranked results; the [archive map](search-evaluation-handoff.md) locates the
full local evidence. The [Similar evaluation summary](similarity-evaluation.md)
records the stored-vector comparison and its limits.

For the deferred authored-label workstream, start with [derived-tag research and
history](derived-tags/README.md). It consolidates the Node implementation, local
concept mappings, Rust seed experiments and ontology/assessment discussion,
including source snapshots and cleanup boundaries.

For the database and ingest changes, see [database and ingest findings](database-ingest-simplification.md).
It records preserved data, release measurements, recovery/query observations and
the measured lower cached-build RSS versus longer elapsed time.

Research notes should capture:

- the question the spike tried to answer;
- the implementation or evaluation shape;
- the useful findings worth carrying forward;
- the negative findings that should prevent repeating the same investigation;
- the resulting product or architecture decision.

For new experiments, keep datasets, rankings, individual judgments, machine-readable
results, experimental scripts and long logs in ignored `scratch/` storage. Commit
concise findings, limitations and resulting decisions here. Reusable developer
tooling belongs with its maintained owner and uses small synthetic test fixtures.
Do not make repository tests or CI depend on local research files. If a spike
branch is preserved remotely, link or name it from the research note and summarize
the durable conclusions here.
