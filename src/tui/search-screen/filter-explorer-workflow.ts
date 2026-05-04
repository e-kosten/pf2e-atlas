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
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type { FilterExplorerComposeTarget, FilterExplorerSelectTargetOutcome } from "../filter-explorer/types.js";

function isUnscopedFilterExplorerField(fieldOption: Pf2eTerminalQueryFieldOption): boolean {
  return fieldOption.value === "rarity" || fieldOption.value === "pack";
}

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
    title?: string;
    queryOverride?: Pf2eTerminalSearchQuery;
    initialDiscoveryMode?: SearchFilterDiscoveryMode;
    initialFieldState?: SearchFilterExplorerFieldState;
    preservedMetadata?: MetadataFilterNode | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    resolveSelectionTarget?: (node: import("../../domain/ontology-types.js").OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
    onQueryChange?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onReturn?: () => void;
    onCancel?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onBack?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onExitRoot?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
    onSelectTarget?: (
      outcome: FilterExplorerSelectTargetOutcome,
      query: Pf2eTerminalSearchQuery,
      fieldState: SearchFilterExplorerFieldState,
      discoveryMode: SearchFilterDiscoveryMode,
    ) => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  closeFilterExplorer: () => void;
} {
  const [filterExplorerSession, setFilterExplorerSession] = React.useState<SearchFilterExplorerSession | null>(null);

  const openFilterExplorer = React.useCallback(
    async ({
      title,
      queryOverride,
      initialDiscoveryMode = "matching",
      initialFieldState,
      preservedMetadata,
      fieldOptions,
      resolveSelectionTarget,
      onQueryChange,
      onReturn,
      onCancel,
      onBack,
      onExitRoot,
      onSelectTarget,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      title?: string;
      queryOverride?: Pf2eTerminalSearchQuery;
      initialDiscoveryMode?: SearchFilterDiscoveryMode;
      initialFieldState?: SearchFilterExplorerFieldState;
      preservedMetadata?: MetadataFilterNode | null;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      resolveSelectionTarget?: (node: import("../../domain/ontology-types.js").OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
      onQueryChange?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onReturn?: () => void;
      onCancel?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onBack?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onExitRoot?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
      onSelectTarget?: (
        outcome: FilterExplorerSelectTargetOutcome,
        query: Pf2eTerminalSearchQuery,
        fieldState: SearchFilterExplorerFieldState,
        discoveryMode: SearchFilterDiscoveryMode,
      ) => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = services.search.normalizeQuery(queryOverride ?? query);
      const scopeCategory = getSearchQueryCategory(scopeQuery);
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      const sessionTitle = title ?? (fieldOptions.length === 1 ? `${fieldOptions[0]!.label} Explorer` : "Filter Explorer");
      if (!scopeCategory && fieldOptions.some((fieldOption) => !isUnscopedFilterExplorerField(fieldOption))) {
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
        const preparedDomain = requestCategory
          ? await services.ontology.loadSearchFilterExplorerDomain({
              request,
              discoveryMode,
            })
          : await services.ontology.loadSearchSemanticsDomain({ discoveryMode });
        return buildSearchFilterExplorerModel(preparedDomain, {
          category: requestCategory ?? scopeCategory,
          subcategory: requestSubcategory ?? scopeSubcategory,
          fieldOptions,
          singleFieldBehavior,
        });
      };

      setFilterExplorerSession({
        title: sessionTitle,
        model: createSearchFilterExplorerLoadingModel(sessionTitle),
        initialDiscoveryMode,
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
        resolveSelectionTarget: resolveSelectionTarget ?? buildSearchFilterExplorerTargetResolver(fieldOptions),
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
        onSelectTarget: onSelectTarget
          ? (outcome, nextQuery, nextFieldState, discoveryMode) => {
              onSelectTarget(outcome, nextQuery, nextFieldState, discoveryMode);
              setFilterExplorerSession(null);
            }
          : undefined,
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
