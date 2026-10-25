import { RouterTestContext } from "../../test/router";
import type { ResultSort } from "../../generated/atlas";
import {
  summaryFixture,
  editorFixture,
  fieldFixture,
  valuesFixture,
  detailFixture,
} from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  render,
  renderHook,
  screen,
  fireEvent,
  waitFor,
} from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  FilterEditorView,
  SearchResultsRequest,
  SearchResultsPage,
  RecordDetailRequest,
} from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
  effectiveResultSort,
} from "../../shared/filters/searchState";
import { KIND_PREFERENCE_KEY } from "./workspaceState";
import { AtlasApiError } from "../../api/atlasApi";
import { useSearchWorkspace } from "./useSearchWorkspace";
import { RelationshipEvidence } from "../../shared/records/RelationshipEvidence";
import { linkLeaf } from "../../shared/filters/linkPredicates";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import type { QueryPredicate } from "../../generated/atlas";

const apiMocks = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  discoverFilterCounts: vi.fn(),
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  searchPage: vi.fn(),
  getRelationshipOccurrences: vi.fn(),
}));

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  discoverFilterCounts: apiMocks.discoverFilterCounts,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  searchPage: apiMocks.searchPage,
  getRelationshipOccurrences: apiMocks.getRelationshipOccurrences,
}));

describe("useSearchWorkspace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    apiMocks.getReadiness.mockResolvedValue({
      status: "ready",
      message: "Ready",
    });
    apiMocks.discoverFilterEditor.mockResolvedValue({ ...editorFixture(), groups: [] });
    apiMocks.discoverFilterValues.mockResolvedValue(valuesFixture("kind"));
    apiMocks.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    apiMocks.searchPage.mockResolvedValue(searchResultsPage());
    history.replaceState(null, "", "/");
  });

  it("opens a submitted canonical key immediately without running text inference", async () => {
    const key = "pathfinder-bestiary:j1mm6bb3hZ56jSrK";
    apiMocks.getRecordDetail.mockResolvedValue(detailFixture(key));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: null, searchProfileUsable: false }),
      { wrapper: queryClientWrapper() },
    );
    act(() =>
      result.current.setSearch({ ...result.current.search, query: ` ${key} ` }),
    );
    expect(result.current.selectedRecordKey).toBe(key);
    await waitFor(() => expect(decodeURIComponent(location.pathname)).toContain(key));
    await waitFor(() =>
      expect(result.current.reading.detail?.record.record_key).toBe(key),
    );
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(result.current.search.query).toBe("");
  });

  it("resets paging for explicit order, keeps it across semantic transitions and recovery", async () => {
    history.replaceState(null, "", "/search?q=fire");
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsPage(["items:witness"], { pageNumber: request.page.number }),
      sort:
        request.sort ??
        (request.criteria.kind === "list_records"
          ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
          : { kind: "relevance" }),
    }));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() =>
      expect(result.current.resultPage?.sort).toEqual({ kind: "relevance" }),
    );
    expect(result.current.search.sort).toBeUndefined();
    act(() => result.current.setPageNumber(3));
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(3));
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        sort: {
          kind: "attributes",
          rules: [{ field: "level", direction: "desc" }],
        } satisfies ResultSort,
      }),
    );
    await waitFor(() =>
      expect(result.current.resultPage?.sort).toEqual({
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      }),
    );
    expect(result.current.pageNumber).toBe(1);
    expect(location.search).toContain("sort=level_desc");
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        query: "",
        similarSeed: "items:seed",
      }),
    );
    await waitFor(() =>
      expect(apiMocks.searchPage.mock.lastCall?.[0].criteria.kind).toBe(
        "similar_records",
      ),
    );
    expect(result.current.search.sort).toEqual({
      kind: "attributes",
      rules: [{ field: "level", direction: "desc" }],
    });
    await act(async () => result.current.refreshResults());
    await waitFor(() =>
      expect(result.current.resultPage?.sort).toEqual({
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      }),
    );
    expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
      expected_artifact_generation: null,
      page: { number: 1 },
    });
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        sort: { kind: "relevance" } as const,
      }),
    );
    await waitFor(() =>
      expect(result.current.resultPage?.sort).toEqual({ kind: "relevance" }),
    );
    act(() =>
      result.current.setSearch({ ...result.current.search, similarSeed: null }),
    );
    expect(result.current.search.sort).toBeUndefined();
    expect(effectiveResultSort(result.current.search)).toEqual({
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    });
  });

  it("rejects wrong-order responses instead of adopting their generation", async () => {
    history.replaceState(null, "", "/search?q=fire&sort=level_desc");
    apiMocks.searchPage.mockResolvedValue({
      ...searchResultsPage(),
      sort: {
        kind: "attributes",
        rules: [{ field: "name", direction: "asc" }],
      } satisfies ResultSort,
    });
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() =>
      expect(result.current.errorMessage).toContain("different order"),
    );
    expect(result.current.resultPage).toBeUndefined();
    expect(
      apiMocks.searchPage.mock.lastCall?.[0].expected_artifact_generation,
    ).toBeNull();
  });

  it("retains loaded Similar rows and the seed title as updating while a new order is pending", async () => {
    history.replaceState(null, "", "/search?similar=items:armor");
    const original: SearchResultsPage = {
      ...searchResultsPage(["items:old"]),
      mode: {
        kind: "similar_records",
        seed: summaryFixture("items:armor", "Leather Armor"),
      },
    };
    let finish!: (page: SearchResultsPage) => void;
    apiMocks.searchPage.mockResolvedValueOnce(original).mockReturnValueOnce(
      new Promise<SearchResultsPage>((resolve) => {
        finish = resolve;
      }),
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage?.rows).toHaveLength(1));
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        sort: {
          kind: "attributes",
          rules: [{ field: "level", direction: "desc" }],
        } satisfies ResultSort,
      }),
    );
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    expect(result.current.resultPage).toEqual(original);
    expect(result.current.resultsRefreshing).toBe(true);
    await act(async () =>
      finish({
        ...original,
        sort: {
          kind: "attributes",
          rules: [{ field: "level", direction: "desc" }],
        } satisfies ResultSort,
        rows: [
          {
            record: summaryFixture("items:new"),
            relationship_evidence: [],
            matches: [],
          },
        ],
      }),
    );
    await waitFor(() =>
      expect(result.current.resultPage?.sort).toEqual({
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      }),
    );
    expect(result.current.resultsRefreshing).toBe(false);
    expect(result.current.resultPage?.rows[0].record.record_key).toBe("items:new");
  });

  it("keeps exact relationship scope through paging, recovery and contextual counts", async () => {
    const constraint = {
      direction: "incoming" as const,
      selection: {
        record_key: "items:armor",
        embedded_path: [{ collection: "items", identity: { Stable: "exact-entry" } }],
        field: null,
        passage: null,
        source_fingerprint: "checked-source",
      },
      include_contents: true,
    };
    history.replaceState(
      null,
      "",
      "/search" +
        searchStateQueryString({
          ...DEFAULT_SEARCH_STATE,
          filter: { kind: "relationship", relationship: constraint },
        }),
    );
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage([], { pageNumber: request.page.number }),
        sort: {
          kind: "attributes",
          rules: [{ field: "name", direction: "asc" }],
        } satisfies ResultSort,
        relationship_clauses: [
          {
            clause_id: request.criteria.filter!.clause_id!,
            relationship:
              request.criteria.filter!.kind === "relationship"
                ? request.criteria.filter!.relationship
                : constraint,
            record: summaryFixture("items:armor", "Leather Armor"),
            title: "Exact entry",
          },
        ],
      }),
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage).toBeDefined());
    expect(result.current.filterCountsEnabled).toBe(true);
    expect(result.current.filterDiscoveryContext).toMatchObject({
      filter: { kind: "relationship", relationship: constraint },
    });
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(2));
    expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
      criteria: { filter: { kind: "relationship", relationship: constraint } },
      expected_artifact_generation: "generation-a",
    });
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: {
          kind: "relationship",
          clause_id: "link",
          relationship: { ...constraint, include_contents: false },
        },
      }),
    );
    await waitFor(() =>
      expect(
        result.current.resultPage?.relationship_clauses[0].relationship
          .include_contents,
      ).toBe(false),
    );
    expect(result.current.pageNumber).toBe(1);
    expect(result.current.filterDiscoveryContext).toMatchObject({
      filter: {
        kind: "relationship",
        clause_id: "link",
        relationship: { ...constraint, include_contents: false },
      },
    });
    await act(async () => result.current.refreshResults());
    await waitFor(() => expect(result.current.resultPage).toBeDefined());
    expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
      criteria: {
        filter: {
          kind: "relationship",
          clause_id: "link",
          relationship: { ...constraint, include_contents: false },
        },
      },
      expected_artifact_generation: null,
      page: { number: 1 },
    });
    expect(
      result.current.search.filter?.kind === "relationship"
        ? result.current.search.filter.relationship.selection
        : null,
    ).toEqual(constraint.selection);
  });

  it("switches ranking independently of the model and resets pagination", async () => {
    history.replaceState(null, "", "/search?q=harrowkin");
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage([], {
          profile: request.search_profile,
          pageNumber: request.page.number,
        }),
        ranking: request.ranking ?? "score",
      }),
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "embeddinggemma" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage?.ranking).toBe("score"));
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(2));
    act(() => result.current.setSearch({ ...result.current.search, ranking: "rrf" }));
    await waitFor(() => expect(result.current.resultPage?.ranking).toBe("rrf"));
    expect(result.current.resultPage?.page.number).toBe(1);
    expect(result.current.resultPage?.search_profile).toBe("embeddinggemma");
    expect(new URLSearchParams(location.search).get("ranking")).toBe("rrf");
    expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
      ranking: "rrf",
      search_profile: "embeddinggemma",
      criteria: { kind: "text_search", query: "harrowkin" },
    });
  });

  it("preserves a standalone spell's selected rank in the search URL and restores it on reload", async () => {
    history.replaceState(null, "", "/search/records/spells:fireball");
    apiMocks.getRecordDetail.mockImplementation((key, selection) => {
      const detail = detailFixture(key);
      detail.selected = { ...detail.selected, spell_rank: selection?.spell_rank };
      return Promise.resolve(detail);
    });
    const first = renderHook(() => useSearchWorkspace({ searchProfile: "bge-base" }), {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() => expect(first.result.current.reading.detail).toBeDefined());
    act(() =>
      first.result.current.reading.changeRank({
        ...first.result.current.reading.detail!.selected,
        spell_rank: 5,
      }),
    );
    await waitFor(() =>
      expect(
        JSON.parse(new URLSearchParams(location.search).get("selection") ?? "null")
          ?.spell_rank,
      ).toBe(5),
    );
    first.unmount();
    const reloaded = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() =>
      expect(reloaded.result.current.reading.detail?.selected.spell_rank).toBe(5),
    );
    expect(apiMocks.getRecordDetail).toHaveBeenLastCalledWith(
      "spells:fireball",
      expect.objectContaining({ spell_rank: 5 }),
      expect.any(AbortSignal),
    );
  });

  it("restores similarity URLs, disables contextual counts, sends stateless pages, and replaces or clears the criterion", async () => {
    localStorage.setItem(
      KIND_PREFERENCE_KEY,
      JSON.stringify({ include: ["creature"], exclude: [] }),
    );
    const filter = { kind: "in" as const, field: "kind", values: ["spell"] };
    const state = {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: "items:seed",
      mode: "similar_records" as const,
      filter,
    };
    const similarUrl = "/search" + searchStateQueryString(state);
    history.replaceState(null, "", similarUrl);
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) =>
      Promise.resolve({
        ...searchResultsPage(),
        page: { ...searchResultsPage().page, number: request.page.number },
        mode: { kind: "similar_records", seed: summaryFixture("items:seed", "Seed") },
      }),
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() => expect(result.current.resultPage).toBeDefined());
    expect(result.current.search.filter).toMatchObject(filter);
    expect(apiMocks.searchPage.mock.calls[0][0].criteria).toEqual({
      kind: "similar_records",
      seed_record_key: "items:seed",
      filter: result.current.search.filter,
    });
    expect(apiMocks.discoverFilterCounts).not.toHaveBeenCalled();
    expect(result.current.filterEditor).toBeDefined();
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(2));
    expect(apiMocks.searchPage.mock.calls[1][0]).toEqual({
      search_profile: "bge-base",
      criteria: {
        kind: "similar_records",
        seed_record_key: "items:seed",
        filter: result.current.search.filter,
      },
      page: { number: 2, size: 25 },
      expected_artifact_generation: "generation-a",
    });
    act(() => result.current.setSearch({ ...result.current.search, query: "dragon" }));
    expect(result.current.search.similarSeed).toBeNull();
    await waitFor(() => expect(window.location.search).not.toContain("similar="));
    // Back/Forward use the same popstate restoration owner as real browser history.
    act(() => {
      history.replaceState(null, "", similarUrl);
      window.dispatchEvent(new PopStateEvent("popstate"));
    });
    expect(result.current.search.similarSeed).toBe("items:seed");
    expect(result.current.pageNumber).toBe(1);
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        similarSeed: null,
        mode: "browse",
      }),
    );
    expect(result.current.search.filter).toMatchObject(filter);
    expect(result.current.search.query).toBe("");
  });

  it("switches a later page atomically, rejects late old rows, and retains exact reading and shared caches", async () => {
    history.replaceState(
      null,
      "",
      "/search" +
        searchStateQueryString({
          ...DEFAULT_SEARCH_STATE,
          query: "dirge",
          mode: "text_search",
          filter: { kind: "in", field: "kind", values: ["spell"] },
        }),
    );
    const oldPage = deferred<SearchResultsPage>();
    const newPage = deferred<SearchResultsPage>();
    apiMocks.getRecordDetail.mockImplementation(
      (_key: string, selection?: RecordDetailRequest) => {
        const detail = detailFixture("spell:heal", "Heal");
        if (!selection) return Promise.resolve(detail);
        const { fields, ...address } = selection;
        return Promise.resolve({
          ...detail,
          selected: { ...address, field: fields[0] ?? null },
        });
      },
    );
    apiMocks.searchPage
      .mockResolvedValueOnce(searchResultsPage(["items:base"]))
      .mockReturnValueOnce(oldPage.promise)
      .mockReturnValueOnce(newPage.promise);
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { result, rerender } = renderHook(
      ({ profile }) => useSearchWorkspace({ searchProfile: profile }),
      { initialProps: { profile: "bge-base" }, wrapper: queryClientWrapper(client) },
    );
    await waitFor(() => expect(result.current.resultPage?.rows).toHaveLength(1));
    act(() => result.current.selectRecord("spell:heal"));
    await waitFor(() => expect(result.current.reading.detail).toBeDefined());
    const exact = {
      ...result.current.reading.request!,
      navigation: {
        ...result.current.reading.request!.navigation,
        embedded_path: [
          { collection: "items", identity: { Stable: "embedded-spell" } },
        ],
        field: "/system/description/value",
        source_fingerprint: "snapshot-a",
      },
    };
    act(() => result.current.reading.inspect(exact));
    const positions = result.current.positions;
    const criteria = result.current.search;
    const graphKey = ["related-records", "shared-graph"];
    client.setQueryData(graphKey, { retained: true });
    act(() => result.current.setPageNumber(4));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    expect(result.current.resultPage?.rows[0].record.record_key).toBe("items:base");
    rerender({ profile: "embeddinggemma" });
    expect(result.current.pageNumber).toBe(1);
    expect(result.current.resultPage).toBeUndefined();
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(3));
    expect(apiMocks.searchPage.mock.calls[2][0]).toMatchObject({
      search_profile: "embeddinggemma",
      page: { number: 1 },
      expected_artifact_generation: "generation-a",
    });
    await act(async () =>
      newPage.resolve(
        searchResultsPage(["items:gemma"], { profile: "embeddinggemma" }),
      ),
    );
    await act(async () =>
      oldPage.resolve(searchResultsPage(["items:old-late"], { pageNumber: 4 })),
    );
    await waitFor(() =>
      expect(result.current.resultPage?.rows[0].record.record_key).toBe("items:gemma"),
    );
    expect(result.current.search).toBe(criteria);
    expect(result.current.selectedRecordKey).toBe("spell:heal");
    expect(result.current.reading.request).toEqual(exact);
    expect(
      JSON.parse(new URLSearchParams(window.location.search).get("selection")!),
    ).toMatchObject(exact.navigation);
    expect(result.current.positions).toBe(positions);
    expect(client.getQueryData(graphKey)).toEqual({ retained: true });
    expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2);
    expect(apiMocks.discoverFilterCounts).not.toHaveBeenCalled();
    expect(
      apiMocks.searchPage.mock.calls.filter(
        ([request]) => request.search_profile === "embeddinggemma",
      ),
    ).toHaveLength(1);
  });

  it("adopts only the current profile's first generation without repeating its inference", async () => {
    seedSearchUrl();
    const abandoned = deferred<SearchResultsPage>();
    apiMocks.searchPage.mockReturnValueOnce(abandoned.promise).mockResolvedValue(
      searchResultsPage(["items:current"], {
        profile: "embeddinggemma",
        generation: "generation-b",
      }),
    );
    const { result, rerender } = renderHook(
      ({ profile }) => useSearchWorkspace({ searchProfile: profile }),
      {
        initialProps: { profile: "bge-base" },
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    rerender({ profile: "embeddinggemma" });
    await waitFor(() =>
      expect(result.current.resultPage?.artifact_generation).toBe("generation-b"),
    );
    await act(async () => abandoned.resolve(searchResultsPage()));
    expect(result.current.resultPage?.artifact_generation).toBe("generation-b");
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(2);
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(3));
    expect(apiMocks.searchPage.mock.calls[2][0]).toMatchObject({
      search_profile: "embeddinggemma",
      expected_artifact_generation: "generation-b",
    });
  });

  it("retains pending explicit URL intent through criterion and record-selection writes without starting retrieval", async () => {
    history.replaceState(null, "", "/search?q=heal&search_profile=embeddinggemma");
    apiMocks.getRecordDetail.mockImplementation((key: string) =>
      Promise.resolve(detailFixture(key)),
    );
    const { result } = renderHook(
      () =>
        useSearchWorkspace({
          searchProfile: null,
          pendingUrlProfile: "embeddinggemma",
        }),
      { wrapper: queryClientWrapper() },
    );
    act(() => result.current.setSearch({ ...result.current.search, query: "frost" }));
    expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
      "embeddinggemma",
    );
    act(() => result.current.selectRecord("spell:heal"));
    await waitFor(() => expect(result.current.reading.detail).toBeDefined());
    expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
      "embeddinggemma",
    );
    act(() => result.current.selectRecord(null));
    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(new URLSearchParams(window.location.search).get("search_profile")).toBe(
      "embeddinggemma",
    );
    expect(new URLSearchParams(window.location.search).get("q")).toBe("frost");
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
  });

  it("rejects a successful response for another profile and keeps model-free Similar usable", async () => {
    seedSearchUrl();
    apiMocks.searchPage.mockResolvedValue(
      searchResultsPage([], { profile: "embeddinggemma" }),
    );
    const { result, rerender } = renderHook(
      ({ usable }) =>
        useSearchWorkspace({ searchProfile: "bge-base", searchProfileUsable: usable }),
      { initialProps: { usable: true }, wrapper: queryClientWrapper() },
    );
    await waitFor(() =>
      expect(result.current.errorMessage).toContain("does not match this request"),
    );
    expect(result.current.resultPage).toBeUndefined();
    rerender({ usable: false });
    apiMocks.searchPage.mockResolvedValue(searchResultsPage(["items:similar"]));
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        query: "",
        similarSeed: "items:seed",
        mode: "similar_records",
      }),
    );
    await waitFor(() => expect(result.current.resultPage?.rows).toHaveLength(1));
    expect(
      apiMocks.searchPage.mock.calls[apiMocks.searchPage.mock.calls.length - 1]?.[0],
    ).toMatchObject({
      search_profile: "bge-base",
      criteria: { kind: "similar_records" },
    });
  });

  it("does not adopt a generation when an abandoned criterion finishes late", async () => {
    history.replaceState(null, "", "/search?similar=items:first");
    const first = deferred<SearchResultsPage>();
    apiMocks.searchPage
      .mockReturnValueOnce(first.promise)
      .mockResolvedValue(searchResultsPage([], { generation: "generation-b" }));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        similarSeed: "items:second",
      }),
    );
    await waitFor(() =>
      expect(result.current.resultPage?.artifact_generation).toBe("generation-b"),
    );
    await act(async () =>
      first.resolve(searchResultsPage([], { generation: "generation-a" })),
    );
    expect(result.current.resultPage?.artifact_generation).toBe("generation-b");
    act(() => result.current.setPageNumber(2));
    await waitFor(() =>
      expect(apiMocks.searchPage).toHaveBeenLastCalledWith(
        expect.objectContaining({
          expected_artifact_generation: "generation-b",
          page: { number: 2, size: 25 },
        }),
        expect.any(AbortSignal),
      ),
    );
  });

  it("adopts the first generation without repeating retrieval", async () => {
    seedSearchUrl();
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() =>
      expect(result.current.resultPage?.artifact_generation).toBe("generation-a"),
    );
    await delay(40);
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
    expect(apiMocks.searchPage).toHaveBeenCalledWith(
      expect.objectContaining({ expected_artifact_generation: null }),
      expect.any(AbortSignal),
    );
  });

  it("pauses a changed generation and refreshes page one without retaining late A reads", async () => {
    seedSearchUrl();
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const oldDetail = deferred<ReturnType<typeof detailFixture>>();
    const oldPage = deferred<SearchResultsPage>();
    const oldCatalog = deferred<FilterEditorView>();
    const freshPage = deferred<SearchResultsPage>();
    const freshCatalog = deferred<FilterEditorView>();
    apiMocks.searchPage
      .mockResolvedValueOnce(searchResultsPage(["spell:old"]))
      .mockReturnValueOnce(oldPage.promise)
      .mockRejectedValueOnce(
        new AtlasApiError(409, "The reference collection has changed", {
          code: "artifact_generation_changed",
          message: "The reference collection has changed",
          recoverable_action: "refresh_results",
        }),
      )
      .mockReturnValue(freshPage.promise);
    apiMocks.getRecordDetail.mockReturnValueOnce(oldDetail.promise).mockResolvedValue({
      ...detailFixture("spell:heal"),
      artifact_generation: "generation-b",
    });
    apiMocks.discoverFilterEditor
      .mockResolvedValueOnce(filterEditor())
      .mockReturnValueOnce(oldCatalog.promise)
      .mockReturnValue(freshCatalog.promise);
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(client),
      },
    );
    await waitFor(() =>
      expect(result.current.resultPage?.artifact_generation).toBe("generation-a"),
    );
    act(() => result.current.selectRecord("spell:heal"));
    await waitFor(() => expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(1));
    // A background catalog read can remain queued when another page reports B.
    act(() => {
      void client.invalidateQueries({ queryKey: ["filter-discovery"] });
    });
    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2));
    act(() => {
      void client.invalidateQueries({ queryKey: ["results"] });
    });
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(result.current.resultsPaused).toBe(true));
    expect(result.current.pageNumber).toBe(2);
    act(() => result.current.setPageNumber(3));
    expect(result.current.pageNumber).toBe(2);
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(3);
    client.setQueryData(["encounter", "draft"], { name: "Preserved encounter" });
    client.setQueryData(["saved-list", "draft"], { name: "Preserved list" });
    const criteria = result.current.search;
    await act(async () => result.current.refreshResults());
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(4));
    expect(apiMocks.searchPage).toHaveBeenLastCalledWith(
      {
        ranking: "score",
        search_profile: "bge-base",
        criteria: { kind: "text_search", query: "dirge", filter: null, mode: "hybrid" },
        page: { number: 1, size: 25 },
        expected_artifact_generation: null,
      },
      expect.any(AbortSignal),
    );
    expect(result.current.resultPage).toBeUndefined();
    expect(result.current.filterEditor).toBeUndefined();
    expect(result.current.search).toEqual(criteria);
    expect(result.current.selectedRecordKey).toBe("spell:heal");
    expect(client.getQueryData(["encounter", "draft"])).toEqual({
      name: "Preserved encounter",
    });
    expect(client.getQueryData(["saved-list", "draft"])).toEqual({
      name: "Preserved list",
    });
    expect(apiMocks.getRecordDetail.mock.calls[0][2].aborted).toBe(true);
    expect(apiMocks.discoverFilterEditor.mock.calls[1][1].aborted).toBe(true);
    await act(async () => {
      oldDetail.resolve(detailFixture("spell:heal", "Old A detail"));
      oldCatalog.resolve(filterEditor());
      oldPage.resolve(searchResultsPage(["spell:late-old"]));
    });
    expect(result.current.reading.detail?.record.title).not.toBe("Old A detail");
    expect(result.current.filterEditor).toBeUndefined();
    expect(
      client
        .getQueryCache()
        .findAll({ queryKey: ["results"] })
        .some((query) => query.queryKey.includes("generation-a")),
    ).toBe(false);
    const editorB = {
      ...filterEditor(),
      artifact_generation: "generation-b",
      groups: [],
    };
    await act(async () => {
      freshPage.resolve(
        searchResultsPage(["spell:new"], { generation: "generation-b" }),
      );
      freshCatalog.resolve(editorB);
    });
    await waitFor(() =>
      expect(result.current.resultPage?.artifact_generation).toBe("generation-b"),
    );
    expect(result.current.resultPage?.page.number).toBe(1);
    expect(result.current.resultPage?.rows[0].record.record_key).toBe("spell:new");
    expect(result.current.filterEditor).toEqual(editorB);
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(4);
    expect(window.location.search).toBe("?search_profile=bge-base&q=dirge");
  });

  it("keeps remembered Kind while editing an authoritative text URL, then clears it on a Kind edit", async () => {
    const preference = { include: ["creature"], exclude: [] };
    localStorage.setItem(KIND_PREFERENCE_KEY, JSON.stringify(preference));
    seedSearchUrl("heal");
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() => expect(result.current.filterEditor).toBeDefined());
    expect(result.current.search.filter).toBeNull();
    act(() =>
      result.current.setSearch({ ...result.current.search, query: "Fireball" }),
    );
    expect(JSON.parse(localStorage.getItem(KIND_PREFERENCE_KEY)!)).toEqual(preference);
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: { kind: "in", field: "kind", values: ["spell"] },
      }),
    );
    expect(JSON.parse(localStorage.getItem(KIND_PREFERENCE_KEY)!)).toEqual({
      include: ["spell"],
      exclude: [],
    });
    act(() => result.current.setSearch({ ...result.current.search, filter: null }));
    expect(localStorage.getItem(KIND_PREFERENCE_KEY)).toBeNull();
  });

  it("debounces search-page requests while preserving immediate search state", async () => {
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
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
    expect(result.current.resultsLoading).toBe(true);
    expect(result.current.resultPage).toBeUndefined();
    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(window.location.search).toBe("?search_profile=bge-base&q=fi");
    await delay(150);
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(result.current.resultsLoading).toBe(true);

    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    const request = apiMocks.searchPage.mock.calls[0][0] as SearchResultsRequest;
    expect(request.criteria).toMatchObject({
      kind: "text_search",
      query: "fi",
    });
  });

  it("can disable search workspace queries for non-search routes", async () => {
    const { result } = renderHook(
      () => useSearchWorkspace({ enabled: false, searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    expect(apiMocks.getReadiness).not.toHaveBeenCalled();
    act(() => result.current.refresh());
    await delay(50);

    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterEditor).not.toHaveBeenCalled();
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    expect(result.current.resultsLoading).toBe(false);
  });

  it("keeps preview selection explicit and restores the search URL when cleared", async () => {
    seedSearchUrl();
    apiMocks.searchPage.mockResolvedValue(
      searchResultsPage(["spell:dirge-of-doom", "spell:heal"]),
    );
    apiMocks.getRecordDetail.mockResolvedValue(detailFixture("spell:heal"));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() =>
      expect(result.current.resultPage?.rows[0]?.record.record_key).toBe(
        "spell:dirge-of-doom",
      ),
    );

    expect(result.current.selectedRecordKey).toBeNull();
    act(() => result.current.selectRecord("spell:heal"));
    expect(result.current.selectedRecordKey).toBe("spell:heal");
    await waitFor(() =>
      expect(window.location.pathname).toBe("/search/records/spell:heal"),
    );
    expect(window.location.search).toBe("?search_profile=bge-base&q=dirge");

    act(() => result.current.selectRecord(null));
    expect(result.current.selectedRecordKey).toBeNull();
    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(window.location.search).toBe("?search_profile=bge-base&q=dirge");
  });

  it("writes shared structured predicates in URL state", async () => {
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

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
              field: "kind",
              values: ["affliction", "character"],
            },
            {
              kind: "not",
              predicate: {
                kind: "in",
                clause_id: "kind-exclude_any",
                field: "kind",
                values: ["character_option"],
              },
            },
          ],
        },
      }),
    );

    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(window.location.search).toBe(
      searchStateQueryString(result.current.search, "bge-base"),
    );
  });

  it("does not query collection states as scalar values", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(
      editorFixture([fieldFixture("embedded.equipment", "collection")]),
    );

    renderHook(() => useSearchWorkspace({ searchProfile: "bge-base" }), {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    await delay(50);

    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ field_id: "embedded.equipment" }),
      expect.any(AbortSignal),
    );
  });

  it("refreshes applicability while retaining the editor when filter criteria change", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

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
              field: "kind",
              values: ["creature"],
            },
          ],
        },
      }),
    );

    await waitFor(() => expect(result.current.search.filter).not.toBeNull());
    expect(result.current.filterEditor).toEqual(filterEditor());
    expect(apiMocks.discoverFilterEditor).toHaveBeenCalledTimes(2);
    expect(apiMocks.discoverFilterEditor).toHaveBeenLastCalledWith(
      { selected_field_ids: [], filter: result.current.search.filter },
      expect.any(AbortSignal),
    );
  });

  it("does not request value discovery for closed catalog choices or hidden controls", async () => {
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditor());
    renderHook(() => useSearchWorkspace({ searchProfile: "bge-base" }), {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    await delay(50);
    expect(apiMocks.discoverFilterValues).not.toHaveBeenCalled();
  });

  it("sends complete criteria and the adopted generation on later pages", async () => {
    seedSearchUrl();
    apiMocks.searchPage
      .mockResolvedValueOnce(searchResultsPage(["spell:dirge-of-doom"]))
      .mockResolvedValue(searchResultsPage(["spell:heal"], { pageNumber: 2 }));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));

    act(() => result.current.setPageNumber(2));

    await waitFor(() =>
      expect(apiMocks.searchPage).toHaveBeenCalledWith(
        {
          ranking: "score",
          search_profile: "bge-base",
          expected_artifact_generation: "generation-a",
          criteria: {
            kind: "text_search",
            query: "dirge",
            filter: null,
            mode: "hybrid",
          },
          page: { number: 2, size: DEFAULT_SEARCH_STATE.pageSize },
        },
        expect.any(AbortSignal),
      ),
    );
    expect(result.current.pageNumber).toBe(2);
  });
  it("keeps selected-detail failures pane-local and retries without changing search/results", async () => {
    seedSearchUrl();
    apiMocks.getRecordDetail
      .mockRejectedValueOnce(new Error("Selected record unavailable"))
      .mockResolvedValueOnce(detailFixture("spell:heal"));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    act(() => result.current.selectRecord("spell:heal"));
    await waitFor(() =>
      expect(result.current.reading.error?.message).toBe("Selected record unavailable"),
    );
    const search = result.current.search;
    const rows = result.current.resultPage;
    expect(result.current.errorMessage).toBeNull();
    act(() => result.current.reading.retry());
    await waitFor(() => expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(2));
    expect(result.current.search).toEqual(search);
    expect(result.current.resultPage).toEqual(rows);
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
  });

  it("keeps previous rows while marking page transitions as refreshing", async () => {
    seedSearchUrl();
    const nextPage = deferred<SearchResultsPage>();
    apiMocks.searchPage
      .mockResolvedValueOnce(searchResultsPage(["spell:dirge-of-doom"]))
      .mockReturnValue(nextPage.promise);
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() =>
      expect(result.current.resultPage?.rows[0]?.record.record_key).toBe(
        "spell:dirge-of-doom",
      ),
    );

    act(() => result.current.setPageNumber(2));

    await waitFor(() => expect(result.current.resultsRefreshing).toBe(true));
    expect(result.current.resultsLoading).toBe(false);
    expect(result.current.resultPage?.rows[0]?.record.record_key).toBe(
      "spell:dirge-of-doom",
    );

    await act(async () => {
      nextPage.resolve(searchResultsPage(["spell:heal"], { pageNumber: 2 }));
      await nextPage.promise;
    });

    await waitFor(() => expect(result.current.resultsRefreshing).toBe(false));
    expect(result.current.resultPage?.page.number).toBe(2);
    expect(result.current.resultPage?.rows[0]?.record.record_key).toBe("spell:heal");
  });

  it("resets page execution when search changes from a later page", async () => {
    seedSearchUrl();
    apiMocks.searchPage
      .mockResolvedValueOnce(searchResultsPage(["spell:dirge-of-doom"]))
      .mockResolvedValue(searchResultsPage(["spell:heal"], { pageNumber: 2 }));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));
    act(() => result.current.setPageNumber(2));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));

    act(() =>
      result.current.setSearch({
        ...DEFAULT_SEARCH_STATE,
        query: "heal",
        mode: "text_search",
      }),
    );

    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(3));
    expect(result.current.pageNumber).toBe(1);
    const request = apiMocks.searchPage.mock.calls[2][0] as SearchResultsRequest;
    expect(request.page).toEqual({
      number: 1,
      size: DEFAULT_SEARCH_STATE.pageSize,
    });
    expect(request.criteria).toMatchObject({
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
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(1));

    history.pushState(
      null,
      "",
      `/search/records/spell%3Aacid-arrow${searchStateQueryString(restoredSearch)}`,
    );
    act(() => window.dispatchEvent(new PopStateEvent("popstate")));

    expect(result.current.search.query).toBe("acid");
    expect(result.current.selectedRecordKey).toBe("spell:acid-arrow");
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    const request = apiMocks.searchPage.mock.calls[1][0] as SearchResultsRequest;
    expect(request.page).toEqual({
      number: 1,
      size: DEFAULT_SEARCH_STATE.pageSize,
    });
    expect(request.criteria).toMatchObject({
      kind: "text_search",
      query: "acid",
    });
  });
});

function queryClientWrapper(
  queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  }),
) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <RouterTestContext>{children}</RouterTestContext>
      </QueryClientProvider>
    );
  };
}

function seedSearchUrl(query = "dirge") {
  history.replaceState(null, "", `/search?q=${encodeURIComponent(query)}`);
}

function searchResultsPage(
  recordKeys: string[] = [],
  options: { pageNumber?: number; generation?: string; profile?: string } = {},
): SearchResultsPage {
  return {
    relationship_clauses: [],
    sort: { kind: "relevance" } as const,
    ranking: "score" as const,
    search_profile: options.profile ?? "bge-base",
    artifact_generation: options.generation ?? "generation-a",
    mode: { kind: "text_search", query: "", mode: "hybrid" },
    page: {
      number: options.pageNumber || 1,
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

function filterEditor(): FilterEditorView {
  const editor = editorFixture([
    fieldFixture("kind", "string", "Kind"),
    fieldFixture("common.traits", "set", "Traits"),
    fieldFixture("source.pack"),
    fieldFixture("level", "number"),
    fieldFixture("spell.basic_save"),
  ]);
  editor.groups[0].fields[4].placement = "addable";
  return editor;
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

it("pages retained link evidence with its producing OR tree while a new NOT tree is pending", async () => {
  vi.clearAllMocks();
  apiMocks.discoverFilterEditor.mockResolvedValue(editorFixture());
  const leaf = linkLeaf(
    {
      direction: "incoming",
      include_contents: true,
      selection: rootNavigation("items:armor"),
    },
    "link",
  );
  const original: QueryPredicate = {
    kind: "any_of",
    children: [leaf, { kind: "boolean_constant", value: false }],
  };
  history.replaceState(
    null,
    "",
    "/search" + searchStateQueryString({ ...DEFAULT_SEARCH_STATE, filter: original }),
  );
  const page = searchResultsPage(["actors:goblin"]);
  page.sort = { kind: "attributes", rules: [{ field: "name", direction: "asc" }] };
  page.mode = { kind: "list_records" };
  apiMocks.searchPage
    .mockImplementationOnce(async (request: SearchResultsRequest) => {
      const filter = request.criteria.filter!;
      const clause = filter.kind === "any_of" ? filter.children[0] : null;
      if (clause?.kind !== "relationship") throw new Error("Missing expected link");
      page.relationship_clauses = [
        {
          clause_id: clause.clause_id!,
          relationship: clause.relationship,
          title: "Leather Armor",
          record: summaryFixture("items:armor", "Leather Armor"),
        },
      ];
      page.rows[0].relationship_evidence = [
        {
          clause_id: clause.clause_id!,
          occurrences: [],
          page: {
            number: 1,
            size: 1,
            count: 0,
            total: 2n,
            has_more: true,
            next_page: 2,
          },
        },
      ];
      return page;
    })
    .mockImplementation(() => new Promise(() => {}));
  apiMocks.getRelationshipOccurrences.mockImplementation(() => new Promise(() => {}));
  let live!: ReturnType<typeof useSearchWorkspace>;
  function Harness() {
    live = useSearchWorkspace({ searchProfile: "bge-base" });
    const p = live.resultPage;
    return p && live.resultFilter ? (
      <RelationshipEvidence
        evidence={p.rows[0].relationship_evidence[0]}
        summary={p.relationship_clauses[0]}
        filter={live.resultFilter}
        generation={p.artifact_generation}
        resultKey="actors:goblin"
      />
    ) : null;
  }
  render(<Harness />, { wrapper: queryClientWrapper() });
  await screen.findByRole("button", { name: "Next links page" });
  const producing = live.resultFilter;
  act(() =>
    live.setSearch({ ...live.search, filter: { kind: "not", predicate: leaf } }),
  );
  await waitFor(() =>
    expect(apiMocks.searchPage.mock.lastCall?.[0].criteria.filter.kind).toBe("not"),
  );
  expect(live.resultFilter).toEqual(producing);
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  await waitFor(() =>
    expect(apiMocks.getRelationshipOccurrences).toHaveBeenCalledWith(
      expect.objectContaining({
        filter: producing,
        clause_id: page.relationship_clauses[0].clause_id,
        expected_artifact_generation: "generation-a",
        page: { number: 2, size: 1 },
      }),
      expect.any(AbortSignal),
    ),
  );
});
