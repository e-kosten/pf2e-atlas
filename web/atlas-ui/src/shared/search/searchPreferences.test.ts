import {
  DEFAULT_SEARCH_PREFERENCES,
  LEGACY_SEARCH_PROFILE_KEY,
  SEARCH_PREFERENCES_KEY,
  readSearchPreferences,
  writeSearchPreferences,
} from "./searchPreferences";

beforeEach(() => localStorage.clear());

it("remembers general preferences across browser-owner launches", () => {
  const preferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    model: "embeddinggemma",
    ranking: "rrf" as const,
    recordSimilarity: {
      method: "content-average" as const,
      decay: 0.75,
      candidateParts: "corresponding" as const,
    },
  };
  expect(writeSearchPreferences(preferences)).toBe(true);
  expect(readSearchPreferences()).toEqual(preferences);
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
