# Maintaining documentation

Read this guide before creating, moving or materially revising documentation,
including contributor and agent instructions. The
[maintain-project-guidance skill](../../.codex/skills/maintain-project-guidance/SKILL.md)
provides the editing workflow; this guide owns repository documentation policy.

## Choose the reader and owner

| Reader or purpose | Maintained owner |
| --- | --- |
| New user: what Atlas does and the default installation | [README](../../README.md) |
| First browser session: find, save and use records | [Getting started](../getting-started.md) |
| Optional installation, search or maintenance tasks | [Installation](../installation.md), [search](../search.md), [setup and maintenance](../setup-and-maintenance.md) |
| Command recipes and JSON behavior | [CLI examples](../cli-examples.md); command definitions own flags/defaults |
| Contributor sequence and publication rules | [CONTRIBUTING](../../CONTRIBUTING.md) |
| Detailed contributor procedures | The relevant guide in this directory |
| Module ownership and current architecture | [Architecture overview](../architecture/overview.md), [runtime](../architecture/runtime.md), [artifact contract](../architecture/artifact-contract.md) |
| Durable decision and its rationale | [ADRs](../architecture/decisions/README.md) |
| Open work and completed work | [Backlog](../backlog/backlog.md) and its history |
| Research findings and uncertainty | [Research](../research/README.md) |
| Cross-surface product copy and interaction | [Product guidelines](../product-guidelines.md) |
| Frontend implementation and design | [Frontend AGENTS](../../web/atlas-ui/AGENTS.md) and its linked guides |

Give a fact or procedure one detailed owner. Other entry points summarize the
next action and link there. A short executable default may appear in both the
README and a task guide; alternate flags, recovery procedures and contract details
should not grow parallel explanations in each.

For example, keep `atlas setup` in the quick start. Put model selection in the
search guide, data-path overrides in setup and maintenance, and developer baseline
selection in development setup. A new user should not need to choose these to
complete the default installation.

## Write for the page's reader

Start with what the reader is trying to do and what they already know. A new
Atlas user may know Pathfinder well while knowing nothing about Atlas, its search
models or its storage. A CLI user may need exact flags and JSON fields; a
contributor may need crate boundaries and build identities. Technical precision
belongs where that reader needs it, with unfamiliar terms explained on first use.

- Lead user guides with a task and a useful example, then introduce controls or
  optional settings. Teach searching by idea before describing ranking options.
  Explain what a choice changes and when to try it; do not list settings without
  helping the reader choose. Avoid claiming a best model without evidence.
- Explain necessary actions and their timing where they first appear. For example,
  setup runs after installation and after upgrading; opening Atlas is a separate
  action. A backup recommendation needs a linked procedure, including how to find
  the files and when to stop running commands.
- Describe user-visible effects rather than internal implementation. In a browser
  guide, explain which content Similar compares instead of describing stored
  vectors or model weights. In a CLI reference, retain literal JSON field names
  and explain the result they represent. Architecture and agent instructions can
  require more detailed contracts; do not erase distinctions their readers need.
- Remove unprompted negatives and reassurance about things the reader has no
  reason to expect. “You do not need a source checkout” distracts from first-run
  setup. Keep a limitation when it changes a real choice: a reference to equipment
  does not prove it is worn, and `--force` can replace customized skill instructions.
- Keep everyday recipes separate from advanced navigation, output contracts and
  diagnostics. Link between them instead of making the reader pass through each
  internal guarantee. Current reference docs describe today's behavior; migration
  history belongs in ADRs, release notes or history.
- Make each command block match its label. A preview block contains only commands
  that preview; put removal or replacement commands in a separate labelled block.
  Explain placeholders and prerequisites before commands that depend on them.
- Future release notes lead with what users can do and what they must do to
  update. Put crate names, transport changes and contributor commands in a
  separate contributor section when useful. Preserve historical release accounts.

## Keep current guidance separate from history

Product and contributor docs describe current behavior and the next useful action.
Verify commands/defaults against their owning source. Distinguish observed behavior,
accepted intended boundaries, proposed work and historical evidence. A passing
validator does not prove a proposal is implemented; one accepted UI example does
not establish a universal design rule.

Architecture docs describe current intended boundaries. ADRs explain the choices
behind them; they are not an implementation checklist. When a decision changes:

1. Update its current architecture owner and affected user/contributor docs.
2. Amend the old ADR's status with the scope and a linked successor. Keep partially
   amended ADRs active while their retained boundaries still apply.
3. Move a fully retired ADR into `docs/architecture/decisions/history/`, retaining
   its number, filename and rationale. Update both indexes and incoming links.

For example, ADR0030's separate local-state file is retired by ADR0051's shared
workspace; the saved-domain owner remains `atlas-local-state`. Archiving the old
file layout must not erase that surviving boundary. Do not rewrite an old
rationale to make it appear that the newer decision was always in force.

Keep temporary task status in the current plan and durable remaining work in the
backlog. Preserve research snapshots and files covered by recorded hashes,
including manifests and historical validation evidence. Update links around those
frozen files rather than editing their contents to match new defaults. Remove duplicated or obsolete explanations
when their useful rationale already has a maintained home or historical record.

## Write contributor and agent instructions

Keep broadly loaded AGENTS files focused on consequential rules and explicit
triggers to read detailed owners. A link should say when it matters. A nested
AGENTS file is not assumed loaded from a root-started task; root routing must
identify it. CONTRIBUTING owns the development sequence. Skills describe how to
perform a task and link to repository policy, rather than maintaining another copy.

State when a rule applies, the preferred action, and any consequential exception.
Add a brief reason or example where it helps distinguish valid choices. Prefer an
observable check over adjectives such as consistent or intuitive. For example,
“check moved links and anchors” gives a verifiable action; “keep docs clean” does not.

Before adding a rule, search its existing owner and related callers. Replace
contradictions and superseded wording. Relocation must preserve consequential
exceptions; brevity alone is not a reason to remove policy.

## Validate the edited documentation

- Inspect the scoped diff for accuracy, duplication and whitespace. Check relative
  links, heading anchors, moved paths and incoming references, including archived
  ADRs. Keep command examples consistent with their real flags and defaults.
- Walk through each page as its intended reader, in reading order. Can a newcomer
  complete the task without understanding internal terms or making optional choices?
  Do command blocks do what their labels promise? Check prerequisites and linked
  instructions for tasks such as backups. Review the literal prose as well as
  links and examples; a shorter sentence can still be vague or unhelpful.
- For a consequential instruction, try a representative case, a likely mistake
  and a boundary/exception. Revise it if both choices appear compliant or it rules
  out an accepted workflow. This is an editorial check, not evidence that another
  agent will follow the instruction.
- Follow [validation policy](validation.md#changed-path-selection-and-hooks).
  Prose-only changes need document checks and independent review, not application
  builds. Report unavailable execution or review evidence explicitly.
