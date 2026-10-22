import { appendLink } from "../filters/linkPredicates";
import { currentAtlasRoute, type AtlasRoute } from "../../app/routes";
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
): AtlasRoute {
  const route = currentAtlasRoute();
  const params = new URLSearchParams(window.location.search);
  const search =
    route.kind === "search"
      ? decodeSearchStateFromParams(params)
      : DEFAULT_SEARCH_STATE;
  const next =
    filter.kind === "relationship"
      ? appendLink(search.filter, filter)
      : search.filter
        ? { kind: "all_of" as const, children: [search.filter, filter] }
        : filter;
  if (route.kind === "search" && next === search.filter) return route;
  return {
    kind: "search",
    selectedRecordKey: null,
    query: searchStateQueryString(
      { ...search, filter: next },
      profile ?? params.get("search_profile"),
    ),
  };
}
