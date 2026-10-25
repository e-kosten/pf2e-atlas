import { createContext, useContext, useEffect, useState } from "react";
import type { AppReadinessView, SearchProfileId } from "../../generated/atlas";
import { atlasRouter } from "../../app/router";
import { useRouterState } from "@tanstack/react-router";
import { searchParam } from "../../app/urlState";
import { currentSearchLocation } from "../../app/searchLocation";
export const SEARCH_PROFILE_PREFERENCE_KEY = "atlas.search.profile.v1";
export const SEARCH_PROFILE_PARAM = "search_profile";
export type SearchProfileSelection = {
  id: SearchProfileId | null;
  pendingUrlProfile: string | null;
  profiles: AppReadinessView["search_profiles"];
  storedVectors: boolean;
  usable: boolean;
  loading: boolean;
  error: string | null;
  canResetLink: boolean;
  resetLink: () => void;
  select: (id: SearchProfileId) => void;
};
export const SearchProfileContext = createContext<SearchProfileSelection>({
  id: null,
  pendingUrlProfile: null,
  profiles: [],
  storedVectors: false,
  usable: false,
  loading: true,
  error: null,
  canResetLink: false,
  resetLink: () => {},
  select: () => {},
});
export function useSearchProfile() {
  return useContext(SearchProfileContext);
}
function urlProfile() {
  return searchParam(atlasRouter.latestLocation.search, SEARCH_PROFILE_PARAM) ?? null;
}
function storedProfile() {
  try {
    return localStorage.getItem(SEARCH_PROFILE_PREFERENCE_KEY);
  } catch {
    return null;
  }
}
type ProfileChoice = {
  id: string;
  source: "url" | "preference" | "default" | "selection";
};
function initialChoice(): ProfileChoice | null {
  const explicit = urlProfile();
  if (explicit !== null) return { id: explicit, source: "url" };
  const preference = storedProfile();
  return preference === null ? null : { id: preference, source: "preference" };
}
function defaultChoice(readiness: AppReadinessView): ProfileChoice {
  const stored = readiness.search_profiles.filter((profile) => profile.stored_vectors);
  const server = stored.find(
    (profile) => profile.id === readiness.default_search_profile && profile.usable,
  );
  const id =
    server?.id ??
    (stored.length === 1 ? stored[0].id : readiness.default_search_profile);
  return { id, source: "default" };
}
/** AtlasApp owns this tab's choice; storage is consulted only on initial launch. */
export function useSearchProfileSelection(
  readiness: AppReadinessView | undefined,
): SearchProfileSelection {
  const [choice, setChoice] = useState(initialChoice);
  const explicitProfile = useRouterState({
    select: (state) => searchParam(state.location.search, SEARCH_PROFILE_PARAM),
  });
  const preferred = readiness?.search_profiles.find(
    (profile) => profile.id === choice?.id,
  );
  // A preference is only a launch default. Explicit links never silently fall back.
  const resolved =
    readiness && (choice === null || choice.source === "preference")
      ? choice && preferred?.stored_vectors && preferred.usable
        ? { id: choice.id, source: "default" as const }
        : defaultChoice(readiness)
      : choice;
  if (resolved !== choice) setChoice(resolved);
  useEffect(() => {
    if (explicitProfile !== undefined)
      setChoice((previous) =>
        previous?.id === explicitProfile
          ? previous
          : { id: explicitProfile, source: "url" },
      );
  }, [explicitProfile]);
  const id = readiness ? resolved!.id : null;
  const profiles = readiness?.search_profiles ?? [];
  const selected = profiles.find((profile) => profile.id === id);
  const single = profiles.filter((profile) => profile.stored_vectors).length === 1;
  const canResetLink = id !== null && (!selected || !selected.stored_vectors);
  const error =
    id === null
      ? null
      : canResetLink
        ? single
          ? "This search link is unavailable for this collection. Reset the search link to continue."
          : `Search model “${selected?.label ?? id}” is not stored in this collection. Choose an available search model.`
        : !selected!.usable
          ? single
            ? "Text search is unavailable for this collection. Check setup to continue. Similar records remain available."
            : `Search model “${selected!.label}” is unavailable for text search. Choose another model or check setup. Stored similar records remain available.`
          : null;
  return {
    id,
    // Preserve explicit link intent while inference still waits for metadata.
    pendingUrlProfile: !readiness && choice?.source === "url" ? choice.id : null,
    profiles,
    storedVectors: selected?.stored_vectors ?? false,
    usable: selected?.usable ?? false,
    loading: !readiness,
    error,
    canResetLink,
    resetLink: () => {
      if (!readiness) return;
      setChoice(defaultChoice(readiness));
      void atlasRouter.navigate({
        to: ".",
        search: (search) => {
          const next = { ...search };
          delete next[SEARCH_PROFILE_PARAM];
          return next;
        },
        replace: true,
        resetScroll: false,
        state: (state) => ({ ...state, atlasSearchPage: 1 }),
      });
    },
    select: (next) => {
      setChoice({ id: next, source: "selection" });
      try {
        localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, next);
      } catch {
        // Optional preferences never prevent a deliberate selection.
      }
      if (currentSearchLocation())
        void atlasRouter.navigate({
          to: ".",
          search: (search) => ({ ...search, [SEARCH_PROFILE_PARAM]: next }),
          replace: true,
          resetScroll: false,
          state: (state) => ({ ...state, atlasSearchPage: 1 }),
        });
    },
  };
}
