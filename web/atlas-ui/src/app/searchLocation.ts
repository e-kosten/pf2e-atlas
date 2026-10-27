import { atlasRouter, type AtlasRouter } from "./router";
import { selectedNavigation, readingView } from "./urlState";
import type { SearchPageInput } from "./destinations";

export function currentSearchLocation(
  router: AtlasRouter = atlasRouter,
): SearchPageInput | null {
  const location = router.latestLocation;
  const matches = router.matchRoutes(location.pathname, location.search);
  const match = matches.find((match) => match.staticData.section === "search");
  if (!match) return null;
  try {
    const recordKey =
      "recordKey" in match.params ? String(match.params.recordKey) : null;
    return {
      selectedRecordKey: recordKey,
      query: location.searchStr,
      selection: recordKey ? selectedNavigation(location.search, recordKey) : undefined,
      landing: recordKey
        ? selectedNavigation(location.search, recordKey, "focus")
        : undefined,
      ...readingView(location.search),
    };
  } catch {
    return null;
  }
}
