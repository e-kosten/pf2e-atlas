import { fireEvent, render, screen } from "@testing-library/react";
import type { ResultWindowPage } from "../../generated/atlas";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { ResultPaneHeader } from "./ResultPaneHeader";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

describe("ResultPaneHeader", () => {
  it("lets users jump directly to a page", () => {
    const setPageNumber = vi.fn();
    render(<ResultPaneHeader workspace={workspace({ setPageNumber })} />);

    const input = screen.getByRole("textbox", { name: "Current page" });
    fireEvent.change(input, { target: { value: "42" } });
    fireEvent.blur(input);

    expect(setPageNumber).toHaveBeenCalledWith(42);
  });

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
    activeResultKey: null,
    focusResult: vi.fn(),
    moveResultSelection: vi.fn(),
    openActiveResult: vi.fn(),
    selectedRecordKey: null,
    selectRecord: vi.fn(),
    pageNumber: 13,
    setPageNumber: vi.fn(),
    resultPage: resultWindowPage(),
    recordDetail: undefined,
    filterEditor: undefined,
    filterValuesByField: {},
    readiness: {} as SearchWorkspaceState["readiness"],
    resultsLoading: false,
    resultsRefreshing: false,
    detailLoading: false,
    detailRefreshing: false,
    detailError: null,
    filterDiscoveryLoading: false,
    diagnostics: {
      activeWindowId: "1",
      detailRequest: null,
      resultRequest: null,
      searchDebouncing: false,
    },
    errorMessage: null,
    refresh: vi.fn(),
    ...overrides,
  };
}

function resultWindowPage(): ResultWindowPage {
  return {
    window_id: 1n,
    mode: { kind: "text_search", query: "" },
    page: {
      number: 13,
      size: 25,
      count: 25,
      total: 14_736n,
      has_more: true,
      next_page: 14,
    },
    rows: [],
  };
}
