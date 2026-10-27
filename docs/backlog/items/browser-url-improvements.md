# Browser URL improvements

Status: in_progress
Priority: now

Implement concise validated reading URLs, stable list slugs, exact list reading restoration, shareable view state, canonical route schemas, and broken-link recovery. Retire legacy JSON reading URLs without compatibility decoding. Pagination adds browser history entries. Unreadable filter links return to empty idle Search with a notice. Localhost and port handling are excluded.

Execution plan: `scratch/plans/2026-10-26-url-improvements.md`. Governing architecture: ADR0049, ADR0057, and ADR0058. The first review checkpoint follows compact reading URLs and stable list navigation; later view-state/recovery work follows user review.
