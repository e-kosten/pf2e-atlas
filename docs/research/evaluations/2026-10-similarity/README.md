# Configurable similarity evaluation

This is the frozen protocol and source-bound input set for evaluating Atlas's
stored-vector Similar methods. It is a developer experiment, not production
ranking policy or a claim that a method is already better.

- [Protocol and judging rubric](protocol.md) defines comparisons, failure handling,
  metric denominators, reviewer blindness and performance measurements.
- `manifest.json` declares 24 scenarios, two stored profiles, 13 method variants,
  comparison strata and the repeated performance subset.
- `seeds.json` binds the 22 unique source seeds and exact selectable sections to
  generation, source hashes, role recipe and both profiles' inventories.
- `corpus.json` freezes a 278-record oracle universe: 240 identity-only stratified
  samples and 38 source-nominated or scenario-seed records. Nominations establish
  coverage opportunities, not relevance grades.
- `freeze.json` binds the protocol, inputs and verifier before ranked results are
  generated. `verify.py` checks those digests and the input joins.

Run the integrity check from the repository root:

```sh
python3 docs/research/evaluations/2026-10-similarity/verify.py
```

The checked source revision is `28d9d156f252d084371d994dc359509b49eadf18`;
the task artifact generation is `a9474928b73e8cf89e84c5ed6990a838:6`.
A different artifact requires an explicit new binding/freeze, not selector
substitution. The source contract and exact section addresses are retained in
`seeds.json`; whole source bodies and quoted packets stay in ignored local
research storage.

The old October search archive is separate and remains unchanged. Its holdouts
are exposed and its grades are not imported here. This experiment's holdout is
independent of this workstream's tuning, not a claim that Pathfinder entities
were globally unseen by reviewers or language models.

## Execution and evidence state

At freeze time no Similar rankings, candidate scores or relevance judgments for
these holdout scenarios have been opened. Exact-name resolution, identity-only
catalog browsing, section inventory and source inspection were used to ground
the inputs. No result-quality findings are implied by this input dataset.

The maintained runner lives in `atlas-dev`; runner help is the authoritative
command syntax. Its reports must include the manifest/corpus/freeze digests,
compiled app revision or dirty-tree identity, artifact generation, profile,
scoring and mapping recipes, effective options, scope, failures and timings.
Do not describe a key-filtered KNN request as exhaustive scoring.

The full source packets and raw executions belong under ignored
`scratch/similarity-evaluation/`. Curated reports may be added after execution;
retain individual reviewer opinions, adjudications and unresolved cases rather
than overwriting earlier evidence. Literal source quotations are represented in
tracked output by source pointers and digests. See the
[research preservation guide](../../search-evaluation-handoff.md).
