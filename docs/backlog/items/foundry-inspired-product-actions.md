# Foundry-inspired product actions

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

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

## Concrete encounter requests

The user requested creature initiative rolling and dice rolling while running
encounters. Start with those workflows rather than broad Macro execution.

- Define how a creature initiative roll chooses an available effective modifier,
  how a user can select another supported basis, and how unknown or situational
  modifiers are handled. Show the die, modifier and total; agree when the result
  updates participant initiative and how deliberate rerolls work.
- Review simple manual dice entry and prepared authored roll/check controls as
  separate entry points over shared bounded dice behavior. Evaluate maintained
  parsing/rolling libraries before adding custom formula logic. Keep unsupported
  Foundry expressions explicit and do not imply full contextual rules execution.
- Agree where results appear, whether any history persists, and which operations
  change encounter state. Initiative and general roll results should not silently
  spend resources or apply damage/conditions. These remain design decisions.
- Evaluate optional, explicitly configured persistent-damage and recovery
  automation after the reminder workflow. The approved encounter design prompts
  for table rolls and offers deliberate HP/condition actions; it does not roll,
  apply damage or decide recovery automatically.

Coordinate the interaction with [encounter usability](rust-web-encounter-runner-polish.md).
Discuss the design with the user before implementation and retain Rust ownership
of product rules rather than duplicating them in frontend controls.
