# atlas-cli

`atlas-cli` owns the command-line surface for the Rust runtime.

This crate should stay thin: it parses arguments, routes commands, formats text or JSON output, reports progress, and chooses exit codes. Durable behavior belongs in the runtime, search, index, ingest, record, embedding, or artifact crate that owns the underlying concern.

## Owns

- Clap command definitions and CLI argument parsing.
- Command routing and exit-code mapping.
- Terminal and JSON presentation for CLI results.
- Progress and tracing setup for long-running commands.

## Should Not Own

- SQLite access policy or raw artifact queries.
- Search semantics, ranking, or result collapse policy.
- Ingest normalization or artifact-writing rules.
- Embedding provider setup beyond exposing user-facing flags.
- Durable path/setup policy that future surfaces also need.

## Boundary Notes

CLI commands should compose through `atlas-runtime` and product-facing service crates where possible. Direct calls into build-time crates such as `atlas-ingest` are acceptable for explicit build/analyze commands, but command modules should keep that orchestration narrow and presentation-focused.
