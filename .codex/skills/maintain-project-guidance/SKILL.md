---
name: maintain-project-guidance
description: "Create or refine project documentation and contributor or agent instructions, especially when turning accepted decisions or recurring mistakes into scoped, verifiable guidance."
---

# Maintain Project Guidance

Maintain guidance that changes the next contributor's decisions without requiring
the conversation that produced it. Use the existing documentation owner; this
skill describes maintenance practice, not a second source of product rules.

## Find the audience, owner and evidence

- Read applicable AGENTS files and the existing guidance for the affected area.
  Follow root AGENTS Documentation Positioning: README for product use,
  CONTRIBUTING for workflow, architecture/ADRs for durable boundaries and
  rationale, and the existing frontend guide for UI rules.
- Distinguish current verified behavior, an accepted intended rule, a proposal,
  and historical evidence. Existing code or one accepted visual example does not
  establish a universal preference. Keep temporary work status in the current
  plan rather than permanent contributor rules.
- Use current code, commands and explicit user decisions to check local claims.
  When external research is needed, prefer primary documentation and cite the
  source supporting the claim. Separate vendor advice from our own synthesis;
  do not require fresh research for every routine documentation edit.

## Write instructions that support decisions

- State when the rule applies, the preferred action and any consequential
  exception. Add a brief reason where it helps contributors generalize correctly.
  Replace vague adjectives such as consistent or intuitive with observable choices.
- For a behavioral rule, give an explicit positive use and a likely failure case.
  Include a boundary example when a default could be applied too broadly. State
  the desired alternative alongside a prohibition; a list of mistakes alone
  leaves the next contributor guessing.
- Match the detail to the reader. Product docs describe capabilities, outcomes
  and recovery; contributor instructions identify owners, constraints and checks.
  Do not turn user-facing prose into an implementation manual.
- Keep broadly loaded AGENTS instructions short and relevant to their scope.
  Link to the maintained detailed owner with an explicit trigger to read it.
  Put an occasional workflow in a skill rather than appending its full procedure
  to every contributor's context. Do not create another tool-specific copy merely
  to repeat the same rule.
- Reconcile old guidance when adding a rule. Search related instructions and
  callers, replace contradictions, remove superseded wording and repair links.
  Preserve necessary scope and rationale while pruning duplication and stale
  catalogs. Do not rewrite unrelated guidance to satisfy an arbitrary line count.

For example, “show facts before editing controls” can incorrectly prohibit the
accepted inline HP editor. Check the shared principle: HP's quiet value button
and spell rank's named action both preserve reading while visibly offering an
edit. Boundary cases include hover-only edit cues and data-entry forms where
inputs should remain visible. Do not merely list HP and spells as exceptions to
an unexplained default. This illustrates scoping; keep actual UI policy in the
[frontend guide](../../../web/atlas-ui/docs/frontend-guidelines.md#reading-and-editing-values).

## Check the guidance and report its limits

- Inspect the scoped diff, formatting, links, anchors and referenced paths.
  Check changed commands or executable examples against their real owner.
  Follow repository validation policy; prose-only edits do not justify unrelated
  expensive gates.
- Apply each consequential instruction to a representative case, a likely wrong
  choice and a boundary/exception. Revise it if both choices appear compliant or
  if it rejects an accepted interaction. This is an editorial check, not proof
  that another agent will follow it.
- Give behavioral guidance a concrete way to inspect the result. For UI, check
  rendered geometry and interaction rather than size props alone. For commands,
  check the relevant exit/output or durable state. Reuse existing checks; do not
  add tests that only assert the instruction's wording.
- Distinguish document checks, observed implementation behavior and independent
  agent adherence. Report what was actually checked. Refine the rule when real
  usage reveals ambiguity; prefer a narrow correction over another blanket rule.

## Research basis

These are vendor recommendations informing the workflow, not evidence that a
particular format guarantees compliance. Consult them when revisiting this
workflow; normal use does not require fetching all sources.

- [OpenAI: AGENTS discovery and scoped instructions](https://learn.chatgpt.com/docs/agent-configuration/agents-md).
- [Anthropic: concrete, organized and noncontradictory instructions](https://code.claude.com/docs/en/memory#write-effective-instructions).
- [Anthropic: concise guidance and checking actual behavior](https://code.claude.com/docs/en/best-practices#write-an-effective-claudemd).
- [Anthropic: relevant, diverse examples and explanatory context](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices).
- [GitHub: short, self-contained repository instructions](https://docs.github.com/en/copilot/concepts/prompting-and-context/custom-instructions#writing-effective-custom-instructions).
