# MCP And Search Reference

This document collects the MCP tool surface and the more detailed search behavior notes that are too reference-heavy for the README.

## MCP Tools

- `pf2e_get_search_semantics`
  Describes the category-first search model, supported boundaries, Pathfinder-native tags, and the available structured filter vocabulary.
- `pf2e_list_filter_values`
  Enumerates live values for a specific filterable field, optionally scoped to a category, subcategory, or set of search families.
- `pf2e_list_packs`
  Lists the available PF2E packs with labels, document types, and record counts.
- `pf2e_get_pack_metadata`
  Returns detailed metadata for one specific pack.
- `pf2e_list_records`
  Lists records with deterministic structured filtering and pagination when stable browse order matters more than ranked retrieval.
- `pf2e_search`
  Runs ranked retrieval over the local corpus using natural-language search and/or structured filters.
- `pf2e_lookup`
  Finds the best exact-name match for a single feat, spell, item, creature, action, rule, or other named record.
- `pf2e_lookup_many`
  Resolves multiple exact-name lookups in one call.
- `pf2e_get_record_by_key`
  Fetches one exact record by canonical `recordKey`.
- `pf2e_get_records_by_key`
  Fetches multiple exact records by canonical `recordKey`.
- `pf2e_collect_rule_question_context`
  High-level rules helper that resolves primary rule names from a narrow question, then gathers outgoing linked support records and optional curated backlinks without synthesizing an answer.
- `pf2e_get_rule_graph`
  Low-level rule-graph primitive that retrieves direct outgoing references and optional curated backlinks for canonical record keys, grouped by direction.

## Search Responses

Search and list responses include:

- `searchProfile` as the user-facing retrieval intent used for the result set
- `category` and `subcategory` as the primary product-facing search boundaries
- `rawRecordType` for internal Foundry traceability when needed
- `descriptionText` when available
- `hasDescription` for filtering and ranking
- `descriptionSnippet` for lightweight discovery
- `sourceCategory` to distinguish core, rules, adventure, and unknown sources
- `searchExplain` on records when `pf2e_search` is called with `explain: true`
- `explain.query` details for literal query normalization when `pf2e_search` is called with `explain: true`

## Search Notes

- The server is read-only.
- Search is category-first. Use `category`, `subcategory`, `scopes`, numeric bounds, and typed `metadata` predicates instead of raw Foundry `recordType`, `documentType`, or `itemCategory`.
- `pf2e_search` supports `searchProfile: "lexical" | "balanced" | "concept"` as the primary retrieval control.
- `pf2e_search` defaults to the `balanced` profile when `query` is present and `searchProfile` is omitted.
- `pf2e_lookup` and `pf2e_lookup_many` remain the exact-name lookup tools; `searchProfile: "lexical"` is the lexical-first ranked-search mode.
- Prefer a short natural-language phrase or sentence with 1-3 concrete anchor terms for `query`. Avoid long comma-separated keyword lists by default.
- `pf2e_get_search_semantics` is the primary discovery surface for categories, subcategories, Pathfinder-native tags, metadata filters, and supported search patterns.
- `pf2e_list_filter_values` enumerates live filter values after you know which category/subcategory or metadata field you want to inspect.
- Search uses a local SQLite index with shared structured filters, FTS-backed lexical search, and hybrid semantic reranking over filtered candidates.
- Ranking considers name, trait, metadata, and description signals, with optional explain output.
- Search does not perform server-side query expansion or hidden semantic-tag inference.
- Semantic scoring runs in the application layer after SQLite hard filters.
- When `hf-local` is configured, the embedding model must already be prepared locally with `npm run refresh-embeddings` or `npm run refresh-external`.
- The SQLite index must already be prepared locally with `npm run refresh-index` or `npm run refresh-external`.
- ANN and SQLite vector extensions are intentionally not required in the current implementation.
- The transport layer is isolated so Streamable HTTP can be added later without rebuilding the data/index layer.
- The vendored PF2E checkout under `vendor/pf2e` is intentionally ignored by this repo's Git history.
