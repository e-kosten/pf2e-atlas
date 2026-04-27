import React from "react";

import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import { createComposeFilterExplorerHostAdapter } from "../filter-explorer/host-adapter.js";
import {
  createFilterExplorerDiscoveryState,
  createFilterExplorerNumericScalarEditHandler,
  createFilterExplorerOutcomeHandler,
} from "../filter-explorer/host-helpers.js";
import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import { cloneFilterExplorerComposeDraft } from "../filter-explorer/compose-state.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { SearchFilterExplorerSession } from "./query-field-builder/query-field-builder-session.js";
import type {
  FilterExplorerDiscoveryState,
  FilterExplorerModeSwitchOption,
} from "../filter-explorer/types.js";
import { isSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";
import {
  planSearchFilterExplorerRefresh,
  shouldApplySearchFilterExplorerRefresh,
} from "./filter-explorer-refresh.js";

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

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = session.initialDiscoveryMode ?? "matching";
  const [model, setModel] = React.useState(session.model);
  const [discoveryMode, setDiscoveryMode] = React.useState<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const [draft, setDraft] = React.useState(() => cloneFilterExplorerComposeDraft(session.draft));
  const [refreshState, setRefreshState] = React.useState<{ pendingMode: SearchFilterDiscoveryMode } | null>(null);
  const draftRef = React.useRef(cloneFilterExplorerComposeDraft(session.draft));
  const discoveryModeRef = React.useRef<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const modelCacheRef = React.useRef(new Map<SearchFilterDiscoveryMode, SearchFilterExplorerSession["model"]>());
  const refreshStateRef = React.useRef<{ pendingMode: SearchFilterDiscoveryMode } | null>(null);
  const refreshRequestIdRef = React.useRef(0);
  const refreshTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);

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
    (nextMode: SearchFilterDiscoveryMode, options: { debounceMs?: number; force?: boolean } = {}) => {
      if (!session.loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        setRefreshState(null);
        discoveryModeRef.current = nextMode;
        return;
      }

      const { debounceMs = DISCOVERY_REFRESH_DEBOUNCE_MS, force = false } = options;
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
        setModel(plan.model);
        setDiscoveryMode(plan.mode);
        discoveryModeRef.current = plan.mode;
        setRefreshState(null);
        return;
      }

      setRefreshState({ pendingMode: plan.pendingMode });

      const executeRefresh = () => {
        refreshTimerRef.current = null;
        void session
          .loadModelForDiscoveryMode!(nextMode)
          .then((nextModel) => {
            if (
              !shouldApplySearchFilterExplorerRefresh({
                currentRequestId: refreshRequestIdRef.current,
                completedRequestId: plan.requestId,
              })
            ) {
              return;
            }
            modelCacheRef.current.set(nextMode, nextModel);
            setModel(nextModel);
            setDiscoveryMode(nextMode);
            discoveryModeRef.current = nextMode;
            setRefreshState(null);
          })
          .catch((error) => {
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
    [invalidateRefreshes, session, terminal],
  );

  React.useEffect(() => {
    refreshStateRef.current = refreshState;
  }, [refreshState]);

  React.useEffect(() => {
    discoveryModeRef.current = discoveryMode;
  }, [discoveryMode]);

  React.useEffect(() => {
    invalidateRefreshes();
    setModel(session.model);
    setDiscoveryMode(initialDiscoveryMode);
    discoveryModeRef.current = initialDiscoveryMode;
    modelCacheRef.current = new Map(
      isSearchFilterExplorerLoadingModel(session.model) ? [] : [[initialDiscoveryMode, session.model]],
    );
    setRefreshState(
      session.loadModelForDiscoveryMode && isSearchFilterExplorerLoadingModel(session.model)
        ? { pendingMode: initialDiscoveryMode }
        : null,
    );
    const nextDraft = cloneFilterExplorerComposeDraft(session.draft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
    if (session.loadModelForDiscoveryMode && isSearchFilterExplorerLoadingModel(session.model)) {
      runModelRefresh(initialDiscoveryMode, { debounceMs: 0, force: true });
    }

    return () => {
      invalidateRefreshes();
    };
  }, [initialDiscoveryMode, invalidateRefreshes, runModelRefresh, session]);

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
          session.onApply(draftRef.current);
        },
        onExitRoot: () => {
          session.onApply(draftRef.current);
        },
        onCancel: () => {
          session.onApply(draftRef.current);
        },
      }),
    [session],
  );

  const onEditScalarTarget = React.useMemo(
    () => createFilterExplorerNumericScalarEditHandler(prompts, terminal),
    [prompts, terminal],
  );
  const host = React.useMemo(
    () =>
      createComposeFilterExplorerHostAdapter({
        resolveTarget: session.resolveSelectionTarget,
        onEditScalarTarget,
      }),
    [onEditScalarTarget, session.resolveSelectionTarget],
  );

  const updateDraft = React.useCallback((nextComposeDraft: typeof draft) => {
    const nextDraft = cloneFilterExplorerComposeDraft(nextComposeDraft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, []);

  return (
    <FilterExplorerScreen
      title={session.title}
      model={model}
      host={host}
      rootDepth={0}
      exitAtRootDepth
      onOutcome={handleOutcome}
      discovery={discovery}
      mode={{
        kind: "compose",
        draft,
        onDraftChange: (nextComposeDraft) => {
          updateDraft(nextComposeDraft);
        },
        onEditScalarTarget,
      }}
    />
  );
}
