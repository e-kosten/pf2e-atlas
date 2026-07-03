---
name: delegated-agent-contract
description: Use when Codex is spawned as a sub-agent, worker, explorer, validator, or remediation agent for delegated repository work, especially plan-orchestration, plan-validation, architecture-sensitive implementation, refactors, or multi-agent slices. Enforces a complete assignment contract with plan excerpts, architecture rules, ownership boundaries, forbidden shortcuts, validation obligations, and a punt-back protocol when the caller did not provide enough information.
---

# Delegated Agent Contract

Use this skill before doing delegated work. Treat the caller's prompt as an assignment contract, not as permission to infer missing architecture or completion rules.

## Contract Gate

Before exploring, editing, validating, or running long commands, check whether the caller provided enough information to do the delegated task safely.

Required for every delegated task:

- task type: implementation, remediation, validation, exploration, or cleanup
- exact scope: plan path, plan section, validation artifact, issue list, or concrete question
- target state: branch, worktree, commit, diff range, or current checkout to inspect
- ownership: files, modules, subsystems, or read-only boundaries you own
- architecture context: relevant docs, ADRs, boundaries, service owners, or explicit statement that none apply
- forbidden shortcuts: shims, compatibility layers, raw data access, direct parsing, duplicated logic, bypassed services, broad fallbacks, or any slice-specific no-go rules
- validation: commands, searches, tests, or evidence required before saying done
- report format: what the caller needs back to integrate or adjudicate the result

If any required item is missing and the gap could affect correctness, architecture, write scope, or validation integrity, do not improvise. Return:

```text
contract incomplete
missing:
- <missing item>
needed because:
- <why this matters>
smallest useful next prompt:
- <what the caller should provide>
```

Only proceed with a reasonable assumption when the missing detail cannot change the implementation boundary or validation verdict. State the assumption in the final report.

## Work Rules

- Validate against the plan and architecture contract, not against a prior completion claim.
- Stay inside the assigned ownership. Do not edit adjacent modules unless the caller explicitly authorizes the expansion or the fix is impossible without it.
- Do not revert or overwrite other agents' changes.
- Do not add transitional shims, adapters, broad fallbacks, compatibility layers, or mixed old/new paths unless the caller explicitly says the plan allows them.
- Do not bypass shared services, boundary facades, lint-enforced owners, or architecture docs to make a slice pass locally.
- Do not duplicate shared interaction, routing, parsing, indexing, or projection logic in a feature controller when the plan or docs identify a shared owner.
- Do not treat build/test success as plan completion unless those checks prove the assigned contract items.
- If the task reveals a consequential architecture decision, scope expansion, or conflict with the plan, stop and punt back instead of patching around it.

## Implementation And Remediation

For code-changing delegated work:

- Read the assigned plan excerpts and architecture docs before editing.
- Identify the intended owner for each behavior before choosing files to modify.
- Prefer direct end-state replacement over bridge code or compatibility seams.
- Add or update tests for the assigned behavior unless the caller explicitly scoped the task as docs-only or validation-only.
- Run the requested validation or explain exactly why it could not run.
- If validation fails for an ordinary slice-local reason, fix it inside scope. If it fails because the contract is wrong or incomplete, punt back.

Final report:

```text
verdict: complete | partial | blocked
scope: <assigned slice>
changed files:
- <path>
validation:
- <command/search/test>: <result>
plan coverage:
- <item>: satisfied | partial | not addressed
architecture compliance:
- <boundary/rule>: preserved | violated | unproven
shortcuts avoided:
- <notable forbidden shortcut checked>
remaining risks:
- <risk or "none known">
```

## Validation

For delegated validation:

- Build a checklist from the assigned plan items and architecture rules.
- Inspect source, tests, docs, generated artifacts, and git/worktree state as relevant.
- Prefer concrete evidence: file references, commands, searches, test names, lint rules, or commit state.
- Report every missing, partial, contradictory, unproven, or shortcut-looking item.
- Mark uncertainty as a gap unless you can resolve it locally within scope.
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
- If the question cannot be answered without broader context, return `contract incomplete` with the missing context.
