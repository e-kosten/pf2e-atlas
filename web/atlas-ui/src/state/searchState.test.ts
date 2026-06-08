import {
  buildFilterDiscoveryContext,
  buildOpenRequest,
  decodeSearchState,
  DEFAULT_SEARCH_STATE,
  encodeSearchState,
  type SearchFormState,
} from "./searchState";

describe("searchState", () => {
  it("builds a browse request with clause filters, sort, and page data", () => {
    const request = buildOpenRequest(DEFAULT_SEARCH_STATE, 3);

    expect(request).toMatchObject({
      mode: {
        kind: "list_records",
        sort: { kind: "record_key" },
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
