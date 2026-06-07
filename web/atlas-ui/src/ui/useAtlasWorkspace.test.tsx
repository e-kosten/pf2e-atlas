import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  FilterValueListView,
  OpenResultWindowRequest,
  ResultWindowPage,
} from "../generated/atlas";
import { DEFAULT_SEARCH_STATE, encodeSearchState } from "../state/searchState";
import { useAtlasWorkspace } from "./useAtlasWorkspace";

const apiMocks = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  openResultWindow: vi.fn(),
  readResultWindowPage: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
  discoverFilterEditor: apiMocks.discoverFilterEditor,
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
    apiMocks.discoverFilterEditor.mockResolvedValue({
      matching_record_count: 0n,
      groups: [],
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

  it("discovers values for count-backed visible editor fields", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
        expect.objectContaining({ field_id: "kind" }),
      ),
    );
    expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "pack" }),
    );
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "level" }),
    );
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "basic_save" }),
    );

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        hiddenFilterIds: ["pack"],
        visibleFilterIds: ["basic_save"],
      }),
    );

    await waitFor(() =>
      expect(apiMocks.discoverFilterValues).toHaveBeenCalledWith(
        expect.objectContaining({ field_id: "basic_save" }),
      ),
    );
  });

  it("passes selected visible fields to editor discovery", async () => {
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        visibleFilterIds: ["publication_title"],
      }),
    );

    await waitFor(() =>
      expect(apiMocks.discoverFilterEditor).toHaveBeenCalledWith(
        expect.objectContaining({
          selected_field_ids: ["publication_title"],
        }),
      ),
    );
  });

  it("keeps previous filter editor data while refreshed discovery is pending", async () => {
    const nextEditor = deferred<FilterEditorView>();
    apiMocks.discoverFilterEditor
      .mockResolvedValueOnce(filterEditor())
      .mockReturnValueOnce(nextEditor.promise);
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(result.current.filterEditor).toEqual(filterEditor()));

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filterClauses: [
          {
            id: "kind-include_any",
            field: "kind",
            operator: "include_any",
            values: ["creature"],
          },
        ],
      }),
    );

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2));
    expect(result.current.filterEditor).toEqual(filterEditor());
    expect(result.current.filterDiscoveryLoading).toBe(true);

    await act(async () => {
      nextEditor.resolve({
        matching_record_count: 1n,
        groups: [],
      });
      await nextEditor.promise;
    });

    await waitFor(() => expect(result.current.filterEditor?.groups).toEqual([]));
  });

  it("keeps previous field values while refreshed value discovery is pending", async () => {
    const nextKindValues = deferred<FilterValueListView>();
    let kindValueRequests = 0;
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    apiMocks.discoverFilterValues.mockImplementation(
      (request: { field_id: string }) => {
        if (request.field_id !== "kind") {
          return Promise.resolve(emptyValues(request.field_id));
        }
        kindValueRequests += 1;
        return kindValueRequests === 1
          ? Promise.resolve(kindValues(["spell", "creature"]))
          : nextKindValues.promise;
      },
    );
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(result.current.filterValuesByField.kind?.options).toHaveLength(2),
    );

    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filterClauses: [
          {
            id: "kind-include_any",
            field: "kind",
            operator: "include_any",
            values: ["creature"],
          },
        ],
      }),
    );

    await waitFor(() => expect(kindValueRequests).toBe(2));
    expect(result.current.filterValuesByField.kind?.options).toHaveLength(2);
    expect(result.current.filterDiscoveryLoading).toBe(true);

    await act(async () => {
      nextKindValues.resolve(kindValues(["spell", "creature", "feat"]));
      await nextKindValues.promise;
    });

    await waitFor(() =>
      expect(result.current.filterValuesByField.kind?.options).toHaveLength(3),
    );
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

function filterEditor(): FilterEditorView {
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
            id: "basic_save",
            label: "Basic Save",
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
        ],
      },
    ],
  };
}

function kindValues(values: string[]): FilterValueListView {
  return {
    field_id: "kind",
    matching_record_count: BigInt(values.length),
    options: values.map((value) => ({
      value,
      label: value,
      count: 1n,
      selected: false,
      disabled: false,
      status: "available",
    })),
  };
}

function emptyValues(fieldId: string): FilterValueListView {
  return {
    field_id: fieldId,
    matching_record_count: 0n,
    options: [],
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
