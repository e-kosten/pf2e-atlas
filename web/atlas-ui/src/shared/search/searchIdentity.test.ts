import type { ResultSort } from "../../generated/atlas";
import {
  assertSearchIdentity,
  sameSearchPageContext,
  sameResultSort,
} from "./searchIdentity";
import { DEFAULT_SEARCH_STATE, buildSearchRequest } from "../filters/searchState";

it("rejects the wrong resolved order before accepting retrieval evidence", () => {
  const response = {
    search_profile: "bge-base",
    artifact_generation: "g",
    sort: {
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    } satisfies ResultSort,
  };
  expect(() =>
    assertSearchIdentity(response, "bge-base", "g", {
      kind: "attributes",
      rules: [{ field: "level", direction: "desc" }],
    }),
  ).toThrow("different order");
  expect(() =>
    assertSearchIdentity(response, "bge-base", "g", {
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    }),
  ).not.toThrow();
});

it("compares complete sort values structurally, with meaningful rule priority and seeds", () => {
  const sort: ResultSort = {
    kind: "attributes",
    rules: [
      { field: "level", direction: "desc" },
      { field: "unit_price", direction: "asc" },
    ],
  };
  expect(
    sameResultSort(sort, {
      rules: [
        { direction: "desc", field: "level" },
        { direction: "asc", field: "unit_price" },
      ],
      kind: "attributes",
    }),
  ).toBe(true);
  expect(sameResultSort(sort, { ...sort, rules: [...sort.rules].reverse() })).toBe(
    false,
  );
  expect(sameResultSort({ kind: "random", seed: 0 }, { seed: 0, kind: "random" })).toBe(
    true,
  );
  expect(
    sameResultSort({ kind: "random", seed: 0 }, { kind: "random", seed: 4294967295 }),
  ).toBe(false);
});

it("allows retention across ordering/pages but isolates eligibility, model, generation and exact source scope", () => {
  const request = buildSearchRequest(
    {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: "items:armor",
      relationships: [
        {
          direction: "incoming",
          include_contents: true,
          selection: {
            record_key: "items:seed",
            embedded_path: [{ collection: "items", identity: { Stable: "owner" } }],
            field: "description",
            source_fingerprint: "source",
            passage: null,
          },
        },
      ],
    },
    1,
    "g",
    "bge-base",
  );
  expect(
    sameSearchPageContext(request, {
      ...request,
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
      page: { number: 2, size: 50 },
    }),
  ).toBe(true);
  for (const changed of [
    { ...request, expected_artifact_generation: "new" },
    { ...request, search_profile: "embeddinggemma" },
    {
      ...request,
      criteria: {
        ...request.criteria,
        filter: { kind: "in" as const, field: "kind", values: ["creature"] },
      },
    },
    {
      ...request,
      relationships: [{ ...request.relationships![0], include_contents: false }],
    },
    {
      ...request,
      relationships: [
        {
          ...request.relationships![0],
          selection: {
            ...request.relationships![0].selection,
            source_fingerprint: "new-source",
          },
        },
      ],
    },
  ])
    expect(sameSearchPageContext(request, changed)).toBe(false);
});
