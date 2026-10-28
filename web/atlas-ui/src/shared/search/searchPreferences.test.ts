import {
  DEFAULT_SEARCH_PREFERENCES,
  LEGACY_SEARCH_PROFILE_KEY,
  SEARCH_PREFERENCES_KEY,
  readSearchPreferences,
  writeSearchPreferences,
} from "./searchPreferences";

beforeEach(() => localStorage.clear());

it("keeps record and trait launch defaults independent", () => {
  const preferences = readSearchPreferences();
  expect(preferences.recordSimilarity.method).toBe("passages");
  expect(preferences.traitSimilarity.method).toBe("baseline");
});

it("preserves a stored trait content method and its inactive values", () => {
  const preferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    traitSimilarity: {
      method: "passages" as const,
      decay: 0.75,
      candidateParts: "any" as const,
    },
  };
  expect(writeSearchPreferences(preferences)).toBe(true);
  expect(readSearchPreferences()).toEqual(preferences);
});

it("remembers general preferences across browser-owner launches", () => {
  const preferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    model: "embeddinggemma",
    ranking: "rrf" as const,
    matchDisplay: "detailed" as const,
    showSmallerWeights: true,
    recordSimilarity: {
      method: "content-average" as const,
      decay: 0.75,
      candidateParts: "corresponding" as const,
    },
  };
  expect(writeSearchPreferences(preferences)).toBe(true);
  expect(readSearchPreferences()).toEqual(preferences);
});

it("defaults to Matching sections without losing existing browser preferences", () => {
  localStorage.setItem(
    SEARCH_PREFERENCES_KEY,
    JSON.stringify({ version: 1, model: "embeddinggemma", ranking: "rrf" }),
  );
  expect(readSearchPreferences()).toMatchObject({
    model: "embeddinggemma",
    ranking: "rrf",
    matchDisplay: "sections",
    showSmallerWeights: false,
  });
});

it("migrates the existing model preference without losing its choice", () => {
  localStorage.setItem(LEGACY_SEARCH_PROFILE_KEY, "embeddinggemma");
  expect(readSearchPreferences().model).toBe("embeddinggemma");
  expect(JSON.parse(localStorage.getItem(SEARCH_PREFERENCES_KEY)!)).toMatchObject({
    version: 1,
    model: "embeddinggemma",
  });
  expect(localStorage.getItem(LEGACY_SEARCH_PROFILE_KEY)).toBeNull();
});

it("does not erase the legacy preference if migration cannot be persisted", () => {
  localStorage.setItem(LEGACY_SEARCH_PROFILE_KEY, "embeddinggemma");
  const write = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  try {
    expect(readSearchPreferences().model).toBe("embeddinggemma");
    expect(localStorage.getItem(LEGACY_SEARCH_PROFILE_KEY)).toBe("embeddinggemma");
  } finally {
    write.mockRestore();
  }
});

it("recovers malformed optional preferences without accepting invalid decay", () => {
  localStorage.setItem(
    SEARCH_PREFERENCES_KEY,
    JSON.stringify({
      version: 1,
      model: "bge-base",
      ranking: "unknown",
      recordSimilarity: { method: "passages", decay: -1, candidateParts: "unknown" },
      traitSimilarity: {
        method: "baseline",
        decay: 1,
        candidateParts: "corresponding",
      },
    }),
  );
  expect(readSearchPreferences()).toEqual({
    ...DEFAULT_SEARCH_PREFERENCES,
    model: "bge-base",
    traitSimilarity: { method: "baseline", decay: 1, candidateParts: "any" },
  });
});

it("uses defaults when storage cannot be read", () => {
  const read = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  try {
    expect(readSearchPreferences()).toEqual(DEFAULT_SEARCH_PREFERENCES);
  } finally {
    read.mockRestore();
  }
});

it("extends existing browser preferences with independent content and family presets", () => {
  const previous = {
    version: 1,
    model: "embeddinggemma",
    ranking: "rrf",
    showSimilarityDetails: true,
    showSmallerWeights: true,
    searchContent: "core",
    sourceSectionsByFamily: { npc: "core", hazard: "all", equipment: "custom" },
    recordSimilarity: {
      method: "passages",
      decay: 0.85,
      candidateParts: "corresponding",
    },
  };
  localStorage.setItem(SEARCH_PREFERENCES_KEY, JSON.stringify(previous));
  const preferences = readSearchPreferences();
  expect(preferences).toMatchObject({
    model: "embeddinggemma",
    ranking: "rrf",
    matchDisplay: "detailed" as const,
    showSmallerWeights: true,
    searchContent: "core",
    sourceSectionsByFamily: { npc: "core", hazard: "all" },
    recordSimilarity: previous.recordSimilarity,
  });
  expect(preferences.sourceSectionsByFamily).not.toHaveProperty("equipment");
  expect(writeSearchPreferences(preferences)).toBe(true);
  expect(readSearchPreferences()).toEqual(preferences);
});

it.each(["hidden", "sections", "detailed"] as const)(
  "remembers the %s match display independently of legacy details",
  (matchDisplay) => {
    localStorage.setItem(
      SEARCH_PREFERENCES_KEY,
      JSON.stringify({ version: 1, matchDisplay, showSimilarityDetails: true }),
    );
    const preferences = readSearchPreferences();
    expect(preferences.matchDisplay).toBe(matchDisplay);
    expect(writeSearchPreferences(preferences)).toBe(true);
    expect(readSearchPreferences().matchDisplay).toBe(matchDisplay);
    expect(
      JSON.parse(localStorage.getItem(SEARCH_PREFERENCES_KEY)!),
    ).not.toHaveProperty("showSimilarityDetails");
  },
);
it.each([
  [true, "detailed"],
  [false, "sections"],
  [undefined, "sections"],
])("migrates legacy details %s to %s", (legacy, expected) => {
  localStorage.setItem(
    SEARCH_PREFERENCES_KEY,
    JSON.stringify({
      version: 1,
      showSimilarityDetails: legacy,
      model: "embeddinggemma",
      ranking: "rrf",
    }),
  );
  expect(readSearchPreferences()).toMatchObject({
    matchDisplay: expected,
    model: "embeddinggemma",
    ranking: "rrf",
  });
});
it("recovers an invalid display preference without accepting arbitrary values", () => {
  localStorage.setItem(
    SEARCH_PREFERENCES_KEY,
    JSON.stringify({ version: 1, matchDisplay: "invalid" }),
  );
  expect(readSearchPreferences().matchDisplay).toBe("sections");
});
