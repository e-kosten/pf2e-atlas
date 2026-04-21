import React from "react";

import {
  DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
  importDerivedTagReviewSession,
  persistDerivedTagReviewSession,
  type DerivedTagReviewServices,
} from "./review-controller.js";
import {
  buildDerivedTagReviewViewModel,
  type DerivedTagReviewScreenModel,
} from "./review-screen-model.js";
import {
  createInitialDerivedTagReviewScreenState,
  DERIVED_TAG_MIGRATION_REVIEW_ACTIONS,
  reduceDerivedTagReviewScreenState,
  type DerivedTagReviewActionId,
} from "./review-screen-state.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../../../tui/terminal-ui.js";
import {
  showTerminalReturnDialog,
  useTerminalInteractionContextAdapters,
} from "../../../tui/interaction-context-adapters.js";
import {
  createTerminalActionTargetInteractionContext,
  createTerminalDetailInteractionContext,
  createTerminalListInteractionContext,
  useTerminalInteractionContextRouter,
} from "../../../tui/interaction-context-router.js";
import { getDerivedTagTerminalTwoPaneLayoutMode } from "../../../tui/two-pane-state.js";
import type { DerivedTagReviewSession } from "../types.js";

export function useDerivedTagReviewScreenController({
  rootPath,
  initialSession,
  onComplete,
  services = DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
}: {
  rootPath: string;
  initialSession: DerivedTagReviewSession;
  onComplete: (result: { imported: boolean; session: DerivedTagReviewSession }) => void;
  services?: DerivedTagReviewServices;
}): { screen: DerivedTagReviewScreenModel } {
  const terminal = useDerivedTagTerminalApp();
  const adapters = useTerminalInteractionContextAdapters();
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    reduceDerivedTagReviewScreenState,
    initialSession,
    createInitialDerivedTagReviewScreenState,
  );
  const [busy, setBusy] = React.useState(false);
  const [persistError, setPersistError] = React.useState<string | null>(null);

  React.useEffect(() => {
    let cancelled = false;

    void persistDerivedTagReviewSession(rootPath, state.session, services)
      .then(() => {
        if (!cancelled) {
          setPersistError(null);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPersistError((error as Error).message);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [rootPath, services, state.session]);

  const screenModel = buildDerivedTagReviewViewModel({
    persistError,
    size,
    state: {
      ...state,
      layoutMode: getDerivedTagTerminalTwoPaneLayoutMode(state),
    },
  });

  const completeReview = React.useCallback(
    (imported: boolean, session: DerivedTagReviewSession) => {
      onComplete({ imported, session });
    },
    [onComplete],
  );

  const handleImport = React.useCallback(async () => {
    setBusy(true);
    try {
      await importDerivedTagReviewSession(rootPath, state.session, services);
      dispatch({ type: "set_imported", imported: true });
      setPersistError(null);
      await terminal.pauseForAnyKey(`Imported session ${state.session.manifest.id}.`);
      completeReview(true, state.session);
    } catch (error) {
      await terminal.pauseForAnyKey(`Import failed: ${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [completeReview, rootPath, services, state.session, terminal]);

  const requestAction = React.useCallback(
    async (actionId: DerivedTagReviewActionId) => {
      if (actionId === "quit") {
        completeReview(state.imported, state.session);
        return;
      }
      if (actionId === "toggle_unresolved") {
        dispatch({ type: "toggle_unresolved" });
        return;
      }
      if (actionId === "import") {
        await handleImport();
        return;
      }
      if (screenModel.items.length === 0) {
        return;
      }
      const item = screenModel.items[state.session.reviewState.currentIndex];
      if (!item) {
        return;
      }
      if (actionId === "approve") {
        dispatch({ type: "apply_decision_status", item, status: "approved" });
      } else if (actionId === "reject") {
        dispatch({ type: "apply_decision_status", item, status: "rejected" });
      } else {
        dispatch({ type: "apply_decision_status", item, status: "needs_review" });
      }
    },
    [completeReview, handleImport, screenModel.items, state.imported, state.session],
  );

  useTerminalInteractionContextRouter({
    enabled: !busy,
    contexts: [
      createTerminalListInteractionContext("list", {
        interactionActions: screenModel.paneInteractionActions,
        pageSize: screenModel.pageSize,
        jumpSize: screenModel.selectionJumpSize,
      }),
      createTerminalDetailInteractionContext("detail", {
        interactionActions: screenModel.paneInteractionActions,
        pageSize: screenModel.pageSize,
        jumpSize: screenModel.detailJumpSize,
      }),
      createTerminalActionTargetInteractionContext("actionTarget", {
        interactionActions: [...screenModel.actionTargetInteractionActions, { id: "help" }],
        state,
        orientation: "horizontal",
      }),
    ],
    onRoute: ({ actionTarget, detail, list }) => {
      if (actionTarget.actionTargetIntent?.kind === "toggle_target") {
        dispatch({ type: "toggle_target" });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "leave_actions") {
        dispatch({ type: "leave_actions" });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "move_action") {
        dispatch({
          type: "move_action",
          delta: actionTarget.actionTargetIntent.delta,
          actionCount: DERIVED_TAG_MIGRATION_REVIEW_ACTIONS.length,
        });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "apply_action") {
        const requestedAction = DERIVED_TAG_MIGRATION_REVIEW_ACTIONS[state.selectedActionIndex]?.id;
        if (requestedAction) {
          void requestAction(requestedAction);
        }
        return;
      }
      if (actionTarget.interactionAction?.id === "help") {
        void showTerminalReturnDialog(adapters, "Derived-Tag Review Help", screenModel.helpLines);
        return;
      }
      if (state.activeTarget === "actions") {
        return;
      }
      const activeContentRoute = state.activePane === "list" ? list : detail;

      if (activeContentRoute.interactionAction?.id === "help") {
        void showTerminalReturnDialog(adapters, "Derived-Tag Review Help", screenModel.helpLines);
        return;
      }
      if (activeContentRoute.interactionAction?.id === "focus") {
        dispatch({ type: "toggle_focus" });
        return;
      }
      if (activeContentRoute.interactionAction?.id === "layout") {
        dispatch({ type: "toggle_layout" });
        return;
      }
      if (activeContentRoute.interactionAction?.id === "close" && state.activePane === "detail") {
        dispatch({ type: "leave_detail" });
        return;
      }
      if (activeContentRoute.navigationAction?.kind === "move") {
        if (state.activePane === "list") {
          if (Math.abs(activeContentRoute.navigationAction.delta) === 1) {
            dispatch({
              type: "move_list_wrapped",
              delta: activeContentRoute.navigationAction.delta,
              itemCount: screenModel.items.length,
            });
          } else {
            dispatch({
              type: "move_list_clamped",
              delta: activeContentRoute.navigationAction.delta,
              itemCount: screenModel.items.length,
            });
          }
          return;
        }
        dispatch({
          type: "move_detail",
          delta: activeContentRoute.navigationAction.delta,
          maxDetailScroll: screenModel.maxDetailScroll,
        });
        return;
      }
      if (activeContentRoute.navigationAction?.kind === "boundary") {
        if (state.activePane === "list") {
          dispatch({
            type: "list_boundary",
            boundary: activeContentRoute.navigationAction.boundary,
            itemCount: screenModel.items.length,
          });
          return;
        }
        dispatch({
          type: "detail_boundary",
          boundary: activeContentRoute.navigationAction.boundary,
          maxDetailScroll: screenModel.maxDetailScroll,
        });
      }
    },
  });

  return { screen: screenModel.screen };
}
