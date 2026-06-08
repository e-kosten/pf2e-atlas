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

  it("starts with no active filters and disables the global clear action", () => {
    render(<AntFilters workspace={workspace()} />);

    expect(
      screen.getByRole("button", { name: /clear search and filters/i }),
    ).toBeDisabled();
  });

  it("enables the global clear action for search text without filters", () => {
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            query: "doom",
            mode: "text_search",
          },
        })}
      />,
    );

    expect(
      screen.getByRole("button", { name: /clear search and filters/i }),
    ).toBeEnabled();
  });

  it("clears search and all filters without changing result options", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            query: "doom",
            mode: "text_search",
            sort: "alphabetical",
            pageSize: 50,
            visibleFilterIds: ["publication_remaster"],
            hiddenFilterIds: ["pack"],
            filterClauses: [
              {
                id: "kind-include_any",
                field: "kind",
                operator: "include_any",
                values: ["spell"],
              },
              {
                id: "level-range",
                field: "level",
                operator: "range",
                values: [],
                range: { min: 2 },
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /clear search and filters/i }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        query: "",
        mode: "browse",
        sort: "alphabetical",
        pageSize: 50,
        visibleFilterIds: [],
        hiddenFilterIds: [],
        filterClauses: [],
      }),
    );
  });

  it("clears a standard field without removing unrelated filter clauses", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [
              {
                id: "kind-include_any",
                field: "kind",
                operator: "include_any",
                values: ["spell"],
              },
              {
                id: "level-range",
                field: "level",
                operator: "range",
                values: [],
                range: { min: 2 },
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /clear kinds filter/i }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: expect.arrayContaining([
          expect.objectContaining({
            field: "level",
            operator: "range",
          }),
        ]),
      }),
    );
    expect(lastSearch(setSearch).filterClauses).not.toContainEqual(
      expect.objectContaining({ field: "kind" }),
    );
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

  it("selects neutral option rows as included values", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    fireEvent.click(screen.getByRole("button", { name: "spell (1)" }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: [
          expect.objectContaining({
            field: "kind",
            operator: "include_any",
            values: ["spell"],
          }),
        ],
      }),
    );
  });

  it("cycles included option rows to excluded values", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [
              {
                id: "kind-include_any",
                field: "kind",
                operator: "include_any",
                values: ["spell", "feat"],
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    fireEvent.click(screen.getByRole("button", { name: "spell (1)" }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: expect.arrayContaining([
          expect.objectContaining({
            field: "kind",
            operator: "exclude_any",
            values: ["spell"],
          }),
        ]),
      }),
    );
    expect(lastSearch(setSearch).filterClauses).toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "include_any",
        values: ["feat"],
      }),
    );
    expect(lastSearch(setSearch).filterClauses).not.toContainEqual(
      expect.objectContaining({
        field: "kind",
        operator: "include_any",
        values: expect.arrayContaining(["spell"]),
      }),
    );
  });

  it("cycles excluded option rows back to neutral", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [
              {
                id: "kind-exclude_any",
                field: "kind",
                operator: "exclude_any",
                values: ["spell"],
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    fireEvent.click(screen.getByRole("button", { name: "spell (1)" }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: [],
      }),
    );
  });

  it("clears included chips without advancing them to excluded", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [
              {
                id: "kind-include_any",
                field: "kind",
                operator: "include_any",
                values: ["spell"],
              },
            ],
          },
          setSearch,
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /remove spell \(1\) filter/i }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: [],
      }),
    );
  });

  it("does not cycle option rows to exclude when the field disallows exclusion", () => {
    const setSearch = vi.fn();
    render(
      <AntFilters
        workspace={workspace({
          search: {
            ...DEFAULT_SEARCH_STATE,
            filterClauses: [
              {
                id: "kind-include_any",
                field: "kind",
                operator: "include_any",
                values: ["spell"],
              },
            ],
          },
          setSearch,
          filterEditor: includeOnlyKindEditor(),
        })}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    fireEvent.click(screen.getByRole("button", { name: "spell (1)" }));

    expect(setSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterClauses: [],
      }),
    );
  });

  it("closes option pickers when clicking outside the control", async () => {
    render(<AntFilters workspace={workspace()} />);

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    expect(await screen.findByRole("button", { name: "spell (1)" })).toBeVisible();

    fireEvent.click(document.body);

    expect(screen.queryByRole("button", { name: "spell (1)" })).not.toBeInTheDocument();
  });

  it("keeps an open option picker stable while refreshed filter values narrow", async () => {
    const { rerender } = render(<AntFilters workspace={workspace()} />);

    fireEvent.click(screen.getByRole("button", { name: /edit kinds filter/i }));
    expect(await screen.findByRole("button", { name: "spell (1)" })).toBeVisible();

    rerender(
      <AntFilters
        workspace={workspace({
          filterValuesByField: {
            kind: {
              field_id: "kind",
              matching_record_count: 1n,
              options: [],
            },
          },
        })}
      />,
    );

    expect(screen.getByRole("button", { name: "spell (1)" })).toBeVisible();
  });
});

function workspace(
  overrides: {
    search?: SearchFormState;
    setSearch?: (next: SearchFormState) => void;
    filterEditor?: FilterEditorView;
    filterValuesByField?: Partial<AtlasWorkspaceState["filterValuesByField"]>;
  } = {},
): AtlasWorkspaceState {
  return {
    search: overrides.search ?? DEFAULT_SEARCH_STATE,
    setSearch: overrides.setSearch ?? vi.fn(),
    filterEditor: overrides.filterEditor ?? editor(),
    filterValuesByField: {
      kind: {
        field_id: "kind",
        matching_record_count: 2n,
        options: [
          {
            value: "spell",
            label: "spell",
            count: 1n,
            selected: false,
            disabled: false,
            status: "available",
          },
        ],
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
      ...overrides.filterValuesByField,
    },
    filterDiscoveryLoading: false,
    errorMessage: null,
  } as unknown as AtlasWorkspaceState;
}

function lastSearch(setSearch: ReturnType<typeof vi.fn>): SearchFormState {
  return setSearch.mock.calls[setSearch.mock.calls.length - 1][0] as SearchFormState;
}

function includeOnlyKindEditor(): FilterEditorView {
  const currentEditor = editor();
  return {
    ...currentEditor,
    groups: currentEditor.groups.map((group) => ({
      ...group,
      fields: group.fields.map((field) =>
        field.id === "kind"
          ? {
              ...field,
              allowed_operators: ["include_any"],
              default_operator: "include_any",
            }
          : field,
      ),
    })),
  };
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
            allowed_operators: ["include_any", "exclude_any"],
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
            allowed_operators: ["include_any", "exclude_any"],
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
