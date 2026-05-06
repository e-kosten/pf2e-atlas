import React from "react";

import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import { usePf2eTerminalAppServices } from "../app-service-context.js";
import {
  createFilterExplorerDiscoveryState,
  createFilterExplorerNumericScalarEditHandler,
  createFilterExplorerOutcomeHandler,
} from "../filter-explorer/host-helpers.js";
import { createComposeFilterExplorerHostAdapter } from "../filter-explorer/host-adapter.js";
import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { SearchFilterExplorerSession } from "./query-field-builder/query-field-builder-session.js";
import type {
  FilterExplorerComposeDraft,
  FilterExplorerDiscoveryState,
  FilterExplorerHostAdapter,
  FilterExplorerModeSwitchOption,
} from "../filter-explorer/types.js";
import type { Pf2eTerminalPreparedFilterExplorerDraft } from "../search/service.js";
import { isSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";
import { planSearchFilterExplorerRefresh, shouldApplySearchFilterExplorerRefresh } from "./filter-explorer-refresh.js";
import { buildSearchFilterExplorerTargetResolver } from "../filter-explorer/search-draft-model.js";
import {
  buildSearchFilterExplorerComposeDraft,
  buildSearchFilterExplorerFieldState,
  type SearchFilterExplorerFieldState,
} from "./filter-explorer-field-state.js";
import { reconcileSearchFilterExplorerModel } from "./filter-explorer-model-reconciliation.js";

const DISCOVERY_REFRESH_DEBOUNCE_MS = 80;
const SEARCH_DISCOVERY_MODE_OPTIONS: readonly FilterExplorerModeSwitchOption<SearchFilterDiscoveryMode>[] = [
  {
    value: "matching",
    label: "Matching Counts",
    description: "Show values and counts from the current matching query context.",
  },
  {
    value: "catalog",
    label: "Catalog Counts",
    description: "Show values and counts from the wider applicability slice only.",
  },
];

function getChangedDiscreteSelectionFields(
  previous: SearchFilterExplorerFieldState,
  next: SearchFilterExplorerFieldState,
): string[] {
  const fields = new Set([...Object.keys(previous.discreteSelections), ...Object.keys(next.discreteSelections)]);
  return [...fields].filter((field) => {
    const previousSelection = previous.discreteSelections[field] ?? { include: [], exclude: [] };
    const nextSelection = next.discreteSelections[field] ?? { include: [], exclude: [] };
    return (
      previousSelection.include.join("\0") !== nextSelection.include.join("\0") ||
      previousSelection.exclude.join("\0") !== nextSelection.exclude.join("\0")
    );
  });
}

function prepareSessionFieldState(
  session: SearchFilterExplorerSession,
  prepareDraft: (query: SearchFilterExplorerSession["query"]) => Pf2eTerminalPreparedFilterExplorerDraft,
): {
  fieldState: SearchFilterExplorerFieldState;
  preservedMetadata: Pf2eTerminalPreparedFilterExplorerDraft["preservedMetadata"];
  scopedFields: Pf2eTerminalPreparedFilterExplorerDraft["scopedFields"];
} {
  const preparedDraft = prepareDraft(session.query);
  return {
    fieldState: session.initialFieldState ?? buildSearchFilterExplorerFieldState(preparedDraft.draft),
    preservedMetadata: session.preservedMetadata ?? preparedDraft.preservedMetadata,
    scopedFields: preparedDraft.scopedFields,
  };
}

export function SearchFilterExplorerScreen({ session }: { session: SearchFilterExplorerSession }): React.JSX.Element {
  const { debug, user } = usePf2eTerminalAppServices();
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = session.initialDiscoveryMode ?? "matching";
  const loadModelForDiscoveryMode = session.loadModelForDiscoveryMode;
  const selectTargetMode = Boolean(session.selectTargetMode);
  const scopedFields = React.useMemo(
    () => session.fieldOptions.map((fieldOption) => fieldOption.value),
    [session.fieldOptions],
  );
  const prepareDraft = React.useCallback(
    (query: typeof session.query) => user.search.prepareFilterExplorerDraft(query, scopedFields),
    [scopedFields, user.search],
  );
  const buildDraft = React.useCallback(
    (): FilterExplorerComposeDraft => buildSearchFilterExplorerComposeDraft(fieldStateRef.current),
    [],
  );
  const [model, setModel] = React.useState(session.model);
  const [, rerenderFieldState] = React.useReducer((value: number) => value + 1, 0);
  const [, rerenderDebugTrace] = React.useReducer((value: number) => value + 1, 0);
  const [discoveryMode, setDiscoveryMode] = React.useState<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const [refreshState, setRefreshState] = React.useState<{ pendingMode: SearchFilterDiscoveryMode } | null>(null);
  const queryRef = React.useRef(session.query);
  const fieldStateRef = React.useRef<SearchFilterExplorerFieldState>(
    prepareSessionFieldState(session, prepareDraft).fieldState,
  );
  const discoveryModeRef = React.useRef<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const modelCacheRef = React.useRef(new Map<SearchFilterDiscoveryMode, SearchFilterExplorerSession["model"]>());
  const preservedMetadataRef = React.useRef(prepareSessionFieldState(session, prepareDraft).preservedMetadata);
  const scopedFieldsRef = React.useRef<readonly (typeof scopedFields)[number][]>(
    prepareSessionFieldState(session, prepareDraft).scopedFields,
  );
  const refreshStateRef = React.useRef<{ pendingMode: SearchFilterDiscoveryMode } | null>(null);
  const refreshRequestIdRef = React.useRef(0);
  const refreshTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);
  const fieldOptionsRef = React.useRef(session.fieldOptions);
  const resolveSelectionTargetRef = React.useRef(session.resolveSelectionTarget);

  React.useEffect(() => {
    if (!debug.enabled) {
      return;
    }

    const interval = setInterval(rerenderDebugTrace, 250);
    return () => {
      clearInterval(interval);
    };
  }, [debug]);

  const clearQueuedRefresh = React.useCallback(() => {
    if (refreshTimerRef.current !== null) {
      clearTimeout(refreshTimerRef.current);
      refreshTimerRef.current = null;
    }
  }, []);

  const invalidateRefreshes = React.useCallback(() => {
    refreshRequestIdRef.current += 1;
    clearQueuedRefresh();
  }, [clearQueuedRefresh]);

  const runModelRefresh = React.useCallback(
    (
      nextMode: SearchFilterDiscoveryMode,
      options: { debounceMs?: number; force?: boolean; targetFields?: readonly string[] } = {},
    ) => {
      if (!loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        setRefreshState(null);
        discoveryModeRef.current = nextMode;
        return;
      }

      const { debounceMs = DISCOVERY_REFRESH_DEBOUNCE_MS, force = false, targetFields } = options;
      if (force) {
        modelCacheRef.current.clear();
      }
      const plan = planSearchFilterExplorerRefresh({
        nextMode,
        displayedMode: discoveryModeRef.current,
        cache: modelCacheRef.current,
        currentRequestId: refreshRequestIdRef.current,
        force,
      });
      invalidateRefreshes();
      refreshRequestIdRef.current = plan.requestId;

      if (plan.kind === "retainCurrent") {
        setRefreshState(null);
        return;
      }

      if (plan.kind === "useCached") {
        setModel((currentModel) =>
          {
            const span = debug.startSpan("filterExplorer.reconcileModel", {
              mode: plan.mode,
              source: "cache",
              targetFields: "all",
            });
            const reconciled = reconcileSearchFilterExplorerModel({
              currentModel,
              refreshedModel: plan.model,
              fieldState: fieldStateRef.current,
              fieldOptions: fieldOptionsRef.current,
              resolveSelectionTarget:
                resolveSelectionTargetRef.current ?? buildSearchFilterExplorerTargetResolver(fieldOptionsRef.current),
            });
            span.end({ rootNodes: reconciled.rootNodes.length });
            return reconciled;
          },
        );
        setDiscoveryMode(plan.mode);
        discoveryModeRef.current = plan.mode;
        setRefreshState(null);
        return;
      }

      setRefreshState({ pendingMode: plan.pendingMode });

      const executeRefresh = () => {
        refreshTimerRef.current = null;
        const loadSpan = debug.startSpan("filterExplorer.loadModel", {
          mode: nextMode,
          source: "session",
          targetFields: targetFields?.join(",") ?? "all",
        });
        void (
          targetFields ? loadModelForDiscoveryMode(nextMode, { targetFields }) : loadModelForDiscoveryMode(nextMode)
        )
          .then((nextModel) => {
            loadSpan.end({ rootNodes: nextModel.rootNodes.length });
            if (
              !shouldApplySearchFilterExplorerRefresh({
                currentRequestId: refreshRequestIdRef.current,
                completedRequestId: plan.requestId,
              })
            ) {
              return;
            }
            if (!targetFields) {
              modelCacheRef.current.set(nextMode, nextModel);
            }
            setModel((currentModel) =>
              {
                const span = debug.startSpan("filterExplorer.reconcileModel", {
                  mode: nextMode,
                  source: "refresh",
                  targetFields: targetFields?.join(",") ?? "all",
                });
                const reconciled = reconcileSearchFilterExplorerModel({
                  currentModel,
                  refreshedModel: nextModel,
                  fieldState: fieldStateRef.current,
                  fieldOptions: fieldOptionsRef.current,
                  resolveSelectionTarget:
                    resolveSelectionTargetRef.current ?? buildSearchFilterExplorerTargetResolver(fieldOptionsRef.current),
                  targetFields,
                });
                span.end({ rootNodes: reconciled.rootNodes.length });
                return reconciled;
              },
            );
            setDiscoveryMode(nextMode);
            discoveryModeRef.current = nextMode;
            setRefreshState(null);
          })
          .catch((error) => {
            loadSpan.end({ error: (error as Error).message });
            if (
              !shouldApplySearchFilterExplorerRefresh({
                currentRequestId: refreshRequestIdRef.current,
                completedRequestId: plan.requestId,
              })
            ) {
              return;
            }
            setRefreshState(null);
            void terminal.pauseForAnyKey(`Could not refresh explorer data.\n\n${(error as Error).message}`);
          });
      };

      if (debounceMs > 0) {
        refreshTimerRef.current = setTimeout(executeRefresh, debounceMs);
        return;
      }

      executeRefresh();
    },
    [debug, invalidateRefreshes, loadModelForDiscoveryMode, terminal],
  );

  React.useEffect(() => {
    refreshStateRef.current = refreshState;
  }, [refreshState]);

  React.useEffect(() => {
    discoveryModeRef.current = discoveryMode;
  }, [discoveryMode]);

  React.useEffect(() => {
    queryRef.current = session.query;
    fieldOptionsRef.current = session.fieldOptions;
    resolveSelectionTargetRef.current = session.resolveSelectionTarget;
  }, [session.fieldOptions, session.query, session.resolveSelectionTarget]);

  React.useEffect(() => {
    invalidateRefreshes();
    const preparedFieldState = prepareSessionFieldState(session, prepareDraft);
    queryRef.current = session.query;
    fieldStateRef.current = preparedFieldState.fieldState;
    preservedMetadataRef.current = preparedFieldState.preservedMetadata;
    scopedFieldsRef.current = preparedFieldState.scopedFields;
    rerenderFieldState();
    setModel(session.model);
    setDiscoveryMode(initialDiscoveryMode);
    discoveryModeRef.current = initialDiscoveryMode;
    modelCacheRef.current = new Map(
      isSearchFilterExplorerLoadingModel(session.model) ? [] : [[initialDiscoveryMode, session.model]],
    );
    setRefreshState(
      loadModelForDiscoveryMode && isSearchFilterExplorerLoadingModel(session.model)
        ? { pendingMode: initialDiscoveryMode }
        : null,
    );
    if (loadModelForDiscoveryMode && isSearchFilterExplorerLoadingModel(session.model)) {
      runModelRefresh(initialDiscoveryMode, { debounceMs: 0, force: true });
    }

    return () => {
      invalidateRefreshes();
    };
  }, [
    initialDiscoveryMode,
    invalidateRefreshes,
    loadModelForDiscoveryMode,
    prepareDraft,
    runModelRefresh,
    session.initialFieldState,
    session.model,
  ]);

  const onDiscoveryModeChange = React.useCallback(
    (nextMode: SearchFilterDiscoveryMode) => {
      if (nextMode === discoveryModeRef.current && refreshStateRef.current === null) {
        return;
      }

      runModelRefresh(nextMode);
    },
    [runModelRefresh],
  );

  const discovery = React.useMemo<FilterExplorerDiscoveryState<SearchFilterDiscoveryMode> | undefined>(
    () =>
      createFilterExplorerDiscoveryState({
        mode: discoveryMode,
        modes: SEARCH_DISCOVERY_MODE_OPTIONS,
        pendingMode: refreshState?.pendingMode,
        isRefreshing: refreshState !== null,
        onModeChange: onDiscoveryModeChange,
        enabled: Boolean(session.loadModelForDiscoveryMode),
      }),
    [discoveryMode, onDiscoveryModeChange, refreshState, session.loadModelForDiscoveryMode],
  );

  const handleOutcome = React.useMemo(
    () =>
      createFilterExplorerOutcomeHandler({
        onBack: () => {
          session.onEvent({ kind: "back", query: queryRef.current, fieldState: fieldStateRef.current });
        },
        onExitRoot: () => {
          session.onEvent({ kind: "exitRoot", query: queryRef.current, fieldState: fieldStateRef.current });
        },
        onCancel: () => {
          session.onEvent({ kind: "cancel", query: queryRef.current, fieldState: fieldStateRef.current });
        },
        onSelectTarget: (outcome) => {
          if (!outcome.result.target) {
            return;
          }
          session.onEvent({
            kind: "selectTarget",
            outcome,
            query: queryRef.current,
            fieldState: fieldStateRef.current,
            discoveryMode: discoveryModeRef.current,
          });
        },
      }),
    [session],
  );

  const onEditScalarTarget = React.useMemo(
    () => createFilterExplorerNumericScalarEditHandler(prompts, terminal),
    [prompts, terminal],
  );

  const applyNextFieldState = React.useCallback(
    (nextFieldState: SearchFilterExplorerFieldState) => {
      const previousFieldState = fieldStateRef.current;
      fieldStateRef.current = nextFieldState;
      rerenderFieldState();
      const nextQuery = user.search.applyFilterExplorerDraft(
        queryRef.current,
        buildSearchFilterExplorerComposeDraft(nextFieldState),
        {
          preservedMetadata: preservedMetadataRef.current,
          scopedFields: scopedFieldsRef.current,
        },
      );
      queryRef.current = nextQuery;
      session.onEvent({ kind: "change", query: nextQuery, fieldState: nextFieldState });
      if (session.refreshOnQueryChange && session.loadModelForDiscoveryMode) {
        const changedDiscreteFields = getChangedDiscreteSelectionFields(previousFieldState, nextFieldState);
        runModelRefresh(discoveryModeRef.current, {
          force: true,
          targetFields: changedDiscreteFields.length === 1 ? changedDiscreteFields : undefined,
        });
      }
    },
    [runModelRefresh, session, user.search],
  );

  const host = React.useMemo<FilterExplorerHostAdapter>(() => {
    const resolveTarget =
      session.resolveSelectionTarget ?? buildSearchFilterExplorerTargetResolver(session.fieldOptions);
    if (selectTargetMode) {
      return {
        resolveTarget,
        describeNode: ({ target }) => ({
          activationStyle: target ? "open" : "none",
        }),
      };
    }

    return createComposeFilterExplorerHostAdapter({
      resolveTarget,
      onEditScalarTarget,
    });
  }, [onEditScalarTarget, selectTargetMode, session.fieldOptions, session.resolveSelectionTarget]);

  const debugSnapshot = debug.enabled ? debug.snapshot() : undefined;

  return (
    <FilterExplorerScreen
      title={session.title}
      model={model}
      host={host}
      rootDepth={0}
      exitAtRootDepth
      onOutcome={handleOutcome}
      discovery={discovery}
      debugTrace={debug}
      debugSnapshot={debugSnapshot}
      mode={
        selectTargetMode
          ? { kind: "inspect-and-open" }
          : {
              kind: "compose",
              draft: buildDraft(),
              onDraftChange: (nextDraft) => {
                applyNextFieldState(buildSearchFilterExplorerFieldState(nextDraft));
              },
              onEditScalarTarget,
              stagedClausesTitle: "Current clauses",
            }
      }
    />
  );
}
