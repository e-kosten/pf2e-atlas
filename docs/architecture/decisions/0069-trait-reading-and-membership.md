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
Schema 43 removes the stored trait-reference relation. Publication and deep
validation check visible resolved links directly against the admitted record
universe, including owned targets. The versioned trait-association digest
attests only exact membership rows. Upgrade authenticates retained snapshots
and verifies their memberships before replacing each existing compatible
digest. Membership, definition provenance and the authored inline links remain
available through the ordinary product paths.
