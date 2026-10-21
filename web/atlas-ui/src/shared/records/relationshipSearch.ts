import { currentAtlasRoute, type AtlasRoute } from "../../app/routes";
import type { RelationshipConstraintView } from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";

/** Known-record entry points replace the current relationship and retain search criteria. */
export function relationshipSearchRoute(
  constraint: RelationshipConstraintView,
  profile?: string | null,
): AtlasRoute {
  const route = currentAtlasRoute();
  const params = new URLSearchParams(window.location.search);
  const search =
    route.kind === "search"
      ? decodeSearchStateFromParams(params)
      : DEFAULT_SEARCH_STATE;
  return {
    kind: "search",
    selectedRecordKey: null,
    query: searchStateQueryString(
      { ...search, relationships: [constraint] },
      profile ?? params.get("search_profile"),
    ),
  };
}
