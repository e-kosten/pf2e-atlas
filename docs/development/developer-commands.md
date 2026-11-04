# Rust developer commands

Build and run developer diagnostics independently of the product CLI, web UI
bundle, and Node:

```bash
cargo run -p atlas-dev -- --help
cargo run -p atlas-dev -- source load --source vendor/pf2e --json
cargo run -p atlas-dev -- source analyze --source vendor/pf2e --json
cargo run -p atlas-dev -- source audit-paths --source vendor/pf2e --record-type npc --json
cargo run -p atlas-dev -- index inspect --index .atlas/workspace.sqlite --json
cargo run -p atlas-dev -- index record actionspf2e:1kGNdIIhuglAjIp9 --index .atlas/workspace.sqlite --json
cargo test -p atlas-dev -p atlas-cli-support
```

These commands use the same global/repo path policy, JSON envelope, and progress
controls as atlas. `source load` reads generated authored DTOs, keeping original
bytes, raw values, provenance and every admission
diagnostic. Its report counts modeled, partial, raw-only and quarantined outcomes
separately. It applies no source defaults, coercions, metric extraction or
embedding work. Embedded children and authored links remain in their source DTOs;
generic rule retention does not imply specific-rule interpretation.
Partial modeled documents produce exit 0; raw-only roots, quarantine, unavailable
packs or empty input produce exit 1 with the report on stdout. Root/manifest
errors produce exit 2. JSON `status: ok` means a report was produced; check the
counts and exit code for completeness. The loading API retains the full source
corpus in memory, with quarantine bytes retained only for successful file reads.
Analysis reports typed source selection and admission without writing SQLite;
path auditing reports observed paths and values. Index inspection reads an
existing artifact without changing it. `index record` decodes its checked source
snapshot and developer provenance; `--original --source vendor/pf2e` explicitly
reads the original clone file only after checking its recorded path and hash.
Source commands' `--manifest` overrides
the Foundry input manifest. Reports go to stdout; `--json` selects JSON.
Use Cargo's release profile for full-corpus processing and ingest performance
measurements, including builds that reuse every embedding. Source preparation,
tokenization and publication validation remain substantial CPU work without
inference; ordinary debug builds are intended for development checks.

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

Setup and index build/check/validate remain operational product commands:

```bash
cargo run --release -p atlas-cli -- index build --no-embeddings --json
```

For timings, build the executable first and measure its invocation separately
from compilation. Keep fresh builds, unchanged updates and content-policy upgrades
as separate cases; capture `--progress always` alongside the JSON report. Use an
isolated workspace copy for experiments, retain verified embedding checkpoints,
and avoid concurrent builds or benchmarks that would distort the comparison.

Foundry compiler research runs through the separate private
[TypeScript package](../../dev-tools/source-contracts/README.md), with npm entry points.
Rust developer commands do not launch TypeScript tools. Neither developer
surface is included in published product bundles; atlas-dev is built locally.

## Eligibility query work probe

Compare the production accepted-root SQL with the general expression compiler
against an existing, current-schema controlled corpus. Upgrade that disposable
workspace deliberately through its normal runtime owner before running the probe.
The probe opens an admitted read-only snapshot; it does not initialize or upgrade
the workspace, acquire sources/models or run inference. Its only output is a new
evidence file whose parent directory already exists. Choose a fresh filename:
existing evidence is never replaced.

Build the test executable first, then run the ignored diagnostic separately from
compilation. Replace the corpus path, evidence path and declared candidate identity
with the inputs being measured:

```bash
cargo test --release --locked -p atlas-index --lib --no-run
ATLAS_QUERY_WORK_CORPUS=/absolute/path/to/controlled/workspace.sqlite \
ATLAS_QUERY_WORK_OUTPUT=/absolute/path/to/new-eligibility-work.json \
ATLAS_QUERY_WORK_CANDIDATE='reviewed candidate and input receipt identity' \
cargo test --release --locked -p atlas-index --lib \
  query::membership_work_probe::controlled_corpus_eligibility_work \
  -- --ignored --exact --nocapture
```

The JSON contains both independently sorted complete accepted-key sets and their
SHA-256 digests, parity/stability results, bound SQL, query plans, preparation and
execution timings, and SQLite VM/full-scan/sort/automatic-index counters. A parity
failure writes its evidence and fails the test. The candidate label is declared
by the caller; the actual test-executable hash, workspace key, reference revision,
context and SQLite identity provide independent provenance.

Each plan warms once, then runs six measured repetitions with alternating order.
`ATLAS_QUERY_WORK_REPEATS` accepts 2–20 repetitions per plan. Preparation includes
SQLite compilation; execution includes binding, row decoding and ID collection.
Predicate lowering, key lookup/sorting, hashing and output are outside those timed
regions. Full-scan steps count forward full-table-scan steps, **not total row
visits**. These are warm-cache SQL observations without an OS-cache reset or a
performance threshold assertion.

The cases include selective/broad AC, level, price and trait predicates, positive
conjunctions, same-child embedded fallback, all-root control and restricted/empty
key scopes. Actual cardinalities establish selectivity on the supplied corpus.
Root eligibility is profile independent. Measure shipped CLI filtered retrieval
separately for each configured profile; these counters do not measure native KNN
work, query inference or end-to-end CLI latency.
