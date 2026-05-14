# Embedding Comparison Harness

This scratch harness compares embedding candidates against the Rust semantic-search path.

It intentionally keeps deterministic measurement separate from subjective result review:

1. Build one isolated Rust artifact per enabled model config.
2. Run the canned query set against each artifact.
3. Emit anonymized review packets for LLM or human scoring.
4. Aggregate build/query metrics and scored review output later.

The current Rust CLI only exposes the default `atlas-embedding` model. Non-default model
configs should stay disabled until the Rust catalog/config surface can select them without
code edits.

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
  summary.json
  summary.md
  models/<model-id>/
    model-config.json
    build-command.json
    build-report.json
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
  review-scores/
```

Review scores are intentionally not produced by the harness. Put scoring JSON files under
`review-scores/` and aggregate them with the deterministic metrics in a later decision pass.

