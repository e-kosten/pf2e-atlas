import { encodeFilterUrl } from "./filterUrl";
import {
  DEFAULT_SEARCH_STATE,
  buildSearchRequest,
  searchStateQueryString,
  decodeSearchStateFromParams,
  buildFilterDiscoveryContext,
  hasExecutableSearch,
} from "./searchState";
describe("source query state", () => {
  it("keeps query and structured predicate across URL reload", () => {
    const state = {
      ...DEFAULT_SEARCH_STATE,
      query: "dragon",
      mode: "text_search" as const,
      filter: {
        kind: "compare" as const,
        field: "actor.level",
        op: "gte" as const,
        value: 4,
      },
    };
    expect(
      decodeSearchStateFromParams(new URLSearchParams(searchStateQueryString(state))),
    ).toMatchObject({
      ...state,
      filter: { ...state.filter, clause_id: expect.any(String) },
    });
    expect(buildSearchRequest(state, 2).criteria).toEqual({
      kind: "text_search",
      query: "dragon",
      mode: "hybrid",
      filter: state.filter,
    });
  });
  it("preserves browse and text context separately", () => {
    expect(buildSearchRequest(DEFAULT_SEARCH_STATE, 1).criteria).toEqual({
      kind: "list_records",
      filter: null,
    });
    expect(
      buildFilterDiscoveryContext({
        ...DEFAULT_SEARCH_STATE,
        query: "dragon",
        mode: "text_search",
      }),
    ).toMatchObject({ text: "dragon", mode: "hybrid" });
  });
  it("ignores retired retrieval URL options and derives text mode from the query", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams("q=dragon&retrieval=semantic"),
    );
    expect(buildSearchRequest(state, 1).criteria).toMatchObject({
      kind: "text_search",
      mode: "hybrid",
    });
    expect(searchStateQueryString(state)).not.toContain("retrieval");
    expect(buildSearchRequest({ ...state, query: "  " }, 1).criteria.kind).toBe(
      "list_records",
    );
  });
  it("blocks malformed URLs and invalid numeric drafts", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams("filter=bad&q=dragon"),
    );
    expect(state.filterError).toBeTruthy();
    expect(hasExecutableSearch(state)).toBe(false);
    expect(() => buildSearchRequest(state, 1)).toThrow();
  });
  it("bounds page sizes without changing valid zero-valued filters", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams(
        "limit=1000&filter=" +
          encodeURIComponent(
            encodeFilterUrl({
              kind: "compare",
              field: "actor.level",
              op: "eq",
              value: 0,
            }),
          ),
      ),
    );
    expect(state.pageSize).toBe(25);
    expect(state.filter).toMatchObject({ value: 0 });
  });
});

it("round trips a root similar criterion and filters without adding a kind filter", () => {
  const state = {
    ...DEFAULT_SEARCH_STATE,
    similarSeed: "spells:root000000000000",
    mode: "similar_records" as const,
    filter: {
      kind: "compare" as const,
      field: "item.level",
      op: "gte" as const,
      value: 2,
    },
  };
  expect(
    decodeSearchStateFromParams(new URLSearchParams(searchStateQueryString(state))),
  ).toMatchObject({
    ...state,
    filter: { ...state.filter, clause_id: expect.any(String) },
  });
  expect(buildSearchRequest(state, 3)).toEqual({
    page: { number: 3, size: 25 },
    expected_artifact_generation: null,
    criteria: {
      kind: "similar_records",
      seed_record_key: state.similarSeed,
      filter: state.filter,
    },
  });
  expect(hasExecutableSearch({ ...state, filter: null })).toBe(true);
  expect(buildFilterDiscoveryContext(state).text).toBeNull();
  const invalid = decodeSearchStateFromParams(
    new URLSearchParams("similar=items:seed&q=dragon"),
  );
  expect(hasExecutableSearch(invalid)).toBe(false);
  expect(() => buildSearchRequest(invalid, 1)).toThrow();
});

it("carries the selected profile through text and Similar deep links and contextual generation", () => {
  const state = {
    ...DEFAULT_SEARCH_STATE,
    query: "dragon",
    mode: "text_search" as const,
    pageSize: 50,
  };
  const params = new URLSearchParams(searchStateQueryString(state, "embeddinggemma"));
  expect(params.get("search_profile")).toBe("embeddinggemma");
  expect(decodeSearchStateFromParams(params)).toMatchObject(state);
  expect(buildSearchRequest(state, 2, "generation-a", "embeddinggemma")).toMatchObject({
    search_profile: "embeddinggemma",
    expected_artifact_generation: "generation-a",
    page: { number: 2, size: 50 },
  });
  expect(
    buildFilterDiscoveryContext(state, "embeddinggemma", "generation-a"),
  ).toMatchObject({
    search_profile: "embeddinggemma",
    expected_artifact_generation: "generation-a",
    text: "dragon",
  });
  const similar = { ...state, query: "", similarSeed: "actors:seed" };
  const similarParams = new URLSearchParams(
    searchStateQueryString(similar, "embeddinggemma"),
  );
  expect(similarParams.get("search_profile")).toBe("embeddinggemma");
  expect(
    buildSearchRequest(
      decodeSearchStateFromParams(similarParams),
      1,
      "generation-a",
      similarParams.get("search_profile")!,
    ),
  ).toMatchObject({
    search_profile: "embeddinggemma",
    criteria: { kind: "similar_records", seed_record_key: "actors:seed" },
  });
});
