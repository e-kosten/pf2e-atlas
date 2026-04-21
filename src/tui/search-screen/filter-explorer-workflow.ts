import React from "react";

import { buildSearchFilterExplorerModel, buildSearchFilterExplorerTargetResolver } from "../filter-explorer/search-draft.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import { getSearchQuerySubcategory } from "../search/query-state.js";
import type { SearchFilterExplorerSession } from "./model.js";
import type { Pf2eTerminalAppServices } from "../app-services.js";

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
    initialDraft?: Pf2eTerminalFilterExplorerDraft;
    onApply: (draft: Pf2eTerminalFilterExplorerDraft) => void;
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
      initialDraft,
      onApply,
      onReturn,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      initialDraft?: Pf2eTerminalFilterExplorerDraft;
      onApply: (draft: Pf2eTerminalFilterExplorerDraft) => void;
      onReturn?: () => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = queryOverride ?? query;
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      const scopedFields = fieldOptions.map((fieldOption) => fieldOption.value);
      if (!scopeQuery.filters.category) {
        await onUnavailable("Choose a category before editing a discoverable query field.");
        return false;
      }
      if (fieldOptions.length === 0) {
        await onUnavailable("No discoverable query fields are available for the current scope.");
        return false;
      }

      const searchSemanticsDomain = services.ontology.loadSearchSemanticsDomain();
      const model = buildSearchFilterExplorerModel(searchSemanticsDomain, {
        category: scopeQuery.filters.category,
        subcategory: scopeSubcategory,
        fieldOptions,
        singleFieldBehavior,
      });
      if (model.rootNodes.length === 0) {
        await onUnavailable("No ontology-backed query explorer is available for that field.");
        return false;
      }

      const preparedDraft =
        initialDraft ??
        services.search.prepareFilterExplorerDraft(scopeQuery, scopedFields).draft;

      setFilterExplorerSession({
        title: fieldOptions.length === 1 ? `${fieldOptions[0]!.label} Explorer` : "Filter Explorer",
        model,
        draft: preparedDraft,
        resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        onApply: (nextDraft) => {
          onApply(nextDraft);
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
