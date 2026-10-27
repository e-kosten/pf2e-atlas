import type { ResultSort } from "../../generated/atlas";
import type { QueryRelationship } from "../../generated/atlas";
import { encodeFilterUrl } from "./filterUrl";
import {
  DEFAULT_SEARCH_STATE,
  buildSearchRequest,
  searchStateQueryString,
  decodeSearchStateFromParams,
  buildFilterDiscoveryContext,
  buildSavedListFilterDiscoveryContext,
  hasExecutableSearch,
  effectiveResultSort,
  normalizeSortTransition,
  encodeSearchExecutionState,
  encodeResultSort,
  decodeResultSort,
} from "./searchState";
describe("result order intent", () => {
  it("round trips complete ordered rules without materializing defaults", () => {
    const sort: ResultSort = {
      kind: "attributes",
      rules: [
        { field: "level", direction: "desc" },
        { field: "unit_price", direction: "asc" },
        { field: "name", direction: "desc" },
      ],
    };
    const params = new URLSearchParams(
      searchStateQueryString({ ...DEFAULT_SEARCH_STATE, sort }),
    );
    expect(params.getAll("sort")).toEqual([
      "level_desc",
      "unit_price_asc",
      "name_desc",
    ]);
    expect(decodeSearchStateFromParams(params).sort).toEqual(sort);
    const old = new URLSearchParams("sort=random&seed=3");
    encodeResultSort(old, sort);
    expect(old.has("seed")).toBe(false);
    encodeResultSort(old, undefined);
    expect(old.size).toBe(0);
  });
  it.each([0, 4294967295])(
    "keeps Random seed %s through URL/request identity and stays idle alone",
    (seed) => {
      const state = {
        ...DEFAULT_SEARCH_STATE,
        sort: { kind: "random", seed } satisfies ResultSort,
      };
      expect(
        decodeSearchStateFromParams(new URLSearchParams(searchStateQueryString(state)))
          .sort,
      ).toEqual(state.sort);
      expect(buildSearchRequest(state, 1).sort).toEqual(state.sort);
      expect(hasExecutableSearch(state)).toBe(false);
      expect(buildFilterDiscoveryContext(state)).toEqual(
        buildFilterDiscoveryContext(DEFAULT_SEARCH_STATE),
      );
    },
  );
  it.each([
    "sort=random",
    "sort=random&seed=",
    "sort=random&seed=-1",
    "sort=random&seed=4294967296",
    "sort=random&seed=1.1",
    "sort=random&seed=1oops",
    "sort=random&seed=1e2",
    "sort=random&seed=Infinity",
    "sort=random&seed=0&seed=0",
    "seed=0",
    "sort=name_asc&seed=1",
    "sort=relevance&seed=1",
    "sort=random&sort=name_asc&seed=1",
    "sort=relevance&sort=relevance",
    "sort=relevance&sort=name_asc",
    "sort=saved_order&sort=name_asc",
    "sort=name_asc&sort=name_desc",
    "sort=level_desc&sort=level_desc",
    "sort=",
    "sort=unknown_asc",
  ])("rejects malformed sort grammar %s", (query) => {
    expect(
      decodeResultSort(new URLSearchParams(query), "search", true).sortError,
    ).toBeTruthy();
    expect(
      decodeSearchStateFromParams(new URLSearchParams(query)).filterError,
    ).toBeTruthy();
  });
  it("keeps automatic defaults out of URLs and requests but preserves an explicit default", () => {
    const automatic = decodeSearchStateFromParams(new URLSearchParams());
    expect(automatic.sort).toBeUndefined();
    expect(effectiveResultSort(automatic)).toEqual({
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    });
    expect(buildSearchRequest(automatic, 1)).not.toHaveProperty("sort");
    expect(
      hasExecutableSearch({
        ...automatic,
        sort: {
          kind: "attributes",
          rules: [{ field: "name", direction: "desc" }],
        } satisfies ResultSort,
      }),
    ).toBe(false);
    const explicit = {
      ...automatic,
      sort: {
        kind: "attributes",
        rules: [{ field: "name", direction: "asc" }],
      } satisfies ResultSort,
    };
    expect(searchStateQueryString(explicit)).toContain("sort=name_asc");
    expect(buildSearchRequest(explicit, 1).sort).toEqual({
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    });
    expect(
      decodeSearchStateFromParams(new URLSearchParams(searchStateQueryString(explicit)))
        .sort,
    ).toEqual({ kind: "attributes", rules: [{ field: "name", direction: "asc" }] });
    expect(encodeSearchExecutionState(explicit)).not.toBe(
      encodeSearchExecutionState(automatic),
    );
  });
  it("defaults semantic criteria to Relevance and lists to Saved order while retaining explicit attributes", () => {
    for (const criterion of [{ query: "fire" }, { similarSeed: "items:seed" }]) {
      const automatic = { ...DEFAULT_SEARCH_STATE, ...criterion };
      expect(effectiveResultSort(automatic)).toEqual({ kind: "relevance" });
      expect(effectiveResultSort(automatic, "saved_list")).toEqual({
        kind: "saved_order",
      });
      const explicit = {
        ...automatic,
        sort: {
          kind: "attributes",
          rules: [{ field: "level", direction: "desc" }],
        } satisfies ResultSort,
      };
      expect(effectiveResultSort(explicit)).toEqual({
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      });
      expect(
        normalizeSortTransition(explicit, { ...explicit, query: "", similarSeed: null })
          .sort,
      ).toEqual({ kind: "attributes", rules: [{ field: "level", direction: "desc" }] });
      const relevant = { ...automatic, sort: { kind: "relevance" } as const };
      expect(
        normalizeSortTransition(relevant, { ...relevant, query: "", similarSeed: null })
          .sort,
      ).toBeUndefined();
    }
  });
  it.each(["sort=unknown", "sort=saved_order", "sort=relevance"])(
    "rejects incompatible search URL %s",
    (query) => {
      const state = decodeSearchStateFromParams(new URLSearchParams(query));
      expect(state.filterError).toBeTruthy();
      expect(hasExecutableSearch(state)).toBe(false);
    },
  );
  it("validates list orders separately and excludes ordering from discovery membership", () => {
    expect(
      decodeSearchStateFromParams(
        new URLSearchParams("q=fire&sort=relevance"),
        "saved_list",
      ).filterError,
    ).toBeTruthy();
    expect(
      decodeSearchStateFromParams(
        new URLSearchParams("q=fire&sort=saved_order"),
        "saved_list",
      ).filterError,
    ).toBeNull();
    const state = {
      ...DEFAULT_SEARCH_STATE,
      sort: {
        kind: "attributes",
        rules: [{ field: "unit_price", direction: "desc" }],
      } satisfies ResultSort,
    };
    expect(buildFilterDiscoveryContext(state)).toEqual(
      buildFilterDiscoveryContext(DEFAULT_SEARCH_STATE),
    );
    expect(buildSavedListFilterDiscoveryContext("research", state)).toEqual(
      buildSavedListFilterDiscoveryContext("research", DEFAULT_SEARCH_STATE),
    );
  });
});
describe("source query state", () => {
  it("preserves ranking independently of the query and rejects unknown strategies", () => {
    for (const ranking of ["rrf", "score"] as const) {
      const state = { ...DEFAULT_SEARCH_STATE, query: "harrowkin", ranking };
      const restored = decodeSearchStateFromParams(
        new URLSearchParams(searchStateQueryString(state)),
      );
      expect(restored.ranking).toBe(ranking);
      expect(buildSearchRequest(restored, 1).ranking).toBe(ranking);
      expect(buildSearchRequest(restored, 1).criteria).toMatchObject({
        kind: "text_search",
        query: "harrowkin",
        mode: "hybrid",
      });
    }
    expect(
      hasExecutableSearch(
        decodeSearchStateFromParams(new URLSearchParams("q=harrowkin&ranking=unknown")),
      ),
    ).toBe(false);
  });
  it("keeps query and structured predicate across URL reload", () => {
    const state = {
      ...DEFAULT_SEARCH_STATE,
      query: "dragon",
      mode: "text_search" as const,
      filter: {
        kind: "compare" as const,
        field: "level",
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
  it("uses lexical list filtering while discovery searches stay hybrid", () => {
    const state = {
      ...DEFAULT_SEARCH_STATE,
      query: "dragon",
      mode: "text_search" as const,
    };
    expect(buildSavedListFilterDiscoveryContext("research", state)).toMatchObject({
      kind: "saved_list",
      list_ref: "research",
      text: "dragon",
      mode: "lexical",
    });
    expect(buildSearchRequest(state, 1).criteria).toMatchObject({ mode: "hybrid" });
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
  it("recovers malformed filter transport while retaining the text query", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams("filter=bad&q=dragon"),
    );
    expect(state.filterError).toBeNull();
    expect(state.filterWasReset).toBe(true);
    expect(hasExecutableSearch(state)).toBe(true);
    expect(buildSearchRequest(state, 1).criteria).toMatchObject({
      kind: "text_search",
      query: "dragon",
      filter: null,
    });
  });
  it("bounds page sizes without changing valid zero-valued filters", () => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams(
        "limit=1000&filter=" +
          encodeURIComponent(
            encodeFilterUrl({
              kind: "compare",
              field: "level",
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
      field: "level",
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

it("recovers obsolete filter links without retaining a legacy decoder", () => {
  const state = decodeSearchStateFromParams(
    new URLSearchParams({ q: "fire", filter: "1.old" }),
  );
  expect(state.filterError).toBeNull();
  expect(state.filterWasReset).toBe(true);
  expect(hasExecutableSearch(state)).toBe(true);
  expect(buildSearchRequest(state, 1).criteria).toMatchObject({
    kind: "text_search",
    query: "fire",
    filter: null,
  });
});

it("carries complete relationship constraints in browse and saved-list discovery contexts", () => {
  const relationship: QueryRelationship = {
    direction: "outgoing",
    include_contents: true,
    selection: {
      record_key: "actors:selected",
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 2 } } },
      ],
      field: "description",
      passage: { kind: "identity" },
      source_fingerprint: "checked-source",
    },
  };
  const state = {
    ...DEFAULT_SEARCH_STATE,

    filter: {
      kind: "all_of" as const,
      children: [
        { kind: "in" as const, field: "kind", values: ["equipment"] },
        {
          kind: "relationship" as const,
          clause_id: "link-0",
          relationship: relationship,
        },
      ],
    },
  };
  expect(buildFilterDiscoveryContext(state, "bge-base", "generation-a")).toEqual({
    kind: "filtered",

    filter: state.filter,
    text: null,
    mode: "hybrid",
    search_profile: "bge-base",
    expected_artifact_generation: "generation-a",
  });
  expect(buildSavedListFilterDiscoveryContext("research", state)).toMatchObject({
    kind: "saved_list",
    list_ref: "research",

    filter: state.filter,
    text: null,
    mode: "lexical",
  });
});

describe("trait search handoffs", () => {
  it("round trips a trait Similar seed and retains its filter", () => {
    const state = {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: "trait:consumable",
      filter: { kind: "boolean_constant", value: false } as const,
    };
    const decoded = decodeSearchStateFromParams(
      new URLSearchParams(searchStateQueryString(state)),
    );
    expect(buildSearchRequest(decoded, 1).criteria).toEqual({
      kind: "similar_traits",
      seed_trait_key: "trait:consumable",
      similarity: { method: "baseline" },
      filter: expect.objectContaining(state.filter),
    });
  });
  it("keeps trait references scoped when adding keywords and filters", () => {
    const state = {
      ...DEFAULT_SEARCH_STATE,
      traitReferenceSeed: "trait:consumable",
      query: "potion",
      filter: { kind: "boolean_constant", value: false } as const,
    };
    const decoded = decodeSearchStateFromParams(
      new URLSearchParams(searchStateQueryString(state)),
    );
    expect(hasExecutableSearch(decoded)).toBe(true);
    expect(buildSearchRequest(decoded, 1).criteria).toEqual({
      kind: "trait_references",
      trait_key: "trait:consumable",
      query: "potion",
      filter: expect.objectContaining(state.filter),
    });
    expect(encodeSearchExecutionState(state)).not.toEqual(
      encodeSearchExecutionState({ ...state, traitReferenceSeed: null }),
    );
  });
});
