import { createContext, useContext, useEffect, useState } from "react";
import type { AppReadinessView, SearchProfileId } from "../../generated/atlas";
import { ATLAS_ROUTE_CHANGE_EVENT } from "../../app/routes";

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
  return new URLSearchParams(window.location.search).get(SEARCH_PROFILE_PARAM);
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
    const restore = () => {
      const explicit = urlProfile();
      if (explicit !== null) setChoice({ id: explicit, source: "url" });
    };
    window.addEventListener("popstate", restore);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, restore);
    return () => {
      window.removeEventListener("popstate", restore);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, restore);
    };
  }, []);
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
      const url = new URL(window.location.href);
      url.searchParams.delete(SEARCH_PROFILE_PARAM);
      history.replaceState(
        { ...history.state, atlasSearchPage: 1 },
        "",
        url.pathname + url.search,
      );
    },
    select: (next) => {
      setChoice({ id: next, source: "selection" });
      try {
        localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, next);
      } catch {
        // Optional preferences never prevent a deliberate selection.
      }
      if (window.location.pathname.startsWith("/search")) {
        const url = new URL(window.location.href);
        url.searchParams.set(SEARCH_PROFILE_PARAM, next);
        history.replaceState(
          { ...history.state, atlasSearchPage: 1 },
          "",
          url.pathname + url.search,
        );
      }
    },
  };
}
