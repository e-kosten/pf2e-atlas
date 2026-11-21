import type { ReactNode } from "react";
import {
  SearchProfileContext,
  type SearchProfileSelection,
} from "../shared/search/SearchProfileContext";
import { readinessFixture } from "./fixtures";
import { DEFAULT_SEARCH_PREFERENCES } from "../shared/search/searchPreferences";

export function SearchProfileProvider({
  children,
  id = "bge-base",
  usable = true,
  preferences = DEFAULT_SEARCH_PREFERENCES,
  resolveWorkspaceDefault,
}: {
  children: ReactNode;
  id?: string;
  usable?: boolean;
  preferences?: SearchProfileSelection["preferences"];
  resolveWorkspaceDefault?: SearchProfileSelection["resolveWorkspaceDefault"];
}) {
  return (
    <SearchProfileContext
      value={{
        id,
        pendingUrlProfile: null,
        profiles: readinessFixture().search_profiles,
        usable,
        resolveWorkspaceDefault,
        storedVectors: true,
        loading: false,
        error: null,
        canResetLink: false,
        resetLink: () => {},
        select: () => {},
        defaultId: "bge-base",
        preferences,
        applyPreferences: () => true,
      }}
    >
      {children}
    </SearchProfileContext>
  );
}
