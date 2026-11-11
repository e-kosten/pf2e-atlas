# ADR 0069: Trait reading and membership

Status: accepted. Amends [ADR0056](0056-trait-reference-entities.md) and the
trait-reference scope described in [ADR0057](0057-relationship-scoped-search.md).

## Context

In the pinned PF2e 8.6.1 corpus, every trait with indexed source references
scopes Search to one record. The GM Screen supplies the incoming occurrences.
Throneglass's outgoing references to Stupefied are available in its definition.
A dedicated Search mode duplicates navigation and gives this limited source
relationship its own retrieval behavior.

## Decision

Trait pages present authored definitions, their prepared inline links and exact
trait membership. **With this trait** previews matching sources, including
embedded entries. Its Search action uses the ordinary root Traits filter.

Retire the References tab, its Search action, the dedicated request and result
mode, and the trait-reference URL scope. Connected discovery uses ordinary static
eligibility and enclosing picker scopes. The private trait-records endpoint
serves membership.

Prepared definition content supplies inline links through the shared reader.
The existing artifact still publishes and validates trait occurrence evidence
as described in [ADR0068](0068-contextual-reference-definitions.md). This change
requires no workspace migration or source preparation.
