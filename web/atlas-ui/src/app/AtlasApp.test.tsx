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
  ResultWindowPage,
} from "../generated/atlas";
import {
  heroPointDeckRecordKey,
  recordDetailFixture as typedRecordDetailFixture,
  recordSummaryFixture,
  rollTableRecordDetailFixture,
  rollTableResultOneLocator,
} from "../test/recordFixtures";
import { AtlasApp } from "./AtlasApp";

const tenSecondTestDeadline = 10_000;

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getReadiness: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedLists: vi.fn(),
  openResultWindow: vi.fn(),
  readResultWindowPage: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
  addSavedListItem: apiMocks.addSavedListItem,
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedLists: apiMocks.getSavedLists,
  openResultWindow: apiMocks.openResultWindow,
  readResultWindowPage: apiMocks.readResultWindowPage,
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
    apiMocks.openResultWindow.mockResolvedValue(resultWindowPage());
    apiMocks.readResultWindowPage.mockResolvedValue(resultWindowPage());
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
    const addEventListener = vi.spyOn(window, "addEventListener");
    try {
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      await waitFor(() =>
        expect(addEventListener).toHaveBeenCalledWith("popstate", expect.any(Function)),
      );
    } finally {
      addEventListener.mockRestore();
    }

    await import("../features/records/RecordViews");
    await waitFor(() => expect(apiMocks.getReadiness).toHaveBeenCalledTimes(1));
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    vi.clearAllMocks();

    await act(async () => {
      history.pushState(null, "", "/records/spell%3Aheal");
      window.dispatchEvent(new PopStateEvent("popstate"));
    });

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "spell:heal",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      "spell:heal",
      undefined,
      expect.any(AbortSignal),
    );

    vi.clearAllMocks();
    await act(async () => {
      history.pushState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
      window.dispatchEvent(new PopStateEvent("popstate"));
    });

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "spell:linked",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });

  it(
    "restores a concise RollTable result selection through browser back and forward",
    async () => {
      const recordKey = heroPointDeckRecordKey;
      history.replaceState(null, "", `/records/${encodeURIComponent(recordKey)}`);
      apiMocks.getRecordDetail.mockResolvedValue(rollTableRecordDetailFixture());
      render(<AtlasApp />, { wrapper: queryClientWrapper() });

      fireEvent.click(await screen.findByRole("link", { name: "Open result 1" }));
      await waitFor(() =>
        expect(window.location.search).toBe(
          `?child=${encodeURIComponent(rollTableResultOneLocator)}`,
        ),
      );
      expect(await screen.findByText("Result 1 selected")).toHaveAttribute(
        "aria-current",
        "page",
      );
      expect(screen.getByRole("region", { name: "Selected result 1" })).toHaveFocus();
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        recordKey,
        { child_locator: rollTableResultOneLocator },
        expect.any(AbortSignal),
      );

      act(() => history.back());
      await waitFor(() => expect(window.location.search).toBe(""));
      expect(
        await screen.findByRole("link", { name: "Open result 1" }),
      ).toBeInTheDocument();

      act(() => history.forward());
      await waitFor(() =>
        expect(window.location.search).toBe(
          `?child=${encodeURIComponent(rollTableResultOneLocator)}`,
        ),
      );
      expect(await screen.findByText("Result 1 selected")).toHaveAttribute(
        "aria-current",
        "page",
      );
      expect(screen.getByRole("region", { name: "Selected result 1" })).toHaveFocus();
    },
    tenSecondTestDeadline,
  );

  it(
    "opens the search side-detail record as a full-page record route",
    async () => {
      history.replaceState(null, "", "/search?q=heal&mode=text");
      apiMocks.openResultWindow.mockResolvedValue(resultWindowPage(["spell:heal"]));
      render(<AtlasApp />, { wrapper: queryClientWrapper() });

      const resultRow = await screen.findByRole("button", { name: /heal/i });
      fireEvent.click(resultRow);
      expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

      vi.clearAllMocks();
      fireEvent.click(await screen.findByRole("link", { name: "Open full page" }));

      await waitFor(() =>
        expect(window.location.pathname).toBe("/records/spell%3Aheal"),
      );
      expect(window.location.search).toBe("");
      expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
      expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
      expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
      expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    },
    tenSecondTestDeadline,
  );

  it(
    "adds the search side-detail record to a saved list",
    async () => {
      history.replaceState(null, "", "/search?q=heal&mode=text");
      apiMocks.openResultWindow.mockResolvedValue(resultWindowPage(["spell:heal"]));
      render(<AtlasApp />, { wrapper: queryClientWrapper() });

      const resultRow = await waitFor(() => {
        const row = document.querySelector<HTMLElement>(".result-row");
        expect(row).not.toBeNull();
        return row!;
      });
      fireEvent.click(resultRow);
      expect(
        await screen.findByText("heal", { selector: "h1,h2,h3,h4,h5,h6" }),
      ).toBeInTheDocument();

      const addToList = await waitFor(() => {
        const button = document.querySelector<HTMLButtonElement>(
          'button[aria-label="Add to saved list"]',
        );
        expect(button).not.toBeNull();
        return button!;
      });
      fireEvent.click(addToList);
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
    },
    tenSecondTestDeadline,
  );
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
  return {
    matching_record_count: 0n,
    groups: [],
  };
}

function resultWindowPage(recordKeys: string[] = []): ResultWindowPage {
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
    rows: recordKeys.map((recordKey) => ({
      record: recordSummaryFixture(recordKey, recordKey.split(":")[1] ?? recordKey),
    })),
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
        item_count: 1n,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ],
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const title = recordKey.split(":")[1] ?? recordKey;
  return typedRecordDetailFixture({ recordKey, title });
}
