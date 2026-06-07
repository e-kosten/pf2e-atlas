import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type { OpenResultWindowRequest, ResultWindowPage } from "../generated/atlas";
import { DEFAULT_SEARCH_STATE } from "../state/searchState";
import { useAtlasWorkspace } from "./useAtlasWorkspace";

const apiMocks = vi.hoisted(() => ({
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  openResultWindow: vi.fn(),
  readResultWindowPage: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
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
    apiMocks.openResultWindow.mockResolvedValue(resultWindowPage());
    history.replaceState(null, "", "/");
  });

  it("debounces result-window requests while preserving immediate search state", async () => {
    const { result } = renderHook(() => useAtlasWorkspace(), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1),
    );

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

    await waitFor(() =>
      expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(2),
    );
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
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

function resultWindowPage(): ResultWindowPage {
  return {
    window_id: 1n,
    mode: { kind: "text_search", query: "" },
    page: {
      number: 1,
      size: 25,
      count: 0,
      total: 0n,
      has_more: false,
    },
    rows: [],
  };
}

function delay(milliseconds: number) {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}
