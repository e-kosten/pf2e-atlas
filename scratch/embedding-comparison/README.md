# Embedding Comparison Harness

This scratch harness compares embedding candidates against the Rust semantic-search path.

It intentionally keeps deterministic measurement separate from subjective result review:

1. Build one isolated Rust artifact per enabled model config.
2. Run the canned query set against each artifact.
3. Emit anonymized review packets for LLM or human scoring.
4. Aggregate build/query metrics and scored review output later.

The Rust CLI exposes every candidate in `models.json` through `--embedding-model`. The
harness keeps running if an individual model fails to load or execute, so an overnight run
can distinguish ready local model caches from candidates that still need provider/export work.

## Run

```bash
node scratch/embedding-comparison/run-comparison.mjs \
  --source vendor/pf2e \
  --models scratch/embedding-comparison/models.json \
  --queries scratch/embedding-comparison/queries.json \
  --output scratch/embedding-comparison/runs/manual
```

By default the script uses `rust/target/release/atlas`. If the binary is absent, build it first:

```bash
cd rust
cargo build --release -p atlas-cli
```

## Outputs

```text
scratch/embedding-comparison/runs/<run-id>/
  run-config.json
  preflight.json
  REVIEW.md
  summary.json
  summary.md
  models/<model-id>/
    model-config.json
    build-command.json
    build-report.json
    build-vectors-report.json
    embedding-metrics.json
    validate-vectors-report.json
    artifact.sqlite
    build.stdout.json
    build.stderr.log
    queries/<query-id>.json
    queries/<query-id>.stdout.json
    queries/<query-id>.stderr.log
  review-packets/<query-id>.md
  review-packets/<query-id>.mapping.json
  score-templates/<query-id>.json
  review-scores/
```

Review scores are intentionally not produced by the harness. Put scoring JSON files under
`review-scores/` and aggregate them with the deterministic metrics:

```bash
node scratch/embedding-comparison/aggregate-scores.mjs \
  --run scratch/embedding-comparison/runs/manual
```

The query suite intentionally mixes terse one-word searches, compact agent-style searches,
and longer natural-language requests so model behavior can be compared across query length.
