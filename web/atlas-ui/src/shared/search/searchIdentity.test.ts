import { assertSearchIdentity, sameSearchPageContext } from "./searchIdentity";
import { DEFAULT_SEARCH_STATE, buildSearchRequest } from "../filters/searchState";

it("rejects the wrong resolved order before accepting retrieval evidence", () => {
  const response = {
    search_profile: "bge-base",
    artifact_generation: "g",
    sort: "name_asc",
  };
  expect(() => assertSearchIdentity(response, "bge-base", "g", "level_desc")).toThrow(
    "different order",
  );
  expect(() =>
    assertSearchIdentity(response, "bge-base", "g", "name_asc"),
  ).not.toThrow();
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
      sort: "level_desc",
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
