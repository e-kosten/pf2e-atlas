import type { ReactNode } from "react";
import { SearchProfileContext } from "../shared/search/SearchProfileContext";
import { readinessFixture } from "./fixtures";

export function SearchProfileProvider({
  children,
  id = "bge-base",
  usable = true,
}: {
  children: ReactNode;
  id?: string;
  usable?: boolean;
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
      }}
    >
      {children}
    </SearchProfileContext>
  );
}
