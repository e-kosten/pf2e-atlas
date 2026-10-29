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

Web retains two independent bounded SQL retrieval workers sharing retained native
profile sessions under [ADR0050](./0050-selectable-search-profiles.md). Every startup report must
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

Requests also carry an optional result order and responses report its effective
value. Omission preserves the workflow default: Name A–Z for browse, Relevance for
text/Similar, Saved order for lists. Explicit attribute intent remains distinct
from an automatic default across mode transitions and URL restoration. Order
changes reset paging without discarding criteria or mutable drafts; invalid
orders and incompatible modes fail before inference.

Order is a tagged domain value: Relevance, Saved order, attributes with one to
three unique name/level/unit-price rules in priority order, or Random with a u32
seed. Later rules break earlier ties; each rule keeps unavailable values last in
both directions. Final name/key ties remain deterministic. Random is a stable
index-owned SHA-256 order over seed and canonical key, with its byte encoding
specified in the artifact contract. It orders all eligible/admitted keys before
paging and does not store mutable order or use SQLite's per-call randomness.
All attribute and Random views of saved lists remain temporary.

Browser URLs use repeated sort tokens in priority order, such as
`sort=level_desc&sort=unit_price_asc`. Relevance and Saved order are sole tokens;
Random uses `sort=random&seed=42`. An incoming Random link requires a valid u32
seed; other modes reject it. Selecting Random generates a seed, while Shuffle or
an explicit seed edit replaces it and resets page1. Paging, Back and recovery
retain it. An order-only bare search remains idle. CLI repeats `--sort` for
attribute rules and accepts `--seed` only with sole Random, generating it when
omitted. Both adapters lower into the same typed contract.

Candidate admission remains independent of order and page. The existing semantic
policy tries 1,024/2,048/4,096 unit hits until the preferred-root stopping target
is met or the largest window is reached. The target of 200 roots is not a result
cap. Every accepted root contributes; hybrid unions the complete lexical lane
before final edition preference. Attribute or Random ordering then sorts the entire accepted
set before paging, retaining its coverage and witnesses. The stateless workflow
still reruns retrieval and does not promise a cross-request frozen candidate set.

Browser URL state retains committed criteria and the complete selected reading
address or parent landing. The browser's common filter controls and Advanced
builder edit one shared in-progress tree; Advanced displays the complete expression.
Simple positive conjunctions can be edited through common controls without
rewriting unrelated branches. Complex expressions retain their grouping and use
the full builder. Successful edits project the executable predicate without
reconstructing the editing tree. Empty authored groups, range editors and
unfinished siblings remain available. An invalid edit retains the last committed
criteria; a completed common-control edit replaces its exact committed condition
while retaining unfinished unrelated editing state. External changed criteria
restore the tree; local URL updates and record-preview navigation do not reset it.
Individual conditions support exclusion independently of genuine group negation.
Operand transitions follow app-service metadata, preserving compatible values
and resetting incompatible shapes before rendering a different input.

Filter URL state uses a versioned compact tuple form of the existing typed
predicate, compressed into a URL-safe alphabet. Clause and collection-scope IDs are editor/discovery identities, omitted
from the URL and reconstructed distinctly and deterministically on restoration.
Groups, literals and collection boundaries remain intact. Backend catalog and
query validation retain semantic authority. Query text, similarity seed and
nondefault page size use ordinary parameters; retrieval mode is derived. This
replaces raw predicate JSON in URLs without another expression language, a
parallel simple-filter dialect or a server-side short-link registry.

Page remains request state and is also represented by the browser `page` query
parameter; page one is omitted. This is the sole browser page authority. Explicit
page changes push history so Back and Forward traverse visited result pages.
Criteria, order, profile, seed and page-size changes reset page one; automatic
page correction replaces the entry. Copied links and reloads request the named
page without promising frozen results. Generation stays in the search owner and
is never a bookmark or search-session identifier. That owner
remains mounted under the persistent Router layout through React Activity, while
its requests are disabled outside Search. [ADR0058](./0058-browser-routing.md)
defines global route and browser-history ownership. Non-search route changes do not reset its committed criteria or page.
Result-list and per-address reader positions are separate in-memory browser state;
restoring them does not claim a frozen result set or survive a full reload. The search owner tracks generation and an epoch. Query keys include
criteria, page and known generation. It adopts a first response only for current
criteria/epoch with a live signal, seeding the final key without another inference
request. Generation mismatch pauses paging. Explicit `Refresh results` cancels
affected immutable queries before removing data, advances the epoch and restarts
at page1, retaining criteria and mutable drafts. Old page data cannot cross that
transition. Exact embedded addresses are never rebound to a different ordinal.
Similar preview reloads its actual owning detail before requesting a new generation.

Search and each picker use the shared browser `useReferenceGeneration` session
for generation adoption, request eligibility and replacement. Criteria, navigation,
form drafts and open/close policy stay with their surface. Picker closure resets
its session immediately; an obsolete recovery cannot publish into a reopened one.
Reference dependency is metadata beside each query definition, including imperative
reads and seeded pages. Retirement snapshots those Query instances, cancels them
before removal and leaves newly created sessions alone. This replaces the separate
query-family registry. Reference-derived encounter reads retain their full mutable
context keys; saved aggregates, receipts and drafts keep their own lifetimes.

Replaceable search pages and filter discovery take an explicit app-owned
`ReadExecutionContext` with an optional shared atomic abandonment flag. One
retrieval submission path checks it before lazy open and admission, immediately
before worker execution, and at safe boundaries between completed retrieval and
app projection. A queued abandoned read returns `OperationCancelled` without
executing its expensive closure. Commands and unrelated reads explicitly choose
uncancellable execution.

Web's read helper holds the abandonment guard in its awaiting async future and
wraps the existing bounded blocking-call helper. Future drop sets the flag; the
blocking closure keeps its permit until the actual work finishes. No ambient
context, client-ID registry, new capability trait, SQL interruption or native
inference abort is introduced. Durable mutations and their precommit reference
reads stay uncancellable under ADR0048.

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
mechanisms. Queued canceled items and waiters can remain until drained. Dropping
a read wait does not immediately release a running permit or stop native work.
Transport propagation needs actual protocol-specific evidence; a dropped-future
unit test does not establish every browser disconnect behavior.
