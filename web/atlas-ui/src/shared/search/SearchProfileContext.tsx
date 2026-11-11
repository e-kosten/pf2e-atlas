import { createContext, useContext, useState } from "react";
import type { AppReadinessView, SearchProfileId } from "../../generated/atlas";
import { useRouter, useRouterState } from "@tanstack/react-router";
import { searchParam } from "../../app/urlState";
import { currentSearchLocation } from "../../app/searchLocation";
import type { SearchAvailability } from "./searchExecution";
import {
  DEFAULT_SEARCH_PREFERENCES,
  LEGACY_SEARCH_PROFILE_KEY,
  readSearchPreferences,
  writeSearchPreferences,
  type SearchPreferences,
} from "./searchPreferences";
export const SEARCH_PROFILE_PREFERENCE_KEY = LEGACY_SEARCH_PROFILE_KEY;
export const SEARCH_PROFILE_PARAM = "search_profile";
export type SearchProfileSelection = SearchAvailability & {
  pendingUrlProfile: string | null;
  profiles: AppReadinessView["search_profiles"];
  loading: boolean;
  error: string | null;
  canResetLink: boolean;
  resetLink: () => void;
  useKeywordSearch?: () => void;
  automaticKeyword?: boolean;
  semanticDefault?: SearchProfileId;
  promoteKeyword?: (id: SearchProfileId) => void;
  select: (id: SearchProfileId) => void;
  defaultId: SearchProfileId | null;
  preferences: SearchPreferences;
  applyPreferences: (
    preferences: SearchPreferences,
    selectedModel?: string | null,
  ) => boolean;
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
  defaultId: null,
  preferences: DEFAULT_SEARCH_PREFERENCES,
  applyPreferences: () => true,
});
export function useSearchProfile() {
  return useContext(SearchProfileContext);
}
type ProfileChoice =
  | { id: string; source: "url" | "preference" | "default" | "selection" }
  | { id: null; source: "keyword" | "keyword_recovery" };
const KEYWORD_CHOICE: ProfileChoice = { id: null, source: "keyword" };
function initialChoice(
  explicit: string | null,
  preference: string | null,
): ProfileChoice | null {
  if (explicit !== null) return { id: explicit, source: "url" };
  return preference === null ? null : { id: preference, source: "preference" };
}
function defaultChoice(readiness: AppReadinessView): ProfileChoice {
  const stored = readiness.search_profiles.filter((profile) => profile.stored_vectors);
  if (stored.length === 0) return KEYWORD_CHOICE;
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
  const router = useRouter();
  const [preferences, setPreferences] = useState(readSearchPreferences);
  const [choice, setChoice] = useState(() =>
    initialChoice(
      searchParam(router.latestLocation.search, SEARCH_PROFILE_PARAM) ?? null,
      preferences.model,
    ),
  );
  const explicitProfile = useRouterState({
    select: (state) => searchParam(state.location.search, SEARCH_PROFILE_PARAM),
  });
  const profiles = readiness?.search_profiles ?? [];
  const stored = profiles.filter((profile) => profile.stored_vectors);
  const preferred = stored.find((profile) => profile.id === choice?.id);
  const metadataReady = readiness?.status === "ready";
  // A preference is only a launch default. Explicit links never silently fall back.
  const resolved =
    metadataReady &&
    (choice === null ||
      choice.source === "preference" ||
      (choice.source === "default" && stored.length === 0))
      ? choice && preferred?.usable
        ? { id: choice.id, source: "default" as const }
        : defaultChoice(readiness)
      : choice;
  if (resolved !== choice) setChoice(resolved);
  const [observedProfile, setObservedProfile] = useState(explicitProfile);
  if (observedProfile !== explicitProfile) {
    setObservedProfile(explicitProfile);
    if (explicitProfile !== undefined && choice?.id !== explicitProfile)
      setChoice({ id: explicitProfile, source: "url" });
  }
  const id = readiness ? (resolved?.id ?? null) : null;
  const selected = profiles.find((profile) => profile.id === id);
  const single = stored.length === 1;
  const keywordSearch =
    metadataReady &&
    (resolved?.source === "keyword" || resolved?.source === "keyword_recovery");
  const semanticDefault = metadataReady
    ? (stored.find((item) => item.id === preferences.model && item.usable)?.id ??
      stored.find((item) => item.id === defaultChoice(readiness).id && item.usable)
        ?.id ??
      stored.find((item) => item.usable)?.id)
    : undefined;
  const canResetLink =
    metadataReady && id !== null && (!selected || !selected.stored_vectors);
  const error =
    !metadataReady || id === null
      ? null
      : canResetLink
        ? stored.length === 0
          ? "This search link needs a semantic index. You can continue with keyword search."
          : single
            ? "This search link is unavailable for this collection. Reset the search link to continue."
            : `Search model “${selected?.label ?? id}” is not stored in this collection. Choose an available search model.`
        : !selected!.usable
          ? single
            ? "Text search is unavailable for this collection. Check setup to continue. Similar records remain available."
            : `Search model “${selected!.label}” is unavailable for text search. Choose another model or check setup. Stored similar records remain available.`
          : null;
  function clearUrlProfile(next: ProfileChoice) {
    setChoice(next);
    void router.navigate({
      to: ".",
      hash: true,
      search: (search) => {
        const next = { ...search };
        delete next[SEARCH_PROFILE_PARAM];
        delete next.page;
        return next;
      },
      replace: true,
      resetScroll: false,
      state: true,
    });
  }
  return {
    id,
    keywordSearch,
    automaticKeyword: resolved?.source === "keyword",
    semanticDefault,
    promoteKeyword: (id) => setChoice({ id, source: "default" }),
    // Preserve explicit link intent while inference still waits for metadata.
    pendingUrlProfile: !readiness && choice?.source === "url" ? choice.id : null,
    profiles,
    storedVectors: metadataReady && (selected?.stored_vectors ?? false),
    usable: metadataReady && (selected?.usable ?? false),
    loading: !readiness,
    error,
    canResetLink,
    defaultId: readiness ? defaultChoice(readiness).id : null,
    preferences,
    applyPreferences: (next, selectedModel = next.model) => {
      setPreferences(next);
      if (selectedModel !== null) setChoice({ id: selectedModel, source: "selection" });
      return writeSearchPreferences(next);
    },
    resetLink: () => {
      if (metadataReady) clearUrlProfile(defaultChoice(readiness));
    },
    useKeywordSearch:
      canResetLink && stored.length === 0
        ? () => clearUrlProfile({ id: null, source: "keyword_recovery" })
        : undefined,
    select: (next) => {
      setChoice({ id: next, source: "selection" });
      const updated = { ...preferences, model: next };
      setPreferences(updated);
      writeSearchPreferences(updated);
      if (currentSearchLocation(router))
        void router.navigate({
          to: ".",
          hash: true,
          search: (search) => ({
            ...search,
            [SEARCH_PROFILE_PARAM]: next,
            page: undefined,
          }),
          replace: true,
          resetScroll: false,
          state: true,
        });
    },
  };
}
