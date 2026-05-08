import {
  clampDerivedTagReviewIndex,
  toggleDerivedTagReviewUnresolvedOnly,
  updateDerivedTagReviewDecisionStatus,
} from "../sessions/review-session.js";
import type { DerivedTagReviewSession } from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";
import {
  createDerivedTagTerminalActionTargetState,
  reduceDerivedTagTerminalActionTargetState,
  type DerivedTagTerminalActionTargetAction,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
} from "../../../tui/action-target.js";
import { moveSelection, moveSelectionWrapped } from "../../../tui/terminal-ui.js";
import {
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
  type DerivedTagTerminalTwoPaneState,
} from "../../../tui/two-pane-state.js";

export type DerivedTagReviewScreenState = DerivedTagTerminalTwoPaneState &
  DerivedTagTerminalActionTargetState & {
    imported: boolean;
    session: DerivedTagReviewSession;
  };

export type DerivedTagReviewScreenAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "set_session"; session: DerivedTagReviewSession }
  | { type: "set_imported"; imported: boolean }
  | { type: "move_list_wrapped"; delta: number; itemCount: number }
  | { type: "move_list_clamped"; delta: number; itemCount: number }
  | { type: "list_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | DerivedTagTerminalActionTargetAction
  | {
      type: "apply_decision_status";
      item: { recordIndex: number; decisionIndex: number };
      status: DerivedTagReviewSession["decisions"][number]["decisions"][number]["status"];
    }
  | { type: "toggle_unresolved" };

export const DERIVED_TAG_MIGRATION_REVIEW_ACTIONS = [
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.APPROVE,
    label: "Approve",
    description: "Mark the current review item approved.",
  },
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.REJECT,
    label: "Reject",
    description: "Mark the current review item rejected.",
  },
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.NEEDS_REVIEW,
    label: "Needs Review",
    description: "Keep the item unresolved for later follow-up.",
  },
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.TOGGLE_UNRESOLVED,
    label: "Toggle Unresolved",
    description: "Switch the queue between all items and unresolved-only items.",
  },
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.IMPORT,
    label: "Lint + Import",
    description: "Run import for the current session after validation.",
  },
  {
    id: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.UI_ACTION.QUIT,
    label: "Quit",
    description: "Finish the review UI and return the current session state.",
  },
] as const satisfies readonly DerivedTagTerminalActionTargetOption[];

export type DerivedTagReviewActionId = (typeof DERIVED_TAG_MIGRATION_REVIEW_ACTIONS)[number]["id"];

export function createInitialDerivedTagReviewScreenState(
  initialSession: DerivedTagReviewSession,
): DerivedTagReviewScreenState {
  return {
    activePane: "list",
    ...createDerivedTagTerminalActionTargetState(),
    detailScroll: 0,
    imported: false,
    layoutMode: "split",
    session: clampDerivedTagReviewIndex(initialSession),
  };
}

function setReviewCurrentIndex(session: DerivedTagReviewSession, nextIndex: number): DerivedTagReviewSession {
  const next = structuredClone(session);
  next.reviewState.currentIndex = nextIndex;
  return next;
}

export function reduceDerivedTagReviewScreenState(
  state: DerivedTagReviewScreenState,
  action: DerivedTagReviewScreenAction,
): DerivedTagReviewScreenState {
  switch (action.type) {
    case "toggle_focus":
    case "toggle_layout":
    case "leave_detail":
    case "set_focus":
      return {
        ...state,
        ...reduceDerivedTagTerminalTwoPaneState(state, action),
      };
    case "set_session":
      return {
        ...state,
        session: clampDerivedTagReviewIndex(action.session),
      };
    case "set_imported":
      return {
        ...state,
        imported: action.imported,
      };
    case "move_list_wrapped":
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(
          state.session,
          moveSelectionWrapped(state.session.reviewState.currentIndex, action.delta, action.itemCount),
        ),
      };
    case "move_list_clamped":
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(
          state.session,
          moveSelection(state.session.reviewState.currentIndex, action.delta, action.itemCount),
        ),
      };
    case "list_boundary":
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(state.session, action.boundary === "start" ? 0 : action.itemCount - 1),
      };
    case "move_detail":
    case "detail_boundary":
      return reduceDerivedTagTerminalTwoPaneState(state, action);
    case "toggle_target":
    case "leave_actions":
    case "move_action":
      return reduceDerivedTagTerminalActionTargetState(state, action);
    case "apply_decision_status":
      return {
        ...state,
        session: clampDerivedTagReviewIndex(
          updateDerivedTagReviewDecisionStatus(state.session, action.item, action.status),
        ),
      };
    case "toggle_unresolved":
      return {
        ...state,
        detailScroll: 0,
        session: clampDerivedTagReviewIndex(toggleDerivedTagReviewUnresolvedOnly(state.session)),
      };
    default:
      return state;
  }
}
