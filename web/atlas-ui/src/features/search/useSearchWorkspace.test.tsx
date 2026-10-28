import { atlasRouter } from "../../app/router";
import { recordDestination, searchDestination } from "../../app/destinations";
import { RouterTestContext } from "../../test/router";
import { App, ConfigProvider } from "antd";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { SearchSettings } from "../../shared/search/SearchSettings";
import {
  similarityConfigurationFixture,
  similarityPartsFixture,
} from "../../test/similarityFixtures";
import type { ResultSort, SimilarityOptions } from "../../generated/atlas";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
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
  RecordDetailView,
  RecordNavigationView,
} from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
  effectiveResultSort,
} from "../../shared/filters/searchState";
import { KIND_PREFERENCE_KEY } from "./workspaceState";
import { AtlasApiError } from "../../api/atlasApi";
import { AtlasConnectionError } from "../../api/availability";
import { useSearchWorkspace } from "./useSearchWorkspace";
import { DEFAULT_SEARCH_PREFERENCES } from "../../shared/search/searchPreferences";
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
  getSimilarityParts: vi.fn(),
}));
const recordSimilarity: SimilarityOptions = {
  method: "passages",
  decay: 0.5,
  parts: null,
  candidate_parts: "any",
};

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterValues: apiMocks.discoverFilterValues,
  discoverFilterCounts: apiMocks.discoverFilterCounts,
  getReadiness: apiMocks.getReadiness,
  getRecordDetail: apiMocks.getRecordDetail,
  searchPage: apiMocks.searchPage,
  getRelationshipOccurrences: apiMocks.getRelationshipOccurrences,
  getSimilarityParts: apiMocks.getSimilarityParts,
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

  it("restores copied pages from the URL and pushes page history for Back and Forward", async () => {
    history.replaceState({ atlasSearchPage: 19 }, "", "/search?q=fire&page=4");
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
      searchResultsPage(["spell:fireball"], { pageNumber: request.page.number }),
    );
    apiMocks.getRecordDetail.mockImplementation(async (key: string) =>
      detailFixture(key),
    );
    const first = renderHook(() => useSearchWorkspace({ searchProfile: "bge-base" }), {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() => expect(first.result.current.resultPage?.page.number).toBe(4));
    expect(apiMocks.searchPage.mock.calls[0][0].page.number).toBe(4);
    const initialLength = history.length;
    act(() => first.result.current.setPageNumber(5));
    await waitFor(() => expect(first.result.current.resultPage?.page.number).toBe(5));
    expect(new URLSearchParams(location.search).get("page")).toBe("5");
    expect(history.length).toBe(initialLength + 1);
    act(() => history.back());
    await waitFor(() => expect(first.result.current.pageNumber).toBe(4));
    act(() => history.forward());
    await waitFor(() => expect(first.result.current.pageNumber).toBe(5));
    act(() => first.result.current.selectRecord("spell:fireball"));
    await waitFor(() => expect(first.result.current.reading.detail).toBeDefined());
    expect(new URLSearchParams(location.search).get("page")).toBe("5");
    const copied = atlasRouter.latestLocation.href;
    first.unmount();
    history.replaceState(null, "", copied);
    const reloaded = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() =>
      expect(reloaded.result.current.resultPage?.page.number).toBe(5),
    );
    expect(apiMocks.searchPage.mock.lastCall?.[0].page.number).toBe(5);
  });

  it("keeps a shared outage distinct from successful empty results", async () => {
    history.replaceState(null, "", "/search?q=fire");
    apiMocks.searchPage.mockRejectedValue(
      new AtlasConnectionError(false, new TypeError("private proxy diagnostic")),
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultsFailed).toBe(true));
    expect(result.current.errorMessage).toBeNull();
    expect(result.current.resultPage).toBeUndefined();
  });

  it("restores an explicit profile and page together from browser history", async () => {
    history.replaceState(null, "", "/search?q=fire&search_profile=bge-base&page=5");
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
      searchResultsPage([], {
        pageNumber: request.page.number,
        profile: request.search_profile,
      }),
    );
    const { result, rerender } = renderHook(
      ({ profile }) => useSearchWorkspace({ searchProfile: profile }),
      { initialProps: { profile: "bge-base" }, wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(5));
    act(() => {
      history.pushState(
        null,
        "",
        "/search?q=fire&search_profile=embeddinggemma&page=4",
      );
      window.dispatchEvent(new PopStateEvent("popstate"));
      rerender({ profile: "embeddinggemma" });
    });
    await waitFor(() =>
      expect(result.current.resultPage?.search_profile).toBe("embeddinggemma"),
    );
    expect(result.current.pageNumber).toBe(4);
    expect(apiMocks.searchPage.mock.lastCall?.[0].page.number).toBe(4);
    expect(new URLSearchParams(location.search).get("page")).toBe("4");
  });

  it("replaces an out-of-range page once using current response bounds and preserves exact reading", async () => {
    history.replaceState(
      { retained: "history" },
      "",
      "/search/records/spell:heal?q=heal&limit=5&page=5&rank=4&record_view=explore#description",
    );
    apiMocks.getRecordDetail.mockImplementation(
      async (key: string, request?: RecordDetailRequest) => ({
        ...detailFixture(key),
        selected: { ...rootNavigation(key), spell_rank: request?.spell_rank },
      }),
    );
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsPage(request.page.number === 3 ? ["spell:heal"] : [], {
        pageNumber: request.page.number,
      }),
      page: {
        number: request.page.number,
        size: 5,
        count: request.page.number === 3 ? 2 : 0,
        total: 12n,
        has_more: false,
        next_page: null,
      },
    }));
    const initialLength = history.length;
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(3));
    expect(
      apiMocks.searchPage.mock.calls.map(([request]) => request.page.number),
    ).toEqual([5, 3]);
    expect(history.length).toBe(initialLength);
    expect(history.state.retained).toBe("history");
    expect(location.hash).toBe("#description");
    const params = new URLSearchParams(location.search);
    expect(params.get("page")).toBe("3");
    expect(params.get("rank")).toBe("4");
    expect(params.get("record_view")).toBe("explore");
    expect(result.current.selectedRecordKey).toBe("spell:heal");
  });

  it("normalizes an empty current result set to page one without a redirect loop", async () => {
    history.replaceState(null, "", "/search?q=ice&limit=5&page=6");
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsPage(),
      page: {
        number: request.page.number,
        size: 5,
        count: 0,
        total: 0n,
        has_more: false,
        next_page: null,
      },
    }));
    const initialLength = history.length;
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(1));
    expect(
      apiMocks.searchPage.mock.calls.map(([request]) => request.page.number),
    ).toEqual([6, 1]);
    expect(new URLSearchParams(location.search).has("page")).toBe(false);
    expect(history.length).toBe(initialLength);
    await delay(40);
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(2);
  });

  it("does not clamp a requested page with placeholder or late prior-criteria totals", async () => {
    seedSearchUrl("fire");
    const nextPage = deferred<SearchResultsPage>();
    apiMocks.searchPage
      .mockResolvedValueOnce({
        ...searchResultsPage(["spell:fire"]),
        page: { ...searchResultsPage().page, total: 1n, count: 1 },
      })
      .mockReturnValueOnce(nextPage.promise);
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.resultPage).toBeDefined());
    act(() => result.current.setPageNumber(5));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    expect(result.current.pageNumber).toBe(5);
    expect(new URLSearchParams(location.search).get("page")).toBe("5");
    act(() => result.current.setSearch({ ...result.current.search, query: "ice" }));
    await act(async () =>
      nextPage.resolve({
        ...searchResultsPage([], { pageNumber: 5 }),
        page: { ...searchResultsPage().page, number: 5, total: 1n },
      }),
    );
    expect(new URLSearchParams(location.search).get("page")).toBeNull();
    expect(new URLSearchParams(location.search).get("q")).toBe("ice");
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
      ...(request.criteria.kind === "similar_records" && request.criteria.similarity
        ? { similarity: similarityConfigurationFixture(request.criteria.similarity) }
        : {}),
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
      expect(result.current.errorMessage).toBe("Try searching again."),
    );
    expect(result.current.resultPage).toBeUndefined();
    expect(
      apiMocks.searchPage.mock.lastCall?.[0].expected_artifact_generation,
    ).toBeNull();
  });

  it("retains loaded Similar rows and the seed title as updating while a new order is pending", async () => {
    history.replaceState(null, "", "/search?similar=items:armor");
    const original: SearchResultsPage = {
      ...searchResultsPage(["items:old"], { similarity: recordSimilarity }),
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
    await waitFor(() => {
      expect(result.current.errorMessage).toBeNull();
      expect(result.current.resultPage).toBeDefined();
    });
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
      criteria: { search_content: "all", kind: "text_search", query: "harrowkin" },
    });
  });

  it.each([
    { rank: undefined, pending: false },
    { rank: 5, pending: false },
    { rank: 5, pending: true },
  ])(
    "applies settings while retaining exact match, focus and requested rank $rank (pending=$pending)",
    async ({ rank, pending }) => {
      const recordKey = "spell:heal";
      history.replaceState(
        null,
        "",
        "/search/records/" +
          recordKey +
          "?q=heal&page=2&record_view=explore&record_tab=similar",
      );
      const match: RecordNavigationView = {
        ...rootNavigation(recordKey),
        field: "/system/description/value",
        passage: {
          kind: "html_section",
          prepared_html_sha256: "prepared-a",
          canonical_text_sha256: "text-a",
          selection_version: "selection-v1",
          section_ordinal: 1,
          label: "Healing",
          chunk_bytes: { start: 10, end: 50 },
        },
        source_fingerprint: "source-a",
        ...(rank === undefined ? {} : { spell_rank: rank }),
      };
      const landing: RecordNavigationView = {
        ...rootNavigation(recordKey),
        embedded_path: [{ collection: "/items", identity: { Stable: "source-entry" } }],
        source_fingerprint: "source-a",
      };
      const section = deferred<RecordDetailView>();
      const selected = {
        ...detailFixture(recordKey),
        selected: { ...match, spell_rank: rank ?? 3 },
      };
      apiMocks.getRecordDetail.mockImplementation(
        (key: string, request?: RecordDetailRequest) =>
          request?.fields?.length
            ? pending
              ? section.promise
              : Promise.resolve(selected)
            : Promise.resolve({
                ...detailFixture(key),
                selected: { ...rootNavigation(key), spell_rank: 3 },
              }),
      );
      apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
        ...searchResultsPage([], {
          profile: request.search_profile,
          pageNumber: request.page.number,
        }),
        ranking: request.ranking ?? "score",
      }));
      const { result, rerender } = renderHook(
        ({ profile }) => useSearchWorkspace({ searchProfile: profile }),
        {
          initialProps: { profile: "bge-base" },
          wrapper: queryClientWrapper(),
        },
      );
      await waitFor(() =>
        expect(result.current.resultPage?.search_profile).toBe("bge-base"),
      );
      await waitFor(() =>
        expect(result.current.reading.detail?.selected.spell_rank).toBe(3),
      );
      const exactRequest = { navigation: rootNavigation(recordKey), match, landing };
      act(() => result.current.reading.inspect(exactRequest));
      await waitFor(() =>
        expect(apiMocks.getRecordDetail).toHaveBeenLastCalledWith(
          recordKey,
          expect.objectContaining({ fields: [match.field], passage: match.passage }),
          expect.any(AbortSignal),
        ),
      );
      if (!pending) {
        await waitFor(() =>
          expect(result.current.reading.detail?.selected).toEqual(selected.selected),
        );
        await waitFor(() =>
          expect(
            selectedNavigation(parseSearchParams(location.search), recordKey),
          ).toEqual(match),
        );
        expect(
          selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
        ).toEqual(landing);
      }
      expect(result.current.reading.request).toBe(exactRequest);
      expect(result.current.reading.canGoBack).toBe(true);
      expect(result.current.pageNumber).toBe(2);
      apiMocks.searchPage.mockClear();
      act(() => {
        result.current.applySettings!(
          { ...result.current.search, ranking: "rrf" },
          "embeddinggemma",
        );
        rerender({ profile: "embeddinggemma" });
      });
      await waitFor(() => expect(result.current.resultPage?.ranking).toBe("rrf"));
      await act(async () => {
        await delay(350);
      });
      expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
      expect(new URLSearchParams(location.search).has("page")).toBe(false);
      expect(new URLSearchParams(location.search).get("record_view")).toBe("explore");
      expect(new URLSearchParams(location.search).get("record_tab")).toBe("similar");
      expect(location.pathname).toBe("/search/records/" + recordKey);
      expect(selectedNavigation(parseSearchParams(location.search), recordKey)).toEqual(
        match,
      );
      expect(
        selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
      ).toEqual(landing);
      expect(result.current.reading.request).toBe(exactRequest);
      expect(result.current.reading.canGoBack).toBe(true);
      if (pending) {
        await act(async () => section.resolve(selected));
        await waitFor(() =>
          expect(result.current.reading.detail?.selected).toEqual(selected.selected),
        );
        expect(
          selectedNavigation(parseSearchParams(location.search), recordKey),
        ).toEqual(match);
        expect(
          selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
        ).toEqual(landing);
      }
      expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
        search_profile: "embeddinggemma",
        ranking: "rrf",
        page: { number: 1 },
      });
    },
  );

  it("applies reloaded B parts without a separate refresh and retires late A reads", async () => {
    history.replaceState(null, "", "/search?similar=hazards%3Abrown-mold");
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const oldPage = deferred<SearchResultsPage>();
    apiMocks.searchPage.mockImplementation((request: SearchResultsRequest) => {
      const similarity =
        request.criteria.kind === "similar_records"
          ? request.criteria.similarity
          : undefined;
      const page = {
        ...searchResultsPage([], {
          generation: request.expected_artifact_generation ?? "generation-a",
        }),
        ...(similarity
          ? { similarity: similarityConfigurationFixture(similarity) }
          : {}),
      };
      return apiMocks.searchPage.mock.calls.length === 2
        ? oldPage.promise
        : Promise.resolve(page);
    });
    apiMocks.getSimilarityParts
      .mockRejectedValueOnce(
        new AtlasApiError(409, "Collection changed", {
          code: "artifact_generation_changed",
          message: "Collection changed",
          recoverable_action: "refresh_results",
        }),
      )
      .mockResolvedValue(similarityPartsFixture());
    apiMocks.getRecordDetail
      .mockImplementationOnce((key: string) =>
        Promise.resolve({ ...detailFixture(key), artifact_generation: "generation-a" }),
      )
      .mockImplementation((key: string) =>
        Promise.resolve({ ...detailFixture(key), artifact_generation: "generation-b" }),
      );
    let workspace!: ReturnType<typeof useSearchWorkspace>;
    function SettingsWorkspace() {
      workspace = useSearchWorkspace({ searchProfile: "bge-base" });
      return (
        <>
          <SearchSettings
            search={workspace.search}
            seed={{ kind: "record", record_key: "hazards:brown-mold" }}
            generation={workspace.resultPage?.artifact_generation}
            onApply={workspace.applySettings!}
          />
          <output aria-label="Result collection">
            {workspace.resultPage?.artifact_generation}
          </output>
        </>
      );
    }
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <SearchProfileProvider>
            <SettingsWorkspace />
          </SearchProfileProvider>
        </App>
      </ConfigProvider>,
      { wrapper: queryClientWrapper(client) },
    );
    await waitFor(() =>
      expect(screen.getByLabelText("Result collection")).toHaveTextContent(
        "generation-a",
      ),
    );
    act(() => {
      workspace.selectRecord("spell:heal");
      workspace.positions.saveList(187);
      workspace.positions.saveReader("spell:heal", 93);
      void client.invalidateQueries({ queryKey: ["results"] });
    });
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledTimes(2));
    await waitFor(() =>
      expect(workspace.reading.detail?.artifact_generation).toBe("generation-a"),
    );
    client.setQueryData(["encounter", "draft"], { name: "Unsaved encounter" });
    fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
    fireEvent.click(screen.getByText("Sections to compare"));
    fireEvent.click(await screen.findByRole("button", { name: "Reload sections" }));
    await screen.findByRole("checkbox", { name: "Description" });
    const selectAll = screen.getByRole("checkbox", { name: "Select all" });
    expect(selectAll).toBeChecked();
    fireEvent.click(selectAll);
    expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
    fireEvent.click(screen.getByRole("checkbox", { name: "Description" }));
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() =>
      expect(screen.getByLabelText("Result collection")).toHaveTextContent(
        "generation-b",
      ),
    );
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(3);
    expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
      expected_artifact_generation: "generation-b",
      criteria: { similarity: { parts: ["description-b"] } },
      page: { number: 1 },
    });
    expect(apiMocks.searchPage.mock.calls[1][1].aborted).toBe(true);
    await act(async () => {
      oldPage.resolve(searchResultsPage(["spell:late-a"]));
      await delay(350);
    });
    expect(workspace.resultPage?.artifact_generation).toBe("generation-b");
    expect(workspace.errorMessage).toBeNull();
    expect(workspace.selectedRecordKey).toBe("spell:heal");
    expect(workspace.reading.request?.navigation.record_key).toBe("spell:heal");
    await waitFor(() =>
      expect(workspace.reading.detail?.artifact_generation).toBe("generation-b"),
    );
    expect(workspace.positions.readList()).toBe(187);
    expect(workspace.positions.readReader("spell:heal")).toBe(93);
    expect(client.getQueryData(["encounter", "draft"])).toEqual({
      name: "Unsaved encounter",
    });
    expect(apiMocks.searchPage).toHaveBeenCalledTimes(3);
    expect(
      client
        .getQueryCache()
        .findAll({ queryKey: ["results"] })
        .some((query) => query.queryKey.includes("generation-a")),
    ).toBe(false);
  });

  it.each([0, 3, 5])(
    "omits a resolved default rank and restores explicitly selected rank %s on reload",
    async (rank) => {
      history.replaceState(null, "", "/search/records/spells:fireball");
      apiMocks.getRecordDetail.mockImplementation((key, selection) => {
        const detail = detailFixture(key);
        detail.selected = {
          ...detail.selected,
          spell_rank: selection?.spell_rank ?? 3,
        };
        return Promise.resolve(detail);
      });
      const first = renderHook(
        () => useSearchWorkspace({ searchProfile: "bge-base" }),
        {
          wrapper: queryClientWrapper(),
        },
      );
      await waitFor(() =>
        expect(first.result.current.reading.detail?.selected.spell_rank).toBe(3),
      );
      expect(new URLSearchParams(location.search).has("rank")).toBe(false);
      act(() =>
        first.result.current.reading.changeRank({
          ...first.result.current.reading.detail!.selected,
          spell_rank: rank,
        }),
      );
      await waitFor(() =>
        expect(new URLSearchParams(location.search).get("rank")).toBe(String(rank)),
      );
      first.unmount();
      const reloaded = renderHook(
        () => useSearchWorkspace({ searchProfile: "bge-base" }),
        {
          wrapper: queryClientWrapper(),
        },
      );
      await waitFor(() =>
        expect(reloaded.result.current.reading.detail?.selected.spell_rank).toBe(rank),
      );
      expect(apiMocks.getRecordDetail).toHaveBeenLastCalledWith(
        "spells:fireball",
        expect.objectContaining({ spell_rank: rank }),
        expect.any(AbortSignal),
      );
    },
  );

  it("preserves externally restored criteria and page while the same record remains selected", async () => {
    const recordKey = "spells:fireball";
    history.replaceState(null, "", "/search/records/" + recordKey + "?q=fire&page=2");
    apiMocks.getRecordDetail.mockResolvedValue(detailFixture(recordKey));
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsPage([recordKey], { pageNumber: request.page.number }),
      page: { ...searchResultsPage().page, number: request.page.number, total: 100n },
    }));
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.reading.detail).toBeDefined());
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(2));
    const readingRequest = result.current.reading.request;
    const visits: string[] = [];
    const unsubscribe = atlasRouter.subscribe("onBeforeNavigate", ({ toLocation }) =>
      visits.push(toLocation.href),
    );
    await act(async () => {
      await atlasRouter.navigate(
        searchDestination({
          selectedRecordKey: recordKey,
          query: "?q=ice&search_profile=bge-base",
          page: 3,
        }),
      );
    });
    await waitFor(() => expect(result.current.search.query).toBe("ice"));
    await waitFor(() => expect(result.current.resultPage?.page.number).toBe(3));
    unsubscribe();
    expect(result.current.reading.request).toBe(readingRequest);
    expect(new URLSearchParams(location.search).get("q")).toBe("ice");
    expect(new URLSearchParams(location.search).get("page")).toBe("3");
    for (const href of visits) {
      const params = new URL(href, location.origin).searchParams;
      expect(params.get("q")).toBe("ice");
      expect(params.get("page")).toBe("3");
    }
  });

  it("restores incoming focus into a retained root reader before serializing its URL", async () => {
    const recordKey = "actors:retained";
    history.replaceState(null, "", "/search/records/" + recordKey);
    apiMocks.getRecordDetail.mockResolvedValue(detailFixture(recordKey));
    const { result, rerender } = renderHook(
      ({ enabled }) => useSearchWorkspace({ enabled, searchProfile: "bge-base" }),
      { initialProps: { enabled: true }, wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.reading.detail).toBeDefined());
    const landing: RecordNavigationView = {
      ...rootNavigation(recordKey),
      embedded_path: [{ collection: "/items", identity: { Stable: "seize" } }],
    };
    const visitedFocus: Array<RecordNavigationView | undefined> = [];
    const unsubscribe = atlasRouter.subscribe("onBeforeNavigate", ({ toLocation }) => {
      if (toLocation.pathname === "/search/records/" + recordKey)
        visitedFocus.push(selectedNavigation(toLocation.search, recordKey, "focus"));
    });
    rerender({ enabled: false });
    await act(async () => {
      await atlasRouter.navigate(recordDestination({ recordKey }));
      await atlasRouter.navigate(
        searchDestination({ selectedRecordKey: recordKey, landing }),
      );
    });
    rerender({ enabled: true });
    await waitFor(() => expect(result.current.reading.landing).toEqual(landing));
    expect(
      selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
    ).toEqual(landing);
    expect(result.current.reading.request).toEqual({
      navigation: rootNavigation(recordKey),
      landing,
    });
    unsubscribe();
    expect(visitedFocus.length).toBeGreaterThan(0);
    for (const focus of visitedFocus) expect(focus).toEqual(landing);
    for (const [key, selection] of apiMocks.getRecordDetail.mock.calls) {
      expect(key).toBe(recordKey);
      expect(selection).toBeUndefined();
    }
    const otherLanding = {
      ...landing,
      embedded_path: [{ collection: "/items", identity: { Stable: "other" } }],
    };
    await act(async () => {
      await atlasRouter.navigate(
        searchDestination({ selectedRecordKey: recordKey, landing: otherLanding }),
      );
    });
    await waitFor(() => expect(result.current.reading.landing).toEqual(otherLanding));
    expect(
      selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
    ).toEqual(otherLanding);
    await act(async () => atlasRouter.history.back());
    await waitFor(() => expect(result.current.reading.landing).toEqual(landing));
    expect(
      selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
    ).toEqual(landing);
    act(() =>
      result.current.reading.changeRank({
        ...rootNavigation(recordKey),
        spell_rank: 0,
      }),
    );
    await waitFor(() =>
      expect(new URLSearchParams(location.search).get("rank")).toBe("0"),
    );
    expect(
      selectedNavigation(parseSearchParams(location.search), recordKey, "focus"),
    ).toEqual(landing);
  });

  it("retains the exact local match request when its URL round trip reorders passage fields", async () => {
    const recordKey = "actors:passage";
    history.replaceState(null, "", "/search/records/" + recordKey + "?q=variance");
    const match: RecordNavigationView = {
      record_key: recordKey,
      embedded_path: [],
      field: "/system/description/value",
      passage: {
        kind: "html_section",
        prepared_html_sha256: "prepared-a",
        canonical_text_sha256: "text-a",
        selection_version: "selection-v1",
        section_ordinal: 1,
        label: "Brood Variance",
        chunk_bytes: { start: 10, end: 50 },
      },
      source_fingerprint: "source-a",
    };
    apiMocks.getRecordDetail.mockImplementation(
      (key: string, request?: RecordDetailRequest) => {
        const detail = detailFixture(key);
        return Promise.resolve(request ? { ...detail, selected: match } : detail);
      },
    );
    const { result } = renderHook(
      () => useSearchWorkspace({ searchProfile: "bge-base" }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(result.current.reading.detail).toBeDefined());
    const request = { navigation: rootNavigation(recordKey), match };
    act(() => result.current.reading.inspect(request));
    await waitFor(() => expect(result.current.reading.detail?.selected).toEqual(match));
    await waitFor(() =>
      expect(selectedNavigation(parseSearchParams(location.search), recordKey)).toEqual(
        match,
      ),
    );
    expect(result.current.reading.request).toBe(request);
    expect(result.current.reading.request?.match).toBe(match);
    expect(apiMocks.getRecordDetail).toHaveBeenLastCalledWith(
      recordKey,
      {
        record_key: recordKey,
        embedded_path: [],
        fields: [match.field],
        passage: match.passage,
        source_fingerprint: "source-a",
      },
      expect.any(AbortSignal),
    );
    expect(apiMocks.getRecordDetail).toHaveBeenCalledTimes(3);
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
        similarity: similarityConfigurationFixture(recordSimilarity),
        page: { ...searchResultsPage().page, number: request.page.number, total: 100n },
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
      search_content: "all",
      source_sections: "all",
      seed_record_key: "items:seed",
      similarity: recordSimilarity,
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
        search_content: "all",
        source_sections: "all",
        seed_record_key: "items:seed",
        similarity: recordSimilarity,
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
      selectedNavigation(
        parseSearchParams(window.location.search),
        exact.navigation.record_key,
      ),
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
      expect(result.current.errorMessage).toBe("Try searching again."),
    );
    expect(result.current.resultPage).toBeUndefined();
    rerender({ usable: false });
    apiMocks.searchPage.mockResolvedValue(
      searchResultsPage(["items:similar"], { similarity: recordSimilarity }),
    );
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
    const similarity = similarityConfigurationFixture({
      method: "passages",
      decay: 0.5,
      parts: null,
      candidate_parts: "any",
    });
    apiMocks.searchPage.mockReturnValueOnce(first.promise).mockResolvedValue({
      ...searchResultsPage([], { generation: "generation-b" }),
      similarity,
    });
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
      first.resolve({
        ...searchResultsPage([], { generation: "generation-a" }),
        similarity,
      }),
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
        criteria: {
          search_content: "all",
          kind: "text_search",
          query: "dirge",
          filter: null,
          mode: "hybrid",
        },
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
    expect(window.location.search).toBe(
      "?search_profile=bge-base&ranking=score&search_content=all&q=dirge",
    );
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
    expect(window.location.search).toBe(
      "?search_profile=bge-base&search_content=all&q=fi",
    );
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

  it("debounces typing with the current persisted rank-based preference and sends it once", async () => {
    apiMocks.searchPage.mockResolvedValue({
      ...searchResultsPage(),
      ranking: "rrf",
    });
    const { result } = renderHook(
      () =>
        useSearchWorkspace({
          searchProfile: "bge-base",
          searchPreferences: { ...DEFAULT_SEARCH_PREFERENCES, ranking: "rrf" },
        }),
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(apiMocks.discoverFilterEditor).toHaveBeenCalled());
    expect(result.current.search.ranking).toBe("rrf");
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    try {
      act(() => {
        result.current.setSearch({ ...result.current.search, query: "f" });
        result.current.setSearch({ ...result.current.search, query: "fi" });
      });
      expect(result.current.search.query).toBe("fi");
      expect(result.current.resultsLoading).toBe(true);
      await act(async () => vi.advanceTimersByTimeAsync(299));
      expect(new URLSearchParams(location.search).get("q")).toBe("fi");
      expect(new URLSearchParams(location.search).get("ranking")).toBe("rrf");
      expect(apiMocks.searchPage).not.toHaveBeenCalled();
      await act(async () => vi.advanceTimersByTimeAsync(1));
      expect(apiMocks.searchPage).toHaveBeenCalledTimes(1);
      expect(apiMocks.searchPage.mock.lastCall?.[0]).toMatchObject({
        ranking: "rrf",
        criteria: { search_content: "all", kind: "text_search", query: "fi" },
      });
      expect(result.current.resultPage?.ranking).toBe("rrf");
    } finally {
      vi.useRealTimers();
    }
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
    expect(window.location.search).toBe(
      "?search_profile=bge-base&ranking=score&search_content=all&q=dirge",
    );

    act(() => result.current.selectRecord(null));
    expect(result.current.selectedRecordKey).toBeNull();
    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(window.location.search).toBe(
      "?search_profile=bge-base&ranking=score&search_content=all&q=dirge",
    );
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
            search_content: "all",
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
  options: {
    pageNumber?: number;
    generation?: string;
    profile?: string;
    similarity?: SimilarityOptions;
  } = {},
): SearchResultsPage {
  return {
    search_content: "all",
    ...(options.similarity
      ? { similarity: similarityConfigurationFixture(options.similarity) }
      : {}),
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
      total: BigInt(
        (options.pageNumber ?? 1) > 1
          ? ((options.pageNumber ?? 1) - 1) * 25 + Math.max(1, recordKeys.length)
          : recordKeys.length,
      ),
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

describe("Core preset handoff preparation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    history.replaceState(null, "", "/search");
    apiMocks.discoverFilterEditor.mockResolvedValue({ ...editorFixture(), groups: [] });
    apiMocks.discoverFilterValues.mockResolvedValue(valuesFixture("kind"));
    apiMocks.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => {
      const criteria = request.criteria;
      if (criteria.kind !== "similar_records") return searchResultsPage();
      const similarity = similarityConfigurationFixture(
        criteria.similarity ?? recordSimilarity,
      );
      similarity.source_sections = criteria.source_sections ?? "all";
      similarity.search_content = criteria.search_content ?? "all";
      return {
        ...searchResultsPage([], {
          generation: "generation-b",
          similarity: criteria.similarity,
        }),
        similarity,
        mode: {
          kind: "similar_records",
          seed: summaryFixture(criteria.seed_record_key),
        },
      };
    });
  });
  it("resolves the typed family before the first ranked request and does not copy selectors into a Core preset", async () => {
    let complete!: (inventory: ReturnType<typeof similarityPartsFixture>) => void;
    apiMocks.getSimilarityParts.mockReturnValue(
      new Promise((resolve) => {
        complete = resolve;
      }),
    );
    const { result } = renderHook(
      () =>
        useSearchWorkspace({
          searchProfile: "bge-base",
          searchPreferences: {
            ...DEFAULT_SEARCH_PREFERENCES,
            sourceSectionsByFamily: { hazard: "core" },
          },
        }),
      { wrapper: queryClientWrapper() },
    );
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        similarSeed: "hazards:brown-mold",
      }),
    );
    await waitFor(() => expect(apiMocks.getSimilarityParts).toHaveBeenCalledOnce());
    expect(apiMocks.searchPage).not.toHaveBeenCalled();
    await act(async () => complete(similarityPartsFixture()));
    await waitFor(() => expect(apiMocks.searchPage).toHaveBeenCalledOnce());
    expect(apiMocks.searchPage.mock.calls[0][0]).toMatchObject({
      criteria: {
        kind: "similar_records",
        source_sections: "core",
        search_content: "all",
        similarity: { parts: null },
      },
      expected_artifact_generation: "generation-b",
    });
    expect(new URLSearchParams(location.search).get("similarity_sections")).toBe(
      "core",
    );
    expect(new URLSearchParams(location.search).has("similarity_part")).toBe(false);
  });
  it("does not let an obsolete family lookup replace a later text query", async () => {
    let complete!: (inventory: ReturnType<typeof similarityPartsFixture>) => void;
    apiMocks.getSimilarityParts.mockReturnValue(
      new Promise((resolve) => {
        complete = resolve;
      }),
    );
    const { result } = renderHook(
      () =>
        useSearchWorkspace({
          searchProfile: "bge-base",
          searchPreferences: {
            ...DEFAULT_SEARCH_PREFERENCES,
            sourceSectionsByFamily: { hazard: "core" },
          },
        }),
      { wrapper: queryClientWrapper() },
    );
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        similarSeed: "hazards:brown-mold",
      }),
    );
    await waitFor(() => expect(apiMocks.getSimilarityParts).toHaveBeenCalledOnce());
    act(() => result.current.setSearch({ ...result.current.search, query: "fire" }));
    await act(async () => complete(similarityPartsFixture()));
    await waitFor(() => expect(result.current.search.query).toBe("fire"));
    expect(result.current.search.similarSeed).toBeNull();
    expect(
      apiMocks.searchPage.mock.calls.every(
        (call) => (call[0] as SearchResultsRequest).criteria.kind !== "similar_records",
      ),
    ).toBe(true);
  });
});
