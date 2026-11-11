import type { SearchResultsRequest } from "../../generated/atlas";

export type SearchAvailability = {
  id: string | null;
  storedVectors: boolean;
  usable: boolean;
  keywordSearch?: boolean;
};

export function searchExecution(
  availability: SearchAvailability,
  kind: SearchResultsRequest["criteria"]["kind"],
) {
  const keyword = availability.keywordSearch === true;
  return {
    profile: keyword ? undefined : (availability.id ?? undefined),
    mode: keyword ? ("lexical" as const) : ("hybrid" as const),
    ready: keyword
      ? kind !== "similar_records"
      : availability.id !== null &&
        (kind === "text_search" ? availability.usable : availability.storedVectors),
  };
}
