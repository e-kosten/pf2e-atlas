import { DEFAULT_SEARCH_PREFERENCES } from "./searchPreferences";
import {
  buildSearchRequest,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";

it("uses remembered preferences only where the URL has no explicit choice", () => {
  const preferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    ranking: "rrf" as const,
    recordSimilarity: {
      method: "content-average" as const,
      decay: 0.75,
      candidateParts: "corresponding" as const,
    },
  };
  const saved = decodeSearchStateFromParams(
    new URLSearchParams("similar=hazards:brown-mold"),
    "search",
    preferences,
  );
  expect(saved.ranking).toBe("rrf");
  expect(saved.similarity).toEqual({
    method: "content-average",
    candidate_parts: "corresponding",
    parts: null,
  });
  const explicit = decodeSearchStateFromParams(
    new URLSearchParams(
      "similar=hazards:brown-mold&ranking=score&similarity_method=passages&similarity_decay=0&candidate_parts=any",
    ),
    "search",
    preferences,
  );
  expect(explicit.ranking).toBe("score");
  expect(explicit.similarity).toEqual({
    method: "passages",
    decay: 0,
    candidate_parts: "any",
    parts: null,
  });
});

it("round trips exact selected parts and their generation into the request", () => {
  const search = decodeSearchStateFromParams(
    new URLSearchParams(
      "similar=hazards:brown-mold&similarity_method=passages&similarity_decay=0.75&candidate_parts=corresponding&similarity_part=one&similarity_part=two&similarity_generation=old-generation",
    ),
  );
  const restored = decodeSearchStateFromParams(
    new URLSearchParams(searchStateQueryString(search)),
  );
  expect(restored.similarity).toEqual(search.similarity);
  expect(
    buildSearchRequest(restored, 1, "current-generation").expected_artifact_generation,
  ).toBe("old-generation");
  expect(buildSearchRequest(restored, 1).criteria).toMatchObject({
    similarity: { parts: ["one", "two"] },
  });
});

it.each([
  "similarity_method=unknown",
  "similarity_method=passages&similarity_decay=NaN",
  "similarity_method=passages&similarity_decay=2",
  "similarity_method=baseline&similarity_decay=0.5",
  "similarity_method=content-average&similarity_decay=0.5",
  "similarity_method=passages&similarity_part=one",
])("keeps invalid explicit settings visible: %s", (query) => {
  expect(
    decodeSearchStateFromParams(
      new URLSearchParams(`similar=hazards:brown-mold&${query}`),
    ).filterError,
  ).toContain("Invalid similarity settings");
});
