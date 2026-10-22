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
  AppReadinessView,
  SearchResultsRequest,
  RecordDetailRequest,
} from "../generated/atlas";
import { AtlasApp } from "./AtlasApp";
import { SEARCH_PROFILE_PREFERENCE_KEY } from "../shared/search/SearchProfileContext";
import { encodeFilterUrl } from "../shared/filters/filterUrl";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  getReadiness: vi.fn(),
  getRecordDetail: vi.fn(),
  getRelatedRecords: vi.fn(),
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
  getRelatedRecords: apiMocks.getRelatedRecords,
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
    apiMocks.getReadiness.mockResolvedValue(readinessFixture());
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
    expect(document.title).toBe("heal · Atlas");
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
    expect(document.title).toBe("heal · Atlas");

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
    expect(screen.getByRole("status", { name: "Searching…" })).toBeVisible();
    expect(screen.queryByRole("textbox", { name: "Current page" })).toBeNull();
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    await act(async () => finish(readinessFixture()));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    expect(apiMocks.searchPage.mock.calls[0][0].search_profile).toBe("embeddinggemma");
    expect(screen.getByText("EmbeddingGemma")).toBeVisible();
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
        selection: JSON.stringify(selection),
        landing: JSON.stringify(landing),
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
      expect(JSON.parse(copiedParams.get("selection")!)).toEqual(selection);
      expect(JSON.parse(copiedParams.get("landing")!)).toEqual(landing);
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      first.unmount();
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
        JSON.parse(new URLSearchParams(window.location.search).get("selection")!),
      ).toEqual(selection);
      expect(
        JSON.parse(new URLSearchParams(window.location.search).get("landing")!),
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
        "/records/spell:heal?search_profile=embeddinggemma",
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
      fireEvent.click(screen.getByRole("button", { name: "Related records" }));
      fireEvent.click(await screen.findByRole("button", { name: "Similar records" }));
      const link = await screen.findByRole("link", {
        name: "See more similar records",
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
      expect(window.location.pathname).toBe("/records/spell:heal");
      reader.unmount();
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
      screen.getByRole("combobox", { name: "Search model" }),
    );
    fireEvent.change(input, { target: { value: "unsubmitted dragon" } });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(await screen.findByRole("button", { name: "Add group" }));
    expect(screen.getAllByRole("button", { name: "Add condition" })).toHaveLength(2);
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
    fireEvent.click(await screen.findByText("EmbeddingGemma"));
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
    expect(screen.queryByText(/Search model|EmbeddingGemma|BGE Base/)).toBeNull();
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
    expect(screen.queryByText(/Search model|EmbeddingGemma|BGE Base/)).toBeNull();
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
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
    fireEvent.click(await screen.findByText("EmbeddingGemma"));
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
