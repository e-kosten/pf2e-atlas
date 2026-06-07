import { fireEvent, render, screen } from "@testing-library/react";
import type { FilterEditorView } from "../../generated/atlas";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../../state/searchState";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";
import { AntFilters } from "./AntFilters";

describe("AntFilters", () => {
  it("renders backend-provided default fields and range metadata", () => {
    render(<AntFilters workspace={workspace()} />);

    expect(screen.getByText("Kinds")).toBeInTheDocument();
    expect(screen.getByText("Pack")).toBeInTheDocument();
    expect(screen.getByText("Min level")).toBeInTheDocument();
    expect(screen.getByText("Max level")).toBeInTheDocument();

    const levelInputs = screen.getAllByRole("spinbutton").slice(0, 2);
    expect(levelInputs[0]).toHaveAttribute("aria-valuemin", "0");
    expect(levelInputs[0]).toHaveAttribute("aria-valuemax", "30");
    expect(levelInputs[0]).toHaveAttribute("step", "1");
    expect(levelInputs[1]).toHaveAttribute("aria-valuemin", "0");
    expect(levelInputs[1]).toHaveAttribute("aria-valuemax", "30");
    expect(levelInputs[1]).toHaveAttribute("step", "1");
  });

  it("renders added filters from backend labels and supports removal", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            visibleFilterIds: ["publication_remaster"],
            filterClauses: [
              ...DEFAULT_SEARCH_STATE.filterClauses,
              {
                id: "publication_remaster-include_any",
                field: "publication_remaster",
                operator: "include_any",
                values: ["true"],
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByText("Added filters (1)"));
    expect(screen.getByText("Remaster")).toBeVisible();

    fireEvent.click(screen.getByText("Remove"));
    expect(setSearch).toHaveBeenCalledWith(
      expect.objectContaining({
        visibleFilterIds: [],
        filterClauses: expect.not.arrayContaining([
          expect.objectContaining({ field: "publication_remaster" }),
        ]),
      }),
    );
  });

  it("renders metric comparison controls from the editor model", () => {
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            visibleFilterIds: ["metric"],
          },
        })}
      />,
    );

    fireEvent.click(screen.getByText("Added filters (1)"));
    expect(screen.getAllByText("Metric")).toHaveLength(2);
    expect(screen.getByText("Operator")).toBeVisible();
    expect(screen.getByText("Value")).toBeVisible();
  });
});

function workspace(
  overrides: {
    search?: SearchFormState;
    setSearch?: (next: SearchFormState) => void;
  } = {},
): AtlasWorkspaceState {
  return {
    search: overrides.search ?? DEFAULT_SEARCH_STATE,
    setSearch: overrides.setSearch ?? vi.fn(),
    filterEditor: editor(),
    filterValuesByField: {
      kind: {
        field_id: "kind",
        matching_record_count: 2n,
        options: [],
      },
      pack: {
        field_id: "pack",
        matching_record_count: 2n,
        options: [],
      },
      publication_remaster: {
        field_id: "publication_remaster",
        matching_record_count: 2n,
        options: [
          {
            value: "true",
            label: "Yes",
            count: 1n,
            selected: false,
            disabled: false,
            status: "available",
          },
        ],
      },
      metric: {
        field_id: "metric",
        matching_record_count: 2n,
        options: [
          {
            value: "spell.area.value",
            label: "Area",
            count: 2n,
            selected: false,
            disabled: false,
            status: "available",
          },
        ],
      },
    },
    filterDiscoveryLoading: false,
    errorMessage: null,
  } as unknown as AtlasWorkspaceState;
}

function editor(): FilterEditorView {
  return {
    matching_record_count: 4n,
    groups: [
      {
        id: "standard",
        label: "Standard",
        fields: [
          {
            id: "kind",
            label: "Kinds",
            control: { kind: "multi_select" },
            placement: "always_visible",
            applicability: "applicable",
            allowed_operators: ["include_any"],
            default_operator: "include_any",
            supports_counts: true,
          },
          {
            id: "level",
            label: "Level",
            control: {
              kind: "range",
              min_label: "Min",
              max_label: "Max",
              min: 0,
              max: 30,
              step: 1,
            },
            placement: "always_visible",
            applicability: "applicable",
            allowed_operators: ["range"],
            default_operator: "range",
            supports_counts: false,
          },
        ],
      },
      {
        id: "source",
        label: "Source",
        fields: [
          {
            id: "pack",
            label: "Pack",
            control: { kind: "multi_select" },
            placement: "initially_visible",
            applicability: "applicable",
            allowed_operators: ["include_any"],
            default_operator: "include_any",
            supports_counts: true,
          },
          {
            id: "publication_remaster",
            label: "Remaster",
            control: {
              kind: "boolean",
              true_label: "Yes",
              false_label: "No",
            },
            placement: "addable",
            applicability: "applicable",
            allowed_operators: ["include_any"],
            default_operator: "include_any",
            supports_counts: true,
          },
          {
            id: "metric",
            label: "Metric",
            control: {
              kind: "metric_comparison",
              key_label: "Metric",
              operator_label: "Operator",
              value_label: "Value",
            },
            placement: "addable",
            applicability: "applicable",
            allowed_operators: ["metric_compare"],
            default_operator: "metric_compare",
            supports_counts: true,
          },
        ],
      },
    ],
  };
}
