import type { Pf2eDataService } from "../../../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult } from "../../../data/vocabulary.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchResult } from "../../../domain/search-types.js";

export type SearchSemanticsDataService = Pick<Pf2eDataService, "getPack" | "listRecords"> &
  Partial<Pick<Pf2eDataService, "search">> & {
    getSearchSemanticsBootstrapSummary: (options?: {
      traitLimitPerCategory?: number;
    }) => SearchSemanticsBootstrapSummaryResult;
  };

export type SearchSemanticsRecordsDataService = {
  listRecords: (request: SearchRequest) => SearchResult;
  search?: (request: SearchRequest) => Promise<SearchResult>;
  getPack?: (packValue: string) => { name: string; label?: string } | undefined;
};

export type ValueNodeQueryOptions = {
  countLabel?: string;
  matchingRequest?: Readonly<SearchRequest>;
};
