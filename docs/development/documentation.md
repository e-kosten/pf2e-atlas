# Maintaining documentation and agent guidance

## Documentation owners

Keep each documentation surface focused on its job:

- `README.md` should describe the current product, what users can do with it, and how to set it up and run it.
- `docs/architecture/` should describe the current intended architecture and durable boundaries that the codebase is expected to follow.
- `docs/backlog/backlog.md` should list only open backlog work using the active status vocabulary in that file.
- `docs/backlog/history/` should hold durable history of completed or retired backlog work.
- `CONTRIBUTING.md` is the contributor entry point and development/PR sequence. Detailed procedures live in `docs/development/`; the architecture overview owns the crate map.
- Historical context, design evolution, tradeoff records, and "why we changed this" material should primarily live in ADRs under `docs/architecture/decisions/`.

When writing or editing docs, prefer statements of current behavior and current ownership over repo-history framing unless the document is explicitly an ADR or backlog/history note.

When creating or materially revising project guidance, including contributor and
agent instructions, use
[maintain-project-guidance](../../.codex/skills/maintain-project-guidance/SKILL.md)
for scoped rules, concrete examples, reconciliation and validation. Keep the
actual product or workflow rules in their existing documentation owner.

## Entry points and detailed owners

Keep root AGENTS focused on instructions that affect most tasks and explicit
triggers for reading detailed owners. Keep CONTRIBUTING a short contributor
entry point. Specialized procedures belong in a coherent task guide; read those
sections when their task applies. A link alone does not explain when it matters.
Frontend AGENTS routes frontend work to the maintained UI guide. Cross-surface
copy and interaction rules live in [product presentation](../product-guidelines.md),
including Rust-projected copy.

Before adding guidance, search the existing owners and replace contradictory or
superseded wording. Maintain one detailed owner instead of copying the procedure
into AGENTS, CONTRIBUTING and skills. Skills describe how to perform a task and
link to repository policy; repository decisions do not belong in global skills.
Retain consequential exceptions and useful examples when relocating text. Check
relative links, heading anchors and incoming references after a move.

Short entry points are an editorial goal, not a reason to discard useful policy.
Codex discovers instructions from the root toward its working directory, with a
combined size budget. Do not assume a nested AGENTS file is automatically loaded
when an agent starts at the root; root routing must explicitly identify the file
to read for that work. Linking detailed material avoids requiring every task to
read every operational procedure.

## Research basis

These sources inform organization; the repository's accepted workflow remains
its own policy. No fresh external research is needed for routine maintenance.

- [GitHub contributor guidelines](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors): discoverable contribution entry point.
- [OpenAI AGENTS discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md): instruction scope, precedence and combined size budget.
- [Anthropic instruction organization](https://code.claude.com/docs/en/memory#write-effective-instructions): concrete, scoped and consistent guidance.
- [Diataxis](https://diataxis.fr/): separate task procedures, reference and explanation.
