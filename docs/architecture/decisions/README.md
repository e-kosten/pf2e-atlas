# Architecture Decisions

This index is the quickest way to scan accepted architecture decision records for PF2e Atlas.

## Current ADRs

- [`0018-rust-default-embedding-model.md`](./0018-rust-default-embedding-model.md): BGE small default; historical pooling/chunk modes are superseded by ADR0046.
- [`0019-rust-cli-search-record-surface.md`](./0019-rust-cli-search-record-surface.md): the CLI separates record identification from result-set search.
- [`0020-rust-content-documents.md`](./0020-rust-content-documents.md): historical RichDocument authority, superseded by ADR0046.
- [`0021-rust-runtime-index-and-retrieval-boundaries.md`](./0021-rust-runtime-index-and-retrieval-boundaries.md): runtime/index/search ownership; historical capability interfaces are superseded by ADR0046.
- [`0022-rust-artifact-policy-ownership.md`](./0022-rust-artifact-policy-ownership.md): index shape/vector ownership; metric authority and graph defaults are superseded by ADR0046.
- [`0023-rust-cli-json-contract.md`](./0023-rust-cli-json-contract.md): JSON envelope and exit behavior; historical record payloads partially superseded by ADR0046.
- [`0024-rust-search-retrieval-and-fusion-controls.md`](./0024-rust-search-retrieval-and-fusion-controls.md): historical ranking/control policy, superseded by ADR0046.
- [`0025-rust-graph-context-retrieval.md`](./0025-rust-graph-context-retrieval.md): graph context retrieval is key-based, one-hop, and separate from search relationship filters.
- [`0026-rust-cli-product-surface.md`](./0026-rust-cli-product-surface.md): PF2e Atlas is a Rust CLI plus first-party skill product; future TUI and derived tags are Rust-owned follow-ups.
- [`0027-rust-runtime-lint-policy.md`](./0027-rust-runtime-lint-policy.md): runtime Rust code denies panic-oriented Clippy lints outside tests, while tests keep assertion-oriented unwrap/expect ergonomics.
- [`0028-rust-tagging-model.md`](./0028-rust-tagging-model.md): Rust tagging is a typed authored-label subsystem with explicit crate ownership, record-centered assignments, and authoritative `record_tags` artifact rows.
- [`0029-local-web-app-boundary.md`](./0029-local-web-app-boundary.md): the local web app uses Axum plus an app-model/app-service boundary over runtime/search, with generated TypeScript contracts and full semantic-search startup for web service mode.
- [`0030-local-state-database.md`](./0030-local-state-database.md): durable mutable local state such as saved lists lives in a separate local-state SQLite database beside the generated artifact.
- [`0031-app-service-record-surfaces.md`](./0031-app-service-record-surfaces.md): historical generic section/profile transport, superseded by ADR0047; app-service composition ownership is retained.
- [`0032-ingest-product-intent.md`](./0032-ingest-product-intent.md): product-purpose requirement for derived projections; historical source retention and metric model partially superseded by ADR0046.

- [`0033-developer-command-surfaces.md`](./0033-developer-command-surfaces.md): product operations use `atlas`, Rust diagnostics use private `atlas-dev`, and compiler research uses private TypeScript npm commands; shared CLI presentation has a narrow support crate.
- [`0034-source-generation-layout.md`](./0034-source-generation-layout.md): a small source pin and locked dependencies reproduce ignored declaration graphs and committed modular Rust; CI checks the complete source-to-Rust path.
- [`0035-source-value-generation-policy.md`](./0035-source-value-generation-policy.md): pre-default source slices, explicit open trait arrays, typed ordered maps, forbidden-member preservation and compiler-proven impossible alternatives.
- [`0036-recursive-source-unions.md`](./0036-recursive-source-unions.md): anchored recursive values, shape-sensitive union identity, typed tuples and compiled generic fixtures.
- [`0037-open-and-indexed-source-values.md`](./0037-open-and-indexed-source-values.md): explicit open JSON domains, named fields plus typed dynamic entries, and the generated complete Item flags source model.
- [`0038-source-templates-and-generic-names.md`](./0038-source-templates-and-generic-names.md): checked template-string domains, concrete generic-instantiation names and distinct compilation/corpus evidence.
- [`0039-authored-rule-inputs.md`](./0039-authored-rule-inputs.md): separate authored collection forms, schema field provenance and contextual corpus/fidelity comparison without claiming Foundry admission.
- [`0040-authored-document-inputs.md`](./0040-authored-document-inputs.md): separate authored document forms, recursive object patches and schema/authored corpus comparison before preparation.
- [`0041-source-admission-and-field-retention.md`](./0041-source-admission-and-field-retention.md): retain useful documents with explicit invalid fields and diagnostics; keep specific rule interpretation atomic and separate from raw retention.
- [`0042-typed-source-loading.md`](./0042-typed-source-loading.md): independently load generated source DTOs with raw bytes, provenance and explicit failures before normalization and storage design.
- [`0043-shared-foundry-model-and-snapshots.md`](./0043-shared-foundry-model-and-snapshots.md): shared generated authored structures and exact typed snapshots support runtime reuse without ingest dependencies or a second family schema.
- [`0044-shared-source-content-interpretation.md`](./0044-shared-source-content-interpretation.md): one shared Foundry parser and explicit HTML/text/fact preparation context; artifact adoption defined by ADR0046.

- [`0045-source-backed-record-enrichment.md`](./0045-source-backed-record-enrichment.md): a key and one retained DTO, internal embedded addressing, separate preparation/relationship outputs and focused borrowed views, with developer reporting and a database-independent ingest handoff.
- [`0046-source-backed-reference-artifact.md`](./0046-source-backed-reference-artifact.md): checked snapshot authority, named typed query projections, HTML caches, attributed lexical/semantic retrieval, concrete reader generation consistency and coordinated consumer replacement.
- [`0047-semantic-record-presentation.md`](./0047-semantic-record-presentation.md): transient family facts, one composed encounter presentation, exact selected navigation and shared browser family components with independent CLI layout.

## Historical ADRs

ADRs 0001-0017 preserve design history for earlier architecture work and migration sequencing. They are retained as context, but current implementation guidance lives in the Rust architecture docs and current ADRs above. ADR 0026 supersedes ADR 0017 for product surface and workspace layout.
