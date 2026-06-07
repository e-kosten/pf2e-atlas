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

| Requirement | Ant Design path | Mantine path | Custom composition cost | Notes |
| --- | --- | --- | --- | --- |
| Text search | `Input.Search` | `TextInput` with `leftSection` icon | Low | AntD has a native search affordance with clear/search button behavior. Mantine keeps the input simpler and composes the icon. |
| Multi-value filters | `Select mode="multiple"` | `MultiSelect` | Low | Compare keyboard behavior, tag overflow, searchable menus, and dense layout behavior. |
| Numeric range filters | `InputNumber` | `NumberInput` | Low | Both have native numeric inputs. Atlas owns range semantics. |
| Boolean/toggle filters | `Checkbox` | `Checkbox` | Low | Both are native. |
| Sort selector | `Select` | `Select` | Low | Both are native; Atlas owns sort DTO mapping. |
| Result table | `Table` with column config, loading, row key | `Table` inside `ScrollArea` with explicit rows | Medium for Mantine | AntD has the more complete data-table primitive. Mantine exposes table building blocks, so row rendering, loading states, empty states, selection, and future virtualization need more Atlas code or another table package. |
| Result paging | `Button` controls | `Button` controls | Medium for both | Backend result-window paging is product-specific, so neither library removes much code here. |
| Trait chips | `Tag` | `Badge` | Low | Both are native visual primitives. |
| Record detail | Shared `RecordPresentation` | Shared `RecordPresentation` | Same for both | Record presentation is Atlas product behavior, not a component-library decision. |
| Diagnostics panel | Shared shell diagnostics | Shared shell diagnostics | Same for both | Diagnostics are implementation instrumentation and should not affect library selection. |
| Theme integration | AntD theme tokens | Mantine theme tokens | Medium for both | Evaluate how much of the shared Atlas theme reaches native component states without per-component overrides. |
| Future keyboard row navigation | Likely table row handlers plus AntD table affordances | Likely explicit row state and handlers | Unknown | This should be tested once the product needs keyboard-first result navigation. |
| Future advanced filters | Native form/select/input primitives plus custom filter-builder state | Native form/select/input primitives plus custom filter-builder state | Unknown | The full filter editor is Atlas-specific. The library comparison should focus on whether each library reduces accessible composition work. |

## Review Guidance

When comparing the two prototypes, separate these questions:

- Component capability: Does the library provide the needed primitive directly?
- Product fit: Does the native primitive match Atlas workflows without fighting it?
- Theme fit: Can shared Atlas tokens produce the desired design language without brittle overrides?
- Accessibility and keyboard behavior: Does the native component already handle expected focus, menu, table, and input behavior?
- Custom code burden: How much Atlas-specific glue remains after using the library's intended primitives?
