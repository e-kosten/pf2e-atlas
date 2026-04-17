import type { NormalizedRecord, SearchCategory, SearchProfile } from "../types.js";
import type { SearchVocabularyResult } from "../data/vocabulary.js";

export type Pf2eTerminalSearchCategoryOption = {
  value: SearchCategory | null;
  label: string;
  description: string;
};

export type Pf2eTerminalSearchProfileOption = {
  value: SearchProfile;
  label: string;
  description: string;
};

export type Pf2eTerminalSearchMode = "lookup" | "search";

export type Pf2eTerminalSearchRequest = {
  category: SearchCategory | null;
  limit: number;
  mode: Pf2eTerminalSearchMode;
  queryText: string;
  searchProfile: SearchProfile;
};

export type Pf2eTerminalSearchSession = {
  request: Pf2eTerminalSearchRequest;
  results: NormalizedRecord[];
  total: number;
  searchProfile: SearchProfile | null;
};

export type Pf2eTerminalSearchService = {
  getCategoryOptions: () => Pf2eTerminalSearchCategoryOption[];
  getProfileOptions: () => Pf2eTerminalSearchProfileOption[];
  runQuery: (request: Pf2eTerminalSearchRequest) => Promise<Pf2eTerminalSearchSession>;
};

type SearchServiceDependencies = {
  getSearchVocabulary: () => SearchVocabularyResult;
  lookup: (name: string, options?: { category?: SearchCategory }) => { match: NormalizedRecord | null; alternatives: NormalizedRecord[] };
  search: (filters: { query: string; limit: number; searchProfile: SearchProfile; category?: SearchCategory }) => Promise<{
    records: NormalizedRecord[];
    total: number;
    searchProfile: SearchProfile | null;
  }>;
};

const SEARCH_PROFILE_OPTIONS: Pf2eTerminalSearchProfileOption[] = [
  {
    value: "balanced",
    label: "Balanced",
    description: "Default hybrid retrieval for concise themed searches.",
  },
  {
    value: "lexical",
    label: "Lexical",
    description: "Exact-wording heavy retrieval for names and precise PF2E terms.",
  },
  {
    value: "concept",
    label: "Concept",
    description: "Semantic-forward retrieval for broader exploratory concept searches.",
  },
];

function formatCategoryLabel(category: SearchCategory): string {
  if (category === "characterCreation") {
    return "Character Creation";
  }
  return `${category[0]!.toUpperCase()}${category.slice(1)}`;
}

export function createPf2eTerminalSearchService(
  dependencies: SearchServiceDependencies,
): Pf2eTerminalSearchService {
  return {
    getCategoryOptions: () => {
      const vocabulary = dependencies.getSearchVocabulary();
      return [
        {
          value: null,
          label: "Any Category",
          description: "Search across the full indexed PF2E corpus.",
        },
        ...vocabulary.categories.map((category: SearchVocabularyResult["categories"][number]) => ({
          value: category.value,
          label: formatCategoryLabel(category.value),
          description: `${category.count} indexed canonical record${category.count === 1 ? "" : "s"}.`,
        })),
      ];
    },
    getProfileOptions: () => SEARCH_PROFILE_OPTIONS,
    runQuery: async (request) => {
      if (request.mode === "lookup") {
        const lookup = dependencies.lookup(request.queryText, request.category ? { category: request.category } : undefined);
        return {
          request,
          results: lookup.match ? [lookup.match, ...lookup.alternatives] : [],
          total: lookup.match ? 1 + lookup.alternatives.length : 0,
          searchProfile: null,
        };
      }

      const result = await dependencies.search({
        query: request.queryText,
        limit: request.limit,
        searchProfile: request.searchProfile,
        ...(request.category ? { category: request.category } : {}),
      });
      return {
        request,
        results: result.records,
        total: result.total,
        searchProfile: result.searchProfile,
      };
    },
  };
}
