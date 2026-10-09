# 0044 Shared source content interpretation

## Status

Accepted library boundary. Product artifact and consumer adoption remain open.

## Context

Generated Foundry DTOs retain authored fields, including markup, unknown values
and diagnostics. Persisting another generic markup tree duplicates that authority
and makes Atlas maintain ordinary HTML formatting. The existing parser already
preserves HTML elements and recognizes Foundry enrichments; reusing it avoids
creating separate macro implementations for web, terminal, search and embeddings.

The bounded comparison used PF2e 6.12.4 at
4cbdaa37d6c33e9519561bae2c59a23e0288cbce, 50 representative cases and 3,996 sampled
fields. Prepared HTML plus facts was only 6.3% smaller after per-field gzip;
ownership and maintenance, rather than size, motivate this direction.

## Decision

atlas-record owns pure Foundry HTML/macro parsing and source-backed preparation.
Ingest owns filesystem discovery, localization catalog loading, resolution context
construction and enrichment execution. Existing callers import the shared parser
directly; there is no ingest re-export facade or copied parser.

The callable preparation API accepts a source-field locator, authored HTML,
explicit audience/implicit DC policy and optional localization/reference
providers. Callers select HTML fields using upstream format metadata; this API
does not guess journal Markdown/plain-text formats from string contents.

Source macros use the pinned handlers/registration as evidence. The old invented
Action/Trait macro interpretations are removed: those macros remain literal with
diagnostics, and neither occurs in the 91,894-field corpus inventory. Actual HTML
action-glyph and pf2-icon classes are interpreted using the upstream font context;
unknown glyph values remain visible and diagnosed rather than guessed.

Source locators identify root record, owned source nodes and relative field.
Valid unique child IDs may be stable; missing/invalid/duplicate IDs require an
explicit snapshot-local identity. Interpretation-local paths and ordinals bind
facts to output, but do not become durable field identities. Identity validation
and cross-rebuild reconciliation belong to the record envelope, not this parser.

Preparation returns sanitized HTML, unwrapped plain text, complete recognized
reference occurrences, visible authored interaction parameters and meaningful
diagnostics. The generic tree is transient for this API. Authored markup remains
in its DTO, and preparation neither persists it again nor replaces that source.
ammonia owns sanitization and URL allowance; html2text owns generic text formatting.
Two bounded DOM adjustments preserve captions and explicit list-number resets
that the formatter otherwise omits. html2text 0.15.5 shares the existing scraper
HTML parser dependency; the comparison with 0.17.1 found no relevant fidelity gain.

Localization uses resolved content, with ignored custom labels retained in source
and diagnosed. Authored custom labels on other enrichments remain labels; their
mechanics are sidecar facts rather than appended prose. Unsupported macros and
dynamic expressions remain literal/authored data with diagnostics. Interpretation
does not execute commands, predicates, formulas or Foundry document preparation.

Reference recognition precedes display/graph/search policy. Hidden occurrences
remain identifiable with explicit visibility; unresolved and sanitizer-blocked
destinations remain evidence, not usable links. Resolver output carries identities
and optional display names, never hard-coded web/CLI routes or copied destination
prose. Embed targets/options remain occurrences without automatic expansion.

Final application rendering binds interactions and record routes to prepared
markers and narrow facts. It must not introduce another Foundry parser or scrape
HTML for mechanics. PreparedSourceContent is an enrichment result containing
source evidence, not an audience-safe HTTP response DTO: application contracts
must select what they expose, including occurrence visibility and diagnostics.

## Consequences and adoption boundary

The current product artifact, record reader, FTS/embedding projections and UI
still use their existing RichDocument contracts. This library unit finishes parser
ownership and establishes a callable preparation contract; it does not claim the
storage replacement is complete. Supersede ADR0020's persisted representation
during the coherent writer/reader/consumer replacement, while preserving its
single interpretation-owner rule.

Before product adoption, select audience defaults, custom-label search mechanics,
Embed expansion, asset resolution and cache policy; measure hydration and cache
size. A cached projection must include interpretation/model/context identity and
exact input hashes. Library/sample tests do not establish Foundry core runtime,
browser or judged retrieval-quality parity.

Source expressions, modifier behavior and runtime preparation remain separate
behavior work. Avoid document-specific repairs, speculative interaction schemas,
compatibility facades and a second persisted/public generic HTML tree.

## Sources

- [Pinned enrichment handlers](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/module/system/text-editor.ts)
- [Pinned visibility handlers](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/scripts/ui/user-visibility.ts)
- [Pinned macro registration](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/scripts/hooks/init.ts)
- [Pinned glyph classes](https://github.com/foundryvtt/pf2e/blob/4cbdaa37d6c33e9519561bae2c59a23e0288cbce/src/styles/_globals.scss)
- [Typed ingest and database follow-up](../../backlog/items/typed-ingest-database-design.md)
