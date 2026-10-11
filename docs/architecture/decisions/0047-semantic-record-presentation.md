# ADR 0047: Semantic Record Presentation

Status: accepted.
Date: 2026-10-10

## Context

CLI, reference browsing, and encounters need the same authored facts, with different
layouts and density. A generic section/value/profile contract dictates layout
while obscuring family meaning. Sending an encounter stat block alongside that
surface duplicates facts and leaves consumers to reconcile two presentations.

Checked Foundry DTOs remain the sole stored authored authority under ADR0046.
This decision changes transient application responses, not the artifact schema,
ingest admission, search projections, embeddings, or local-state schema.

## Decision

App-service projects selected records into explicit semantic family responses
exported by app-model. Each presentation contains selected identity, prepared
content, useful immediate owned-document navigation, and a tagged family body.
The detail envelope retains the root summary and complete selected address.
Creature, hazard, spell, activity, physical-reference, and table responses carry
only facts needed by current consumers. Other families intentionally use content
and owner links until a useful family presentation is designed.

Reference and encounter composition share authored actor extraction. Encounters
apply existing bounded runtime policy to those facts and send one composed
presentation beside durable participant state. Modifier explanations accompany
the actual adjusted facts. There is no public duplicate stat-block transport.
The private encounter arithmetic workspace supports existing bounded rules; it
is not another response or an authored reference projection owner.

Borrowed record views remain storage and presentation neutral. They preserve
availability and typed values; app-service owns labels and semantic composition.
Renderers format those values independently, including signs and units. Neither
renderer parses source DTOs or prose to recover mechanics.

Authored action and casting costs carry a readable label and an optional typed
glyph for one, two or three actions, reaction and free action. Variable, timed
and passive costs retain text; unavailable costs retain their fact state. Physical
usage instructions remain separate from action costs. Spell form/heightening
patches preserve explicit clears. Optional baseline spell requirements, cost and
casting-entry facts omit missing/null and empty or whitespace-only prose only
under a known typed system. Nonempty authored values are preserved verbatim;
unavailable parents and invalid facts remain explicit. The checked source retains
all original values. This baseline presentation policy does not omit spell patch
clears or normalize source data.

Baseline spell area and defense are absent only when a known typed system
explicitly supplies null, meaning no authored area or defense. Missing or invalid
fields and unavailable system parents retain explicit facts; area and defense
patches independently preserve clears. The baseline web reader omits the raw
casting-entry identifier; checked parent context supplies product grouping;
the source fact, JSON contract and CLI presentation retain it.
Reference readers omit missing optional price units and IWR exception/double
qualifiers, and absent baseline damage categories. Null/invalid IWR qualifiers
and required magnitudes remain visible. Damage captions use presentation order,
while authored component IDs remain unchanged for correlation.
Known-empty optional Notes and heightening damage-increment text have no reading
row; unavailable facts and patch clears remain explicit. Spell forms use authored
names or ordered captions, preserving their IDs for identity and invalid/null
name facts for inspection.

Selected immediate Actor casting entries carry checked parent context and their
associated local spell roster. Association uses the actual typed source location
and unique checked owner identity; duplicate or unavailable associations are not
guessed. Spell activities expose base rank and explicit authored cast rank,
without deriving cantrip rank or executing casting. Root activities already carry
the roster and do not duplicate it inside each casting entry. Browser layouts
reuse this meaning in compact spellcasting groups and selected entry pages.

Hazards carry an explicit defense-block display fact. Known AC and maximum HP
both equal to zero make the primary AC/HP/hardness block inapplicable, following
the pinned upstream hazard policy. The original facts remain in the response;
zero saving throws stay valid and independently visible. Unavailable AC or HP
does not imply the block is absent. Detection displays the authored modifier and
prepared details without asserting a fully executed Foundry Stealth DC.

Each relationship occurrence includes source navigation from its actual root,
owner chain and field. Incoming occurrences navigate the referring source, not
the graph seed. Snapshot-local owners carry the current artifact fingerprint;
stable owners do not need one. Repeated occurrence ordinals remain distinct.

Authored spell damage, heightening, and form changes are reference facts. Their
display does not execute selected-rank casting, form application, resource use,
or arbitrary Foundry rules. Parent-owned copies retain local customizations and
exact navigation rather than resolving to a canonical replacement.

Prepared sanitized HTML and its field-local marker facts remain the content
contract. CLI uses html2text at read time; the browser uses DOMPurify and native
Ant overlays. No markup AST or alternative stored rendering is introduced.

Full selected identity, including owner chain, field, passage, and fingerprint,
drives detail requests and query caching. Reader transitions preserve that
identity. Root summaries and search rows stay body-free; richer detail decodes
one root and batches selected content.

## Consequences

Family layouts can evolve without another stored authored model or a generic
layout interpreter. New response facts need a concrete consumer and typed source
evidence. New compact search facts instead require deliberate query/index design.
CLI and browser share meaning without sharing density or display structure.

Within the browser, family presentation components are reused across detail,
preview, comparison, and encounter consumers. Feature components compose their
own controls around that shared presentation. Generic interactions reuse the
shared Ant-based primitives. A new component needs a clear responsibility or
actual reuse need; local copies of the same family layout are not independent
presentation owners.

Reference reading puts descriptive prose before grouped mechanics; encounters
prioritize current combat state and actions. Family sections adapt to actual pane
width rather than assuming full-window width. Owned ability disclosures request
the exact selected detail lazily and reuse the checked detail/cache identity and
prepared content renderer. Expanding an ability preserves its parent context;
an explicit native link remains available for full-page reading. These are
transient layout/workflow choices, not additional stored content or family models.

This replaces the section/value/profile decision and migration fallback in
ADR0031, while retaining its app-service composition boundary. Old transport,
renderers, profiles, and fixtures are removed together; no compatibility shim
is retained.
