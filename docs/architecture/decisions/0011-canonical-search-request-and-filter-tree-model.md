# ADR 0011: Canonical Search Request And Filter Tree Model

- Status: Accepted
- Date: 2026-04-23

## Context

ADR 0010 established `SearchRequest` as the shared semantic search contract, but the contract shape still reflected an intermediate model:

- `intent` remained the top-level discriminant
- `parts` remained the main structured-filter carrier
- several filter concepts still lived as first-class root fields or embedded policy mini-languages
- TUI and MCP still had too much room to evolve their own parallel meanings around shared search semantics

The planning work for the next search-contract pass makes a stronger architectural choice. The repository now needs one durable shared data model that defines search meaning across MCP, TUI, ontology-origin flows, and backend compilation without preserving overlapping public shapes.

## Decision

Adopt one canonical shared search-request model with these durable rules:

- the shared request is a discriminated union keyed by `mode`
- the top-level modes are `browse`, `search`, and `lookup`
- `search` meaning and `filter` meaning are distinct
  - `search` carries text-retrieval inputs such as query text, exclude text, and profile where that mode allows them
  - `filter` carries result-set constraints
- `filter` is a boolean tree of atomic leaves
- the only canonical boolean composition shapes are:
  - `anyOf`
  - `allOf`
  - `not`
- filter concepts such as scope, links, pack, numeric matchers, metadata predicates, and metric predicates live as atomic canonical leaves under that tree
- exact record-link filters are represented by the symmetric canonical leaves `linksTo` and `linkedFrom`
  - `linksTo` matches records with an outgoing reference edge to the given target record key
  - `linkedFrom` matches records with an incoming reference edge from the given source record key
- top-level numeric matchers preserve exact, strict, inclusive, and bounded-range meaning as distinct canonical variants
- metadata predicates remain atomic in the canonical model
  - multiplicity is expressed through boolean composition, not plural payload shapes
- surface sugar is allowed only at the transport or editor edge
  - any friendly or shorthand input must lower into the canonical model before it crosses the shared contract boundary
- search-execution DTOs remain compiled output owned by `src/search/`
  - they are not alternate public/shared contracts

This ADR intentionally replaces the older shared-contract assumptions that kept `intent` / `parts` as the durable public shape.

## Consequences

- MCP, TUI, and ontology-origin flows must converge on the same `mode` / `search` / `filter` model before shared backend execution begins.
- The repository should not preserve alias request fields, parallel public filter shapes, or compatibility readers for the replaced shared contract.
- Search compilation, normalization, and validation remain centralized, but they now lower from the canonical filter tree instead of from the old `parts` shape.
- Surface-local editing or transport affordances may still exist, but only as edge sugar that lowers into the canonical model.
- Future shared search concepts should be added to this canonical model once and then lowered centrally, rather than reopening surface-specific semantic contracts.
