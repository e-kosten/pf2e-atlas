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
    onDraftChange?: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onApply: (
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void;
    onReturn?: () => void;
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
      onDraftChange,
      onApply,
      onReturn,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      initialPreparedDraft?: Pf2eTerminalPreparedFilterExplorerDraft;
      onDraftChange?: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onApply: (
        draft: Pf2eTerminalFilterExplorerDraft,
        context: Pf2eTerminalPreparedFilterExplorerContext,
      ) => void;
      onReturn?: () => void;
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

      const buildPreparedModel = async (
        discoveryMode: "matching" | "catalog",
      ): Promise<ReturnType<typeof buildSearchFilterExplorerModel>> => {
        const preparedDomain = await services.ontology.loadSearchFilterExplorerDomain({
          request: scopeQuery,
          discoveryMode,
        });
        return buildSearchFilterExplorerModel(preparedDomain, {
          category: scopeCategory,
          subcategory: scopeSubcategory,
          fieldOptions,
          singleFieldBehavior,
        });
      };

      const preparedDraft = initialPreparedDraft ?? services.search.prepareFilterExplorerDraft(scopeQuery, scopedFields);

      setFilterExplorerSession({
        title,
        model: createSearchFilterExplorerLoadingModel(title),
        initialDiscoveryMode: "matching",
        loadModelForDiscoveryMode: (mode) => buildPreparedModel(mode),
        draft: preparedDraft.draft,
        onDraftChange: (nextDraft) => {
          setFilterExplorerSession((currentSession) =>
            currentSession
              ? {
                  ...currentSession,
                  draft: nextDraft,
                }
              : currentSession,
          );
          onDraftChange?.(nextDraft, {
            preservedMetadata: preparedDraft.preservedMetadata,
            scopedFields: preparedDraft.scopedFields,
          });
        },
        resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        onApply: (nextDraft) => {
          onApply(nextDraft, {
            preservedMetadata: preparedDraft.preservedMetadata,
            scopedFields: preparedDraft.scopedFields,
          });
          setFilterExplorerSession(null);
          onReturn?.();
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
