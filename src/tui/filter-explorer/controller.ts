import React from "react";

import {
  createDerivedTagTerminalActionTargetState,
  getDerivedTagTerminalActionTargetInteractionActions,
  reduceDerivedTagTerminalActionTargetState,
} from "../action-target.js";
import type { DerivedTagTerminalPointerEvent } from "../framework/types.js";
import {
  measureTerminalListDetailPresentation,
  useTerminalListDetailNotification,
  useTerminalListDetailInteractionRouter,
} from "../list-detail-presentation.js";
import { renderPageDocumentModel } from "../page-document/model.js";
import {
  createPageDocumentInteractionState,
  enterPageDocumentTargetMode,
  getFocusedPageDocumentSection,
  getSelectedPageDocumentTarget,
  leavePageDocumentTargetMode,
  movePageDocumentSection,
  movePageDocumentSectionBoundary,
  movePageDocumentTarget,
  movePageDocumentTargetBoundary,
} from "../page-document/interaction.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { getDerivedTagTerminalTwoPaneLayoutMode } from "../two-pane-state.js";
import {
  buildFilterExplorerBreadcrumb,
  buildFilterExplorerDetailLines,
  canDrillIntoFilterExplorerNode,
  createFilterExplorerBrowserUiState,
  getFilterExplorerBrowserSelection,
  getFilterExplorerDetailTitle,
  normalizeFilterExplorerBrowserState,
} from "./browser.js";
import {
  cloneFilterExplorerComposeDraft,
  getFilterExplorerDiscreteClause,
  getFilterExplorerScalarClause,
  normalizeFilterExplorerComposeDraft,
} from "./compose-state.js";
import { resolveFilterExplorerHostTarget } from "./host-adapter.js";
import {
  buildFilterExplorerControllerContext,
  handleFilterExplorerInteractionRoute,
} from "./controller-routing.js";
import {
  type FilterExplorerAction,
  filterExplorerReducer,
} from "./controller-state.js";
import type { FilterExplorerInteractionRoute } from "./controller-types.js";
import {
  buildFilterExplorerActionEntries,
  buildFilterExplorerComposeDetailLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerBrowserUiState,
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerOptions,
} from "./types.js";

function useComposeSelectionState(mode: FilterExplorerComposeMode | null): [
  FilterExplorerComposeDraft,
  (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
] {
  const isControlled = mode?.draft !== undefined;
  const [internalDraft, setInternalDraft] = React.useState<FilterExplorerComposeDraft>(() =>
    normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft),
  );

  React.useEffect(() => {
    setInternalDraft(
      normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft),
    );
  }, [mode?.draft, mode?.initialDraft]);

  const currentDraft = React.useMemo(
    () => normalizeFilterExplorerComposeDraft(mode?.draft ?? internalDraft),
    [internalDraft, mode?.draft],
  );
  const updateSelection = React.useCallback(
    (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => {
      if (!mode) {
        return;
      }

      const next = normalizeFilterExplorerComposeDraft(updater(currentDraft));
      if (!isControlled) {
        setInternalDraft(next);
      }
      mode.onDraftChange?.(cloneFilterExplorerComposeDraft(next));
    },
    [currentDraft, isControlled, mode],
  );

  return [currentDraft, updateSelection];
}

export function useFilterExplorerController(options: FilterExplorerOptions): FilterExplorerControllerContext {
  const adapters = useTerminalInteractionContextAdapters();
  const terminal = useDerivedTagTerminalApp();
  const { notification, showNotification } = useTerminalListDetailNotification();
  const composeMode = options.mode.kind === "compose" ? options.mode : null;
  const selectionPresentation = options.host.selectionPresentation;
  const [draft, updateDraft] = useComposeSelectionState(composeMode);
  const size = useDerivedTagTerminalSize();
  const [actionTargetState, dispatchActionTarget] = React.useReducer(
    reduceDerivedTagTerminalActionTargetState,
    undefined,
    () => createDerivedTagTerminalActionTargetState(),
  );
  const [pageInteractionState, setPageInteractionState] = React.useState(createPageDocumentInteractionState);
  const [state, dispatch] = React.useReducer(
    (current: FilterExplorerBrowserUiState, action: FilterExplorerAction) =>
      filterExplorerReducer(options.model, current, action),
    { model: options.model, initialSnapshot: options.initialSnapshot },
    ({ model, initialSnapshot }) => createFilterExplorerBrowserUiState(model, initialSnapshot),
  );

  React.useEffect(() => {
    dispatch({ type: "normalize" });
  }, [options.model]);

  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode({
    activePane: state.activePane,
    detailScroll: state.browserState.detailScroll,
    layoutMode: state.layoutMode,
  });
  const normalizedBrowserState = normalizeFilterExplorerBrowserState(options.model, state.browserState);
  const selection = getFilterExplorerBrowserSelection(options.model, normalizedBrowserState);
  const selectedTarget = resolveFilterExplorerHostTarget(options.host, selection.currentNode);
  const pageDocument =
    options.mode.kind === "inspect-and-open" ? options.host.resolvePageDocument?.(selection.currentNode) ?? null : null;
  React.useEffect(() => {
    setPageInteractionState(createPageDocumentInteractionState());
  }, [pageDocument?.recordKey, selection.currentNode?.id]);
  const selectedPageTarget = pageDocument
    ? getSelectedPageDocumentTarget({
        document: pageDocument,
        state: pageInteractionState,
      })
    : null;
  const initialPageDetailLines = React.useMemo(
    () =>
      pageDocument
        ? renderPageDocumentModel(pageDocument, {
            selectedTargetNodeId: selectedPageTarget?.nodeId,
          })
        : null,
    [pageDocument, selectedPageTarget?.nodeId],
  );
  const detailDraft = options.host.getDraft?.() ?? draft;
  const detailLines =
    composeMode || selectionPresentation
      ? buildFilterExplorerComposeDetailLines({
          mode: {
            kind: "compose",
            detailTitle: composeMode?.detailTitle ?? selectionPresentation?.detailTitle,
            emptySelectionText: composeMode?.emptySelectionText ?? selectionPresentation?.emptySelectionText,
            focusedClauseTitle: composeMode?.focusedClauseTitle ?? selectionPresentation?.focusedClauseTitle,
            stagedClausesTitle: composeMode?.stagedClausesTitle ?? selectionPresentation?.selectionTitle,
          },
          draft: detailDraft,
          currentNodeLabel: selection.currentNode?.label,
          selectedTarget,
          selectedDiscreteClause: getFilterExplorerDiscreteClause(selectedTarget, detailDraft),
          selectedScalarClause: getFilterExplorerScalarClause(selectedTarget, detailDraft),
          baseDetailLines:
            selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }],
        })
      : initialPageDetailLines
        ? initialPageDetailLines
      : buildFilterExplorerDetailLines(options.model, normalizedBrowserState);
  const presentationMetrics = measureTerminalListDetailPresentation({
    terminalWidth: size.width,
    terminalHeight: size.height,
    footerLineCount: 2,
    notification,
    transitionStatus: options.transitionStatus,
    detailLines,
    detailScroll: normalizedBrowserState.detailScroll,
    layoutMode,
    leftWidth: 46,
    hyperlinkSupport: terminal.capabilities.hyperlinkSupport,
  });
  const focusedPageSection = pageDocument
    ? getFocusedPageDocumentSection({
        document: pageDocument,
        state: pageInteractionState,
        scroll: normalizedBrowserState.detailScroll,
        bodyHeight: presentationMetrics.bodyHeight,
      })
    : null;
  const detailInteractionState =
    pageDocument == null
      ? { kind: "none" as const }
      : pageInteractionState.mode.kind === "target"
        ? { kind: "target" as const }
        : {
            kind: "section" as const,
            canEnterTargets: Boolean(focusedPageSection?.targetNodeIds.length),
          };
  const detailTargetActionId =
    pageInteractionState.mode.kind === "target" && selectedPageTarget
      ? selectedPageTarget.target.kind === "record" && selectedPageTarget.target.action === "preview"
        ? "preview"
        : "open"
      : null;
  const pageDetailLines = React.useMemo(
    () =>
      pageDocument
        ? renderPageDocumentModel(pageDocument, {
            activeSectionId: focusedPageSection?.id,
            selectedTargetNodeId: selectedPageTarget?.nodeId,
          })
        : null,
    [focusedPageSection?.id, pageDocument, selectedPageTarget?.nodeId],
  );
  const detailLinesWithFocus =
    composeMode || selectionPresentation
      ? detailLines
      : pageDetailLines
        ? pageDetailLines
        : detailLines;
  const detailTitle = composeMode || selectionPresentation
    ? composeMode?.detailTitle ?? selectionPresentation?.detailTitle ?? "Detail"
    : pageDocument
      ? `${pageDocument.title}${focusedPageSection?.title ? ` | ${focusedPageSection.title}` : ""}`
      : getFilterExplorerDetailTitle(options.model, normalizedBrowserState);
  const screenPresentationMetrics = measureTerminalListDetailPresentation({
    terminalWidth: size.width,
    terminalHeight: size.height,
    footerLineCount: 2,
    notification,
    transitionStatus: options.transitionStatus,
    detailLines: detailLinesWithFocus,
    detailScroll: normalizedBrowserState.detailScroll,
    layoutMode,
    leftWidth: 46,
    hyperlinkSupport: terminal.capabilities.hyperlinkSupport,
  });
  const maxDetailScroll = screenPresentationMetrics.maxDetailScroll;
  const effectiveState =
    normalizedBrowserState.detailScroll > maxDetailScroll
      ? { ...normalizedBrowserState, detailScroll: maxDetailScroll }
      : normalizedBrowserState;
  const currentNodeHasChildren = canDrillIntoFilterExplorerNode(selection.currentNode);
  const searchIndicator = state.searchMode
    ? ` | /${state.searchInput}`
    : effectiveState.filter
      ? ` | /${effectiveState.filter}`
      : "";

  const browserContext: FilterExplorerBrowserContext = {
    state,
    effectiveState,
    selection,
    currentNode: selection.currentNode,
    currentNodeHasChildren,
    breadcrumb: buildFilterExplorerBreadcrumb(options.model, selection),
    bodyHeight: screenPresentationMetrics.bodyHeight,
    detailWidth: screenPresentationMetrics.detailWidth,
    detailLines: detailLinesWithFocus,
    visibleDetailLines: screenPresentationMetrics.visibleDetailLines,
    detailTitle,
    layoutMode,
    maxDetailScroll,
    detailJumpSize: screenPresentationMetrics.detailJumpSize,
    detailPageSize: screenPresentationMetrics.pageSize,
    selectionJumpSize: screenPresentationMetrics.selectionJumpSize,
    searchIndicator,
    pageDocument,
    pageInteractionState,
    focusedPageSection,
    selectedPageTarget,
    detailInteractionState,
    detailTargetActionId,
  };
  const handleDetailPointerEvent = React.useCallback(
    (event: DerivedTagTerminalPointerEvent) => {
      if (event.kind !== "wheel") {
        return false;
      }
      dispatch({ type: "move_detail", delta: event.deltaY, maxDetailScroll: browserContext.maxDetailScroll });
      return true;
    },
    [browserContext.maxDetailScroll, dispatch],
  );

  const baseContext = buildFilterExplorerControllerContext({
    options,
    browser: browserContext,
    draft,
    actionEntries: [],
    actionTargetState,
    onDetailPointerEvent: handleDetailPointerEvent,
    notification,
  });
  const actionEntries = buildFilterExplorerActionEntries(baseContext);
  const controllerContext = React.useMemo(
    () =>
      buildFilterExplorerControllerContext({
        options,
        browser: browserContext,
        draft,
        actionEntries,
        actionTargetState,
        onDetailPointerEvent: handleDetailPointerEvent,
        notification,
      }),
    [actionEntries, actionTargetState, browserContext, draft, handleDetailPointerEvent, notification, options],
  );
  const interactionActions = getFilterExplorerInteractionActions(baseContext, actionEntries.length > 0);

  const handleRoute = React.useCallback(
    (route: FilterExplorerInteractionRoute) => {
      if ((browserContext.state.activePane === "detail" || browserContext.layoutMode === "detail-only") && pageDocument) {
        const detailNavigationAction =
          browserContext.layoutMode === "detail-only"
            ? route.detailNavigationAction ?? route.listNavigationAction
            : route.detailNavigationAction;
        const scrollTo = (nextScroll: number) => {
          dispatch({
            type: "move_detail",
            delta: nextScroll - browserContext.effectiveState.detailScroll,
            maxDetailScroll: browserContext.maxDetailScroll,
          });
        };

        if (
          (route.interactionAction?.id === "back" || route.interactionAction?.id === "return") &&
          detailInteractionState.kind === "target"
        ) {
          setPageInteractionState(leavePageDocumentTargetMode());
          return;
        }

        if (detailNavigationAction?.kind === "confirm" && detailInteractionState.kind === "section") {
          const entered = enterPageDocumentTargetMode({
            document: pageDocument,
            scroll: browserContext.effectiveState.detailScroll,
            bodyHeight: browserContext.bodyHeight,
            maxScroll: browserContext.maxDetailScroll,
          });
          if (entered.state.mode.kind === "section") {
            showNotification({
              message: "The active section has no targets.",
              tone: "warning",
            });
            return;
          }
          setPageInteractionState(entered.state);
          scrollTo(entered.scroll);
          return;
        }

        if (detailNavigationAction?.kind === "confirm" && detailInteractionState.kind === "target") {
          const target = selectedPageTarget?.target;
          if (!target) {
            return;
          }

          const handled =
            options.mode.kind === "inspect-and-open"
              ? options.host.activatePageTarget?.({
                  target,
                  controller: controllerContext,
                })
              : false;
          if (handled) {
            setPageInteractionState(createPageDocumentInteractionState());
            return;
          }

          showNotification({
            message:
              target.kind === "searchPivot"
                ? "Page pivots are unavailable in this host."
                : target.kind === "record"
                  ? "Record targets are not wired yet."
                  : "This target opens outside the current page flow.",
            tone: "warning",
          });
          return;
        }

        if (detailNavigationAction?.kind === "cursorMove") {
          if (detailInteractionState.kind === "target") {
            const moved = movePageDocumentTarget({
              document: pageDocument,
              state: pageInteractionState,
              bodyHeight: browserContext.bodyHeight,
              maxScroll: browserContext.maxDetailScroll,
              delta: detailNavigationAction.delta,
            });
            setPageInteractionState(moved.state);
            scrollTo(moved.scroll);
            return;
          }

          scrollTo(
            movePageDocumentSection({
              document: pageDocument,
              scroll: browserContext.effectiveState.detailScroll,
              bodyHeight: browserContext.bodyHeight,
              maxScroll: browserContext.maxDetailScroll,
              delta: detailNavigationAction.delta,
            }),
          );
          return;
        }

        if (detailNavigationAction?.kind === "cursorBoundary") {
          if (detailInteractionState.kind === "target") {
            const moved = movePageDocumentTargetBoundary({
              document: pageDocument,
              state: pageInteractionState,
              bodyHeight: browserContext.bodyHeight,
              maxScroll: browserContext.maxDetailScroll,
              boundary: detailNavigationAction.boundary,
            });
            setPageInteractionState(moved.state);
            scrollTo(moved.scroll);
            return;
          }

          scrollTo(
            movePageDocumentSectionBoundary({
              document: pageDocument,
              boundary: detailNavigationAction.boundary,
              bodyHeight: browserContext.bodyHeight,
              maxScroll: browserContext.maxDetailScroll,
            }),
          );
          return;
        }

        if (
          detailNavigationAction?.kind === "viewportScrollSmall" ||
          detailNavigationAction?.kind === "viewportScrollLarge" ||
          detailNavigationAction?.kind === "viewportPage"
        ) {
          if (pageInteractionState.mode.kind === "target") {
            setPageInteractionState(leavePageDocumentTargetMode());
          }
          dispatch({
            type: "move_detail",
            delta: detailNavigationAction.delta,
            maxDetailScroll: browserContext.maxDetailScroll,
          });
          return;
        }

        if (detailNavigationAction?.kind === "viewportEdge") {
          if (pageInteractionState.mode.kind === "target") {
            setPageInteractionState(leavePageDocumentTargetMode());
          }
          dispatch({
            type: "detail_boundary",
            boundary: detailNavigationAction.boundary,
            maxDetailScroll: browserContext.maxDetailScroll,
          });
          return;
        }
      }

      handleFilterExplorerInteractionRoute({
        route,
        adapters,
        browserContext,
        options,
        draft,
        updateDraft,
        dispatch,
        actionEntries,
        actionTargetState,
        dispatchActionTarget,
        showNotification,
      });
    },
    [
      actionEntries,
      actionTargetState,
      adapters,
      browserContext,
      detailInteractionState.kind,
      dispatch,
      draft,
      controllerContext,
      options,
      pageDocument,
      pageInteractionState,
      selectedPageTarget?.target,
      showNotification,
      updateDraft,
    ],
  );

  useTerminalListDetailInteractionRouter({
    list: {
      interactionActions,
      pageSize: browserContext.detailPageSize,
      jumpSize: browserContext.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    },
    detail: {
      interactionActions,
      pageSize: browserContext.detailPageSize,
      jumpSize:
        detailInteractionState.kind === "none" ? browserContext.detailJumpSize : browserContext.selectionJumpSize,
      mode: detailInteractionState.kind === "none" ? "viewport" : "hybrid",
      includeConfirmKeys: detailInteractionState.kind !== "none",
      includeHorizontalConfirmKeys: detailInteractionState.kind !== "none",
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
    textEntry: {
      interactionActions: [{ id: "cancel" }],
    },
    actionTarget:
      actionEntries.length > 0
        ? {
            interactionActions: [
              ...getDerivedTagTerminalActionTargetInteractionActions(actionTargetState, "horizontal"),
              { id: "help" },
            ],
            state: actionTargetState,
            orientation: "horizontal",
          }
        : undefined,
    transitionStatus: options.transitionStatus,
    onRoute: ({ actionTarget, detail, list, textEntry }) => {
      handleRoute({
        event: textEntry?.event ?? list.event,
        interactionAction:
          browserContext.state.activePane === "detail" || browserContext.layoutMode === "detail-only"
            ? detail.interactionAction
            : list.interactionAction,
        actionTargetInteractionAction: actionTarget?.interactionAction,
        actionTargetIntent: actionTarget?.actionTargetIntent,
        searchModeAction: textEntry?.interactionAction?.id === "cancel" ? { id: "cancel" } : undefined,
        textEntryIntent: textEntry?.textEntryIntent,
        listNavigationAction: list.navigationAction,
        detailNavigationAction: detail.navigationAction,
      });
    },
  });

  return controllerContext;
}
