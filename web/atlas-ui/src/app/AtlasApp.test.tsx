import {
  detailFixture,
  summaryFixture,
  editorFixture,
  readinessFixture,
  fieldFixture,
} from "../test/fixtures";
import { Grid } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render as renderDom,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  RecordDetailView,
  SearchResultsPage,
  AppReadinessView,
  SearchResultsRequest,
  RecordDetailRequest,
} from "../generated/atlas";
import { encodeReadingAddress } from "./readingUrl";
import { selectedNavigation, parseSearchParams } from "./urlState";
import { AtlasApp } from "./AtlasApp";
import { atlasRouter } from "./router";
const routerSlot = vi.hoisted(() => ({
  current: null as import("./router").AtlasRouter | null,
}));
vi.mock("./router", () => ({
  get atlasRouter() {
    return routerSlot.current;
  },
}));
// Transform the real route graph once during module loading. Per-test hook
// deadlines measure fixture setup, not cold imports of the application routes.
const { createAtlasRouter } =
  await vi.importActual<typeof import("./router")>("./router");
import { SEARCH_PROFILE_PREFERENCE_KEY } from "../shared/search/SearchProfileContext";
import { encodeFilterUrl } from "../shared/filters/filterUrl";
import { COLOR_SCHEME_STORAGE_KEY } from "../shared/theme/atlasTheme";

const apiMocks = vi.hoisted(() => ({
  addEncounterRecordParticipant: vi.fn(),
  addSavedListItem: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getReadiness: vi.fn(),
  verifyBackendCompatibility: vi.fn(),
  getRecordDetail: vi.fn(),
  getRelatedRecords: vi.fn(),
  getEncounters: vi.fn(),
  getSavedLists: vi.fn(),
  filterSavedList: vi.fn(),
  searchPage: vi.fn(),
}));

vi.mock("../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../api/atlasApi")>()),
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  addSavedListItem: apiMocks.addSavedListItem,
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  getReadiness: apiMocks.getReadiness,
  verifyBackendCompatibility: apiMocks.verifyBackendCompatibility,
  getRecordDetail: apiMocks.getRecordDetail,
  getRelatedRecords: apiMocks.getRelatedRecords,
  getEncounters: apiMocks.getEncounters,
  getSavedLists: apiMocks.getSavedLists,
  filterSavedList: apiMocks.filterSavedList,
  searchPage: apiMocks.searchPage,
}));

describe("AtlasApp routing", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installLocalStorage();
    Element.prototype.scrollIntoView = vi.fn();
    window.localStorage.clear();
    window.sessionStorage.clear();
    history.replaceState(null, "", "/search");
    apiMocks.getReadiness.mockResolvedValue(readinessFixture());
    apiMocks.verifyBackendCompatibility.mockResolvedValue(true);
    apiMocks.discoverFilterEditor.mockResolvedValue(emptyFilterEditor());
    apiMocks.discoverFilterValues.mockResolvedValue({
      field_id: "kind",
      matching_record_count: 0n,
      options: [],
    });
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsPage(),
      sort:
        request.sort ??
        (request.criteria.kind === "list_records"
          ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
          : { kind: "relevance" }),
    }));
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.getRelatedRecords.mockImplementation((request) =>
      Promise.resolve({
        selection: request.selection,
        artifact_generation: request.artifact_generation,
        seed: summaryFixture(request.selection.record_key),
        references: { kind: "empty" },
        counterparts: { kind: "empty" },
        variants: { kind: "empty" },
      }),
    );
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    routerSlot.current = createAtlasRouter();
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

  it("keeps invalid-link recovery in the chosen theme and moves keyboard recovery focus to main content", async () => {
    localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, "dark");
    history.replaceState(null, "", "/search/records/spell:heal?q=heal&at=99.bad");
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    const message = await screen.findByText("This link couldn’t be restored");
    expect(message.closest("[data-theme]")).toHaveAttribute("data-theme", "dark");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    const recovery = screen.getByRole("link", { name: "Return to results" });
    recovery.focus();
    expect(recovery).toHaveFocus();
    fireEvent.click(recovery, { detail: 0 });
    await waitFor(() => expect(location.pathname).toBe("/search"));
    await waitFor(() => expect(document.getElementById("atlas-main")).toHaveFocus());
    expect(
      await screen.findByRole("searchbox", { name: "Search records" }),
    ).not.toHaveFocus();
    expect(document.getElementById("atlas-main")).toHaveAttribute("tabindex", "-1");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
  });

  it("replaces a broken filter link with empty idle Search and a dismissible inline notice", async () => {
    localStorage.setItem(
      "atlas.search.kind-default.v1",
      JSON.stringify({ include: ["creature"], exclude: [] }),
    );
    const params = new URLSearchParams({
      q: "fire",
      filter: "2.obsolete",
      rank: "4",
      page: "9",
      record_view: "explore",
    });
    history.replaceState(null, "", "/search/records/spell:heal?" + params);
    const length = history.length;
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    const notice = await screen.findByText(
      "This link couldn’t be restored. Search has been reset.",
    );
    await waitFor(() => expect(location.pathname).toBe("/search"));
    expect(new URLSearchParams(location.search).get("q")).toBeNull();
    expect(new URLSearchParams(location.search).has("filter")).toBe(false);
    expect(new URLSearchParams(location.search).has("rank")).toBe(false);
    expect(history.length).toBe(length);
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue("");
    fireEvent.click(
      within(notice.closest("[role=alert]")!).getByRole("button", { name: "close" }),
    );
    await waitFor(() =>
      expect(
        screen.queryByText("This link couldn’t be restored. Search has been reset."),
      ).toBeNull(),
    );
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
  });

  it.each(["filter=3.broken&rank=bad", "filter=3.broken&selection=obsolete"])(
    "recovers a saved-list filter failure before reading validation: %s",
    async (query) => {
      history.replaceState(null, "", "/lists/research/records/spell:heal?" + query);
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      await screen.findByText("This link couldn’t be restored. Search has been reset.");
      await waitFor(() => expect(location.pathname).toBe("/search"));
      expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue("");
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
      expect(apiMocks.filterSavedList).not.toHaveBeenCalled();
    },
  );

  it.each([
    "/lists/research/records/spell:heal?filter=2.obsolete&q=other&rank=8",
    "/search/records/spell:heal?filter=3.broken&rank=bad",
    "/search/records/spell:heal?filter=3.broken&selection=obsolete",
  ])(
    "clears retained Search criteria, reading, page and unfinished input after transport failure: %s",
    async (brokenLink) => {
      history.replaceState(null, "", "/search/records/spell:heal?q=heal&page=4&rank=4");
      apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
        ...searchResultsPage(),
        page: { ...searchResultsPage().page, number: request.page.number },
        sort: { kind: "relevance" },
      }));
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      const input = await screen.findByRole("searchbox", { name: "Search records" });
      await screen.findByRole("heading", { name: "heal" });
      await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalled());
      fireEvent.change(input, { target: { value: "Unsubmitted draft" } });
      const requestsBefore = apiMocks.searchPage.mock.calls.length;
      const detailReadsBefore = apiMocks.getRecordDetail.mock.calls.length;
      act(() => {
        history.pushState(null, "", brokenLink);
        fireEvent.popState(window);
      });
      await screen.findByText("This link couldn’t be restored. Search has been reset.");
      await waitFor(() => expect(location.pathname).toBe("/search"));
      expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue("");
      expect(screen.queryByRole("heading", { name: "heal" })).toBeNull();
      expect(new URLSearchParams(location.search).get("page")).toBeNull();
      expect(new URLSearchParams(location.search).get("q")).toBeNull();
      expect(apiMocks.searchPage).toHaveBeenCalledTimes(requestsBefore);
      expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(detailReadsBefore);
      expect(apiMocks.filterSavedList).not.toHaveBeenCalled();
    },
  );

  it("restores record pages from browser history without running search queries", async () => {
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
    expect(document.title).toBe("heal · Atlas");
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      "spell:heal",
      undefined,
      expect.any(AbortSignal),
    );

    vi.clearAllMocks();
    history.pushState(null, "", "/records/spell%3Alinked");
    fireEvent.popState(window);
    await act(() => vi.dynamicImportSettled());

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "heal" })).not.toBeInTheDocument();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(document.title).toBe("linked · Atlas");

    history.pushState(null, "", "/search");
    fireEvent.popState(window);
    await waitFor(() => expect(document.title).toBe("Search · Atlas"));
    history.pushState(null, "", "/lists");
    fireEvent.popState(window);
    await waitFor(() => expect(document.title).toBe("Lists · Atlas"));
  });

  it("waits for metadata before searching and gives the URL priority over preference and launch default", async () => {
    history.replaceState(null, "", "/search?q=heal&search_profile=embeddinggemma");
    localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, "bge-base");
    let finish!: (value: AppReadinessView) => void;
    apiMocks.getReadiness.mockReturnValue(
      new Promise<AppReadinessView>((resolve) => {
        finish = resolve;
      }),
    );
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage(),
        search_profile: request.search_profile,
        sort:
          request.sort ??
          (request.criteria.kind === "list_records"
            ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
            : { kind: "relevance" }),
      }),
    );
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    await act(() => vi.dynamicImportSettled());
    await waitFor(() => expect(apiMocks.getReadiness).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
    expect(screen.queryByRole("status", { name: "Searching…" })).toBeNull();
    expect(await screen.findByRole("status", { name: "Searching…" })).toBeVisible();
    expect(screen.queryByRole("textbox", { name: "Current page" })).toBeNull();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    await act(async () => finish(readinessFixture()));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage.mock.calls[0][0].search_profile).toBe("embeddinggemma");
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    const settings = screen.getByRole("dialog", { name: "Search settings" });
    fireEvent.click(
      within(settings).getByRole("button", { name: /Advanced settings/ }),
    );
    const model = within(settings).getByRole("combobox", { name: "Search model" });
    expect(model).toBeEnabled();
    fireEvent.mouseDown(model);
    expect(
      await screen.findByRole("option", { name: "EmbeddingGemma" }),
    ).toHaveAttribute("aria-selected", "true");
  });

  it.each([false, true])(
    "preserves an explicit copied/reloaded reader URL through delayed metadata (single profile: %s)",
    async (single) => {
      const selection = {
        record_key: "spell:heal",
        embedded_path: [],
        field: "/system/description/value",
        passage: null,
        source_fingerprint: "snapshot-a",
      };
      const landing = {
        ...selection,
        field: null,
        embedded_path: [{ collection: "items", identity: { Stable: "landing" } }],
      };
      const filter = encodeFilterUrl({
        kind: "in",
        field: "kind",
        values: ["spell"],
      });
      const params = new URLSearchParams({
        q: "heal",
        filter,
        search_profile: "embeddinggemma",
        at: encodeReadingAddress(selection.record_key, selection, false)!,
        focus: encodeReadingAddress(selection.record_key, landing, true)!,
      });
      history.replaceState(null, "", "/search/records/spell:heal?" + params);
      localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, "bge-base");
      let finish!: (value: AppReadinessView) => void;
      apiMocks.getReadiness.mockReturnValue(
        new Promise<AppReadinessView>((resolve) => {
          finish = resolve;
        }),
      );
      apiMocks.getRecordDetail.mockImplementation(
        (key: string, request?: RecordDetailRequest) => {
          const detail = recordDetailFixture(key);
          if (!request) return Promise.resolve(detail);
          const { fields, ...address } = request;
          return Promise.resolve({
            ...detail,
            selected: { ...address, field: fields[0] ?? null },
          });
        },
      );
      apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
        Promise.resolve({
          ...searchResultsPage(),
          search_profile: request.search_profile,
          sort:
            request.sort ??
            (request.criteria.kind === "list_records"
              ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
              : { kind: "relevance" }),
        }),
      );
      const first = render(<AtlasApp />, { wrapper: queryClientWrapper() });
      expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
      expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
        "embeddinggemma",
      );
      const input = screen.getByRole("searchbox", { name: "Search records" });
      fireEvent.change(input, { target: { value: "frost" } });
      fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
      await waitFor(() =>
        expect(new URLSearchParams(window.location.search).get("q")).toBe("frost"),
      );
      const copied = window.location.pathname + window.location.search;
      const copiedParams = new URLSearchParams(window.location.search);
      expect(copiedParams.get("search_profile")).toBe("embeddinggemma");
      expect(copiedParams.get("filter")).toBe(filter);
      expect(
        selectedNavigation(
          parseSearchParams(copiedParams.toString()),
          selection.record_key,
        ),
      ).toEqual(selection);
      expect(
        selectedNavigation(
          parseSearchParams(copiedParams.toString()),
          selection.record_key,
          "focus",
        ),
      ).toEqual(landing);
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      first.unmount();
      routerSlot.current = createAtlasRouter();
      history.replaceState(null, "", copied);
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      const readiness = readinessFixture();
      if (single) readiness.search_profiles = [readiness.search_profiles[0]];
      await act(async () => finish(readiness));
      if (single) {
        expect(await screen.findByRole("alert")).toHaveTextContent(
          "search link is unavailable",
        );
        expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
        expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
          "embeddinggemma",
        );
        expect(apiMocks.searchPage).not.toHaveBeenCalled();
        fireEvent.change(screen.getByRole("searchbox", { name: "Search records" }), {
          target: { value: "Unsubmitted after reload" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Reset search link" }));
        await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
        expect(apiMocks.searchPage.mock.calls[0][0]).toMatchObject({
          search_profile: "bge-base",
          criteria: {
            query: "frost",
            filter: { field: "kind", values: ["spell"] },
          },
        });
        expect(
          new URLSearchParams(window.location.search).get("search_profile"),
        ).toBeNull();
        expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue(
          "Unsubmitted after reload",
        );
      } else {
        await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
        expect(apiMocks.searchPage.mock.calls[0][0]).toMatchObject({
          search_profile: "embeddinggemma",
          criteria: { query: "frost" },
        });
      }
      expect(
        selectedNavigation(
          parseSearchParams(window.location.search),
          selection.record_key,
        ),
      ).toEqual(selection);
      expect(
        selectedNavigation(
          parseSearchParams(window.location.search),
          selection.record_key,
          "focus",
        ),
      ).toEqual(landing);
      expect(screen.getByRole("heading", { name: "heal" })).toBeVisible();
    },
  );

  it.each(["unknown", "embeddinggemma"])(
    "retains unavailable URL selection %s without fallback or inference, while the reader still opens",
    async (profile) => {
      history.replaceState(
        null,
        "",
        "/search/records/spell:heal?q=heal&search_profile=" + profile,
      );
      const readiness = readinessFixture();
      readiness.search_profiles[1].usable = false;
      readiness.search_profiles[1].assets_ready = false;
      apiMocks.getReadiness.mockResolvedValue(readiness);
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
      expect(await screen.findByRole("alert")).toHaveTextContent(
        profile === "unknown" ? "not stored" : "unavailable for text search",
      );
      expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
        profile,
      );
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
    },
  );

  it.each([false, true])(
    "keeps a pending explicit selection in native Similar links copied to a fresh app (single profile: %s)",
    async (single) => {
      history.replaceState(
        null,
        "",
        "/search/records/spell:heal?search_profile=embeddinggemma",
      );
      localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, "bge-base");
      let finish!: (value: AppReadinessView) => void;
      apiMocks.getReadiness.mockReturnValue(
        new Promise<AppReadinessView>((resolve) => {
          finish = resolve;
        }),
      );
      apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
        Promise.resolve({
          ...searchResultsPage(),
          search_profile: request.search_profile,
          sort:
            request.sort ??
            (request.criteria.kind === "list_records"
              ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
              : { kind: "relevance" }),
        }),
      );
      const reader = render(<AtlasApp />, { wrapper: queryClientWrapper() });
      expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
      fireEvent.click(screen.getByRole("button", { name: "Explore related records" }));
      fireEvent.click(await screen.findByRole("tab", { name: "Similar" }));
      const link = await screen.findByRole("link", {
        name: "Search similar records",
      });
      const href = link.getAttribute("href")!;
      const copied = new URL(href, window.location.origin);
      expect(copied.searchParams.get("search_profile")).toBe("embeddinggemma");
      expect(copied.searchParams.get("similar")).toBe("spell:heal");
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(1);
      expect(apiMocks.getRelatedRecords).toHaveBeenCalledTimes(1);
      for (const nativeClick of [{ metaKey: true }, { ctrlKey: true }, { button: 1 }])
        expect(fireEvent.click(link, nativeClick)).toBe(true);
      expect(window.location.pathname).toBe("/search/records/spell:heal");
      reader.unmount();
      routerSlot.current = createAtlasRouter();
      history.replaceState(null, "", copied.pathname + copied.search);
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      await act(() => vi.dynamicImportSettled());
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      const readiness = readinessFixture();
      if (single) readiness.search_profiles = [readiness.search_profiles[0]];
      await act(async () => finish(readiness));
      if (single) {
        const error = await screen.findByRole("alert");
        expect(error).toHaveTextContent("search link is unavailable");
        expect(error).not.toHaveTextContent(/model|choose|BGE Base|EmbeddingGemma/i);
        expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
        expect(apiMocks.searchPage).not.toHaveBeenCalled();
        expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
          "embeddinggemma",
        );
        fireEvent.click(screen.getByRole("button", { name: "Reset search link" }));
      }
      await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
      expect(apiMocks.searchPage.mock.calls[0][0]).toMatchObject({
        search_profile: single ? "bge-base" : "embeddinggemma",
        criteria: { kind: "similar_records", seed_record_key: "spell:heal" },
      });
      if (single)
        expect(
          new URLSearchParams(window.location.search).get("search_profile"),
        ).toBeNull();
      expect(new URLSearchParams(window.location.search).get("similar")).toBe(
        "spell:heal",
      );
    },
  );

  it("switches the shared model without submitting the keyword draft or closing the reader", async () => {
    const breakpoint = vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
    history.replaceState(null, "", "/search/records/spell:heal?q=heal");
    apiMocks.discoverFilterEditor.mockResolvedValue(
      editorFixture([fieldFixture("defenses.hp.maximum", "number", "Maximum HP")]),
    );
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage(["spell:heal"]),
        search_profile: request.search_profile,
        sort:
          request.sort ??
          (request.criteria.kind === "list_records"
            ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
            : { kind: "relevance" }),
      }),
    );
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
    const input = screen.getByRole("searchbox", { name: "Search records" });
    expect(input.closest(".filter-controls__search-row")).toContainElement(
      screen.getByRole("button", { name: "Search settings" }),
    );
    fireEvent.change(input, { target: { value: "unsubmitted dragon" } });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(await screen.findByRole("button", { name: "Add group" }));
    expect(screen.getAllByRole("button", { name: "Add condition" })).toHaveLength(2);
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    fireEvent.click(screen.getByRole("button", { name: /Advanced settings/ }));
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
    fireEvent.click(await screen.findByText("EmbeddingGemma"));
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    await waitFor(() =>
      expect(
        apiMocks.searchPage.mock.calls[apiMocks.searchPage.mock.calls.length - 1]?.[0],
      ).toMatchObject({
        search_profile: "embeddinggemma",
        criteria: { query: "heal" },
        page: { number: 1 },
      }),
    );
    expect(input).toHaveValue("unsubmitted dragon");
    expect(screen.getByRole("heading", { name: "heal" })).toBeVisible();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(1);
    // The popover can close when another control opens; its unfinished tree survives.
    const advanced = screen.getByRole("button", { name: "Advanced filters" });
    if (advanced.getAttribute("aria-expanded") !== "true") fireEvent.click(advanced);
    expect(
      await screen.findAllByRole("button", { name: "Add condition" }),
    ).toHaveLength(2);
    breakpoint.mockRestore();
  });

  it("uses a single stored profile without showing a model chooser", async () => {
    const readiness = readinessFixture();
    readiness.search_profiles[1] = {
      ...readiness.search_profiles[1],
      stored_vectors: false,
      assets_ready: false,
      usable: false,
    };
    apiMocks.getReadiness.mockResolvedValue(readiness);
    history.replaceState(null, "", "/search?q=heal");
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage.mock.calls[0][0].search_profile).toBe("bge-base");
    expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    const settings = screen.getByRole("dialog", { name: "Search settings" });
    expect(
      within(settings).queryByRole("combobox", { name: "Search model" }),
    ).toBeNull();
    expect(within(settings).queryByText("Embedding model")).toBeNull();
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
  });

  it("resets an unavailable single-profile search link without a chooser or loss of criteria and reader", async () => {
    const readiness = readinessFixture();
    readiness.search_profiles = [readiness.search_profiles[0]];
    apiMocks.getReadiness.mockResolvedValue(readiness);
    const filter = encodeFilterUrl({
      kind: "in",
      field: "kind",
      values: ["spell"],
    });
    history.replaceState(
      null,
      "",
      "/search/records/spell:heal?q=heal&filter=" +
        encodeURIComponent(filter) +
        "&search_profile=embeddinggemma",
    );
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "search link is unavailable",
    );
    expect(await screen.findByRole("heading", { name: "heal" })).toBeVisible();
    expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
    expect(screen.getByRole("alert")).not.toHaveTextContent(
      /model|EmbeddingGemma|BGE Base/i,
    );
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    fireEvent.change(screen.getByRole("searchbox", { name: "Search records" }), {
      target: { value: "Unsubmitted draft" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Reset search link" }));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage.mock.calls[0][0]).toMatchObject({
      search_profile: "bge-base",
      criteria: { query: "heal", filter: { field: "kind", values: ["spell"] } },
    });
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
    const params = new URLSearchParams(window.location.search);
    expect(params.get("search_profile")).toBeNull();
    expect(params.get("q")).toBe("heal");
    expect(params.get("filter")).toBe(filter);
    expect(window.location.pathname).toBe("/search/records/spell:heal");
    expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue(
      "Unsubmitted draft",
    );
    expect(screen.getByRole("heading", { name: "heal" })).toBeVisible();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(1);
  });

  it("ignores a stale browser preference on a normal single-profile visit", async () => {
    const readiness = readinessFixture();
    readiness.search_profiles = [readiness.search_profiles[0]];
    apiMocks.getReadiness.mockResolvedValue(readiness);
    localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, "embeddinggemma");
    history.replaceState(null, "", "/search?q=heal");
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage.mock.calls[0][0].search_profile).toBe("bge-base");
    expect(screen.queryByRole("alert")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    const settings = screen.getByRole("dialog", { name: "Search settings" });
    expect(
      within(settings).queryByRole("combobox", { name: "Search model" }),
    ).toBeNull();
    expect(within(settings).queryByText("Embedding model")).toBeNull();
  });

  it("uses ordinary availability copy when the sole stored profile lacks text-search assets", async () => {
    const readiness = readinessFixture();
    readiness.search_profiles = [
      { ...readiness.search_profiles[0], assets_ready: false, usable: false },
    ];
    apiMocks.getReadiness.mockResolvedValue(readiness);
    history.replaceState(null, "", "/search?q=heal&search_profile=bge-base");
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Text search is unavailable");
    expect(error).not.toHaveTextContent(/model|choose|BGE Base|EmbeddingGemma/i);
    expect(screen.queryByRole("combobox", { name: "Search model" })).toBeNull();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
  });

  it("can launch and switch retrieval when optional browser storage is denied", async () => {
    vi.mocked(localStorage.getItem).mockImplementation(() => {
      throw new Error("Storage denied");
    });
    vi.mocked(localStorage.setItem).mockImplementation(() => {
      throw new Error("Storage denied");
    });
    history.replaceState(null, "", "/search?q=heal");
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage(),
        search_profile: request.search_profile,
        sort:
          request.sort ??
          (request.criteria.kind === "list_records"
            ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
            : { kind: "relevance" }),
      }),
    );
    render(<AtlasApp />, { wrapper: queryClientWrapper() });
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    fireEvent.click(screen.getByRole("button", { name: /Advanced settings/ }));
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
    fireEvent.click(await screen.findByText("EmbeddingGemma"));
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    expect(apiMocks.searchPage.mock.calls[1][0].search_profile).toBe("embeddinggemma");
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

    fireEvent.click(screen.getByRole("button", { name: "Add to list" }));
    const dialog = await screen.findByRole("dialog", { name: "Add to list" });
    const selector = within(dialog).getByRole("combobox");
    await waitFor(() => expect(selector).toBeEnabled());
    fireEvent.mouseDown(selector);
    fireEvent.click(await screen.findByText("Research"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
        list_ref: "list_research",
        record_ref: "spell:heal",
      }),
    );
  }, 10_000);

  it.each(["creature", "hazard", "equipment"] as const)(
    "adds a linked %s from Search without leaving the selected reader",
    async (kind) => {
      history.replaceState(null, "", "/search/records/spell:heal?q=heal");
      apiMocks.searchPage.mockResolvedValue(searchResultsPage(["spell:heal"]));
      const linkedKey = `${kind}:linked`;
      apiMocks.getRecordDetail.mockImplementation(async (key: string) => {
        const detail = detailFixture(
          key,
          key === linkedKey ? `Linked ${kind}` : "heal",
          key === linkedKey ? undefined : linkedKey,
        );
        if (key === linkedKey) detail.record.kind = kind;
        return detail;
      });
      apiMocks.getEncounters.mockResolvedValue({
        encounters: [
          {
            encounter_key: "enc_ambush",
            slug: "ambush",
            name: "Ambush",
            status: "draft",
          },
        ],
      });
      apiMocks.addEncounterRecordParticipant.mockResolvedValue({
        encounter_key: "enc_ambush",
      });
      render(<AtlasApp />, { wrapper: queryClientWrapper() });
      const selected = await screen.findByRole("link", { name: /^heal/, hidden: true });
      const nested = await screen.findByRole("link", { name: "Nested Rule" });
      const reader = document.querySelector<HTMLElement>(".reference-reader__scroll")!;
      reader.scrollTop = 425;
      const input = screen.getByRole("searchbox", { name: "Search records" });
      fireEvent.change(input, { target: { value: "Unsubmitted idea" } });
      const url = location.href;
      fireEvent.click(nested);
      const preview = await screen.findByRole("dialog", { name: "Reference" });
      await within(preview).findByRole("heading", { name: `Linked ${kind}` });
      const compatible = kind !== "equipment";
      const trigger = within(preview).getByRole("button", {
        name: compatible ? "Add to list or encounter" : "Add to list",
      });
      fireEvent.click(trigger);
      const picker = await screen.findByRole("dialog", {
        name: compatible ? "Add to…" : "Add to list",
      });
      if (compatible)
        fireEvent.click(within(picker).getByRole("tab", { name: "Encounters" }));
      else expect(within(picker).queryByRole("tab", { name: "Encounters" })).toBeNull();
      const selector = within(picker).getByRole("combobox");
      await waitFor(() => expect(selector).toBeEnabled());
      fireEvent.mouseDown(selector);
      fireEvent.click(await screen.findByText(compatible ? "Ambush" : "Research"));
      fireEvent.click(within(picker).getByRole("button", { name: "Add" }));
      const mutation = compatible
        ? apiMocks.addEncounterRecordParticipant
        : apiMocks.addSavedListItem;
      await waitFor(() => expect(mutation).toHaveBeenCalledTimes(1));
      expect(mutation).toHaveBeenCalledWith(
        compatible
          ? { encounter_ref: "enc_ambush", record_ref: linkedKey, quantity: 1 }
          : { list_ref: "list_research", record_ref: linkedKey },
      );
      await waitFor(() => expect(picker).not.toBeVisible());
      expect(preview).toBeVisible();
      await waitFor(() => expect(trigger).toHaveFocus());
      expect(location.href).toBe(url);
      expect(input).toHaveValue("Unsubmitted idea");
      expect(reader.scrollTop).toBe(425);
      expect(selected).toHaveAttribute("aria-current", "true");
    },
    10_000,
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
  return editorFixture();
}

function searchResultsPage(recordKeys: string[] = []): SearchResultsPage {
  return {
    search_content: "all",
    relationship_clauses: [],
    sort: { kind: "relevance" } as const,
    ranking: "score" as const,
    search_profile: "bge-base",
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
    rows: recordKeys.map((key) => ({
      record: summaryFixture(key),
      relationship_evidence: [],
      matches: [],
    })),
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

function render(
  ui: ReactNode,
  options?: import("@testing-library/react").RenderOptions,
) {
  atlasRouter.updateLatestLocation();
  return renderDom(ui, options);
}
