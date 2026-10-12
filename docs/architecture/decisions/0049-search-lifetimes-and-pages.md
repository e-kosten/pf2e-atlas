# ADR 0049: Coherent Search Lifetimes and Stateless Pages

Status: accepted.
Date: 2026-10-12

## Context

Atomic artifact publication protects each opened reader, but independently opened
web workers or composed command reads can observe different files. Preparation
context equality alone does not prove workers opened the same physical instance.
The previous app window registry retained criteria and reran retrieval on every
page. Its handles added expiration and browser recovery without freezing ranking
or reusing query inference. Query-budget errors were reported as model availability
failures.

## Decision

Index retains the opaque, non-serialized file identity established during a
successful reader open, together with the complete preparation-context hash.
Search exposes that identity narrowly for application composition. It does not
expose SQLite or filesystem operations to app-service. The identity retains the
existing same-file library guarantees; it is not a stronger filesystem identity
or hostile-filesystem contract.

Web retains two independent bounded retrieval workers. Every startup report must
match both physical instance and complete context before a sender becomes usable.
Any startup failure closes the queue and joins all started workers before
returning. Later publication does not change a running service's readers; restart
adopts the new artifact. Commands lazily retain their first successful service
across all composed reference reads. Query-embedding, lexical and stored-vector
modes stay explicit. Source-free local-state commands do not open the artifact.

The public artifact generation is the full `SourceArtifactBuildContext` SHA-256.
Snapshot-local navigation uses the separate source fingerprint. Physical identity
stays process-local. Same-context republication can have the same public generation
while worker startup still requires one physical instance.

One `search_page` operation and `POST /api/search/page` accept criteria, page and
an optional expected generation. Responses return generation, existing mode and
page facts, rows with attributed matches, and retrieval coverage. List, text and
stored-vector Similar retain their eligibility, ranking and count semantics.
Expected generation is checked before embedding/ranking. A mismatch returns
`ArtifactGenerationChanged`, HTTP409, with expected and actual hashes. Window
registries, identifiers, expiration, open/read operations and recovery are removed.

Browser URL state retains committed criteria; page and generation remain request
state. The search owner tracks generation and an epoch. Query keys include
criteria, page and known generation. It adopts a first response only for current
criteria/epoch with a live signal, seeding the final key without another inference
request. Generation mismatch pauses paging. Explicit `Refresh results` cancels
affected immutable queries before removing data, advances the epoch and restarts
at page1, retaining criteria and mutable drafts. Old page data cannot cross that
transition. Exact embedded addresses are never rebound to a different ordinal.
Similar preview reloads its actual owning detail before requesting a new generation.

Text retrieval and semantic contextual discovery share an exhaustive conversion
of embedding errors. Empty input and token-budget overflow are invalid requests,
HTTP400, with useful counts and a shortening instruction. Model/cache/inference
failures retain availability classification; document-only preparation failures
at this boundary are internal execution errors. Unsupported optional facet
contexts are still checked before inference. No query clipping or lexical fallback
conceals an invalid semantic request.

## Consequences

One running web service or command has a coherent reference lifetime. Stateless
requests remove handle state without claiming less inference work: every page
reruns retrieval, semantic candidates remain bounded, and no exact frozen ranking
or new cache is promised. A generation change deliberately restarts paging while
preserving user intent and durable state. No artifact schema, ingest, ranking or
family-presentation redesign is implied.

Active SQL/native inference interruption, read deadlines and query-vector/candidate
caches require separate evidence and decisions. Stateless pages add none of those
mechanisms.
