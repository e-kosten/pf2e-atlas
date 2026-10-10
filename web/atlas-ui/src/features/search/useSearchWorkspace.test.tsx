import {
  summaryFixture,
  editorFixture,
  fieldFixture,
  valuesFixture,
} from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  FilterValueListView,
  OpenResultWindowRequest,
  ResultWindowPage,
} from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
} from "../../shared/filters/searchState";
import { useSearchWorkspace } from "./useSearchWorkspace";

const apiMocks = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  openResultWindow: vi.fn(),
  readResultWindowPage: vi.fn(),
}));

vi.mock("../../api/atlasApi", () => ({
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  openResultWindow: apiMocks.openResultWindow,
  readResultWindowPage: apiMocks.readResultWindowPage,
}));

describe("useSearchWorkspace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    apiMocks.getReadiness.mockResolvedValue({
      status: "ready",
      message: "Ready",
    });
    apiMocks.discoverFilterEditor.mockResolvedValue({ ...editorFixture(), groups: [] });
    apiMocks.discoverFilterValues.mockResolvedValue(valuesFixture("record.kind"));
    apiMocks.openResultWindow.mockResolvedValue(resultWindowPage());
    apiMocks.readResultWindowPage.mockResolvedValue(resultWindowPage());
    history.replaceState(null, "", "/");
  });

  it("debounces result-window requests while preserving immediate search state", async () => {
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).toHaveBeenCalled();

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
    expect(window.location.pathname).toBe("/search");
    expect(window.location.search).toBe("?q=fi&mode=text_search");
    await delay(150);
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));
    expect(result.current.diagnostics.searchDebouncing).toBe(false);
    expect(result.current.diagnostics.resultRequest).toMatchObject({
      kind: "open_window",
    });
    const request = apiMocks.openResultWindow.mock
      .calls[0][0] as OpenResultWindowRequest;
    expect(request.mode).toMatchObject({
      kind: "text_search",
      query: "fi",
    });
  });

  it("can disable search workspace queries for non-search routes", async () => {
    const { result } = renderHook(() => useSearchWorkspace({ enabled: false }), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.getReadiness).toHaveBeenCalledTimes(1));
    act(() => result.current.refresh());
    await waitFor(() => expect(apiMocks.getReadiness).toHaveBeenCalledTimes(2));
    await delay(50);

    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    expect(result.current.resultsLoading).toBe(false);
  });

  it("tracks keyboard result selection separately from opened detail routes", async () => {
    seedSearchUrl();
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom", "spell:heal"]),
    );
    apiMocks.getRecordDetail.mockResolvedValue({ record_key: "spell:heal" });
    const { result } = renderHook(() => useSearchWorkspace(), {
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
    expect(window.location.pathname).toBe("/search/records/spell%3Aheal");
    expect(window.location.search).toBe("?q=dirge&mode=text_search");

    act(() => result.current.selectRecord(null));
    expect(result.current.selectedRecordKey).toBeNull();
    expect(window.location.pathname).toBe("/search");
    expect(window.location.search).toBe("?q=dirge&mode=text_search");
  });

  it("writes shared structured predicates in URL state", async () => {
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());

    act(() =>
      result.current.setSearch({
        ...DEFAULT_SEARCH_STATE,
        filter: {
          kind: "all_of",
          children: [
            {
              kind: "in",
              clause_id: "kind-include_any",
              field: "record.kind",
              values: ["affliction", "character"],
            },
            {
              kind: "not",
              predicate: {
                kind: "in",
                clause_id: "kind-exclude_any",
                field: "record.kind",
                values: ["character_option"],
              },
            },
          ],
        },
      }),
    );

    expect(window.location.pathname).toBe("/search");
    expect(window.location.search).toBe(
      "?filter=" + encodeURIComponent(JSON.stringify(result.current.search.filter)),
    );
  });

  it("discovers values for count-backed visible editor fields", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
        expect.objectContaining({ field_id: "record.kind" }),
      ),
    );
    expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "source.pack" }),
    );
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "actor.level" }),
    );
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "spell.basic_save" }),
    );

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        hiddenFilterIds: ["source.pack"],
        visibleFilterIds: ["spell.basic_save"],
      }),
    );

    await waitFor(() =>
      expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
        expect.objectContaining({ field_id: "spell.basic_save" }),
      ),
    );
  });

  it("does not query collection states as scalar values", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(
      editorFixture([fieldFixture("actor.items", "collection")]),
    );

    renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    await delay(50);

    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "common.traits" }),
    );
  });

  it("passes selected visible fields to editor discovery", async () => {
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        visibleFilterIds: ["common.publication.title"],
      }),
    );

    await waitFor(() =>
      expect(apiMocks.discoverFilterEditor).toHaveBeenCalledWith(
        expect.objectContaining({
          selected_field_ids: ["common.publication.title"],
        }),
      ),
    );
  });

  it("keeps previous filter editor data while refreshed discovery is pending", async () => {
    const nextEditor = deferred<FilterEditorView>();
    apiMocks.discoverFilterEditor
      .mockResolvedValueOnce(filterEditor())
      .mockReturnValueOnce(nextEditor.promise);
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(result.current.filterEditor).toEqual(filterEditor()));

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: {
          kind: "all_of",
          children: [
            {
              kind: "in",
              clause_id: "kind-include_any",
              field: "record.kind",
              values: ["creature"],
            },
          ],
        },
      }),
    );

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2));
    expect(result.current.filterEditor).toEqual(filterEditor());
    expect(result.current.filterDiscoveryLoading).toBe(true);

    await act(async () => {
      nextEditor.resolve({ ...editorFixture(), groups: [] });
      await nextEditor.promise;
    });

    await waitFor(() => expect(result.current.filterEditor?.groups).toEqual([]));
  });

  it("does not discover values from placeholder editor data during editor refresh", async () => {
    const nextEditor = deferred<FilterEditorView>();
    apiMocks.discoverFilterEditor
      .mockResolvedValueOnce(filterEditor())
      .mockReturnValueOnce(nextEditor.promise);
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(result.current.filterEditor).toEqual(filterEditor()));
    await waitFor(() => expect(apiMocks.discoverFilterValues).toHaveBeenCalled());
    const valueRequestsBeforeRefresh = apiMocks.discoverFilterValues.mock.calls.length;

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: {
          kind: "all_of",
          children: [
            {
              kind: "in",
              clause_id: "kind-include_any",
              field: "record.kind",
              values: ["creature"],
            },
          ],
        },
      }),
    );

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2));
    await delay(50);

    expect(apiMocks.discoverFilterValues).toHaveBeenCalledTimes(
      valueRequestsBeforeRefresh,
    );

    await act(async () => {
      nextEditor.resolve({ ...editorFixture(), groups: [] });
      await nextEditor.promise;
    });
  });

  it("keeps previous field values while refreshed value discovery is pending", async () => {
    const nextKindValues = deferred<FilterValueListView>();
    let kindValueRequests = 0;
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    apiMocks.discoverFilterValues.mockImplementation(
      (request: { field_id: string }) => {
        if (request.field_id !== "record.kind") {
          return Promise.resolve(emptyValues(request.field_id));
        }
        kindValueRequests += 1;
        return kindValueRequests === 1
          ? Promise.resolve(kindValues(["spell", "creature"]))
          : nextKindValues.promise;
      },
    );
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(
        result.current.filterValuesByField["record.kind"]?.values.options,
      ).toHaveLength(2),
    );

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: {
          kind: "all_of",
          children: [
            {
              kind: "in",
              clause_id: "kind-include_any",
              field: "record.kind",
              values: ["creature"],
            },
          ],
        },
      }),
    );

    await waitFor(() => expect(kindValueRequests).toBe(2));
    expect(
      result.current.filterValuesByField["record.kind:kind-include_any"]?.values
        .options,
    ).toHaveLength(2);
    expect(result.current.filterDiscoveryLoading).toBe(true);

    await act(async () => {
      nextKindValues.resolve(kindValues(["spell", "creature", "feat"]));
      await nextKindValues.promise;
    });

    await waitFor(() =>
      expect(
        result.current.filterValuesByField["record.kind:kind-include_any"]?.values
          .options,
      ).toHaveLength(3),
    );
  });

  it("reads later pages from the current result window", async () => {
    seedSearchUrl();
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockResolvedValue(
      resultWindowPage(["spell:heal"], { pageNumber: 2, windowId: 7n }),
    );
    const { result } = renderHook(() => useSearchWorkspace(), {
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
    seedSearchUrl();
    const nextPage = deferred<ResultWindowPage>();
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockReturnValue(nextPage.promise);
    const { result } = renderHook(() => useSearchWorkspace(), {
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
    seedSearchUrl();
    apiMocks.openResultWindow.mockResolvedValue(
      resultWindowPage(["spell:dirge-of-doom"], { windowId: 7n }),
    );
    apiMocks.readResultWindowPage.mockResolvedValue(
      resultWindowPage(["spell:heal"], { pageNumber: 2, windowId: 7n }),
    );
    const { result } = renderHook(() => useSearchWorkspace(), {
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
    seedSearchUrl();
    const restoredSearch = {
      ...DEFAULT_SEARCH_STATE,
      query: "acid",
      mode: "text_search" as const,
    };
    const { result } = renderHook(() => useSearchWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));

    history.pushState(
      null,
      "",
      `/search/records/spell%3Aacid-arrow${searchStateQueryString(restoredSearch)}`,
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

function seedSearchUrl(query = "dirge") {
  history.replaceState(
    null,
    "",
    `/search?q=${encodeURIComponent(query)}&mode=text_search`,
  );
}

function resultWindowPage(
  recordKeys: string[] = [],
  options: { pageNumber?: number; windowId?: bigint } = {},
): ResultWindowPage {
  return {
    window_id: options.windowId || 1n,
    mode: { kind: "text_search", query: "", mode: "hybrid" },
    page: {
      number: options.pageNumber || 1,
      size: 25,
      count: recordKeys.length,
      total: BigInt(recordKeys.length),
      has_more: false,
      next_page: null,
    },
    rows: recordKeys.map((key) => ({ record: summaryFixture(key), matches: [] })),
    coverage: null,
  };
}

function filterEditor(): FilterEditorView {
  const editor = editorFixture([
    fieldFixture("record.kind", "string", "Kind"),
    fieldFixture("common.traits", "set", "Traits"),
    fieldFixture("source.pack"),
    fieldFixture("actor.level", "number"),
    fieldFixture("spell.basic_save"),
  ]);
  editor.groups[0].fields[4].placement = "addable";
  return editor;
}

function kindValues(values: string[]): FilterValueListView {
  return valuesFixture("record.kind", values);
}

function emptyValues(fieldId: string): FilterValueListView {
  return valuesFixture(fieldId);
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
