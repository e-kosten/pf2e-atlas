# atlas-ingest

`atlas-ingest` owns build-time transformation from Foundry PF2E source data into a complete Rust artifact.

This crate loads source records, parses Foundry-specific content, normalizes records, enriches relationships, generates source-backed records, prepares and runs embedding work for artifact builds, and writes complete SQLite artifacts.

## Owns

- Vendored Foundry PF2E source loading and source signatures.
- Foundry-specific JSON, HTML, and macro parsing.
- Normalization into `atlas-record` models.
- Generated source-backed records.
- Alias, remaster-link, taxonomy, variant, metric, visibility, and reference extraction.
- Build-time embedding orchestration and cache reuse.
- Complete artifact writing through the `atlas-artifact` contract.

## Should Not Own

- Public embedding-specific APIs or model catalog policy.
- Runtime search orchestration.
- Read-only index access for product surfaces.
- CLI presentation.
- Broad crate-root behavior unrelated to ingest phases.

## Boundary Notes

Keep ingest behavior in phase-specific modules such as `source`, `records`, `generated`, `embeddings`, and `artifact`. The crate root should remain a narrow public facade. New source policy belongs near the ingest concern that owns it, not in generic helpers.
