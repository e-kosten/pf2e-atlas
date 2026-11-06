import type { ReactNode } from "react";
import { SearchProfileContext } from "../shared/search/SearchProfileContext";
import { readinessFixture } from "./fixtures";
import { DEFAULT_SEARCH_PREFERENCES } from "../shared/search/searchPreferences";

export function SearchProfileProvider({
  children,
  id = "bge-base",
  usable = true,
  preferRemaster = true,
}: {
  children: ReactNode;
  id?: string;
  usable?: boolean;
  preferRemaster?: boolean;
}) {
  return (
    <SearchProfileContext
      value={{
        id,
        pendingUrlProfile: null,
        profiles: readinessFixture().search_profiles,
        usable,
        storedVectors: true,
        loading: false,
        error: null,
        canResetLink: false,
        resetLink: () => {},
        select: () => {},
        defaultId: "bge-base",
        preferences: DEFAULT_SEARCH_PREFERENCES,
        preferRemaster,
        applyPreferences: () => true,
      }}
    >
      {children}
    </SearchProfileContext>
  );
}
