import { RouterTestContext } from "../../test/router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactElement, ReactNode } from "react";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import { summaryFixture } from "../../test/fixtures";
import type { SearchResultsPage } from "../../generated/atlas";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import { SearchView } from "./SearchView";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";

function render(ui: ReactElement) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return rtlRender(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <RouterTestContext>
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      </RouterTestContext>
    ),
  });
}

describe("search pane composition", () => {
  it("keeps the visible chosen model beside the search input with an accessible label", () => {
    render(
      <SearchProfileProvider>
        <SearchView workspace={workspace()} />
      </SearchProfileProvider>,
    );
    const input = screen.getByRole("searchbox", { name: "Search records" });
    const row = input.closest(".filter-controls__search-row");
    expect(row).toContainElement(
      screen.getByRole("combobox", { name: "Search model" }),
    );
    expect(within(row as HTMLElement).getByText("BGE Base")).toBeVisible();
    expect(screen.queryByText("Search model")).toBeNull();
  });

  it("distinguishes idle drafts, pending search, completed empty and local error without a zero pager", () => {
    const state = workspace({ resultPage: undefined });
    const content = (next: SearchWorkspaceState) => (
      <SearchProfileProvider>
        <SearchView workspace={next} />
      </SearchProfileProvider>
    );
    const { rerender } = render(content(state));
    expect(screen.getByText("Search or choose filters.")).toBeVisible();
    fireEvent.change(screen.getByRole("searchbox", { name: "Search records" }), {
      target: { value: "unsubmitted" },
    });
    expect(screen.getByText("Search or choose filters.")).toBeVisible();
    expect(state.setSearch).not.toHaveBeenCalled();
    expect(screen.queryByRole("textbox", { name: "Current page" })).toBeNull();

    const searching = {
      ...state,
      search: { ...state.search, query: "heal" },
      resultsLoading: true,
    };
    rerender(content(searching));
    expect(screen.getByRole("status", { name: "Searching…" })).toBeVisible();
    expect(screen.queryByText("No matching records.")).toBeNull();
    expect(screen.queryByRole("textbox", { name: "Current page" })).toBeNull();

    const empty = {
      ...searchResultsPage(),
      page: {
        ...searchResultsPage().page,
        number: 1,
        total: 0n,
        count: 0,
        has_more: false,
        next_page: null,
      },
    };
    rerender(content({ ...searching, resultsLoading: false, resultPage: empty }));
    expect(screen.getByText("No matching records.")).toBeVisible();
    expect(screen.queryByRole("status", { name: "Searching…" })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Records per page" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Next page" })).toBeNull();

    rerender(
      content({
        ...searching,
        resultsLoading: true,
        errorMessage: "Search connection lost",
      }),
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Search connection lost");
    expect(screen.queryByText("No matching records.")).toBeNull();
    expect(screen.queryByRole("status", { name: "Searching…" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(state.refresh).toHaveBeenCalledOnce();
  });

  it.each([
    { ...DEFAULT_SEARCH_STATE, similarSeed: "items:seed" },
    {
      ...DEFAULT_SEARCH_STATE,
      filter: { kind: "boolean_constant" as const, value: false },
    },
  ])("treats committed Similar or filter-only criteria as a search", (search) => {
    render(
      <SearchProfileProvider>
        <SearchView
          workspace={workspace({ search, resultPage: undefined, resultsLoading: true })}
        />
      </SearchProfileProvider>,
    );
    expect(screen.getByRole("status", { name: "Searching…" })).toBeVisible();
    expect(screen.queryByText("Search or choose filters.")).toBeNull();
  });

  it("keeps an invalid committed criterion distinct from idle guidance", () => {
    render(
      <SearchProfileProvider>
        <SearchView
          workspace={workspace({
            resultFilter: null,
            resultPage: undefined,
            search: { ...DEFAULT_SEARCH_STATE, filterError: "Invalid filter link" },
          })}
        />
      </SearchProfileProvider>,
    );
    expect(screen.queryByText("Search or choose filters.")).toBeNull();
    expect(screen.queryByText("No matching records.")).toBeNull();
    expect(screen.getByText("Invalid filter link")).toBeVisible();
  });
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
    const criterion = screen.getByText("Similar to");
    const body = criterion.closest(".filter-toolbar");
    expect(body).toBeVisible();
    expect(body).toContainElement(
      screen.getByRole("searchbox", { name: "Similar search" }),
    );
    expect(screen.getByRole("searchbox", { name: "Similar search" })).toHaveValue(
      "Seed",
    );
    expect(screen.getByRole("combobox", { name: "Search ranking" })).toBeDisabled();
    expect(screen.getByText("Top matches")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Clear similar criterion" }));
    expect(setSearch).toHaveBeenCalledWith({
      ...state.search,
      similarSeed: null,
      mode: "browse",
    });

    rerender(<SearchView workspace={workspace()} />);
    expect(screen.queryByText("Similar to")).toBeNull();
    expect(screen.getByText("14,736 records")).toBeVisible();
    expect(container.querySelector(".filter-toolbar")).toBeVisible();
  });
});

describe("ResultPaneHeader", () => {
  it("distinguishes bounded search matches from exhaustive record totals", () => {
    const page = searchResultsPage();
    const { rerender } = render(
      <ResultPaneHeader
        workspace={workspace({
          resultPage: {
            ...page,
            coverage: {
              exhaustive: false,
              semantic_unit_window: 1024,
              candidate_roots: 664,
              count_basis: "bounded candidates",
            },
            page: { ...page.page, total: 664n },
          },
        })}
      />,
    );
    expect(screen.getByText("Top 664 matches")).toBeVisible();
    expect(screen.queryByText(/candidate records|1024 units/)).toBeNull();
    rerender(<ResultPaneHeader workspace={workspace()} />);
    expect(screen.getByText("14,736 records")).toBeVisible();
  });
  it.each([
    undefined,
    {
      ...searchResultsPage(),
      page: { ...searchResultsPage().page, number: 1, total: 0n, count: 0 },
    },
  ])("does not render pagination without a nonempty result page", (resultPage) => {
    render(<ResultPaneHeader workspace={workspace({ resultPage })} />);
    expect(screen.queryByRole("textbox", { name: "Current page" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Next page" })).toBeNull();
  });
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

  it("keeps page navigation together while sort and page-size controls can wrap separately", () => {
    render(<ResultPaneHeader workspace={workspace()} />);
    const navigation = screen.getByRole("group", { name: "Result pages" });
    expect(
      within(navigation).getByRole("button", { name: "Previous page" }),
    ).toBeInTheDocument();
    expect(
      within(navigation).getByRole("textbox", { name: "Current page" }),
    ).toBeInTheDocument();
    expect(
      within(navigation).getByRole("button", { name: "Next page" }),
    ).toBeInTheDocument();
    expect(within(navigation).queryByRole("combobox")).toBeNull();
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
    resultFilter: null,
    search: DEFAULT_SEARCH_STATE,
    setSearch: vi.fn(),
    recoverFilterUrl: vi.fn(),
    selectedRecordKey: null,
    selectRecord: vi.fn(),
    pageNumber: 13,
    setPageNumber: vi.fn(),
    resultPage: searchResultsPage(),
    reading: {
      changeRank: vi.fn(),
      request: null,
      inspect: vi.fn(),
      select: vi.fn(),
      canGoBack: false,
      restoringHistory: false,
      back: vi.fn(),
      detail: undefined,
      parentDetail: undefined,
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
      readReaderFocus: vi.fn(),
      saveReaderFocus: vi.fn(),
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
    relationship_clauses: [],
    sort: { kind: "relevance" } as const,
    ranking: "score" as const,
    search_profile: "bge-base",
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
