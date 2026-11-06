# ADR0067: Source-backed remaster relationships over current records

Status: Accepted, 2026-11-05. Revised 2026-11-06: explicit counterparts and all matching results; automatic suppression removed.

## Context

Atlas's name aliases and legacy/remaster pairs lose the distinctions present in
Foundry's change descriptions. A single pair also cannot preserve multiple
citations, grouped destinations, absent records or same-ID name changes. A useful
relationship does not by itself establish a preferred search replacement.

The product retains current content at stable record keys. Saved records follow
updates at their chosen key. Historical document versions, pinning and a universal
rules-concept identity are outside this design.

## Decision

Keep one current checked body per record and existing record identity. Represent
source-described remaster changes as typed declarations with attributed origin,
change kinds, contextual qualifiers, ordered old/new endpoint descriptions and
supporting evidence. Endpoints may contain an exact source reference, a scoped
name or source discussion; missing old content does not create a synthetic record.
Compound changes and intentionally grouped targets remain grouped.

Current bindings are rebuildable projections, separate from authored declarations.
They support bounded record/term context, name lookup and counterpart navigation.
Keep multiple supporting declarations for the same fact. Neither matching names,
matching bodies nor membership in a broad subject group establishes identity.

Journal extraction uses existing audience-selected prepared content and authored
reference evidence, preserving exclusions within fields. Citations bind the checked
record field hash plus a selected statement selector; they do not bypass selection
or claim an independent full reinterpretation of source content. Selected non-record source inputs retain the bytes and identity needed
for offline validation of their declarations. Preparation and validation use the
owning shared source policy; they never execute upstream TypeScript. Captures
support installed current claims, not a historical document archive.

Atlas-reviewed correspondences, when supplied, have explicit Atlas origin, exact
scope and evidence/validation conditions. They must not appear upstream-authored.
Rebuild bindings on ingest. A relevant failed condition withdraws derived authority;
unrelated source revision or ordinary errata alone is not a blanket invalidation.
Apply these conditions to derived lookup and counterpart availability. The shared
compiled manifest/recipe identity participates in offline preparation admission;
a curated-only binary change requires normal source-backed preparation before
reference reads resume. Keep that identity separate from Foundry authorship.

## Consumer policy

Search, browse, Similar, traits and discovery retain every independently matching
record. Publication era and counterpart relationships do not hide results or
change counts. There is no remaster-preference request, URL/settings mode, CLI
switch or dormant suppression policy.

A small explicit Atlas manifest supplies reviewed exact old/new record keys and
source-family scope. Normal preparation validates current endpoints. This is a
maintained mapping, not automatic discovery from names, native IDs or general
UUID redirects. Journal extraction continues to supply the hundreds of old-name
aliases and compact change statements. Useful journal context can explain a
reviewed mapping without presenting its exact-root identity as Foundry-authored.

Exact opening and saved hydration keep the requested key. Same-key source updates
change displayed current content; a replacement relationship never silently
rebinds a save to another key. Filtered/searched saved lists retain every
independently matching chosen member. Owned content keeps its checked parent
context and customization; an origin entry's remaster information is separately
labelled source context.

App-service projects relationship meaning, contextual targets, readable evidence
and availability into typed product DTOs. Browser and CLI render these facts.
App-service consolidates compatible statements into logical relationships with
ordered inline text and exact record references. Supporting declaration IDs retain
bounded evidence; consolidation does not establish new record identity.
Reading shows concise inline context above rules, without duplicate names or
source-origin badges. Useful journal explanations are linked directly without
exposing internal provenance. Full attributed evidence stays available through
explicit API/CLI inspection. App-service projects Versions availability from actual
current alternatives or suggested variants; text-only former names and explanation
links do not create the tab. Reader explanation actions open the supplied journal
context in the reference drawer. Explore omits the reader notice in every tab;
Versions shows its relationships once with comparison actions below them.
Comparison keeps both identities and rules without
reciprocal notices.
Explicit term API/CLI lookup stays separate from record counts and strict identity
lookup. Ordinary Search does not add an automatic source-statement panel.

## Publication and upgrade

Append schema migrations and replace the old pair/opaque-alias authority and its
callers. Reprepare source information that previous projections discarded. Do not
invent richer evidence by splitting old opaque strings or retain parallel old/new
authorities.

Declarations, captures, bindings and alias supports participate in
reference generation. Metadata-only changes invalidate affected reads without
requiring unrelated native inference. Reuse unchanged compatible inputs through
the existing publication owner. Remaster metadata alone does not invalidate source
content or passage layouts: their actual dependency checks and physical profile
validation still apply. Ordinary no-op setup skips publication and keeps
generation; unchanged remaster content skips its row rewrites. Explicit committed
publication retains the existing storage revision increment even when content is
identical.

Migration rollback, same-contract publication rollback and post-upgrade readiness
are distinct guarantees. Preserve saved state and upgrade recovery. A committed
contract upgrade may require explicit source-backed setup before reference reads
resume; do not reset data or add an old-version serving shim.

See the [research findings](../../research/remaster-information.md),
[artifact contract](../artifact-contract.md), [runtime](../runtime.md), and
[ADR0065](0065-input-owned-reference-publication.md) for the existing ownership and
publication boundaries retained by this decision.
