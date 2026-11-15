import { atlasRouter, type AtlasRouter } from "../../app/router";
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
  router: AtlasRouter = atlasRouter,
) {
  const route = currentSearchLocation(router);
  const params = new URLSearchParams(
    router === atlasRouter ? window.location.search : router.latestLocation.searchStr,
  );
  const search =
    route !== null ? decodeSearchStateFromParams(params) : DEFAULT_SEARCH_STATE;
  const next =
    filter.kind === "relationship"
      ? appendLink(search.filter, filter)
      : search.filter
        ? { kind: "all_of" as const, children: [search.filter, filter] }
        : filter;
  if (route !== null && next === search.filter) {
    return searchDestination({ ...route, query: `?${params}` });
  }
  return searchDestination({
    selectedRecordKey: null,
    query: searchStateQueryString(
      {
        ...search,
        filter: next,
      },
      profile ?? params.get("search_profile"),
    ),
  });
}
