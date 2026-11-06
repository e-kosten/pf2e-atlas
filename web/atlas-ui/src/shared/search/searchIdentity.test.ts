import type { ResultSort } from "../../generated/atlas";
import {
  assertSearchIdentity,
  assertSimilarityIdentity,
  assertSimilarityInventoryIdentity,
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
  const relationship: import("../../generated/atlas").QueryRelationship = {
    direction: "incoming",
    include_contents: true,
    selection: {
      record_key: "items:seed",
      embedded_path: [{ collection: "items", identity: { Stable: "owner" } }],
      field: "description",
      passage: null,
      source_fingerprint: "source",
    },
  };
  const filter = { kind: "relationship" as const, clause_id: "link-0", relationship };
  const request = buildSearchRequest(
    { ...DEFAULT_SEARCH_STATE, similarSeed: "items:armor", filter },
    1,
    "g",
    "bge-base",
  );
  expect(
    sameSearchPageContext(request, {
      ...request,
      sort: { kind: "attributes", rules: [{ field: "level", direction: "desc" }] },
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
      criteria: {
        ...request.criteria,
        filter: {
          ...filter,
          relationship: { ...relationship, include_contents: false },
        },
      },
    },
    {
      ...request,
      criteria: {
        ...request.criteria,
        filter: {
          ...filter,
          relationship: {
            ...relationship,
            selection: { ...relationship.selection, source_fingerprint: "new-source" },
          },
        },
      },
    },
  ])
    expect(sameSearchPageContext(request, changed)).toBe(false);
});

it("rejects mismatched result scope and source presets before accepting Core evidence", () => {
  const response = {
    search_profile: "bge-base",
    artifact_generation: "g",
    search_content: "all" as const,
  };
  expect(() =>
    assertSearchIdentity(response, "bge-base", "g", undefined, "core"),
  ).toThrow("different search content");
  const options = {
    method: "passages" as const,
    parts: null,
    candidate_parts: "any" as const,
    decay: 0.5,
  };
  const page = {
    similarity: {
      options,
      source_sections: "all" as const,
      search_content: "core" as const,
    },
  };
  expect(() => assertSimilarityIdentity(page, options, "core", "core")).toThrow(
    "different similarity settings",
  );
  expect(() =>
    assertSimilarityIdentity(
      { ...page, similarity: { ...page.similarity, source_sections: "core" } },
      options,
      "core",
      "core",
    ),
  ).not.toThrow();
});
it("rejects a stale or differently addressed source inventory", () => {
  const inventory = {
    seed: { kind: "record" as const, record_key: "hazards:one" },
    artifact_generation: "g",
    search_profile: "bge-base",
    parts: [],
    role_recipe: "r",
    supports_corresponding_parts: true,
    core: {
      family: "hazard" as const,
      selectors: [],
      representation: "identity_only" as const,
      fallback_reason: "no_core_prose" as const,
      recipe: "core-sections/v1",
    },
  };
  expect(() =>
    assertSimilarityInventoryIdentity(inventory, inventory.seed, "bge-base", "g"),
  ).not.toThrow();
  expect(() =>
    assertSimilarityInventoryIdentity(
      inventory,
      { kind: "record", record_key: "hazards:two" },
      "bge-base",
      "g",
    ),
  ).toThrow("do not match this source");
  expect(() =>
    assertSimilarityInventoryIdentity(inventory, inventory.seed, "bge-base", "retired"),
  ).toThrow("reference collection changed");
});
