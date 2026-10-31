---
name: maintain-project-guidance
description: "Create or refine project documentation and contributor or agent instructions, especially when turning accepted decisions or recurring mistakes into scoped, verifiable guidance."
---

# Maintain project guidance

Use this workflow for documentation and contributor/agent instruction changes.
[Documentation ownership](../../../docs/development/documentation.md) owns the
repository's audience map, ADR lifecycle and maintenance checks. Read it and
applicable AGENTS instructions before editing; keep actual product and workflow
rules in their maintained owners.

## Establish the change

1. Identify the reader, the decision or task the text should help with, and its
   existing owner. State what the reader already knows and the action they need
   to complete. Search related guidance before creating another document.
2. Check the claim against current source, commands and explicit user decisions.
   Separate observed behavior, accepted intent, proposals and historical evidence.
   Use primary sources if external research is needed; routine maintenance does
   not require fresh research.
3. Choose the smallest complete consolidation: retain one authoritative explanation,
   update entry points and remove redundant or superseded guidance. Preserve
   consequential exceptions and provenance-bound historical evidence.

## Edit and reconcile

Apply the owner's [audience guidance](../../../docs/development/documentation.md#write-for-the-pages-reader).
Read the page in the reader's task order. Put a useful example before optional
settings; separate advanced reference where needed. Check that labels, command
blocks and prerequisite instructions agree. State an instruction's trigger,
preferred action and meaningful exception. Add a reason or example when it helps distinguish the right choice.
Do not turn one accepted case into a universal preference.

Reconcile affected instructions and callers in the same change. Follow the
owner's ADR retirement procedure when needed. Keep temporary status in the task
plan. Avoid tool-specific copies of repository rules.

## Verify and report

Apply the [documentation checks](../../../docs/development/documentation.md#validate-the-edited-documentation),
including audience/task fit, incoming links, anchors, command examples and
representative/boundary cases for consequential instructions. Follow the normal contribution sequence
for commits, independent review, validation and publication.

Report what became simpler, what was removed or relocated, checks actually run
and remaining limitations. Distinguish editorial checks, observed implementation
behavior and evidence of agent adherence; none substitutes for the others.
