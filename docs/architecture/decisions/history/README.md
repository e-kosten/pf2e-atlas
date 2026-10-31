# Historical architecture decisions

These records preserve retired implementations and migration rationale. Their
original decisions describe the system at that time; they are not current
contributor instructions. Earlier references to `decisions/<filename>` identify
the same numbered records now in this directory. Use the [active index](../README.md) and
[architecture overview](../../overview.md) for changes.

## Retired TypeScript MCP and TUI

ADRs 0001–0016 belong to the TypeScript application retired by
[ADR0026](../0026-rust-cli-product-surface.md). ADR0017 records the migration.

- [ADR 0001: Shared Backend, Separate MCP and Editorial Surfaces](0001-shared-backend-separate-surfaces.md)
- [ADR 0002: Readonly Ontology and Explicit Storage Boundary](0002-readonly-ontology-and-explicit-storage-boundary.md)
- [ADR 0003: Lint-Enforced Derived-Tag Boundaries](0003-lint-enforced-derived-tag-boundaries.md)
- [ADR 0004: Non-Tag Ownership Import Boundaries](0004-non-tag-ownership-import-boundaries.md)
- [ADR 0005: Live Search-Semantics Exploration](0005-live-search-semantics-exploration.md)
- [ADR 0006: Shared TUI Interaction Contracts](0006-shared-tui-interaction-contracts.md)
- [ADR 0007: Render-Ready Route Transitions](0007-render-ready-route-transitions.md)
- [ADR 0008: Search Filters And Concern-Specific Tag Facades](0008-search-filters-and-concern-specific-tag-facades.md)
- [ADR 0009: Shared List/Detail Presentation Layer And Behavior Contracts](0009-shared-list-detail-presentation-layer.md)
- [ADR 0010: Shared SearchRequest Semantic Contract](0010-shared-search-request-contract.md)
- [ADR 0011: Canonical Search Request And Filter Tree Model](0011-canonical-search-request-and-filter-tree-model.md)
- [ADR 0012: Shared App-Facing Discovery Service Boundary](0012-app-facing-discovery-service-boundary.md)
- [ADR 0013: TUI Canonical Search State And Derived Editor Projections](0013-tui-canonical-search-state-and-derived-editor-projections.md)
- [ADR 0014: Shared Search Result Presentation Capabilities](0014-shared-search-result-presentation-capabilities.md)
- [ADR 0015: Shared Explorer Host Contract And Live Group Editing](0015-shared-explorer-host-contract-and-live-group-editing.md)
- [0016 Metadata Field Ownership Split](0016-metadata-field-ownership-split.md)
- [0017 Rust Runtime And CLI-First Migration](0017-rust-runtime-cli-first-migration.md)

## Replaced Rust designs

| Record | Current decision |
| --- | --- |
| [0018 Rust Default Embedding Model](0018-rust-default-embedding-model.md) | [Selectable profiles](../0050-selectable-search-profiles.md) and [source-backed retrieval](../0046-source-backed-reference-artifact.md) |
| [0020 Rust Rich Documents](0020-rust-content-documents.md) | [Source-backed content](../0046-source-backed-reference-artifact.md) |
| [0024 Rust Search Retrieval And Fusion Controls](0024-rust-search-retrieval-and-fusion-controls.md) | [Source-backed retrieval](../0046-source-backed-reference-artifact.md) and [field-aware fusion](../0053-field-aware-search-fusion.md) |
| [ADR 0030: Local State Database](0030-local-state-database.md) | [Persistent workspace](../0051-persistent-workspace.md); saved-domain ownership remains in local-state |
| [ADR 0031: App-Service Record Surfaces](0031-app-service-record-surfaces.md) | [Semantic presentation](../0047-semantic-record-presentation.md); app-service composition remains |
