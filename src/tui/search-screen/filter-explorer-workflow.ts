import React from "react";

import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../filter-explorer/search-draft-model.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import type { MetadataFilterNode } from "../search/metadata-filter-draft.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import type { SearchFilterExplorerSession } from "./model.js";
import type { Pf2eTerminalAppServices } from "../app-services.js";
import { createSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";
import type { SearchFilterExplorerFieldState } from "./filter-explorer-field-state.js";

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
    initialFieldState?: SearchFilterExplorerFieldState;
    preservedMetadata?: MetadataFilterNode | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    onQueryChange?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onReturn?: () => void;
    onCancel?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onBack?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onExitRoot?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  closeFilterExplorer: () => void;
} {
  const [filterExplorerSession, setFilterExplorerSession] = React.useState<SearchFilterExplorerSession | null>(null);

  const openFilterExplorer = React.useCallback(
    async ({
      queryOverride,
      initialFieldState,
      preservedMetadata,
      fieldOptions,
      onQueryChange,
      onReturn,
      onCancel,
      onBack,
      onExitRoot,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      initialFieldState?: SearchFilterExplorerFieldState;
      preservedMetadata?: MetadataFilterNode | null;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      onQueryChange?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onReturn?: () => void;
      onCancel?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onBack?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onExitRoot?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
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
        initialFieldState,
        preservedMetadata,
        fieldOptions,
        refreshOnQueryChange: Boolean(onQueryChange),
        onQueryChange: (nextQuery, nextFieldState) => {
          currentQueryRef.current = nextQuery;
          setFilterExplorerSession((currentSession) =>
            currentSession
              ? {
                  ...currentSession,
                  query: nextQuery,
                }
              : currentSession,
          );
          onQueryChange?.(nextQuery, nextFieldState);
        },
        resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        onBack: (nextQuery, nextFieldState) => {
          onBack?.(nextQuery, nextFieldState);
          setFilterExplorerSession(null);
          onReturn?.();
        },
        onExitRoot: (nextQuery, nextFieldState) => {
          onExitRoot?.(nextQuery, nextFieldState);
          setFilterExplorerSession(null);
        },
        onCancel: (nextQuery, nextFieldState) => {
          onCancel?.(nextQuery, nextFieldState);
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
