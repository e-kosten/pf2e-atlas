import {
  buildOpenRequest,
  decodeSearchState,
  DEFAULT_SEARCH_STATE,
  encodeSearchState,
  type SearchFormState,
} from "./searchState";

describe("searchState", () => {
  it("builds a browse request with kind filters, sort, and page data", () => {
    const request = buildOpenRequest(DEFAULT_SEARCH_STATE, 3);

    expect(request).toMatchObject({
      mode: {
        kind: "list_records",
        sort: { kind: "record_key" },
        filter: {
          clauses: [
            {
              id: "kind-include_any",
              field: "kind",
              operator: "include_any",
              values: ["spell", "feat", "equipment"],
            },
          ],
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

  it("builds rarity, trait, excluded trait, and level clauses", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      rarity: ["uncommon", "rare"],
      traits: ["auditory", "mental"],
      traitOperator: "include_all",
      excludedTraits: ["incapacitation"],
      levelMin: 2,
      levelMax: 5,
    };

    const request = buildOpenRequest(state, 1);
    const clauses =
      request.mode.kind === "list_records"
        ? request.mode.filter?.clauses
        : request.mode.filter?.clauses;

    expect(clauses).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          field: "rarity",
          operator: "include_any",
          values: ["uncommon", "rare"],
        }),
        expect.objectContaining({
          field: "traits",
          operator: "include_all",
          values: ["auditory", "mental"],
        }),
        expect.objectContaining({
          field: "traits",
          operator: "exclude_any",
          values: ["incapacitation"],
        }),
        expect.objectContaining({
          id: "level-range",
          field: "level",
          operator: "range",
          range: { min: 2, max: 5 },
        }),
      ]),
    );
  });

  it("builds dynamic option, boolean, and range filter clauses", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      optionFilters: {
        traditions: ["arcane", "occult"],
        size: ["lg"],
      },
      booleanFilters: {
        basic_save: "true",
      },
      rangeFilters: {
        bulk_value: { min: 1, max: 3 },
      },
    };

    const request = buildOpenRequest(state, 1);
    const clauses =
      request.mode.kind === "list_records"
        ? request.mode.filter?.clauses
        : request.mode.filter?.clauses;

    expect(clauses).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          field: "traditions",
          operator: "include_any",
          values: ["arcane", "occult"],
        }),
        expect.objectContaining({
          field: "size",
          operator: "include_any",
          values: ["lg"],
        }),
        expect.objectContaining({
          field: "basic_save",
          operator: "include_any",
          values: ["true"],
        }),
        expect.objectContaining({
          id: "bulk_value-range",
          field: "bulk_value",
          operator: "range",
          range: { min: 1, max: 3 },
        }),
      ]),
    );
  });

  it("encodes and decodes unicode search state", () => {
    const state: SearchFormState = {
      ...DEFAULT_SEARCH_STATE,
      query: "鬼火 café",
      mode: "text_search",
      traits: ["fire"],
      levelMin: 1,
      levelMax: null,
    };

    const encoded = encodeSearchState(state);
    expect(encoded).toContain("%");
    expect(decodeSearchState(encoded)).toEqual(state);
  });

  it("falls back to defaults for missing or malformed encoded state", () => {
    expect(decodeSearchState(null)).toEqual(DEFAULT_SEARCH_STATE);
    expect(decodeSearchState("%E0%A4%A")).toEqual(DEFAULT_SEARCH_STATE);
    expect(decodeSearchState(encodeURIComponent("{"))).toEqual(
      DEFAULT_SEARCH_STATE,
    );
  });

  it("normalizes wrong-shaped encoded fields independently", () => {
    const encoded = encodeURIComponent(
      JSON.stringify({
        query: 7,
        mode: "unsupported",
        kinds: ["spell", 12],
        rarity: ["rare"],
        traits: "fire",
        traitOperator: "exclude_any",
        excludedTraits: ["mental"],
        levelMin: "1",
        levelMax: Number.POSITIVE_INFINITY,
        sort: "unknown",
        pageSize: -4,
        includeDiagnostics: "true",
      }),
    );

    expect(decodeSearchState(encoded)).toEqual({
      ...DEFAULT_SEARCH_STATE,
      rarity: ["rare"],
      excludedTraits: ["mental"],
      pageSize: 1,
    });
  });

  it("clamps valid page sizes from encoded state", () => {
    expect(
      decodeSearchState(encodeURIComponent(JSON.stringify({ pageSize: 250 })))
        .pageSize,
    ).toBe(100);
  });
});
