import React from "react";

import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import { usePf2eTerminalAppServices } from "../app-service-context.js";
import {
  cycleFilterExplorerDiscreteClause,
  getFilterExplorerDiscreteClause,
  getFilterExplorerScalarClause,
  isFilterExplorerScalarTarget,
  setFilterExplorerScalarClause,
} from "../filter-explorer/compose-state.js";
import {
  createFilterExplorerDiscoveryState,
  createFilterExplorerNumericScalarEditHandler,
  createFilterExplorerOutcomeHandler,
} from "../filter-explorer/host-helpers.js";
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
import { isSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";
import {
  planSearchFilterExplorerRefresh,
  shouldApplySearchFilterExplorerRefresh,
} from "./filter-explorer-refresh.js";
import { buildSearchFilterExplorerTargetResolver } from "../filter-explorer/search-draft-model.js";
import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";

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

function mergeExplorerNodeTrees(
  visibleNodes: readonly OntologyNode[],
  refreshedNodes: readonly OntologyNode[],
): OntologyNode[] {
  const refreshedById = new Map(refreshedNodes.map((node) => [node.id, node]));

  return visibleNodes.map((visibleNode) => {
    const refreshedNode = refreshedById.get(visibleNode.id);
    if (!refreshedNode) {
      return visibleNode;
    }

    const visibleChildren = visibleNode.children;
    const refreshedChildren = refreshedNode.children;
    if (!visibleChildren || !refreshedChildren) {
      return refreshedNode;
    }

    return {
      ...refreshedNode,
      children: mergeExplorerNodeTrees(visibleChildren, refreshedChildren),
    };
  });
}

function mergeExplorerModelPreservingVisibleNodes(
  visibleModel: OntologyDomainModel,
  refreshedModel: OntologyDomainModel,
): OntologyDomainModel {
  if (visibleModel.id !== refreshedModel.id) {
    return refreshedModel;
  }

  return {
    ...refreshedModel,
    rootNodes: mergeExplorerNodeTrees(visibleModel.rootNodes, refreshedModel.rootNodes),
  };
}

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const { user } = usePf2eTerminalAppServices();
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = session.initialDiscoveryMode ?? "matching";
  const loadModelForDiscoveryMode = session.loadModelForDiscoveryMode;
  const scopedFields = React.useMemo(() => session.fieldOptions.map((fieldOption) => fieldOption.value), [session.fieldOptions]);
  const prepareDraft = React.useCallback(
    (query: typeof session.query) => user.search.prepareFilterExplorerDraft(query, scopedFields),
    [scopedFields, user.search],
  );
  const buildDraft = React.useCallback(
    (): FilterExplorerComposeDraft => draftRef.current,
    [],
  );
  const [model, setModel] = React.useState(session.model);
  const [, rerenderDraft] = React.useReducer((value: number) => value + 1, 0);
  const [discoveryMode, setDiscoveryMode] = React.useState<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const [refreshState, setRefreshState] = React.useState<{ pendingMode: SearchFilterDiscoveryMode } | null>(null);
  const queryRef = React.useRef(session.query);
  const draftRef = React.useRef<FilterExplorerComposeDraft>(prepareDraft(session.query).draft);
  const discoveryModeRef = React.useRef<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const modelCacheRef = React.useRef(new Map<SearchFilterDiscoveryMode, SearchFilterExplorerSession["model"]>());
  const preservedMetadataRef = React.useRef(prepareDraft(session.query).preservedMetadata);
  const scopedFieldsRef = React.useRef<readonly typeof scopedFields[number][]>(prepareDraft(session.query).scopedFields);
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
      if (!loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        setRefreshState(null);
        discoveryModeRef.current = nextMode;
        return;
      }

      const { debounceMs = DISCOVERY_REFRESH_DEBOUNCE_MS, force = false } = options;
      const preserveVisibleNodes = force && nextMode === discoveryModeRef.current;
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
        void loadModelForDiscoveryMode(nextMode)
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
            setModel((currentModel) =>
              preserveVisibleNodes && !isSearchFilterExplorerLoadingModel(currentModel)
                ? mergeExplorerModelPreservingVisibleNodes(currentModel, nextModel)
                : nextModel,
            );
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
    [invalidateRefreshes, loadModelForDiscoveryMode, terminal],
  );

  React.useEffect(() => {
    refreshStateRef.current = refreshState;
  }, [refreshState]);

  React.useEffect(() => {
    discoveryModeRef.current = discoveryMode;
  }, [discoveryMode]);

  React.useEffect(() => {
    queryRef.current = session.query;
  }, [session.query]);

  React.useEffect(() => {
    invalidateRefreshes();
    const preparedDraft = prepareDraft(session.query);
    queryRef.current = session.query;
    draftRef.current = preparedDraft.draft;
    preservedMetadataRef.current = preparedDraft.preservedMetadata;
    scopedFieldsRef.current = preparedDraft.scopedFields;
    rerenderDraft();
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
  }, [initialDiscoveryMode, invalidateRefreshes, loadModelForDiscoveryMode, prepareDraft, runModelRefresh, session.model]);

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
          session.onBack?.(queryRef.current);
        },
        onExitRoot: () => {
          session.onExitRoot?.(queryRef.current);
        },
        onCancel: () => {
          session.onCancel?.(queryRef.current);
        },
      }),
    [session],
  );

  const onEditScalarTarget = React.useMemo(
    () => createFilterExplorerNumericScalarEditHandler(prompts, terminal),
    [prompts, terminal],
  );

  const applyNextDraft = React.useCallback(
    (nextDraft: FilterExplorerComposeDraft) => {
      draftRef.current = nextDraft;
      rerenderDraft();
      const nextQuery = user.search.applyFilterExplorerDraft(queryRef.current, nextDraft, {
        preservedMetadata: preservedMetadataRef.current,
        scopedFields: scopedFieldsRef.current,
      });
      queryRef.current = nextQuery;
      session.onQueryChange(nextQuery);
      if (session.refreshOnQueryChange && session.loadModelForDiscoveryMode) {
        runModelRefresh(discoveryModeRef.current, { force: true });
      }
    },
    [runModelRefresh, session, user.search],
  );

  const host = React.useMemo<FilterExplorerHostAdapter>(
    () => ({
      resolveTarget: session.resolveSelectionTarget ?? buildSearchFilterExplorerTargetResolver(session.fieldOptions),
      getDraft: () => buildDraft(),
      selectionPresentation: {
        selectionTitle: "Current clauses",
      },
      describeNode: (args) => {
        const { node, target } = args;
        if (!target) {
          return { activationStyle: "none" as const };
        }

        const draft = buildDraft();
        if (isFilterExplorerScalarTarget(target)) {
          const clause = getFilterExplorerScalarClause(target, draft);
          return {
            activationStyle: "edit" as const,
            stateBadge: clause ? { kind: "custom" as const, text: "ƒ", tone: "accent" } : { kind: "custom" as const, text: "·", tone: "dim" },
            suffixText: node && clause ? clause.summaryLabel : undefined,
          };
        }

        const operator = getFilterExplorerDiscreteClause(target, draft)?.operator;
        return {
          activationStyle: "toggle" as const,
          stateBadge:
            operator === "include"
              ? { kind: "include" as const }
              : operator === "exclude"
                ? { kind: "exclude" as const }
                : { kind: "off" as const },
        };
      },
      activateTarget: ({ target }) => {
        const currentDraft = buildDraft();
        if (isFilterExplorerScalarTarget(target)) {
          void Promise.resolve(
            onEditScalarTarget({
              target,
              currentClause: getFilterExplorerScalarClause(target, currentDraft),
              draft: currentDraft,
            }),
          ).then((nextClause) => {
            if (nextClause === undefined) {
              return;
            }
            applyNextDraft(setFilterExplorerScalarClause(target, nextClause, currentDraft));
          });
          return true;
        }

        applyNextDraft(cycleFilterExplorerDiscreteClause(target, currentDraft, 1));
        return true;
      },
    }),
    [applyNextDraft, buildDraft, onEditScalarTarget, session.fieldOptions, session.resolveSelectionTarget],
  );

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
        kind: "inspect-and-open",
        onEditScalarTarget,
      }}
    />
  );
}
