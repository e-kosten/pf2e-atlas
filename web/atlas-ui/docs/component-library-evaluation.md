# Component Library Decision

Ant Design is the selected component library for the Atlas web prototype.

The comparison started with parallel Ant Design and Mantine implementations over the same Atlas workflow. The implementation used shared API/client/state code, shared generated DTOs, shared URL/result-window behavior, and shared record detail rendering so the library comparison focused on component fit rather than duplicated product behavior.

## Decision

Use Ant Design for the V1 web UI. Remove the Mantine parallel implementation and the component-library selector.

AntD better fits the current product shape: a dense local search console with dynamic filters, table-heavy result browsing, loading/error/empty states, and future advanced form workflows. The AntD path uses native `Form`, `Collapse`, `Input.Search`, `Select`, `InputNumber`, `Checkbox`, `Empty`, `Alert`, and `Table` components for the core workflow. Mantine remained pleasant and composable, but the prototype showed more Atlas-owned composition for sectioned filter forms and result-table behavior.

## Retained Evaluation Notes

| Requirement           | Ant Design path                                         | Former Mantine path                              | Decision signal                                                                |
| --------------------- | ------------------------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------ |
| Text search           | `Form.Item` plus `Input.Search`                         | `TextInput` with composed icon                   | AntD supplies the native search affordance.                                    |
| Multi-value filters   | `Form.Item` plus `Select mode="multiple"`               | `MultiSelect`                                    | Both are viable. AntD integrates more naturally with the selected form layout. |
| Numeric range filters | `InputNumber`                                           | `NumberInput`                                    | Both are viable; Atlas owns range semantics.                                   |
| Boolean filters       | `Checkbox`                                              | `Checkbox`                                       | Both are viable.                                                               |
| Dense filter grouping | `Collapse` with vertical `Form`                         | Stacked inputs or explicit accordion composition | AntD reduced custom layout code for the expanded-filter slice.                 |
| Add-filter states     | `Select` loading plus `Empty` not-found content         | Plain select state                               | AntD represented dynamic filter discovery states with less custom code.        |
| Result table          | `Table` with columns, loading, row key, row class hooks | Table primitives inside scroll area              | AntD has the stronger data-table primitive for Atlas result browsing.          |
| Record detail         | Shared Atlas renderer                                   | Shared Atlas renderer                            | Not a component-library differentiator.                                        |
| Theme integration     | Shared Atlas tokens adapted to AntD theme config        | Shared tokens adapted to Mantine provider        | Both needed adapters; AntD is now the only supported adapter.                  |

## Follow-Up

Keep AntD as the default path while tightening the UI architecture.

The first state cleanup removed effect-driven repair logic for result focus and detail-pane expansion. Active result selection is now derived from the current result rows plus the user's focused-row preference. Detail collapse state records the selected record key it was collapsed for, so selecting a different record expands detail without a follow-up effect. This keeps interaction transitions explicit enough to enable `react-hooks/set-state-in-effect`.

Future UI state work should keep this direction: TanStack Query owns server/cache state, URL/search state remains durable and shareable, and local workspace interaction state should be either event-driven or derived from current data instead of synchronized by corrective effects.
