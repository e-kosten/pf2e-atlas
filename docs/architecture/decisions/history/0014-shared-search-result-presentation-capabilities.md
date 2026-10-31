# ADR 0014: Shared Search Result Presentation Capabilities

> This records the retired TypeScript MCP/TUI architecture. For current owners,
> read the [architecture overview](../../overview.md) and
> [active decisions](../README.md). [ADR0026](../0026-rust-cli-product-surface.md)
> records the product replacement.


- Status: Historical (accepted for the retired TypeScript runtime)
- Date: 2026-04-23

## Context

The next lookup and result-reader pass needs mode-specific result treatment, especially around lookup match strength:

- lookup results may need grouped sections by match type
- flat result modes may still need row-level metadata such as badges or subtitles
- detail panes may need light optional metadata lines

Those needs are not unique to lookup as a subsystem. They are result-presentation capabilities that can apply across search surfaces. If lookup owns them directly, the repository will grow bespoke rendering pathways where a shared presentation seam should exist.

## Decision

Treat grouping and lightweight result metadata as shared result-view capabilities, not as lookup-only rendering behavior.

The durable shared capabilities are:

- optional grouping axes for result lists
- shared section-header rendering for grouped result sets
- shared row-presentation metadata such as badges or subtitles
- shared optional detail metadata lines for selected results

Lookup is only a consumer of those capabilities. In practice:

- lookup may supply `matchType` as a grouping or row/detail presentation input
- browse and ordinary search may choose not to use a grouping axis
- the shared result-view/presentation layer remains the owner of the rendering hooks and behavior shape
- lookup-specific screens or controllers must not become the durable owner of grouped-result rendering behavior

## Consequences

- Lookup can feel distinct without creating a parallel result-reader architecture.
- Result grouping and lightweight metadata become reusable capabilities for future search surfaces instead of one-off lookup code.
- Shared presentation owners remain the right home for list/detail result-rendering extensibility, consistent with the repo’s broader TUI architecture direction.
