import type { OntologyDomainModel } from "../domain/ontology-types.js";
import type { OntologyNodeQuery } from "../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../domain/search-field-domains.js";
import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import type {
  DerivedTagWorkbenchMode,
  DerivedTagReviewDecisionKind,
  DerivedTagReviewSession,
} from "../tags/editorial.js";
import type { OntologyInspectExplorerSnapshot } from "./ontology-explorer/inspect-screen.js";
import type { Pf2eTerminalSearchSession } from "./search/service.js";
import { moveSelectionWrapped } from "./framework/input.js";

export const PF2E_APP_ROUTE_KIND = {
  AREAS: "areas",
  TAG_REFINEMENT: "tag_refinement",
  SEARCH: "search",
  ONTOLOGY: "ontology",
  REVIEW: "review",
} as const;

export const PF2E_APP_AREA_ID = {
  TAG_REFINEMENT: "tag_refinement",
  ONTOLOGY_SEARCH: "ontology_search",
  SEARCH: "search",
} as const;

export const PF2E_SEARCH_ROUTE_ENTRY_KIND = {
  EDITOR: "editor",
  RESULTS: "results",
} as const;

export const PF2E_SEARCH_ROUTE_ORIGIN_KIND = {
  ONTOLOGY: "ontology",
} as const;

export type Pf2eAppAreaId = (typeof PF2E_APP_AREA_ID)[keyof typeof PF2E_APP_AREA_ID];

export type Pf2eOntologyRoute = {
  kind: (typeof PF2E_APP_ROUTE_KIND)["ONTOLOGY"];
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  snapshot?: OntologyInspectExplorerSnapshot;
};

export type Pf2eSearchRouteOrigin = {
  kind: (typeof PF2E_SEARCH_ROUTE_ORIGIN_KIND)["ONTOLOGY"];
  route: Pf2eOntologyRoute;
};

export type Pf2eSearchEditorRoute = {
  kind: (typeof PF2E_APP_ROUTE_KIND)["SEARCH"];
  entry: (typeof PF2E_SEARCH_ROUTE_ENTRY_KIND)["EDITOR"];
  initialQuery?: OntologyNodeQuery;
  initialSession?: never;
  origin?: Pf2eSearchRouteOrigin;
};

export type Pf2eSearchResultsRoute = {
  kind: (typeof PF2E_APP_ROUTE_KIND)["SEARCH"];
  entry: (typeof PF2E_SEARCH_ROUTE_ENTRY_KIND)["RESULTS"];
  initialQuery?: never;
  initialSession: Pf2eTerminalSearchSession;
  origin?: Pf2eSearchRouteOrigin;
};

export type Pf2eSearchRoute = Pf2eSearchEditorRoute | Pf2eSearchResultsRoute;

export type Pf2eAppRoute =
  | { kind: (typeof PF2E_APP_ROUTE_KIND)["AREAS"] }
  | { kind: (typeof PF2E_APP_ROUTE_KIND)["TAG_REFINEMENT"] }
  | Pf2eSearchRoute
  | Pf2eOntologyRoute
  | { kind: (typeof PF2E_APP_ROUTE_KIND)["REVIEW"]; session: DerivedTagReviewSession };

export type CreatePf2eDerivedTagSessionOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  decisionKind?: DerivedTagReviewDecisionKind;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export type StartPf2eDerivedTagSessionMode = "review_all" | DerivedTagWorkbenchMode;

export type Pf2eAppState = {
  routeStack: Pf2eAppRoute[];
  selectedAreaIndex: number;
  tagRefinementSelectedIndex: number;
};

type Pf2eAppAction =
  | { type: "move_area"; delta: number; itemCount: number }
  | { type: "move_tag_refinement"; delta: number; itemCount: number }
  | { type: "set_tag_refinement_index"; index: number; itemCount: number }
  | { type: "push_route"; route: Pf2eAppRoute }
  | { type: "replace_route"; route: Pf2eAppRoute }
  | { type: "pop_route" };

export function createPf2eOntologyRoute({
  model,
  initialDiscoveryMode,
  loadModelForDiscoveryMode,
  snapshot,
}: {
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  snapshot?: OntologyInspectExplorerSnapshot;
}): Pf2eOntologyRoute {
  return {
    kind: PF2E_APP_ROUTE_KIND.ONTOLOGY,
    model,
    ...(initialDiscoveryMode ? { initialDiscoveryMode } : {}),
    ...(loadModelForDiscoveryMode ? { loadModelForDiscoveryMode } : {}),
    snapshot,
  };
}

export function createPf2eSearchEditorRoute({
  initialQuery,
  origin,
}: {
  initialQuery?: OntologyNodeQuery;
  origin?: Pf2eSearchRouteOrigin;
} = {}): Pf2eSearchEditorRoute {
  return {
    kind: PF2E_APP_ROUTE_KIND.SEARCH,
    entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.EDITOR,
    ...(initialQuery ? { initialQuery } : {}),
    ...(origin ? { origin } : {}),
  };
}

export function createPf2eSearchResultsRoute({
  initialSession,
  origin,
}: {
  initialSession: Pf2eTerminalSearchSession;
  origin?: Pf2eSearchRouteOrigin;
}): Pf2eSearchResultsRoute {
  return {
    kind: PF2E_APP_ROUTE_KIND.SEARCH,
    entry: PF2E_SEARCH_ROUTE_ENTRY_KIND.RESULTS,
    initialSession,
    ...(origin ? { origin } : {}),
  };
}

export function createPf2eAppState(initialRoute: Pf2eAppRoute = { kind: PF2E_APP_ROUTE_KIND.AREAS }): Pf2eAppState {
  return {
    routeStack: [initialRoute],
    selectedAreaIndex: 0,
    tagRefinementSelectedIndex: 0,
  };
}

export function getCurrentPf2eAppRoute(state: Pf2eAppState): Pf2eAppRoute {
  return state.routeStack[state.routeStack.length - 1] ?? { kind: PF2E_APP_ROUTE_KIND.AREAS };
}

export function canPopPf2eAppRoute(state: Pf2eAppState): boolean {
  return state.routeStack.length > 1;
}

export function pf2eAppReducer(state: Pf2eAppState, action: Pf2eAppAction): Pf2eAppState {
  switch (action.type) {
    case "move_area":
      return {
        ...state,
        selectedAreaIndex:
          action.itemCount <= 0 ? 0 : moveSelectionWrapped(state.selectedAreaIndex, action.delta, action.itemCount),
      };
    case "move_tag_refinement":
      return {
        ...state,
        tagRefinementSelectedIndex:
          action.itemCount <= 0
            ? 0
            : moveSelectionWrapped(state.tagRefinementSelectedIndex, action.delta, action.itemCount),
      };
    case "set_tag_refinement_index":
      return {
        ...state,
        tagRefinementSelectedIndex:
          action.itemCount <= 0 ? 0 : Math.max(0, Math.min(action.index, action.itemCount - 1)),
      };
    case "push_route":
      return {
        ...state,
        routeStack: [...state.routeStack, action.route],
      };
    case "replace_route":
      return {
        ...state,
        routeStack: [...state.routeStack.slice(0, -1), action.route],
      };
    case "pop_route":
      return {
        ...state,
        routeStack: state.routeStack.length > 1 ? state.routeStack.slice(0, -1) : state.routeStack,
      };
    default:
      return state;
  }
}
