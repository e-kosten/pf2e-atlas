import React from "react";

import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../filter-explorer/search-draft-model.js";
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../search/service.js";
import type { MetadataFilterNode } from "../search/metadata-filter-draft.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import type { SearchFilterExplorerSession } from "./model.js";
import type { Pf2eTerminalAppServices } from "../app-services.js";
import { createSearchFilterExplorerLoadingModel } from "./filter-explorer-loading-model.js";
import type { SearchFilterExplorerFieldState } from "./filter-explorer-field-state.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type { FilterExplorerComposeTarget } from "../filter-explorer/types.js";
import type { SearchFilterExplorerSessionEvent } from "./filter-explorer-session-events.js";

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
    resolveSelectionTarget?: (
      node: import("../../domain/ontology-types.js").OntologyNode | undefined,
    ) => FilterExplorerComposeTarget | undefined;
    onEvent?: (event: SearchFilterExplorerSessionEvent) => void;
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
      onEvent,
      singleFieldBehavior = "list",
    }: {
      title?: string;
      queryOverride?: Pf2eTerminalSearchQuery;
      initialDiscoveryMode?: SearchFilterDiscoveryMode;
      initialFieldState?: SearchFilterExplorerFieldState;
      preservedMetadata?: MetadataFilterNode | null;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      resolveSelectionTarget?: (
        node: import("../../domain/ontology-types.js").OntologyNode | undefined,
      ) => FilterExplorerComposeTarget | undefined;
      onEvent?: (event: SearchFilterExplorerSessionEvent) => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = services.search.normalizeQuery(queryOverride ?? query);
      const scopeCategory = getSearchQueryCategory(scopeQuery);
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      const sessionTitle =
        title ?? (fieldOptions.length === 1 ? `${fieldOptions[0]!.label} Explorer` : "Filter Explorer");
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
        options: { targetFields?: readonly string[] } = {},
      ): Promise<ReturnType<typeof buildSearchFilterExplorerModel>> => {
        const request = services.search.normalizeQuery(currentQueryRef.current);
        const requestCategory = getSearchQueryCategory(request);
        const requestSubcategory = getSearchQuerySubcategory(request);
        const modelFieldOptions = options.targetFields
          ? fieldOptions.filter((fieldOption) => options.targetFields?.includes(fieldOption.value))
          : fieldOptions;
        const preparedDomain = requestCategory
          ? await services.ontology.loadSearchFilterExplorerDomain({
              request,
              discoveryMode,
              targetFields: modelFieldOptions.map((fieldOption) => fieldOption.value),
            })
          : await services.ontology.loadSearchSemanticsDomain({ discoveryMode });
        return buildSearchFilterExplorerModel(preparedDomain, {
          category: requestCategory ?? scopeCategory,
          subcategory: requestSubcategory ?? scopeSubcategory,
          fieldOptions: modelFieldOptions,
          singleFieldBehavior,
        });
      };

      setFilterExplorerSession({
        title: sessionTitle,
        model: createSearchFilterExplorerLoadingModel(sessionTitle),
        initialDiscoveryMode,
        loadModelForDiscoveryMode: (mode, options) => buildPreparedModel(mode, options),
        query: scopeQuery,
        initialFieldState,
        preservedMetadata,
        fieldOptions,
        refreshOnQueryChange: Boolean(onEvent),
        resolveSelectionTarget: resolveSelectionTarget ?? buildSearchFilterExplorerTargetResolver(fieldOptions),
        selectTargetMode: Boolean(resolveSelectionTarget),
        onEvent: (event) => {
          if (event.kind === "change") {
            currentQueryRef.current = event.query;
            setFilterExplorerSession((currentSession) =>
              currentSession
                ? {
                    ...currentSession,
                    query: event.query,
                  }
                : currentSession,
            );
          } else {
            setFilterExplorerSession(null);
          }
          onEvent?.(event);
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
