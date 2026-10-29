# Atlas UI agent guidance

This package owns React browser presentation over generated Rust app contracts.
Read the root [development sequence](../../CONTRIBUTING.md#development-loop).
The detailed UI owner is [frontend guidelines](docs/frontend-guidelines.md);
read the relevant sections below before changing their behavior.

## Ownership and composition

- Keep product semantics in Rust app-service/app-model. Render generated facts
  and use the thin API client; do not duplicate rules, retrieval or filtering.
- Use Ant Design and its component tokens for generic controls and states.
  Atlas components serve product-specific reference and encounter presentation.
- Before adding a feature-local interaction, inspect `src/shared/ui`. Compose
  existing shared primitives with feature requests, labels and DTOs. Move repeated
  behavior to its existing shared owner rather than copying it.
- Use semantic theme tokens and Atlas variables, not ad hoc colors or palettes.
- Review rendered copy and interactions, including backend-projected text, against
  [product presentation](../../docs/product-guidelines.md).
- Audit authored fields through source views and projections before redesigning a
  record family; a current presentation DTO is not proof of source completeness.

## Read the guide for the change

| Before changing                                    | Frontend guide section                                                                                                                            |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Composition, hierarchy, density or disclosures     | [Design principles](docs/frontend-guidelines.md#design-principles)                                                                                |
| Errors, unavailable states, retries or recovery    | [Error feedback and recovery](docs/frontend-guidelines.md#error-feedback-and-recovery)                                                            |
| Navigation, native links or browser history        | [Routing and navigation](docs/frontend-guidelines.md#routing-and-navigation)                                                                      |
| Repeated controls, toolbars or destructive actions | [Shared UI layer](docs/frontend-guidelines.md#shared-ui-layer)                                                                                    |
| Inline editing and persistent edit cues            | [Reading and editing values](docs/frontend-guidelines.md#reading-and-editing-values)                                                              |
| CSS, scroll containers, insets and content frames  | [Styling](docs/frontend-guidelines.md#styling) and [padding ownership](docs/frontend-guidelines.md#padding-and-control-group-ownership)           |
| Small tasks, dialogs, popovers or drawers          | [Choosing an interaction surface](docs/frontend-guidelines.md#choosing-an-interaction-surface)                                                    |
| Selected records, linked references and previews   | [In-place reference reading](docs/frontend-guidelines.md#in-place-reference-reading)                                                              |
| Record facts, family layouts or completeness       | [Record presentation](docs/frontend-guidelines.md#record-presentation) and [grouping facts](docs/frontend-guidelines.md#grouping-reference-facts) |
| Layout, touch, narrow/short views or zoom          | [Responsive acceptance](docs/frontend-guidelines.md#responsive-design-and-acceptance)                                                             |
| Dice, encounter panels or saved mutations          | [Mutable workflows](docs/frontend-guidelines.md#mutable-workflows)                                                                                |

## Feedback and validation

For substantive visible UI work, share an early functional preview and obtain
user feedback before expensive final validation. Minimal UI changes and internal
refactors can skip that checkpoint or offer a later preview. Keep existing tests
enabled and disclose unfinished behavior; the checkpoint does not establish final
acceptance or replace independent review.

For frontend-only work, reuse a compatible running backend with Vite hot reload.
Human previews use the full-corpus development workspace and verified query models;
the model-free fixture serves automated acceptance and targeted fault checks.
Preserve the task preview URL and saved state. See
[setup and preview](../../docs/development/setup-and-preview.md).

Run focused tests during iteration, then the full frontend verification once
review fixes are complete:

```bash
npm --prefix web/atlas-ui run verify
```

Before PR creation, run affected browser flows and broaden coverage for shared
layout, navigation, overlays, forms and responsive behavior. CI owns the full
applicable device/browser matrix by default. Use
[validation](../../docs/development/validation.md) for selection, commands and
failure diagnosis. Preserve tracked visual references during refactors.

Obtain independent subagent review and fix confirmed findings before opening the
PR; local logical commits do not require that review gate. Presentation changes
also require Astra visual/interaction review with actual browser inspection.
Report candidate, viewports, input methods and unavailable evidence honestly:
automated checks, emulation, actual devices, human feedback and hosted CI are
different evidence.
