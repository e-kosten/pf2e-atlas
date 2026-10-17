# Record Presentation and UI Architecture Review

Status: done
Owner: Codex
Last reviewed: 2026-10-10

## Outcome

Reviewed the checked source, prepared content, borrowed reads, application
contracts, CLI and browser consumers against actual source examples. The earlier
[integration PR 7](https://github.com/e-kosten/pf2e-atlas/pull/7) supplied feature
requirements, not implementation authority. The resulting presentation boundary
is recorded in [ADR 0047](../../../architecture/decisions/0047-semantic-record-presentation.md).

Transient semantic family responses replace the generic section/value/profile
contract. Selected identity, complete owned navigation and prepared content remain
explicit. Creatures, hazards and authored spells share extraction between root
and owned detail. Encounters compose the authored actor baseline once per record
with existing bounded runtime adjustments and return one presentation alongside
participant state. The private arithmetic workspace is not public transport.

The CLI and browser independently format the same facts. Shared browser family
components are reused by detail, reader, preview, comparison and encounter
surfaces; feature components own their controls. Generic UI uses existing Ant
primitives, including context-aware source-control dialogs. Narrow panes wrap
prose while wide authored tables scroll within their content boundary.

Checked Foundry DTOs remain the sole stored authored authority. Artifact schema,
ingest admission, search projections, embedding policy and local-state storage
are unchanged. Root summaries and search rows remain body-free. Exact selected
detail performs one root decode and batches selected content; encounter roster
hydration does not eagerly load content and shares its authored baseline across
copies of a record.

## Validation scope

Source-grounded service and CLI assertions cover IWR qualifiers, known zero and
unavailable facts, hazard organization, partial spell forms and heightening,
casting associations, parent-local spell identity, and retained physical/table
facts. Navigation tests cover reader previews, follow-link, close, promotion and
Back/Forward. Encounter tests cover authored adjustment, modifier explanations,
independent participant state, HP edit intent and bounded read counters.

Two independent Astra reviews inspect implementation and actual CLI output plus
light, dark and narrow browser layouts and interactions. The user approved real
Playwright screenshots and interactions after Safari MCP failed before opening a
page. Browser evidence uses the existing 220-record semantic sample; CLI evidence
also uses the complete 25,641-root lexical artifact. No artifact rebuild or full
semantic inference is required for this presentation change. Local validation,
hosted CI and human visual approval remain separate milestones.

## Remaining feature work

Dedicated equipment, journal and table layouts remain in
[record detail polish](../../items/rust-web-record-detail-polish.md). Narrow
physical-reference and table variants preserve their existing useful facts;
other families intentionally present prepared content and owned links.
Automatic spell/form evaluation, resource tracking, arbitrary Foundry rules,
macro execution and table draws are outside this completed slice.

## Related

- [Architecture overview](../../../architecture/overview.md)
- [Runtime architecture](../../../architecture/runtime.md)
- [Frontend guidelines](../../../../web/atlas-ui/docs/frontend-guidelines.md)
