# Component Library Evaluation

This prototype compares Ant Design and Mantine against the Atlas product workflow, not as isolated widget demos. Keep shared Atlas behavior shared, use each library's native components where possible, and count custom composition as part of the library trade-off.

## Shared Baseline

These surfaces should stay shared while evaluating libraries:

- API client, DTO imports, workspace state, result-window semantics, URL state, diagnostics, and search request construction.
- Theme source of truth in `src/ui/atlasTheme.ts`, with library-specific provider adapters.
- Record detail rendering through `src/ui/recordPresentation.tsx`.
- Atlas-specific behavior such as selecting a record, following references, result paging, and diagnostics timing.

## Library-Owned Surfaces

Library-owned files intentionally isolate the comparison points:

- Ant Design filters: `src/ui/ant/AntFilters.tsx`
- Ant Design results: `src/ui/ant/AntResults.tsx`
- Mantine filters: `src/ui/mantine/MantineFilters.tsx`
- Mantine results: `src/ui/mantine/MantineResults.tsx`

When adding product behavior, prefer adding the behavior to shared state first, then expose it through each library's native components. If one library needs manual composition, document that as evaluation signal instead of hiding it behind an artificial parity wrapper.

## Evaluation Matrix

| Requirement                     | Ant Design path                                                      | Mantine path                                                         | Custom composition cost                     | Notes                                                                                                                                                                                                                       |
| ------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Text search                     | `Form.Item` plus `Input.Search`                                      | `TextInput` with `leftSection` icon                                  | Low                                         | AntD has a native search affordance with clear/search button behavior. Mantine keeps the input simpler and composes the icon.                                                                                               |
| Multi-value filters             | `Form.Item` plus `Select mode="multiple"`                            | `MultiSelect`                                                        | Low                                         | Compare keyboard behavior, tag overflow, searchable menus, and dense layout behavior.                                                                                                                                       |
| Numeric range filters           | `InputNumber`                                                        | `NumberInput`                                                        | Low                                         | Both have native numeric inputs. Atlas owns range semantics.                                                                                                                                                                |
| Boolean/toggle filters          | `Checkbox`                                                           | `Checkbox`                                                           | Low                                         | Both are native.                                                                                                                                                                                                            |
| Sort selector                   | `Select`                                                             | `Select`                                                             | Low                                         | Both are native; Atlas owns sort DTO mapping.                                                                                                                                                                               |
| Dense filter panel grouping     | `Collapse` with `Form layout="vertical"`                             | Plain stacked inputs/groups                                          | Medium for Mantine if matched               | AntD now supplies the stronger native form-and-section structure. Mantine can likely match this with `Accordion` plus explicit composition, but that would be additional implementation rather than current baseline.       |
| Add-filter empty/loading states | `Select` loading plus `Empty` not-found content                      | Plain `Select`                                                       | Low for AntD, medium for Mantine if matched | AntD's empty/loading affordances fit the dynamic filter catalog with little custom code. Mantine can represent the state, but the current prototype does less out of the box.                                               |
| Result table                    | `Table` with column config, loading, row key                         | `Table` inside `ScrollArea` with explicit rows                       | Medium for Mantine                          | AntD has the more complete data-table primitive. Mantine exposes table building blocks, so row rendering, loading states, empty states, selection, and future virtualization need more Atlas code or another table package. |
| Result paging                   | `Button` controls                                                    | `Button` controls                                                    | Medium for both                             | Backend result-window paging is product-specific, so neither library removes much code here.                                                                                                                                |
| Trait chips                     | `Tag`                                                                | `Badge`                                                              | Low                                         | Both are native visual primitives.                                                                                                                                                                                          |
| Record detail                   | Shared `RecordPresentation`                                          | Shared `RecordPresentation`                                          | Same for both                               | Record presentation is Atlas product behavior, not a component-library decision.                                                                                                                                            |
| Diagnostics panel               | Shared shell diagnostics                                             | Shared shell diagnostics                                             | Same for both                               | Diagnostics are implementation instrumentation and should not affect library selection.                                                                                                                                     |
| Theme integration               | AntD theme tokens                                                    | Mantine theme tokens                                                 | Medium for both                             | Evaluate how much of the shared Atlas theme reaches native component states without per-component overrides.                                                                                                                |
| Future keyboard row navigation  | Likely table row handlers plus AntD table affordances                | Likely explicit row state and handlers                               | Unknown                                     | This should be tested once the product needs keyboard-first result navigation.                                                                                                                                              |
| Future advanced filters         | Native form/select/input primitives plus custom filter-builder state | Native form/select/input primitives plus custom filter-builder state | Unknown                                     | The full filter editor is Atlas-specific. The library comparison should focus on whether each library reduces accessible composition work.                                                                                  |

## Review Guidance

When comparing the two prototypes, separate these questions:

- Component capability: Does the library provide the needed primitive directly?
- Product fit: Does the native primitive match Atlas workflows without fighting it?
- Theme fit: Can shared Atlas tokens produce the desired design language without brittle overrides?
- Accessibility and keyboard behavior: Does the native component already handle expected focus, menu, table, and input behavior?
- Custom code burden: How much Atlas-specific glue remains after using the library's intended primitives?

## Current Direction

Ant Design is the provisional default after the expanded-filter slice. The AntD version now uses native `Form`, `Collapse`, `Input.Search`, `Select`, `InputNumber`, `Checkbox`, `Empty`, `Alert`, and `Table` components for the main search workflow. Mantine remains useful as a comparison target, but the current implementation shows more Atlas-owned composition in Mantine for dense search-console behavior, especially sectioned filter forms and result-table behavior.

Before removing Mantine, evaluate one more component-heavy pass only if a near-term feature stresses components AntD has not yet proven: advanced filter editing, keyboard-first table navigation, drawer/modal detail navigation, or virtualized results. If the next slice continues to favor AntD, remove Mantine and convert this document into a short implementation decision note.
