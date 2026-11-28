import {
  decodeSearchStateFromParams,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { currentSearchLocation } from "../../app/searchLocation";
import { atlasRouter, type AtlasRouter } from "../../app/router";
import type { QueryFieldDefinition, QueryPredicate } from "../../generated/atlas";
import {
  commonCondition,
  choiceClauses,
  conditionValues,
  isExcluded,
  type SearchSubmission,
} from "../../shared/filters/filterControls";
import {
  DEFAULT_SEARCH_PREFERENCES,
  type SearchPreferences,
} from "../../shared/search/searchPreferences";
export const KIND_PREFERENCE_KEY = "atlas.search.kind-default.v2";
type KindPreference = {
  include: string[];
  exclude: string[];
};
function readKindPreference(): KindPreference | null {
  try {
    const stored = localStorage.getItem(KIND_PREFERENCE_KEY);
    // Bound restored browser data while allowing source pack IDs up to the query byte budget.
    if (!stored || stored.length > 128 * 1024) return null;
    const value: unknown = JSON.parse(stored);
    if (
      !value ||
      typeof value !== "object" ||
      !("include" in value) ||
      !("exclude" in value)
    )
      return null;
    const valid = (values: unknown): values is string[] =>
      Array.isArray(values) &&
      values.length <= 128 &&
      values.every((entry) => typeof entry === "string" && entry.trim().length > 0) &&
      new Set(values).size === values.length;
    const include = value.include,
      exclude = value.exclude;
    if (
      !valid(include) ||
      !valid(exclude) ||
      include.some((entry) => exclude.includes(entry))
    )
      return null;
    return { include, exclude };
  } catch {
    return null;
  }
}
export function rememberKindPreference(
  predicate: QueryPredicate | null,
  field: QueryFieldDefinition,
): void {
  const condition = commonCondition(predicate, field);
  if (condition.status === "advanced") return;
  const clauses = choiceClauses(condition.clause);
  const preference: KindPreference = {
    include: clauses.filter((clause) => !isExcluded(clause)).flatMap(conditionValues),
    exclude: clauses.filter(isExcluded).flatMap(conditionValues),
  };
  try {
    if (preference.include.length || preference.exclude.length)
      localStorage.setItem(KIND_PREFERENCE_KEY, JSON.stringify(preference));
    else localStorage.removeItem(KIND_PREFERENCE_KEY);
  } catch {
    /* Browser preferences are optional; committed criteria remain authoritative. */
  }
}
function preferredKindPredicate(): QueryPredicate | null {
  const preference = readKindPreference();
  if (!preference) return null;
  const children: QueryPredicate[] = [];
  for (const [exclude, values] of [
    [false, preference.include],
    [true, preference.exclude],
  ] as const) {
    if (!values.length) continue;
    const leaf: QueryPredicate = {
      kind: "in",
      clause_id: crypto.randomUUID(),
      field: "kind",
      values,
    };
    children.push(exclude ? { kind: "not", predicate: leaf } : leaf);
  }
  return children.length > 1 ? { kind: "all_of", children } : (children[0] ?? null);
}
export type WorkspaceInteractionState = {
  search: SearchFormState;
  selectedRecordKey: string | null;
  activeSearch: SearchFormState;
  resetToken?: string;
  acceptedSearchSubmission?: SearchSubmission;
};
export type WorkspaceInteractionEvent =
  | {
      type: "url.restored";
      resetToken?: string;
      search: SearchFormState;
      selectedRecordKey: string | null;
    }
  | {
      type: "search.changed";
      search: SearchFormState;
    }
  | {
      type: "search.settingsApplied" | "search.submitted";
      search: SearchFormState;
      submission?: SearchSubmission;
    }
  | {
      type: "search.executionCommitted";
      search: SearchFormState;
    }
  | {
      type: "record.selected";
      recordKey: string | null;
      submission?: SearchSubmission;
    };
export function initialWorkspaceInteractionState(
  preferences: SearchPreferences = DEFAULT_SEARCH_PREFERENCES,
  router: AtlasRouter = atlasRouter,
): WorkspaceInteractionState {
  let search = decodeSearchStateFromParams(
    new URLSearchParams(router.latestLocation.searchStr),
    "search",
    preferences,
  );
  const route = currentSearchLocation(router);
  if (
    router.latestLocation.pathname === "/search" &&
    !router.latestLocation.searchStr &&
    !router.latestLocation.state.atlasSearchReset
  )
    search = { ...search, filter: preferredKindPredicate() };
  return {
    search,
    selectedRecordKey: route?.selectedRecordKey ?? null,
    activeSearch: search,
    resetToken: router.latestLocation.state.atlasSearchReset,
  };
}
export function workspaceInteractionReducer(
  state: WorkspaceInteractionState,
  event: WorkspaceInteractionEvent,
): WorkspaceInteractionState {
  switch (event.type) {
    case "url.restored":
      return {
        ...state,
        search: event.search,
        activeSearch: event.search,
        resetToken: event.resetToken,
        selectedRecordKey: event.selectedRecordKey,
        acceptedSearchSubmission: undefined,
      };
    case "search.changed":
      return {
        ...state,
        search: event.search,
      };
    case "search.submitted":
    case "search.settingsApplied":
      return {
        ...state,
        search: event.search,
        activeSearch: event.search,
        acceptedSearchSubmission: event.submission ?? state.acceptedSearchSubmission,
      };
    case "search.executionCommitted":
      if (event.search !== state.search || event.search === state.activeSearch) {
        return state;
      }
      return {
        ...state,
        activeSearch: event.search,
      };
    case "record.selected":
      return {
        ...state,
        selectedRecordKey: event.recordKey,
        acceptedSearchSubmission: event.submission ?? state.acceptedSearchSubmission,
      };
  }
}
