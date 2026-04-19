import React from "react";

import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type { Pf2eTerminalSearchQuery } from "./search-service.js";
import {
  type SearchFacetPickerSession,
  applyFacetPickerSelectionsToRequest,
  buildFacetPickerInitialSelections,
} from "./search-screen-model.js";
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

    const fieldOptions = services.search.getFacetFieldOptions(query.filters.category, query.filters.subcategory);
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
      initialSelections: buildFacetPickerInitialSelections(
        query,
        fieldOptions.map((option) => option.value),
      ),
      scopedFields: fieldOptions.map((option) => option.value),
    });
  }, [onUnavailable, query, services.ontology, services.search]);

  const applyFacetPicker = React.useCallback(
    (selection: OntologyPickerSelectionMap) => {
      if (!facetPickerSession) {
        return;
      }
      applyQueryUpdate((request) =>
        applyFacetPickerSelectionsToRequest(request, selection, facetPickerSession.scopedFields),
      );
      setFacetPickerSession(null);
    },
    [applyQueryUpdate, facetPickerSession],
  );

  return {
    facetPickerSession,
    openFacetPicker,
    applyFacetPicker,
  };
}
