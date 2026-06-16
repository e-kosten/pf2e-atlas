import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  RecordDetailView,
  ResultWindowPage,
} from "../generated/atlas";
import { AtlasPrototypeApp } from "./AtlasPrototypeApp";

const apiMocks = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getReadiness: vi.fn(),
  getRecordDetail: vi.fn(),
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

describe("AtlasPrototypeApp routing", () => {
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
  });

  it("restores record and reader views from browser history without running search queries", async () => {
    render(<AtlasPrototypeApp />, { wrapper: queryClientWrapper() });

    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));
    vi.clearAllMocks();

    history.pushState(null, "", "/records/spell%3Aheal");
    window.dispatchEvent(new PopStateEvent("popstate"));

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("spell:heal");

    vi.clearAllMocks();
    history.pushState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    window.dispatchEvent(new PopStateEvent("popstate"));

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });

  it("opens the search side-detail record as a full-page record route", async () => {
    apiMocks.openResultWindow.mockResolvedValue(resultWindowPage(["spell:heal"]));
    render(<AtlasPrototypeApp />, { wrapper: queryClientWrapper() });

    const resultKey = await screen.findByText("spell:heal");
    const resultButton = resultKey.closest("button");
    if (!resultButton) {
      throw new Error("result row button was not rendered");
    }
    fireEvent.click(resultButton);
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    vi.clearAllMocks();
    fireEvent.click(screen.getByRole("link", { name: "Open full page" }));

    await waitFor(() => expect(window.location.pathname).toBe("/records/spell%3Aheal"));
    expect(window.location.search).toBe("");
    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });
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
      record: {
        record_key: recordKey,
        title: recordKey.split(":")[1] ?? recordKey,
        kind: "spell",
        kind_label: "Spell",
      },
    })),
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const title = recordKey.split(":")[1] ?? recordKey;
  return {
    record_key: recordKey,
    title,
    kind: "spell",
    presentation: {
      record_key: recordKey,
      kind: "spell",
      title,
      identity: [],
      badges: [],
      sections: [],
    },
  };
}
