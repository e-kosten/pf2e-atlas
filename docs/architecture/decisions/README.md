# Architecture Decisions

This index is the quickest way to scan accepted architecture decision records for PF2e Atlas.

## Current ADRs

- [`0056-trait-reference-entities.md`](./0056-trait-reference-entities.md): typed trait identities, pinned source definitions, exact membership/reference evidence and stored-vector Similar into normal records.

- [`0054-derived-tag-deferral.md`](./0054-derived-tag-deferral.md): derived-tag development is deferred pending demonstrated retrieval gaps; existing file validation remains, with history and future design preserved as research.

- [`0053-field-aware-search-fusion.md`](./0053-field-aware-search-fusion.md): one field-aware lexical projection and independent temporary RRF/score comparison without re-embedding.

- [`0019-rust-cli-search-record-surface.md`](./0019-rust-cli-search-record-surface.md): the CLI separates record identification from result-set search.
- [`0021-rust-runtime-index-and-retrieval-boundaries.md`](./0021-rust-runtime-index-and-retrieval-boundaries.md): runtime/index/search ownership; historical capability interfaces are superseded by ADR0046.
- [`0022-rust-artifact-policy-ownership.md`](./0022-rust-artifact-policy-ownership.md): index shape/vector ownership; metric authority and graph defaults are superseded by ADR0046.
- [`0023-rust-cli-json-contract.md`](./0023-rust-cli-json-contract.md): JSON envelope and exit behavior; historical record payloads partially superseded by ADR0046.
- [`0025-rust-graph-context-retrieval.md`](./0025-rust-graph-context-retrieval.md): graph context retrieval is key-based, one-hop, and separate from search relationship filters.
- [`0026-rust-cli-product-surface.md`](./0026-rust-cli-product-surface.md): CLI and first-party skill ownership; browser workflows follow ADR 0029, with TUI and derived tags as Rust-owned follow-ups.
- [`0027-rust-runtime-lint-policy.md`](./0027-rust-runtime-lint-policy.md): runtime Rust code denies panic-oriented Clippy lints outside tests, while tests keep assertion-oriented unwrap/expect ergonomics.
- [`0028-rust-tagging-model.md`](./0028-rust-tagging-model.md): historical Rust tagging design and implemented file model; roadmap and future design commitments partially superseded by ADR0054. Artifact tag rows remain unimplemented.
- [`0029-local-web-app-boundary.md`](./0029-local-web-app-boundary.md): Axum/app-model/app-service ownership, generated contracts and full semantic startup; result-window state and command lifetimes are superseded by ADR0049.
- [`0030-local-state-database.md`](./0030-local-state-database.md): durable mutable local state such as saved lists lives in a separate local-state SQLite database beside the generated artifact.
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
- [`0048-atomic-mutable-workflows.md`](./0048-atomic-mutable-workflows.md): local-state transaction boundaries, revision-checked participant replacements and durable mutation receipts independent of reference hydration.
- [`0049-search-lifetimes-and-pages.md`](./0049-search-lifetimes-and-pages.md): coherent worker/command artifact lifetimes, stateless generation-checked pages, deliberate browser recovery, cooperative read abandonment and typed query-input errors.
- [`0050-selectable-search-profiles.md`](./0050-selectable-search-profiles.md): two pinned baseline profiles, one shared artifact, profile-scoped KNN/reuse and shared retained inference sessions.
- [`0051-persistent-workspace.md`](./0051-persistent-workspace.md): accepted persistent workspace, shared bounded sessions, one migration ledger and state-preserving incremental publication; implementation in progress.

- [`0052-pinned-reference-source.md`](./0052-pinned-reference-source.md): one immutable PF2e revision drives model generation and default data acquisition; supplied sources remain user-managed.

## Historical ADRs

Historical ADRs preserve earlier architecture and migration decisions. Read their
status and successor links to understand which parts remain applicable; current
editing guidance is in the architecture docs and the active decisions above.

- [`0020-rust-content-documents.md`](./0020-rust-content-documents.md): historical RichDocument authority, superseded by ADR0046.
- [`0018-rust-default-embedding-model.md`](./0018-rust-default-embedding-model.md): historical model/default comparison, superseded by ADR0050 and ADR0046.
- [`0024-rust-search-retrieval-and-fusion-controls.md`](./0024-rust-search-retrieval-and-fusion-controls.md): historical ranking/control policy, superseded by ADR0046.
- [`0031-app-service-record-surfaces.md`](./0031-app-service-record-surfaces.md): historical generic section/profile transport, superseded by ADR0047; app-service composition ownership is retained.
