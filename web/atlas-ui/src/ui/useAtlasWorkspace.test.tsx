import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type { OpenResultWindowRequest, ResultWindowPage } from "../generated/atlas";
import { DEFAULT_SEARCH_STATE, encodeSearchState } from "../state/searchState";
import { useAtlasWorkspace } from "./useAtlasWorkspace";

const apiMocks = vi.hoisted(() => ({
  discoverFilterFields: vi.fn(),
  discoverFilterValues: vi.fn(),
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  openResultWindow: vi.fn(),
  readResultWindowPage: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
  discoverFilterFields: apiMocks.discoverFilterFields,
  discoverFilterValues: apiMocks.discoverFilterValues,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  openResultWindow: apiMocks.openResultWindow,
  readResultWindowPage: apiMocks.readResultWindowPage,
}));

describe("useAtlasWorkspace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    apiMocks.getReadiness.mockResolvedValue({
      status: "ready",
      message: "Ready",
    });
    apiMocks.discoverFilterFields.mockResolvedValue({
      matching_record_count: 0n,
      fields: [],
    });
    apiMocks.discoverFilterValues.mockResolvedValue({
      field_id: "kind",
      matching_record_count: 0n,
      options: [],
    });
    apiMocks.openResultWindow.mockResolvedValue(resultWindowPage());
    apiMocks.readResultWindowPage.mockResolvedValue(resultWindowPage());
    history.replaceState(null, "", "/");
  });

  it("debounces result-window requests while preserving immediate search state", async () => {
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));
    expect(apiMocks.discoverFilterFields).toHaveBeenCalled();

    act(() => {
      result.current.setSearch({
        ...DEFAULT_SEARCH_STATE,
        query: "f",
        mode: "text_search",
      });
      result.current.setSearch({
        ...DEFAULT_SEARCH_STATE,
        query: "fi",
        mode: "text_search",
      });
    });

    expect(result.current.search.query).toBe("fi");
    expect(result.current.diagnostics.searchDebouncing).toBe(true);
    expect(window.location.search).toContain("fi");
    await delay(150);
    expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1);

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(2));
    expect(result.current.diagnostics.searchDebouncing).toBe(false);
    expect(result.current.diagnostics.resultRequest).toMatchObject({
      kind: "open_window",
    });
    const request = apiMocks.openResultWindow.mock
      .calls[1][0] as OpenResultWindowRequest;
    expect(request.mode).toMatchObject({
      kind: "text_search",
      query: "fi",
    });
  });

  it("tracks keyboard result selection separately from opened detail routes", async () => {
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom", "spell:heal"]),
    );
    apiMocks.getRecordDetail.mockResolvedValue({ record_key: "spell:heal" });
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(result.current.activeResultKey).toBe("spell:dirge-of-doom"),
    );

    act(() => result.current.moveResultSelection("next"));
    expect(result.current.activeResultKey).toBe("spell:heal");
    expect(result.current.selectedRecordKey).toBeNull();

    act(() => result.current.openActiveResult());
    expect(result.current.selectedRecordKey).toBe("spell:heal");
    expect(window.location.pathname).toBe("/records/spell%3Aheal");

    act(() => result.current.selectRecord(null));
    expect(result.current.selectedRecordKey).toBeNull();
    expect(window.location.pathname).toBe("/");
  });

  it("reads later pages from the current result window", async () => {
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockResolvedValue(
      resultWindowPage(["spell:heal"], { pageNumber: 2, windowId: 7n }),
    );
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));

    act(() => result.current.setPageNumber(2));

    await waitFor(() =>
      expect(apiMocks.readResultWindowPage).toHaveBeenCalledWith(7n, {
        page: { number: 2, size: DEFAULT_SEARCH_STATE.pageSize },
      }),
    );
    expect(result.current.pageNumber).toBe(2);
    expect(result.current.diagnostics.resultRequest).toMatchObject({
      kind: "page",
    });
  });

  it("keeps previous rows while marking page transitions as refreshing", async () => {
    const nextPage = deferred<ResultWindowPage>();
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockReturnValue(nextPage.promise);
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(result.current.activeResultKey).toBe("spell:dirge-of-doom"),
    );

    act(() => result.current.setPageNumber(2));

    await waitFor(() => expect(result.current.resultsRefreshing).toBe(true));
    expect(result.current.resultsLoading).toBe(false);
    expect(result.current.resultPage?.rows[0]?.record.record_key).toBe(
      "spell:dirge-of-doom",
    );

    await act(async () => {
      nextPage.resolve(
        resultWindowPage(["spell:heal"], { pageNumber: 2, windowId: 7n }),
      );
      await nextPage.promise;
    });

    await waitFor(() => expect(result.current.resultsRefreshing).toBe(false));
    expect(result.current.resultPage?.page.number).toBe(2);
    expect(result.current.resultPage?.rows[0]?.record.record_key).toBe("spell:heal");
  });

  it("resets page execution when search changes from a later page", async () => {
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockResolvedValue(
      resultWindowPage(["spell:heal"], { pageNumber: 2, windowId: 7n }),
    );
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(apiMocks.readResultWindowPage).toHaveBeenCalledTimes(1));

    act(() =>
      result.current.setSearch({
        ...DEFAULT_SEARCH_STATE,
        query: "heal",
        mode: "text_search",
      }),
    );

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(2));
    expect(apiMocks.readResultWindowPage).toHaveBeenCalledTimes(1);
    expect(result.current.pageNumber).toBe(1);
    const request = apiMocks.openResultWindow.mock
      .calls[1][0] as OpenResultWindowRequest;
    expect(request.page).toEqual({
      number: 1,
      size: DEFAULT_SEARCH_STATE.pageSize,
    });
    expect(request.mode).toMatchObject({
      kind: "text_search",
      query: "heal",
    });
  });

  it("restores URL search and selected record without waiting for typing debounce", async () => {
    const restoredSearch = {
      ...DEFAULT_SEARCH_STATE,
      query: "acid",
      mode: "text_search" as const,
    };
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));

    history.pushState(
      null,
      "",
      `/records/spell%3Aacid-arrow?s=${encodeSearchState(restoredSearch)}`,
    );
    act(() => window.dispatchEvent(new PopStateEvent("popstate")));

    expect(result.current.search.query).toBe("acid");
    expect(result.current.selectedRecordKey).toBe("spell:acid-arrow");
    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(2));
    const request = apiMocks.openResultWindow.mock
      .calls[1][0] as OpenResultWindowRequest;
    expect(request.page).toEqual({
      number: 1,
      size: DEFAULT_SEARCH_STATE.pageSize,
    });
    expect(request.mode).toMatchObject({
      kind: "text_search",
      query: "acid",
    });
  });
});

function queryClientWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

function resultWindowPage(
  recordKeys: string[] = [],
  options: { pageNumber?: number; windowId?: bigint } = {},
): ResultWindowPage {
  return {
    window_id: options.windowId ?? 1n,
    mode: { kind: "text_search", query: "" },
    page: {
      number: options.pageNumber ?? 1,
      size: 25,
      count: 0,
      total: 0n,
      has_more: false,
    },
    rows: recordKeys.map((recordKey) => ({
      record: {
        record_key: recordKey,
        title: recordKey.split(":")[1] ?? recordKey,
        kind: "spell",
        kind_label: "Spell",
      },
    })),
  };
}

function delay(milliseconds: number) {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((innerResolve) => {
    resolve = innerResolve;
  });
  return { promise, resolve };
}
