import type {
  CoreSectionFamily,
  SearchContentScope,
  SearchRanking,
} from "../../generated/atlas";

export const SEARCH_PREFERENCES_KEY = "atlas.search.preferences.v1";
export const LEGACY_SEARCH_PROFILE_KEY = "atlas.search.profile.v1";

export type SimilarityMethod = "passages" | "content-average" | "baseline";
export type CandidateParts = "any" | "corresponding";
export type SimilarityPreferences = {
  method: SimilarityMethod;
  decay: number;
  candidateParts: CandidateParts;
};
export type SearchPreferences = {
  version: 1;
  model: string | null;
  ranking: SearchRanking;
  searchContent: SearchContentScope;
  sourceSectionsByFamily: Partial<Record<CoreSectionFamily, "all" | "core">>;
  showSimilarityDetails: boolean;
  showSmallerWeights: boolean;
  recordSimilarity: SimilarityPreferences;
  traitSimilarity: SimilarityPreferences;
};

export const DEFAULT_SIMILARITY_PREFERENCES: SimilarityPreferences = {
  method: "passages",
  decay: 0.5,
  candidateParts: "any",
};
export const DEFAULT_TRAIT_SIMILARITY_PREFERENCES: SimilarityPreferences = {
  ...DEFAULT_SIMILARITY_PREFERENCES,
  method: "baseline",
};
export const DEFAULT_SEARCH_PREFERENCES: SearchPreferences = {
  version: 1,
  model: null,
  ranking: "score",
  searchContent: "all",
  sourceSectionsByFamily: {},
  showSimilarityDetails: false,
  showSmallerWeights: false,
  recordSimilarity: DEFAULT_SIMILARITY_PREFERENCES,
  traitSimilarity: DEFAULT_TRAIT_SIMILARITY_PREFERENCES,
};

function object(value: unknown): Record<string, unknown> | null {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function similarity(value: unknown, trait = false): SimilarityPreferences {
  const input = object(value);
  const defaults = trait
    ? DEFAULT_TRAIT_SIMILARITY_PREFERENCES
    : DEFAULT_SIMILARITY_PREFERENCES;
  return {
    method:
      input?.method === "passages" ||
      input?.method === "content-average" ||
      input?.method === "baseline"
        ? input.method
        : defaults.method,
    decay:
      typeof input?.decay === "number" &&
      Number.isFinite(input.decay) &&
      input.decay >= 0 &&
      input.decay <= 1
        ? input.decay
        : 0.5,
    candidateParts:
      !trait && input?.candidateParts === "corresponding" ? "corresponding" : "any",
  };
}

export function readSearchPreferences(): SearchPreferences {
  try {
    const stored: unknown = JSON.parse(
      localStorage.getItem(SEARCH_PREFERENCES_KEY) ?? "null",
    );
    const input = object(stored);
    if (input?.version === 1)
      return {
        version: 1,
        model: typeof input.model === "string" ? input.model : null,
        ranking: input.ranking === "rrf" ? "rrf" : "score",
        searchContent: input.searchContent === "core" ? "core" : "all",
        sourceSectionsByFamily: Object.fromEntries(
          Object.entries(object(input.sourceSectionsByFamily) ?? {}).filter(
            ([, value]) => value === "all" || value === "core",
          ),
        ),
        showSimilarityDetails: input.showSimilarityDetails === true,
        showSmallerWeights: input.showSmallerWeights === true,
        recordSimilarity: similarity(input.recordSimilarity),
        traitSimilarity: similarity(input.traitSimilarity, true),
      };

    const model = localStorage.getItem(LEGACY_SEARCH_PROFILE_KEY);
    if (model) {
      const migrated = { ...DEFAULT_SEARCH_PREFERENCES, model };
      // Retire the legacy key only after the replacement is safely stored.
      if (writeSearchPreferences(migrated))
        localStorage.removeItem(LEGACY_SEARCH_PROFILE_KEY);
      return migrated;
    }
  } catch {
    // Persistence is optional; an explicit search choice still works in this tab.
  }
  return DEFAULT_SEARCH_PREFERENCES;
}

export function writeSearchPreferences(preferences: SearchPreferences): boolean {
  try {
    localStorage.setItem(SEARCH_PREFERENCES_KEY, JSON.stringify(preferences));
    localStorage.removeItem(LEGACY_SEARCH_PROFILE_KEY);
    return true;
  } catch {
    return false;
  }
}
