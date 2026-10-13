import { fireEvent, render, screen } from "@testing-library/react";
import { summaryFixture } from "../../test/fixtures";
import type { SearchResultsPage } from "../../generated/atlas";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { SearchView } from "./SearchView";

describe("search pane composition", () => {
  it("keeps the Similar criterion and filters in one body, and clears only the criterion", () => {
    const setSearch = vi.fn();
    const state = workspace({
      setSearch,
      search: {
        ...DEFAULT_SEARCH_STATE,
        similarSeed: "items:seed",
        filter: { kind: "all_of", clause_id: "root", children: [] },
      },
      resultPage: {
        ...searchResultsPage(),
        mode: { kind: "similar_records", seed: summaryFixture("items:seed", "Seed") },
      },
    });
    const { container, rerender } = render(<SearchView workspace={state} />);
    const criterion = screen.getByText("Similar to Seed");
    const body = criterion.closest(".filter-toolbar");
    expect(body).toBeVisible();
    expect(body).toContainElement(
      screen.getByRole("searchbox", { name: "Search records" }),
    );
    expect(screen.getByText("Top matches")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Clear similar criterion" }));
    expect(setSearch).toHaveBeenCalledWith({
      ...state.search,
      similarSeed: null,
      mode: "browse",
    });

    rerender(<SearchView workspace={workspace()} />);
    expect(screen.queryByText("Similar to Seed")).toBeNull();
    expect(screen.getByText("14,736 records")).toBeVisible();
    expect(container.querySelector(".filter-toolbar")).toBeVisible();
  });
});

describe("ResultPaneHeader", () => {
  it("labels bounded similarity as Top matches without an exhaustive record total", () => {
    render(
      <ResultPaneHeader
        workspace={workspace({
          resultPage: {
            ...searchResultsPage(),
            mode: { kind: "similar_records", seed: summaryFixture("items:seed") },
            coverage: {
              exhaustive: false,
              semantic_unit_window: 4096,
              candidate_roots: 10,
              count_basis: "bounded candidates",
            },
          },
        })}
      />,
    );
    expect(screen.getByText("Top matches")).toBeVisible();
    expect(screen.queryByText(/candidate records|Bounded semantic/)).toBeNull();
    expect(screen.getByRole("button", { name: "Next page" })).toBeEnabled();
  });

  it("lets users jump directly to a page", () => {
    const setPageNumber = vi.fn();
    render(<ResultPaneHeader workspace={workspace({ setPageNumber })} />);

    const input = screen.getByRole("textbox", { name: "Current page" });
    fireEvent.change(input, { target: { value: "42" } });
    fireEvent.blur(input);

    expect(setPageNumber).toHaveBeenCalledWith(42);
  });

  it.each(["resultsPaused", "resultsRecovering"] as const)(
    "disables pagination while %s without changing the criterion",
    (state) => {
      const setPageNumber = vi.fn();
      render(
        <ResultPaneHeader workspace={workspace({ [state]: true, setPageNumber })} />,
      );
      expect(screen.getByRole("button", { name: "Previous page" })).toBeDisabled();
      expect(screen.getByRole("button", { name: "Next page" })).toBeDisabled();
      expect(screen.getByRole("textbox", { name: "Current page" })).toBeDisabled();
      expect(screen.getByRole("combobox", { name: "Records per page" })).toBeDisabled();
      fireEvent.click(screen.getByRole("button", { name: "Next page" }));
      expect(setPageNumber).not.toHaveBeenCalled();
    },
  );

  it("clamps page jumps to the known page count", () => {
    const setPageNumber = vi.fn();
    render(<ResultPaneHeader workspace={workspace({ setPageNumber })} />);

    const input = screen.getByRole("textbox", { name: "Current page" });
    fireEvent.change(input, { target: { value: "999" } });
    fireEvent.blur(input);

    expect(setPageNumber).toHaveBeenCalledWith(590);
    expect(input).toHaveValue("590");
  });
});

function workspace(
  overrides: Partial<SearchWorkspaceState> = {},
): SearchWorkspaceState {
  return {
    search: DEFAULT_SEARCH_STATE,
    setSearch: vi.fn(),
    selectedRecordKey: null,
    selectRecord: vi.fn(),
    pageNumber: 13,
    setPageNumber: vi.fn(),
    resultPage: searchResultsPage(),
    reading: {
      request: null,
      inspect: vi.fn(),
      detail: undefined,
      landing: undefined,
      error: null,
      loading: false,
      retry: vi.fn(),
    },
    positions: {
      readList: () => 0,
      saveList: vi.fn(),
      readReader: () => 0,
      saveReader: vi.fn(),
    },
    filterEditor: undefined,
    filterDiscoveryContext: {
      kind: "filtered",
      filter: null,
      text: null,
      mode: "hybrid",
    },
    vocabularyScope: { kind: "artifact" },
    filterDiscoveryError: null,
    resultsLoading: false,
    resultsRefreshing: false,
    filterDiscoveryLoading: false,
    errorMessage: null,
    refresh: vi.fn(),
    refreshResults: vi.fn(),
    ...overrides,
  };
}

function searchResultsPage(): SearchResultsPage {
  return {
    artifact_generation: "generation-a",
    mode: { kind: "text_search", query: "", mode: "hybrid" },
    page: {
      number: 13,
      size: 25,
      count: 25,
      total: 14_736n,
      has_more: true,
      next_page: 14,
    },
    rows: [],
    coverage: null,
  };
}
