import {
  DEFAULT_SEARCH_STATE,
  buildOpenRequest,
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
    ).toEqual(state);
    expect(buildOpenRequest(state, 2).mode).toEqual({
      kind: "text_search",
      query: "dragon",
      mode: "hybrid",
      filter: state.filter,
    });
  });
  it("preserves browse and text context separately", () => {
    expect(buildOpenRequest(DEFAULT_SEARCH_STATE, 1).mode).toEqual({
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
    expect(buildOpenRequest(state, 1).mode).toMatchObject({
      kind: "text_search",
      mode: "hybrid",
    });
    expect(searchStateQueryString(state)).not.toContain("retrieval");
    expect(buildOpenRequest({ ...state, query: "  " }, 1).mode.kind).toBe(
      "list_records",
    );
  });
  it("blocks malformed URLs and invalid numeric drafts", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams("filter=bad&q=dragon"),
    );
    expect(state.filterError).toBeTruthy();
    expect(hasExecutableSearch(state)).toBe(false);
    expect(() => buildOpenRequest(state, 1)).toThrow();
  });
  it("bounds page sizes without changing valid zero-valued filters", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams(
        "limit=1000&filter=" +
          encodeURIComponent(
            JSON.stringify({
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
  ).toEqual(state);
  expect(buildOpenRequest(state, 3)).toEqual({
    page: { number: 3, size: 25 },
    mode: {
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
  expect(() => buildOpenRequest(invalid, 1)).toThrow();
});
