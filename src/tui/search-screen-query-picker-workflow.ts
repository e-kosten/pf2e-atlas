import React from "react";

import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalQueryFieldSelectionMap, Pf2eTerminalSearchQuery } from "./search-service.js";
import type { SearchQueryFieldPickerSession } from "./search-screen-model.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";

export function useSearchQueryFieldPickerWorkflow({
  query,
  services,
  onUnavailable,
}: {
  query: Pf2eTerminalSearchQuery;
  services: Pick<Pf2eTerminalAppServices["user"], "ontology" | "search">;
  onUnavailable: (message: string) => Promise<void>;
}): {
  queryFieldPickerSession: SearchQueryFieldPickerSession | null;
  openQueryFieldPicker: (options: {
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
    onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
  }) => Promise<boolean>;
  closeQueryFieldPicker: () => void;
} {
  const [queryFieldPickerSession, setQueryFieldPickerSession] = React.useState<SearchQueryFieldPickerSession | null>(null);

  const openQueryFieldPicker = React.useCallback(
    async ({
      fieldOptions,
      initialSelections = {},
      onApply,
    }: {
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
      onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
    }): Promise<boolean> => {
      if (!query.filters.category) {
        await onUnavailable("Choose a category before editing a discoverable query field.");
        return false;
      }
      if (fieldOptions.length === 0) {
        await onUnavailable("No discoverable query fields are available for the current scope.");
        return false;
      }

      const searchSemanticsDomain = services.ontology.loadDomain("searchSemantics");
      const model = buildSearchFacetPickerModel(searchSemanticsDomain, {
        category: query.filters.category,
        subcategory: query.filters.subcategory,
        fieldOptions,
      });
      if (model.rootNodes.length === 0) {
        await onUnavailable("No ontology-backed query picker is available for that field.");
        return false;
      }

      setQueryFieldPickerSession({
        model,
        initialSelections: initialSelections as OntologyPickerSelectionMap,
        applySelection: (selection) => {
          onApply(selection as Pf2eTerminalQueryFieldSelectionMap);
          setQueryFieldPickerSession(null);
        },
      });
      return true;
    },
    [onUnavailable, query.filters.category, query.filters.subcategory, services.ontology],
  );

  const closeQueryFieldPicker = React.useCallback(() => {
    setQueryFieldPickerSession(null);
  }, []);

  return {
    queryFieldPickerSession,
    openQueryFieldPicker,
    closeQueryFieldPicker,
  };
}
