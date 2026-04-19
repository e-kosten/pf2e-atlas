import React from "react";

import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type {
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./search-service.js";
import type { SearchFacetPickerSession } from "./search-screen-model.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";

export function useSearchFacetWorkflow({
  query,
  applyQueryUpdate,
  services,
  onUnavailable,
}: {
  query: Pf2eTerminalSearchQuery;
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  services: Pick<Pf2eTerminalAppServices["user"], "ontology" | "search">;
  onUnavailable: (message: string) => Promise<void>;
}): {
  facetPickerSession: SearchFacetPickerSession | null;
  openFacetPicker: () => Promise<void>;
  applyFacetPicker: (selection: OntologyPickerSelectionMap) => void;
} {
  const [facetPickerSession, setFacetPickerSession] = React.useState<SearchFacetPickerSession | null>(null);

  const openFacetPicker = React.useCallback(async () => {
    if (!query.filters.category) {
      await onUnavailable("Choose a category before editing a discoverable facet filter.");
      return;
    }

    const fieldOptions = services.search
      .getQueryFieldOptions(query.filters.category, query.filters.subcategory)
      .filter((option) => ["set", "enumString", "boolean"].includes(option.fieldType))
      .sort((left, right) => {
        if (left.editor === right.editor) {
          return left.label.localeCompare(right.label);
        }
        return left.editor === "ontologyPicker" ? -1 : 1;
      });
    if (fieldOptions.length === 0) {
      await onUnavailable("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const searchSemanticsDomain = services.ontology.loadDomain("searchSemantics");
    const model = buildSearchFacetPickerModel(searchSemanticsDomain, {
      category: query.filters.category,
      subcategory: query.filters.subcategory,
      fieldOptions,
    });
    if (model.rootNodes.length === 0) {
      await onUnavailable("No ontology-backed facet hierarchy is available for that scope.");
      return;
    }

    setFacetPickerSession({
      model,
      initialSelections: services.search.buildDiscoverableQueryFieldSelections(
        query,
        fieldOptions.map((option) => option.value),
      ) as OntologyPickerSelectionMap,
      scopedFields: fieldOptions.map((option) => option.value),
    });
  }, [onUnavailable, query, services.ontology, services.search]);

  const applyFacetPicker = React.useCallback(
    (selection: OntologyPickerSelectionMap) => {
      if (!facetPickerSession) {
        return;
      }
      applyQueryUpdate((request) =>
        services.search.applyDiscoverableQueryFieldSelections(
          request,
          selection as Pf2eTerminalQueryFieldSelectionMap,
          facetPickerSession.scopedFields,
        ),
      );
      setFacetPickerSession(null);
    },
    [applyQueryUpdate, facetPickerSession, services.search],
  );

  return {
    facetPickerSession,
    openFacetPicker,
    applyFacetPicker,
  };
}
