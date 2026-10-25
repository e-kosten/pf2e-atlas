import { atlasRouter, type AtlasRouter } from "./router";
import { selectedNavigation } from "./urlState";
import type { SearchPageInput } from "./destinations";

export function currentSearchLocation(
  router: AtlasRouter = atlasRouter,
): SearchPageInput | null {
  const location = router.latestLocation;
  const matches = router.matchRoutes(location.pathname, location.search);
  const match = matches.find((match) => match.staticData.section === "search");
  if (!match) return null;
  try {
    return {
      selectedRecordKey:
        "recordKey" in match.params ? String(match.params.recordKey) : null,
      query: location.searchStr,
      selection: selectedNavigation(location.search),
      landing: selectedNavigation(location.search, "landing"),
    };
  } catch {
    return null;
  }
}
