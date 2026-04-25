import React from "react";

import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import { cloneFilterExplorerComposeDraft } from "../filter-explorer/compose-state.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { SearchFilterExplorerSession } from "./query-field-builder/query-field-builder-session.js";
import { promptNumericScalarClause } from "../filter-explorer/scalar-editor.js";
import type { FilterExplorerDiscoveryMode, FilterExplorerDiscoveryState } from "../filter-explorer/types.js";

const DISCOVERY_REFRESH_DEBOUNCE_MS = 80;
const LOADING_EXPLORER_MODEL_ID = "searchFilterExplorer:loading";

function isLoadingExplorerModel(model: SearchFilterExplorerSession["model"]): boolean {
  return model.id === LOADING_EXPLORER_MODEL_ID || model.rootNodes.length === 0;
}

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = session.initialDiscoveryMode ?? "matching";
  const [model, setModel] = React.useState(session.model);
  const [discoveryMode, setDiscoveryMode] = React.useState<FilterExplorerDiscoveryMode>(initialDiscoveryMode);
  const [draft, setDraft] = React.useState(() => cloneFilterExplorerComposeDraft(session.draft));
  const [refreshState, setRefreshState] = React.useState<{ pendingMode: FilterExplorerDiscoveryMode } | null>(null);
  const draftRef = React.useRef(cloneFilterExplorerComposeDraft(session.draft));
  const modelCacheRef = React.useRef(new Map<FilterExplorerDiscoveryMode, SearchFilterExplorerSession["model"]>());
  const refreshStateRef = React.useRef<{ pendingMode: FilterExplorerDiscoveryMode } | null>(null);
  const refreshRequestIdRef = React.useRef(0);
  const refreshTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearQueuedRefresh = React.useCallback(() => {
    if (refreshTimerRef.current !== null) {
      clearTimeout(refreshTimerRef.current);
      refreshTimerRef.current = null;
    }
  }, []);

  const runModelRefresh = React.useCallback(
    (nextMode: FilterExplorerDiscoveryMode, options: { debounceMs?: number; force?: boolean } = {}) => {
      if (!session.loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        setRefreshState(null);
        return;
      }

      const { debounceMs = DISCOVERY_REFRESH_DEBOUNCE_MS, force = false } = options;
      if (!force && refreshStateRef.current?.pendingMode === nextMode) {
        return;
      }

      if (!force) {
        const cachedModel = modelCacheRef.current.get(nextMode);
        if (cachedModel) {
          setModel(cachedModel);
          setDiscoveryMode(nextMode);
          setRefreshState(null);
          return;
        }
      }

      setRefreshState({ pendingMode: nextMode });
      const requestId = refreshRequestIdRef.current + 1;
      refreshRequestIdRef.current = requestId;
      clearQueuedRefresh();

      const executeRefresh = () => {
        refreshTimerRef.current = null;
        void session
          .loadModelForDiscoveryMode!(nextMode)
          .then((nextModel) => {
            if (refreshRequestIdRef.current !== requestId) {
              return;
            }
            modelCacheRef.current.set(nextMode, nextModel);
            setModel(nextModel);
            setDiscoveryMode(nextMode);
            setRefreshState(null);
          })
          .catch((error) => {
            if (refreshRequestIdRef.current !== requestId) {
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
    [clearQueuedRefresh, session, terminal],
  );

  React.useEffect(() => {
    refreshStateRef.current = refreshState;
  }, [refreshState]);

  React.useEffect(() => {
    clearQueuedRefresh();
    refreshRequestIdRef.current += 1;
    setModel(session.model);
    setDiscoveryMode(initialDiscoveryMode);
    modelCacheRef.current = new Map(
      isLoadingExplorerModel(session.model) ? [] : [[initialDiscoveryMode, session.model]],
    );
    setRefreshState(session.loadModelForDiscoveryMode && isLoadingExplorerModel(session.model) ? { pendingMode: initialDiscoveryMode } : null);
    const nextDraft = cloneFilterExplorerComposeDraft(session.draft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
    if (session.loadModelForDiscoveryMode && isLoadingExplorerModel(session.model)) {
      runModelRefresh(initialDiscoveryMode, { debounceMs: 0, force: true });
    }

    return () => {
      clearQueuedRefresh();
      refreshRequestIdRef.current += 1;
    };
  }, [clearQueuedRefresh, initialDiscoveryMode, runModelRefresh, session]);

  const onDiscoveryModeChange = React.useCallback(
    (nextMode: FilterExplorerDiscoveryMode) => {
      if (nextMode === discoveryMode || refreshState?.pendingMode === nextMode) {
        return;
      }

      runModelRefresh(nextMode);
    },
    [discoveryMode, refreshState?.pendingMode, runModelRefresh],
  );

  const discovery = React.useMemo<FilterExplorerDiscoveryState | undefined>(() => {
    if (!session.loadModelForDiscoveryMode) {
      return undefined;
    }

    return {
      mode: discoveryMode,
      pendingMode: refreshState?.pendingMode,
      isRefreshing: refreshState !== null,
      onModeChange: onDiscoveryModeChange,
    };
  }, [discoveryMode, onDiscoveryModeChange, refreshState, session.loadModelForDiscoveryMode]);

  const applyDraft = React.useCallback(() => {
    session.onApply(draftRef.current);
  }, [session]);

  const updateDraft = React.useCallback((nextComposeDraft: typeof draft) => {
    const nextDraft = cloneFilterExplorerComposeDraft(nextComposeDraft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, []);

  return (
    <FilterExplorerScreen
      title={session.title}
      model={model}
      rootDepth={0}
      exitAtRootDepth
      onExit={applyDraft}
      discovery={discovery}
      mode={{
        kind: "compose",
        draft,
        onDraftChange: (nextComposeDraft) => {
          updateDraft(nextComposeDraft);
        },
        resolveSelectionTarget: session.resolveSelectionTarget,
        onEditScalarTarget: async ({ target, currentClause }) => {
          if (target.valueType !== "number") {
            return undefined;
          }

          const nextClause = await promptNumericScalarClause(prompts, terminal, {
            title: target.editorLabel ?? `${target.fieldLabel} / ${target.subjectLabel}`,
            currentClause:
              currentClause?.operator === "between"
                ? { op: "between", min: currentClause.min, max: currentClause.max }
                : typeof currentClause?.value === "number"
                  ? { op: currentClause.operator, value: currentClause.value }
                  : null,
          });

          if (nextClause === undefined) {
            return undefined;
          }
          if (nextClause === null) {
            return null;
          }

          return nextClause.op === "between"
            ? { operator: "between", min: nextClause.min, max: nextClause.max }
            : { operator: nextClause.op, value: nextClause.value };
        },
      }}
    />
  );
}
