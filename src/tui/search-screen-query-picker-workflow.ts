import React from "react";

import { buildSearchFacetPickerModel as buildSearchQueryFieldSelectionPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import {
  getSearchQuerySubcategory,
  type Pf2eTerminalQueryFieldOption,
  type Pf2eTerminalQueryFieldSelectionMap,
  type Pf2eTerminalSearchQuery,
} from "./search-service.js";
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
  selectionPickerSession: SearchQueryFieldPickerSession | null;
  openQueryFieldPicker: (options: {
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
    onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
  }) => Promise<boolean>;
  closeQueryFieldPicker: () => void;
} {
  const querySubcategory = React.useMemo(() => getSearchQuerySubcategory(query), [query]);
  const [selectionPickerSession, setSelectionPickerSession] = React.useState<SearchQueryFieldPickerSession | null>(null);

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
      const model = buildSearchQueryFieldSelectionPickerModel(searchSemanticsDomain, {
        category: query.filters.category,
        subcategory: querySubcategory,
        fieldOptions,
      });
      if (model.rootNodes.length === 0) {
        await onUnavailable("No ontology-backed query picker is available for that field.");
        return false;
      }

      setSelectionPickerSession({
        model,
        initialSelections: initialSelections as OntologyPickerSelectionMap,
        applySelection: (selection) => {
          onApply(selection as Pf2eTerminalQueryFieldSelectionMap);
          setSelectionPickerSession(null);
        },
      });
      return true;
    },
    [onUnavailable, query.filters.category, querySubcategory, services.ontology],
  );

  const closeQueryFieldPicker = React.useCallback(() => {
    setSelectionPickerSession(null);
  }, []);

  return {
    selectionPickerSession,
    openQueryFieldPicker,
    closeQueryFieldPicker,
  };
}
