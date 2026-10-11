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

This replaces the section/value/profile decision and migration fallback in
ADR0031, while retaining its app-service composition boundary. Old transport,
renderers, profiles, and fixtures are removed together; no compatibility shim
is retained.
