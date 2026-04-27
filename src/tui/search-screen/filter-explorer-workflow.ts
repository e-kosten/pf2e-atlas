import React from "react";

import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../filter-explorer/search-draft-model.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
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
    initialDraft?: Pf2eTerminalFilterExplorerDraft;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    onQueryChange?: (query: Pf2eTerminalSearchQuery) => void;
    onReturn?: () => void;
    onCancel?: (query: Pf2eTerminalSearchQuery) => void;
    onBack?: (query: Pf2eTerminalSearchQuery) => void;
    onExitRoot?: (query: Pf2eTerminalSearchQuery) => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  closeFilterExplorer: () => void;
} {
  const [filterExplorerSession, setFilterExplorerSession] = React.useState<SearchFilterExplorerSession | null>(null);

  const openFilterExplorer = React.useCallback(
    async ({
      queryOverride,
      initialDraft,
      fieldOptions,
      onQueryChange,
      onReturn,
      onCancel,
      onBack,
      onExitRoot,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      initialDraft?: Pf2eTerminalFilterExplorerDraft;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      onQueryChange?: (query: Pf2eTerminalSearchQuery) => void;
      onReturn?: () => void;
      onCancel?: (query: Pf2eTerminalSearchQuery) => void;
      onBack?: (query: Pf2eTerminalSearchQuery) => void;
      onExitRoot?: (query: Pf2eTerminalSearchQuery) => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = services.search.normalizeQuery(queryOverride ?? query);
      const scopeCategory = getSearchQueryCategory(scopeQuery);
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      const title = fieldOptions.length === 1 ? `${fieldOptions[0]!.label} Explorer` : "Filter Explorer";
      if (!scopeCategory) {
        await onUnavailable("Choose a category before editing a discoverable query field.");
        return false;
      }
      if (fieldOptions.length === 0) {
        await onUnavailable("No discoverable query fields are available for the current scope.");
        return false;
      }

      const currentQueryRef = {
        current: scopeQuery,
      };

      const buildPreparedModel = async (
        discoveryMode: "matching" | "catalog",
      ): Promise<ReturnType<typeof buildSearchFilterExplorerModel>> => {
        const request = services.search.normalizeQuery(currentQueryRef.current);
        const requestCategory = getSearchQueryCategory(request);
        const requestSubcategory = getSearchQuerySubcategory(request);
        const preparedDomain = await services.ontology.loadSearchFilterExplorerDomain({
          request,
          discoveryMode,
        });
        return buildSearchFilterExplorerModel(preparedDomain, {
          category: requestCategory ?? scopeCategory,
          subcategory: requestSubcategory ?? scopeSubcategory,
          fieldOptions,
          singleFieldBehavior,
        });
      };

      setFilterExplorerSession({
        title,
        model: createSearchFilterExplorerLoadingModel(title),
        initialDiscoveryMode: "matching",
        loadModelForDiscoveryMode: (mode) => buildPreparedModel(mode),
        query: scopeQuery,
        initialDraft,
        fieldOptions,
        refreshOnQueryChange: Boolean(onQueryChange),
        onQueryChange: (nextQuery) => {
          currentQueryRef.current = nextQuery;
          setFilterExplorerSession((currentSession) =>
            currentSession
              ? {
                  ...currentSession,
                  query: nextQuery,
                }
              : currentSession,
          );
          onQueryChange?.(nextQuery);
        },
        resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        onBack: (nextQuery) => {
          onBack?.(nextQuery);
          setFilterExplorerSession(null);
          onReturn?.();
        },
        onExitRoot: (nextQuery) => {
          onExitRoot?.(nextQuery);
          setFilterExplorerSession(null);
        },
        onCancel: (nextQuery) => {
          onCancel?.(nextQuery);
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
