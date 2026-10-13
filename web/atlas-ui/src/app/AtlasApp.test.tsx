import { detailFixture, summaryFixture, editorFixture } from "../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  RecordDetailView,
  SearchResultsPage,
} from "../generated/atlas";
import { AtlasApp } from "./AtlasApp";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getReadiness: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedLists: vi.fn(),
  searchPage: vi.fn(),
}));

vi.mock("../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../api/atlasApi")>()),
  addSavedListItem: apiMocks.addSavedListItem,
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedLists: apiMocks.getSavedLists,
  searchPage: apiMocks.searchPage,
}));

describe("AtlasApp routing", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installLocalStorage();
    Element.prototype.scrollIntoView = vi.fn();
    window.localStorage.clear();
    history.replaceState(null, "", "/search");
    apiMocks.getReadiness.mockResolvedValue({
      status: "ready",
      message: "Ready",
    });
    apiMocks.discoverFilterEditor.mockResolvedValue(emptyFilterEditor());
    apiMocks.discoverFilterValues.mockResolvedValue({
      field_id: "kind",
      matching_record_count: 0n,
      options: [],
    });
    apiMocks.searchPage.mockResolvedValue(searchResultsPage());
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    apiMocks.addSavedListItem.mockImplementation(
      ({ record_ref, list_ref }: { record_ref: string; list_ref: string }) =>
        Promise.resolve({
          list_key: list_ref,
          slug: "research",
          record_key: record_ref,
          record_name: "heal",
          outcome: "added",
        }),
    );
  });

  it("restores record and reader views from browser history without running search queries", async () => {
    render(<AtlasApp />, { wrapper: queryClientWrapper() });

    await waitFor(() => expect(apiMocks.getReadiness).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    vi.clearAllMocks();

    history.pushState(null, "", "/records/spell%3Aheal");
    fireEvent.popState(window);
    // Wait for the real lazy route import before starting the data/render assertion.
    // Cold module transforms on CI are independent of the mocked request latency.
    await act(() => vi.dynamicImportSettled());

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      "spell:heal",
      undefined,
      expect.any(AbortSignal),
    );

    vi.clearAllMocks();
    history.pushState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    fireEvent.popState(window);
    await act(() => vi.dynamicImportSettled());

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });

  it("opens the search side-detail record as a full-page record route", async () => {
    history.replaceState(null, "", "/search?q=heal&mode=text");
    apiMocks.searchPage.mockResolvedValue(searchResultsPage(["spell:heal"]));
    render(<AtlasApp />, { wrapper: queryClientWrapper() });

    const resultRow = await screen.findByRole("link", { name: /^heal/ });
    fireEvent.click(resultRow);
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    vi.clearAllMocks();
    fireEvent.click(screen.getByRole("link", { name: "Open full page" }));

    await waitFor(() => expect(window.location.pathname).toBe("/records/spell:heal"));
    expect(window.location.search).toBe("");
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });

  it("adds the search side-detail record to a saved list", async () => {
    history.replaceState(null, "", "/search?q=heal&mode=text");
    apiMocks.searchPage.mockResolvedValue(searchResultsPage(["spell:heal"]));
    render(<AtlasApp />, { wrapper: queryClientWrapper() });

    const resultRow = await screen.findByRole("link", { name: /^heal/ });
    fireEvent.click(resultRow);
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Add to saved list" }));
    const dialog = await screen.findByRole("dialog", { name: "Add to List" });
    const selector = within(dialog).getByRole("combobox");
    fireEvent.mouseDown(selector);
    fireEvent.click(await screen.findByText("Research"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
        list_ref: "research",
        record_ref: "spell:heal",
      }),
    );
  }, 10_000);
});

function installLocalStorage() {
  const store = new Map<string, string>();
  const storage = {
    clear: vi.fn(() => store.clear()),
    getItem: vi.fn((key: string) => store.get(key) ?? null),
    key: vi.fn((index: number) => Array.from(store.keys())[index] ?? null),
    removeItem: vi.fn((key: string) => store.delete(key)),
    setItem: vi.fn((key: string, value: string) => store.set(key, value)),
    get length() {
      return store.size;
    },
  };
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    value: storage,
  });
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: storage,
  });
}

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

function emptyFilterEditor(): FilterEditorView {
  return editorFixture();
}

function searchResultsPage(recordKeys: string[] = []): SearchResultsPage {
  return {
    artifact_generation: "generation-a",
    mode: { kind: "text_search", query: "", mode: "hybrid" },
    page: {
      number: 1,
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

function savedListIndexFixture() {
  return {
    lists: [
      {
        list_key: "list_research",
        slug: "research",
        name: "Research",
        description: "Campaign prep",
        tags: ["arc-one"],
        item_count: 1,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ],
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  return detailFixture(recordKey);
}
