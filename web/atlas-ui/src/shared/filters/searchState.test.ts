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
      retrievalMode: "lexical" as const,
    };
    expect(
      decodeSearchStateFromParams(new URLSearchParams(searchStateQueryString(state))),
    ).toEqual(state);
    expect(buildOpenRequest(state, 2).mode).toEqual({
      kind: "text_search",
      query: "dragon",
      mode: "lexical",
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
