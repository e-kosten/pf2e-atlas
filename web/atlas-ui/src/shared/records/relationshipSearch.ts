import { searchDestination } from "../../app/destinations";
import { appendLink } from "../filters/linkPredicates";
import { currentSearchLocation } from "../../app/searchLocation";
import type { QueryPredicate } from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";
/** Known-record links AND one exact condition with the complete active expression. */
export function relationshipSearchRoute(
  filter: QueryPredicate,
  profile?: string | null,
  preferRemaster = true,
) {
  const route = currentSearchLocation();
  const params = new URLSearchParams(window.location.search);
  const search =
    route !== null ? decodeSearchStateFromParams(params) : DEFAULT_SEARCH_STATE;
  const next =
    filter.kind === "relationship"
      ? appendLink(search.filter, filter)
      : search.filter
        ? { kind: "all_of" as const, children: [search.filter, filter] }
        : filter;
  if (route !== null && next === search.filter) {
    if (!params.has("versions"))
      params.set("versions", preferRemaster ? "prefer" : "all");
    return searchDestination({ ...route, query: `?${params}` });
  }
  return searchDestination({
    selectedRecordKey: null,
    query: searchStateQueryString(
      {
        ...search,
        preferRemaster: params.has("versions") ? search.preferRemaster : preferRemaster,
        filter: next,
      },
      profile ?? params.get("search_profile"),
    ),
  });
}
