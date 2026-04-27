import React from "react";

import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../filter-explorer/search-draft-model.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalPreparedFilterExplorerContext,
  Pf2eTerminalPreparedFilterExplorerDraft,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import type { SearchFilterExplorerSession } from "./model.js";
import type { Pf2eTerminalAppServices } from "../app-services.js";
import { createSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";

export function useSearchFilterExplorerWorkflow({
  query,
  services,
  onUnavailable,
}: {
  query: Pf2eTerminalSearchQuery;
  services: Pick<Pf2eTerminalAppServices["user"], "ontology" | "search">;
  onUnavailable: (message: string) => Promise<void>;
}): {
  filterExplorerSession: SearchFilterExplorerSession | null;
  openFilterExplorer: (options: {
    queryOverride?: Pf2eTerminalSearchQuery;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    initialPreparedDraft?: Pf2eTerminalPreparedFilterExplorerDraft;
    buildQueryForDraft?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => Pf2eTerminalSearchQuery;
    onDraftChange?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onApply: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onReturn?: () => void;
    onCancel?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onBack?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onExitRoot?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  closeFilterExplorer: () => void;
} {
  const [filterExplorerSession, setFilterExplorerSession] = React.useState<SearchFilterExplorerSession | null>(null);

  const openFilterExplorer = React.useCallback(
    async ({
      queryOverride,
      fieldOptions,
      initialPreparedDraft,
      buildQueryForDraft,
      onDraftChange,
      onApply,
      onReturn,
      onCancel,
      onBack,
      onExitRoot,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      initialPreparedDraft?: Pf2eTerminalPreparedFilterExplorerDraft;
      buildQueryForDraft?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => Pf2eTerminalSearchQuery;
      onDraftChange?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onApply: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onReturn?: () => void;
      onCancel?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onBack?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onExitRoot?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = services.search.normalizeQuery(queryOverride ?? query);
      const scopeCategory = getSearchQueryCategory(scopeQuery);
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      const scopedFields = fieldOptions.map((fieldOption) => fieldOption.value);
      const title = fieldOptions.length === 1 ? `${fieldOptions[0]!.label} Explorer` : "Filter Explorer";
      if (!scopeCategory) {
        await onUnavailable("Choose a category before editing a discoverable query field.");
        return false;
      }
      if (fieldOptions.length === 0) {
        await onUnavailable("No discoverable query fields are available for the current scope.");
        return false;
      }

      const preparedDraft = initialPreparedDraft ?? services.search.prepareFilterExplorerDraft(scopeQuery, scopedFields);
      const preparedContext: Pf2eTerminalPreparedFilterExplorerContext = {
        preservedMetadata: preparedDraft.preservedMetadata,
        scopedFields: preparedDraft.scopedFields,
      };
      const currentDraftRef = {
        current: preparedDraft.draft,
      };

      const buildPreparedModel = async (
        discoveryMode: "matching" | "catalog",
      ): Promise<ReturnType<typeof buildSearchFilterExplorerModel>> => {
        const request = services.search.normalizeQuery(
          buildQueryForDraft?.(currentDraftRef.current, preparedContext) ?? scopeQuery,
        );
        const requestCategory = getSearchQueryCategory(request);
        const requestSubcategory = getSearchQuerySubcategory(request);
        const preparedDomain = await services.ontology.loadSearchFilterExplorerDomain({
          request,
          discoveryMode,
        });
        return buildSearchFilterExplorerModel(preparedDomain, {
          category: requestCategory ?? scopeCategory,
          subcategory: requestSubcategory,
          fieldOptions,
          singleFieldBehavior,
        });
      };

      setFilterExplorerSession({
        title,
        model: createSearchFilterExplorerLoadingModel(title),
        initialDiscoveryMode: "matching",
        loadModelForDiscoveryMode: (mode) => buildPreparedModel(mode),
        draft: preparedDraft.draft,
        refreshOnDraftChange: Boolean(buildQueryForDraft),
        onDraftChange: (nextDraft) => {
          currentDraftRef.current = nextDraft;
          setFilterExplorerSession((currentSession) =>
            currentSession
              ? {
                  ...currentSession,
                  draft: nextDraft,
                }
              : currentSession,
          );
          onDraftChange?.(nextDraft, preparedContext);
        },
        resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        onApply: (nextDraft) => {
          onApply(nextDraft, preparedContext);
          setFilterExplorerSession(null);
        },
        onBack: (nextDraft) => {
          if (onBack) {
            onBack(nextDraft, preparedContext);
          } else {
            onApply(nextDraft, preparedContext);
          }
          setFilterExplorerSession(null);
          onReturn?.();
        },
        onExitRoot: (nextDraft) => {
          if (onExitRoot) {
            onExitRoot(nextDraft, preparedContext);
          } else {
            onApply(nextDraft, preparedContext);
          }
          setFilterExplorerSession(null);
        },
        onCancel: (nextDraft) => {
          onCancel?.(nextDraft, preparedContext);
          setFilterExplorerSession(null);
        },
      });
      return true;
    },
    [onUnavailable, query, services.ontology, services.search],
  );

  const closeFilterExplorer = React.useCallback(() => {
    setFilterExplorerSession(null);
  }, []);

  return {
    filterExplorerSession,
    openFilterExplorer,
    closeFilterExplorer,
  };
}
