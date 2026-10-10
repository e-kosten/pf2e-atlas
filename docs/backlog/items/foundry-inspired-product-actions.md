# Foundry-inspired product actions

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-09

Review upstream Macro documents, enrichments and runtime hooks as inspiration for
useful Atlas controls, such as rolling an authored formula or presenting a
supported check. Macros remain developer-only and arbitrary Foundry JavaScript
is never executed by the current product.

Start from a concrete user workflow and the checked DTO/prepared interaction
facts. Determine which actions can use existing libraries and which require
explicit gameplay context. Preserve unsupported runtime expressions as authored
data; a parsed predicate or formula does not prove that Atlas can execute the
Foundry behavior correctly.

Modifier callbacks, runtime Predicate behavior and ChoiceSet evaluation require
separate contextual semantics. Retained source-contract evidence is useful for
that study; persisted JSON types alone do not include every runtime callback.
Do not infer executable mechanics from trait-description prose.

Define bounded action contracts and tests before adding UI behavior. Keep
readiness, display, dice execution and contextual gameplay adjudication distinct.
