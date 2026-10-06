---
name: delegated-agent-contract
description: Use when Codex is spawned as a sub-agent, worker, explorer, validator, or remediation agent for delegated repository work, especially architecture-sensitive implementation, refactors, and independent validation. Establishes a concise outcome-scoped brief, autonomous implementation envelope, risk-based validation, and escalation boundary.
---

# Delegated Agent Contract

Use this skill before delegated work. Identify the explicitly current brief and compatible corrections; arrival order alone does not make a delayed callback authoritative. Execute autonomously inside the outcome and semantic boundaries; do not turn likely file lists or procedural history into exhaustive permission gates.

## Brief Gate

Before material edits or expensive commands, confirm that the brief supplies enough context to preserve the intended outcome.

Required when material to the task:

- task type: implementation, remediation, validation, exploration, or cleanup
- outcome and accepted base, branch, worktree, commit, diff, or current context
- governing architectural and source-fidelity invariants
- non-goals or boundaries that prevent a materially different outcome
- risk-appropriate validation and applicable final gates
- escalation conditions and requested handoff shape

Likely files and modules may be included for coordination, but they are not an exhaustive permission allowlist. The outcome authorizes mechanically necessary callers, imports, tests, current-behavior docs, generated bindings, formatting, and refactor cleanup. Report those edits; do not request path-by-path or count-only amendments.

If a missing item could materially change the product outcome, architecture, source disposition, acceptance strength, external effects, or expected cost, return:

```text
brief incomplete
missing:
- <missing item>
needed because:
- <why this matters>
smallest useful next prompt:
- <what the caller should provide>
```

Proceed with reasonable implementation assumptions when they stay within the outcome and invariants. State material assumptions in the handoff.

## Short Brief And Handoff

Use this reusable structure for ordinary work; omit empty optional lines.

```text
Task: <implementation | remediation | validation | exploration | cleanup>
Outcome: <observable completed result>
Base/context: <current checkout, candidate, plan/issue, or accepted state>
Invariants: <applicable architecture, source fidelity, behavior, and acceptance that must remain true>
Non-goals: <materially different outcomes not authorized, when needed>
Validation: <focused iteration checks and applicable final gates>
Escalate if: <decision, weakening, external/destructive action, or unexpected cost threshold>
Handoff: <changed behavior/files, checks/results, baseline failures, reused evidence/limits, risks>
```

A newer brief or correction supersedes conflicting procedural instructions only when it comes from the user or the decision owner authorized to change that requirement and is explicitly current or names what it supersedes. Acknowledge the current base/candidate and ignore stale callbacks or commands. Do not require authorization shell scripts, checksum packages, publication wrappers, a launcher, or a new task framework for ordinary repository work. Existing semantic requirements and genuine source/artifact/candidate trust-boundary checks still apply.

## Work Rules

- Validate against the outcome, actual diff, architecture, and production paths—not a prior completion claim.
- Complete necessary cross-file consequences inside the outcome without administrative escalation.
- Do not revert or overwrite other agents' changes.
- Do not add transitional shims, adapters, broad fallbacks, compatibility layers, or mixed old/new paths unless the brief explicitly requires an incremental strategy.
- Do not bypass shared services, boundary facades, lint-enforced owners, or architecture docs to make a slice pass locally.
- Do not duplicate shared interaction, routing, parsing, indexing, or projection logic in a feature controller when the plan or docs identify a shared owner.
- Do not treat build/test success as completion unless those checks prove the outcome and invariants.
- Escalate product or architecture choices, source-disposition changes, acceptance weakening, destructive/external actions, work beyond the non-goals, or substantial unexpected cost. Do not escalate file-count changes or ordinary implementation choices.
- Preserve source/artifact integrity and candidate provenance where those identities are actually consumed. Do not replicate authorization metadata at ordinary coordination boundaries.
- Follow the repository model-selection policy in `AGENTS.md`; never claim an unverified model.

## Implementation And Remediation

For code-changing delegated work:

- Read the assigned plan excerpts and architecture docs before editing.
- Identify the intended owner for each behavior before choosing files to modify.
- Prefer direct end-state replacement over bridge code or compatibility seams.
- Add or update tests for the assigned behavior unless the caller explicitly scoped the task as docs-only or validation-only.
- Use focused validation during edits and ordinary slice handoff. Run full repository gates only when this handoff owns the integrated/final candidate or an existing repository trigger requires them; do not repeat unchanged full-gate evidence.
- Reuse validation evidence only when its relevant source, toolchain, candidate, generator, and policy inputs are unchanged; disclose the reuse and its limitations.
- Use embedding/full semantic-artifact validation at final artifact or end-to-end gates when those inputs changed, not for routine UI, docs, or unit loops.
- If validation fails for an ordinary outcome-local reason, fix it. Distinguish pre-existing baseline failures from candidate regressions, report both, and never silently waive them.

Final report:

```text
verdict: complete | partial | blocked
scope: <assigned slice>
changed files:
- <path>
validation:
- <command/search/test>: <result>
outcome coverage:
- <item>: satisfied | partial | not addressed
architecture compliance:
- <boundary/rule>: preserved | violated | unproven
evidence reuse:
- <reused evidence and limitation, or "none">
baseline failures:
- <pre-existing failure and disposition, or "none observed">
remaining risks:
- <risk or "none known">
```

## Validation

For delegated validation:

- Build a risk-based checklist from the outcome and architecture rules.
- Inspect the actual diff, production paths, source, tests, docs, generated artifacts, and git/worktree state as relevant.
- Prefer concrete evidence: file references, commands, searches, test names, lint rules, or commit state.
- Use adversarial, mutation, rollback, identity, and residue checks where they target plausible failures.
- Batch related findings so remediation can address the boundary coherently.
- Report every material missing, partial, contradictory, unproven, silently waived, or shortcut-looking item.
- Separate baseline failures from new regressions; neither disappears from the report.
- Reuse evidence only for unchanged relevant inputs and state the limitation.
- Mark material uncertainty as a gap unless you can resolve it locally.
- Do not fix issues unless the caller explicitly asks for remediation.

Final report:

```text
verdict: pass | fail | uncertain
items checked:
- <assigned item>
evidence:
- <file, command, search, or test>
gaps:
- <missing, partial, contradicted, or unproven item>
baseline failures:
- <pre-existing failure and disposition, or "none observed">
architecture concerns:
- <shortcut, owner drift, shim, boundary bypass, or "none found">
recommended follow-up:
- <smallest remediation or extra validation>
```

## Exploration

For delegated exploration:

- Answer the bounded question only.
- Cite concrete files or docs.
- Do not make code changes unless the caller explicitly asks.
- If the question cannot be answered without broader context, return `brief incomplete` with the missing context.
