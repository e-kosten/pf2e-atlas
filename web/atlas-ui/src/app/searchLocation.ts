import { atlasRouter } from "./router";
import { selectedNavigation } from "./urlState";
import type { SearchPageInput } from "./destinations";

export function currentSearchLocation(): SearchPageInput | null {
  const location = atlasRouter.latestLocation;
  const matches = atlasRouter.matchRoutes(location.pathname, location.search);
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
