import React from "react";

import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type { Pf2eTerminalSearchRequest } from "./search-service.js";
import {
  type SearchFacetPickerSession,
  applyFacetPickerSelectionsToRequest,
  buildFacetPickerInitialSelections,
} from "./search-screen-model.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";

export function useSearchFacetWorkflow({
  draft,
  applyDraftUpdate,
  services,
  onUnavailable,
}: {
  draft: Pf2eTerminalSearchRequest;
  applyDraftUpdate: (update: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest) => void;
  services: Pick<Pf2eTerminalAppServices["user"], "ontology" | "search">;
  onUnavailable: (message: string) => Promise<void>;
}): {
  facetPickerSession: SearchFacetPickerSession | null;
  openFacetPicker: () => Promise<void>;
  applyFacetPicker: (selection: OntologyPickerSelectionMap) => void;
} {
  const [facetPickerSession, setFacetPickerSession] = React.useState<SearchFacetPickerSession | null>(null);

  const openFacetPicker = React.useCallback(async () => {
    if (!draft.filters.category) {
      await onUnavailable("Choose a category before editing a discoverable facet filter.");
      return;
    }

    const fieldOptions = services.search.getFacetFieldOptions(draft.filters.category, draft.filters.subcategory);
    if (fieldOptions.length === 0) {
      await onUnavailable("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const searchSemanticsDomain = services.ontology.loadDomain("searchSemantics");
    const model = buildSearchFacetPickerModel(searchSemanticsDomain, {
      category: draft.filters.category,
      subcategory: draft.filters.subcategory,
      fieldOptions,
    });
    if (model.rootNodes.length === 0) {
      await onUnavailable("No ontology-backed facet hierarchy is available for that scope.");
      return;
    }

    setFacetPickerSession({
      model,
      initialSelections: buildFacetPickerInitialSelections(
        draft,
        fieldOptions.map((option) => option.value),
      ),
      scopedFields: fieldOptions.map((option) => option.value),
    });
  }, [draft, onUnavailable, services.ontology, services.search]);

  const applyFacetPicker = React.useCallback(
    (selection: OntologyPickerSelectionMap) => {
      if (!facetPickerSession) {
        return;
      }
      applyDraftUpdate((request) =>
        applyFacetPickerSelectionsToRequest(request, selection, facetPickerSession.scopedFields),
      );
      setFacetPickerSession(null);
    },
    [applyDraftUpdate, facetPickerSession],
  );

  return {
    facetPickerSession,
    openFacetPicker,
    applyFacetPicker,
  };
}
