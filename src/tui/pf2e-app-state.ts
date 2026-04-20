import type { OntologyNodeQuery } from "../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
} from "../tags/editorial/types.js";
import type { OntologyInspectExplorerSnapshot } from "./ontology-explorer/inspect-screen.js";
import { moveSelectionWrapped } from "./framework/input.js";

export type Pf2eOntologyRoute = {
  kind: "ontology";
  snapshot?: OntologyInspectExplorerSnapshot;
};

export type Pf2eSearchRouteOrigin = {
  kind: "ontology";
  route: Pf2eOntologyRoute;
};

export type Pf2eSearchRoute = {
  kind: "search";
  initialQuery?: OntologyNodeQuery;
  origin?: Pf2eSearchRouteOrigin;
};

export type Pf2eAppRoute =
  | { kind: "areas" }
  | { kind: "tag_refinement" }
  | Pf2eSearchRoute
  | Pf2eOntologyRoute
  | { kind: "review"; session: DerivedTagMigrationSession };

export type CreatePf2eDerivedTagSessionOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  decisionKind?: DerivedTagMigrationReviewDecisionKind;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export type StartPf2eDerivedTagSessionMode = "review_all" | DerivedTagMigrationMode;

export type Pf2eAppState = {
  routeStack: Pf2eAppRoute[];
  selectedAreaIndex: number;
  tagRefinementSelectedIndex: number;
};

export type Pf2eAppAction =
  | { type: "move_area"; delta: number; itemCount: number }
  | { type: "move_tag_refinement"; delta: number; itemCount: number }
  | { type: "set_tag_refinement_index"; index: number; itemCount: number }
  | { type: "push_route"; route: Pf2eAppRoute }
  | { type: "replace_route"; route: Pf2eAppRoute }
  | { type: "pop_route" };

export function createPf2eAppState(initialRoute: Pf2eAppRoute = { kind: "areas" }): Pf2eAppState {
  return {
    routeStack: [initialRoute],
    selectedAreaIndex: 0,
    tagRefinementSelectedIndex: 0,
  };
}

export function getCurrentPf2eAppRoute(state: Pf2eAppState): Pf2eAppRoute {
  return state.routeStack[state.routeStack.length - 1] ?? { kind: "areas" };
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
