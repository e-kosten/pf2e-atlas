import {
  buildFilterDiscoveryContext,
  buildOpenRequest,
  decodeSearchStateFromParams,
  decodeSearchState,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  encodeSearchState,
  searchStateQueryString,
  type SearchFormState,
} from "./searchState";

describe("searchState", () => {
  it("builds a browse request with clause filters, sort, and page data", () => {
    const request = buildOpenRequest(DEFAULT_SEARCH_STATE, 3);

    expect(request).toMatchObject({
      mode: {
        kind: "list_records",
        sort: { kind: "alphabetical" },
        filter: {
          clauses: [],
        },
      },
      page: { number: 3, size: 25 },
      include_diagnostics: false,
    });
  });

  it("builds a text-search request with a trimmed query and no list sort", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      mode: "text_search",
      query: "  Dirge of Doom  ",
      includeDiagnostics: true,
    };

    const request = buildOpenRequest(state, 1);

    expect(request.mode).toMatchObject({
      kind: "text_search",
      query: "Dirge of Doom",
      filter: expect.any(Object),
    });
    expect("sort" in request.mode).toBe(false);
    expect(request.include_diagnostics).toBe(true);
  });

  it("builds a browse request with random list sort", () => {
    const request = buildOpenRequest(
      {
        ...DEFAULT_SEARCH_STATE,
        sort: "random",
      },
      1,
    );

    expect(request.mode.kind).toBe("list_records");
    if (request.mode.kind === "list_records") {
      expect(request.mode.sort.kind).toBe("random");
      if (request.mode.sort.kind === "random") {
        expect(typeof request.mode.sort.seed).toBe("bigint");
      }
    }
  });

  it("keeps text-search execution state independent from browse sort", () => {
    const textState: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      mode: "text_search",
      query: "heal",
      sort: "level_desc",
    };
    const browseState: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      mode: "browse",
      sort: "level_desc",
    };

    expect(decodeURIComponent(encodeSearchExecutionState(textState))).not.toContain(
      "level_desc",
    );
    expect(decodeURIComponent(encodeSearchExecutionState(browseState))).toContain(
      "level_desc",
    );
  });

  it("builds filter discovery context from clauses only", () => {
    const context = buildFilterDiscoveryContext({
      ...DEFAULT_SEARCH_STATE,
      mode: "text_search",
      query: "fireball",
      filterClauses: [
        ...DEFAULT_SEARCH_STATE.filterClauses,
        {
          id: "rarity-include_any",
          field: "rarity",
          operator: "include_any",
          values: ["rare"],
        },
      ],
    });

    expect(context).toEqual({
      kind: "filtered",
      filter: {
        clauses: [expect.objectContaining({ field: "rarity", values: ["rare"] })],
      },
    });
  });

  it("preserves range and metric comparison clauses", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      filterClauses: [
        ...DEFAULT_SEARCH_STATE.filterClauses,
        {
          id: "level-range",
          field: "level",
          operator: "range",
          values: [],
          range: { min: 2, max: 5 },
        },
        {
          id: "metric-metric_compare",
          field: "metric",
          operator: "metric_compare",
          values: [],
          metric: {
            key: "spell.area.value",
            op: "gte",
            value: 10,
          },
        },
      ],
    };

    const request = buildOpenRequest(state, 1);
    const clauses =
      request.mode.kind === "list_records"
        ? request.mode.filter?.clauses
        : request.mode.filter?.clauses;

    expect(clauses).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "level-range",
          field: "level",
          operator: "range",
          range: { min: 2, max: 5 },
        }),
        expect.objectContaining({
          id: "metric-metric_compare",
          field: "metric",
          operator: "metric_compare",
          metric: {
            key: "spell.area.value",
            op: "gte",
            value: 10,
          },
        }),
      ]),
    );
  });

  it("encodes and decodes unicode search state", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      query: "鬼火 café",
      mode: "text_search",
      filterClauses: [
        ...DEFAULT_SEARCH_STATE.filterClauses,
        {
          id: "traits-include_all",
          field: "traits",
          operator: "include_all",
          values: ["fire"],
        },
      ],
    };

    const encoded = encodeSearchState(state);
    expect(encoded).toContain("%");
    expect(decodeSearchState(encoded)).toEqual(state);
  });

  it("omits default search state from URL query strings", () => {
    expect(encodeSearchState(DEFAULT_SEARCH_STATE)).toBe("%7B%7D");
    expect(searchStateQueryString(DEFAULT_SEARCH_STATE)).toBe("");
  });

  it("writes compact query params for simple search state", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      query: "heal",
      mode: "text_search",
      filterClauses: [
        {
          id: "kind-include_any",
          field: "kind",
          operator: "include_any",
          values: ["spell", "feat"],
        },
        {
          id: "kind-exclude_any",
          field: "kind",
          operator: "exclude_any",
          values: ["character_option"],
        },
        {
          id: "traits-include_all",
          field: "traits",
          operator: "include_all",
          values: ["vitality"],
        },
        {
          id: "level-range",
          field: "level",
          operator: "range",
          values: [],
          range: { min: 1, max: 5 },
        },
      ],
    };

    expect(decodeSearchState(encodeSearchState(state))).toEqual(state);
    const params = new URLSearchParams(searchStateQueryString(state).slice(1));
    expect(params.get("q")).toBe("heal");
    expect(params.get("mode")).toBe("text");
    expect(params.getAll("kind")).toEqual(["spell", "feat"]);
    expect(params.getAll("exclude-kind")).toEqual(["character_option"]);
    expect(params.getAll("trait")).toEqual(["vitality"]);
    expect(params.get("level")).toBe("1..5");
    expect(params.has("s")).toBe(false);
  });

  it("round-trips excluded traits through compact query params", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      filterClauses: [
        {
          id: "traits-exclude_any",
          field: "traits",
          operator: "exclude_any",
          values: ["incapacitation"],
        },
      ],
    };

    const query = searchStateQueryString(state);

    expect(query).toBe("?exclude-trait=incapacitation");
    expect(decodeSearchStateFromParams(new URLSearchParams(query.slice(1)))).toEqual(
      state,
    );
  });

  it("reads compact query params into search state", () => {
    const params = new URLSearchParams(
      "q=heal&kind=spell&kind=feat&exclude-kind=character_option&trait=vitality&any-trait=fire&exclude-trait=incapacitation&rarity=rare&exclude-rarity=unique&pack=Actions&exclude-pack=Bestiary&level=1..5&price=10&metric=ac.value%3E%3D25&sort=alphabetical&limit=50",
    );

    expect(decodeSearchStateFromParams(params)).toEqual({
      ...DEFAULT_SEARCH_STATE,
      query: "heal",
      mode: "text_search",
      sort: "alphabetical",
      pageSize: 50,
      filterClauses: [
        {
          id: "kind-include_any",
          field: "kind",
          operator: "include_any",
          values: ["spell", "feat"],
        },
        {
          id: "kind-exclude_any",
          field: "kind",
          operator: "exclude_any",
          values: ["character_option"],
        },
        {
          id: "traits-include_all",
          field: "traits",
          operator: "include_all",
          values: ["vitality"],
        },
        {
          id: "traits-include_any",
          field: "traits",
          operator: "include_any",
          values: ["fire"],
        },
        {
          id: "traits-exclude_any",
          field: "traits",
          operator: "exclude_any",
          values: ["incapacitation"],
        },
        {
          id: "rarity-include_any",
          field: "rarity",
          operator: "include_any",
          values: ["rare"],
        },
        {
          id: "rarity-exclude_any",
          field: "rarity",
          operator: "exclude_any",
          values: ["unique"],
        },
        {
          id: "pack-include_any",
          field: "pack",
          operator: "include_any",
          values: ["Actions"],
        },
        {
          id: "pack-exclude_any",
          field: "pack",
          operator: "exclude_any",
          values: ["Bestiary"],
        },
        {
          id: "level-range",
          field: "level",
          operator: "range",
          values: [],
          range: { min: 1, max: 5 },
        },
        {
          id: "price-range",
          field: "price",
          operator: "range",
          values: [],
          range: { min: 10, max: 10 },
        },
        {
          id: "metric-gte-ac.value",
          field: "metric",
          operator: "metric_compare",
          values: [],
          metric: { key: "ac.value", op: "gte", value: 25 },
        },
      ],
    });
  });

  it("uses readable generic params for non-compact filter clauses", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      visibleFilterIds: ["size"],
      filterClauses: [
        {
          id: "size-include_any",
          field: "size",
          operator: "include_any",
          values: ["med"],
        },
        {
          id: "size-exclude_any",
          field: "size",
          operator: "exclude_any",
          values: ["lg"],
        },
        {
          id: "source-include_all",
          field: "source",
          operator: "include_all",
          values: ["GM Core"],
        },
        {
          id: "bulk_value-range",
          field: "bulk_value",
          operator: "range",
          values: [],
          range: { min: 1, max: 3 },
        },
      ],
    };

    const query = searchStateQueryString(state);
    expect(query).toBe(
      "?include-size=med&exclude-size=lg&require-source=GM+Core&range-bulk_value=1..3&show-filter=size",
    );
    expect(decodeSearchStateFromParams(new URLSearchParams(query.slice(1)))).toEqual(
      state,
    );
  });

  it("uses no filter clauses when encoded state does not include clause state", () => {
    const encoded = encodeURIComponent(
      JSON.stringify({
        rarity: ["uncommon", "rare"],
        traits: ["auditory", "mental"],
        traitOperator: "include_all",
        excludedTraits: ["incapacitation"],
        levelMin: 2,
        levelMax: 5,
        optionFilters: {
          traditions: ["arcane", "occult"],
        },
        booleanFilters: {
          basic_save: "true",
        },
        rangeFilters: {
          bulk_value: { min: 1, max: 3 },
        },
      }),
    );

    expect(decodeSearchState(encoded).filterClauses).toEqual([]);
  });

  it("falls back to defaults for missing or malformed encoded state", () => {
    expect(decodeSearchState(null)).toEqual(DEFAULT_SEARCH_STATE);
    expect(decodeSearchState("%E0%A4%A")).toEqual(DEFAULT_SEARCH_STATE);
    expect(decodeSearchState(encodeURIComponent("{"))).toEqual(DEFAULT_SEARCH_STATE);
  });

  it("normalizes wrong-shaped encoded fields independently", () => {
    const encoded = encodeURIComponent(
      JSON.stringify({
        query: 7,
        mode: "unsupported",
        filterClauses: [
          {
            id: "rarity-include_any",
            field: "rarity",
            operator: "include_any",
            values: ["rare"],
          },
          {
            id: "empty-values",
            field: "traits",
            operator: "include_all",
            values: [],
          },
          {
            id: "bad-metric",
            field: "metric",
            operator: "metric_compare",
            metric: { key: "spell.area.value", op: "gte", value: "10" },
            values: [],
          },
        ],
        sort: "unknown",
        pageSize: -4,
        includeDiagnostics: "true",
      }),
    );

    expect(decodeSearchState(encoded)).toEqual({
      ...DEFAULT_SEARCH_STATE,
      filterClauses: [
        {
          id: "rarity-include_any",
          field: "rarity",
          operator: "include_any",
          values: ["rare"],
        },
      ],
      pageSize: 1,
    });
  });

  it("clamps valid page sizes from encoded state", () => {
    expect(
      decodeSearchState(encodeURIComponent(JSON.stringify({ pageSize: 250 }))).pageSize,
    ).toBe(100);
  });
});
