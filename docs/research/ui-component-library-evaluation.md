# UI component library evaluation

## Decision

Retain Ant Design for generic Atlas controls. The bounded comparison with Mantine
and shadcn/Base UI demonstrated feasible alternatives while leaving the principal
Atlas workflow owners in place. Their integration and refinement costs outweighed
the demonstrated maintenance benefit for the current application.

This reaffirms the component choice in [ADR0029](../architecture/decisions/0029-local-web-app-boundary.md).
The [frontend guide](../../web/atlas-ui/docs/frontend-guidelines.md#ant-design)
owns implementation policy.

## Question and scope

The evaluation asked whether another component system would better support
Atlas's established visual direction, simplify frontend ownership, or make
agent-driven maintenance more reliable. It used the application at source
`ace20ee2a6140df3d41acb46150cf055a87a2b5e` as the Ant control.

All lanes exercised the same rich Common/Advanced filters, full-page and nested
reference reading, and encounter HP/condition editing. Record renderers,
generated contracts and Rust-owned semantics were retained. Alternative lanes
replaced a finite shared-control footprint and continued to use Ant elsewhere.
Fresh maintainers then applied the same shared spacing and action-state task.
Checks sampled both appearances, keyboard/pointer interaction, narrow and short
viewports, nested focus/scroll return, value-only updates and browser zoom.

## Findings

- **Product workflow ownership persisted.** Rich selection, paging, nested Read
  actions, reader history, draft/mutation recovery and work-area geometry
  remained Atlas responsibilities in every lane. Replacing generic primitives
  preserved those compositions.
- **Mantine supplied useful finished controls.** Searchable Select and numeric
  controls provided supported mechanics and styling hooks. Overlay lifecycle,
  focus handoff and retained Ant forms still required Atlas integration. The
  bounded migration left the shared workflow owners in place.
- **shadcn/Base UI increased direct control and maintained surface.** Copied
  component recipes and primitive composition exposed markup and variants
  directly. Atlas also owned their styling assumptions, native link semantics,
  numeric accessibility and overlay integration. Scoped styling without the
  recipes' usual reset exposed resting button fills and cursor regressions.
- **Shared changes required an inventory of actual owners.** Both alternatives
  initially missed a retained Ant condition-row Remove action while updating
  the adjacent numeric step action. Broad ordinary-action styling also erased
  semantic selection backgrounds in one lane. Mixed-library results include
  that coexistence burden.
- **API discovery and rendered acceptance served different purposes.** Official
  skills, documentation tools and installed types helped agents choose supported
  mechanisms. Browser inspection and interaction traces exposed clipping,
  native activation and queued focus behavior that automated passes had missed.

## Limits

This was one observational comparison and one maintenance task per approach.
Review assistance, limited blinding and interleaved initial-candidate repairs
bound the agent-effort comparison. The partial migrations measured coexistence
with Ant; complete removal, production upgrade maintenance and broader task
classes remain unmeasured.

Ant and Mantine completed the bounded maintenance review. The shadcn candidate
passed its frontend gate and selected browser checks, with overflow/recovery
follow-up and one state sequence incomplete when the investigation closed.
User review also reported less-smooth drawer motion in the alternative preview;
frame timing was unmeasured. Physical devices and screen readers were outside the
executed coverage. These results support the component decision for the tested
Atlas workflows, with the detailed experimental evidence retained separately.

## When to reconsider

Start a future evaluation from a reproducible Ant limitation in a current Atlas
workflow. Identify the shared owner or mechanism an alternative would simplify,
then measure the resulting source ownership, interaction quality and maintenance
work on that workflow. Reuse the existing product requirements and supported Ant
customization hooks as the baseline.
