import {
  decodeSearchStateFromParams,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { currentSearchLocation } from "../../app/searchLocation";
import { atlasRouter } from "../../app/router";
import type { QueryFieldDefinition, QueryPredicate } from "../../generated/atlas";
import {
  commonCondition,
  choiceClauses,
  conditionValues,
  isExcluded,
} from "../../shared/filters/filterControls";
import {
  DEFAULT_SEARCH_PREFERENCES,
  type SearchPreferences,
} from "../../shared/search/searchPreferences";
export const KIND_PREFERENCE_KEY = "atlas.search.kind-default.v1";
type KindPreference = {
  include: string[];
  exclude: string[];
};
function readKindPreference(): KindPreference | null {
  try {
    const value: unknown = JSON.parse(
      localStorage.getItem(KIND_PREFERENCE_KEY) ?? "null",
    );
    if (
      !value ||
      typeof value !== "object" ||
      !("include" in value) ||
      !("exclude" in value)
    )
      return null;
    const valid = (values: unknown): values is string[] =>
      Array.isArray(values) &&
      values.length <= 100 &&
      values.every(
        (entry) =>
          typeof entry === "string" && entry.trim().length > 0 && entry.length <= 100,
      ) &&
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
  pageNumber: number;
  activeSearch: SearchFormState;
};
export type WorkspaceInteractionEvent =
  | {
      type: "url.restored";
      pageNumber?: number;
      search: SearchFormState;
      selectedRecordKey: string | null;
    }
  | {
      type: "search.changed";
      search: SearchFormState;
    }
  | {
      type: "search.settingsApplied";
      search: SearchFormState;
    }
  | {
      type: "search.executionCommitted";
      search: SearchFormState;
    }
  | {
      type: "resultPage.changed";
      pageNumber: number;
    }
  | {
      type: "record.selected";
      recordKey: string | null;
    };
export function initialWorkspaceInteractionState(
  preferences: SearchPreferences = DEFAULT_SEARCH_PREFERENCES,
): WorkspaceInteractionState {
  let search = decodeSearchStateFromParams(
    new URLSearchParams(atlasRouter.latestLocation.searchStr),
    "search",
    preferences,
  );
  const route = currentSearchLocation();
  if (
    atlasRouter.latestLocation.pathname === "/search" &&
    !atlasRouter.latestLocation.searchStr
  )
    search = { ...search, filter: preferredKindPredicate() };
  return {
    search,
    selectedRecordKey: route?.selectedRecordKey ?? null,
    pageNumber:
      !search.filterWasReset &&
      Number.isSafeInteger(atlasRouter.latestLocation.state.atlasSearchPage) &&
      (atlasRouter.latestLocation.state.atlasSearchPage ?? 0) > 0
        ? atlasRouter.latestLocation.state.atlasSearchPage!
        : 1,
    activeSearch: search,
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
        selectedRecordKey: event.selectedRecordKey,
        pageNumber:
          Number.isSafeInteger(event.pageNumber) && event.pageNumber! > 0
            ? event.pageNumber!
            : 1,
      };
    case "search.changed":
      return {
        ...state,
        search: event.search,
      };
    case "search.settingsApplied":
      return {
        ...state,
        search: event.search,
        activeSearch: event.search,
        pageNumber: 1,
      };
    case "search.executionCommitted":
      if (event.search === state.activeSearch) {
        return state;
      }
      return {
        ...state,
        activeSearch: event.search,
        pageNumber: 1,
      };
    case "resultPage.changed":
      return {
        ...state,
        pageNumber: event.pageNumber,
      };
    case "record.selected":
      return {
        ...state,
        selectedRecordKey: event.recordKey,
      };
  }
}
