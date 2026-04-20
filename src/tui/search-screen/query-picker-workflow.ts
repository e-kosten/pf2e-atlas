import React from "react";

import { buildSearchFacetPickerModel as buildSearchQueryFieldSelectionPickerModel } from "../ontology-explorer/facet-picker-model.js";
import {
  buildHostedOntologyPickerInitialSnapshot,
  clearHostedOntologyPickerContract,
  registerHostedOntologyPickerContract,
} from "../ontology-explorer/picker-hosting.js";
import type { OntologyPickerSelectionMap } from "../ontology-explorer/picker-screen.js";
import {
  getSearchQuerySubcategory,
  type Pf2eTerminalQueryFieldOption,
  type Pf2eTerminalQueryFieldSelectionMap,
  type Pf2eTerminalSearchQuery,
} from "../search/service.js";
import type { SearchQueryFieldPickerSession } from "./model.js";
import type { Pf2eTerminalAppServices } from "../app-services.js";

type HostedPickerInitialSnapshotCandidate = {
  drillToFirstChild?: boolean;
  selectedNodeIds: string[];
};

function buildHostedPickerInitialSnapshotCandidates(options: {
  category: string;
  subcategory: string | null;
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  singleFieldBehavior: "list" | "directValues";
}): HostedPickerInitialSnapshotCandidate[] {
  const { category, subcategory, fieldOptions, singleFieldBehavior } = options;
  const categoryId = `searchSemantics:${category}`;
  const candidates: HostedPickerInitialSnapshotCandidate[] = [];
  const singleField = fieldOptions.length === 1 ? fieldOptions[0] : null;

  if (singleFieldBehavior === "directValues" && singleField) {
    if (subcategory) {
      candidates.push({
        selectedNodeIds: [
          categoryId,
          `${category}:subcategories`,
          `${category}:subcategory:${subcategory}`,
          `${category}:${subcategory}:metadataFields`,
          `${category}:${subcategory}:field:${singleField.value}`,
        ],
        drillToFirstChild: true,
      });
    }
    candidates.push({
      selectedNodeIds: [categoryId, `${category}:metadataFields`, `${category}:field:${singleField.value}`],
      drillToFirstChild: true,
    });
    if (singleField.value === "derivedTags") {
      candidates.push({
        selectedNodeIds: [categoryId, `${category}:commonDerivedTags`],
        drillToFirstChild: true,
      });
    }
    if (singleField.value === "traits") {
      candidates.push({
        selectedNodeIds: [categoryId, `${category}:commonTraits`],
        drillToFirstChild: true,
      });
    }
  }

  if (subcategory) {
    candidates.push({
      selectedNodeIds: [categoryId, `${category}:subcategories`, `${category}:subcategory:${subcategory}`],
    });
  }
  candidates.push({
    selectedNodeIds: [categoryId],
    drillToFirstChild: true,
  });

  return candidates;
}

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
    queryOverride?: Pf2eTerminalSearchQuery;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
    onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
    onReturn?: () => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  closeQueryFieldPicker: () => void;
} {
  const [selectionPickerSession, setSelectionPickerSession] = React.useState<SearchQueryFieldPickerSession | null>(
    null,
  );

  const openQueryFieldPicker = React.useCallback(
    async ({
      queryOverride,
      fieldOptions,
      initialSelections = {},
      onApply,
      onReturn,
      singleFieldBehavior = onReturn ? "directValues" : "list",
    }: {
      queryOverride?: Pf2eTerminalSearchQuery;
      fieldOptions: Pf2eTerminalQueryFieldOption[];
      initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
      onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
      onReturn?: () => void;
      singleFieldBehavior?: "list" | "directValues";
    }): Promise<boolean> => {
      const scopeQuery = queryOverride ?? query;
      const scopeSubcategory = getSearchQuerySubcategory(scopeQuery);
      if (!scopeQuery.filters.category) {
        await onUnavailable("Choose a category before editing a discoverable query field.");
        return false;
      }
      if (fieldOptions.length === 0) {
        await onUnavailable("No discoverable query fields are available for the current scope.");
        return false;
      }

      const searchSemanticsDomain = services.ontology.loadDomain("searchSemantics");
      const model = buildSearchQueryFieldSelectionPickerModel(searchSemanticsDomain, {
        category: scopeQuery.filters.category,
        subcategory: scopeSubcategory,
        fieldOptions,
      });
      if (model.rootNodes.length === 0) {
        await onUnavailable("No ontology-backed query picker is available for that field.");
        return false;
      }

      const initialSnapshot =
        buildHostedPickerInitialSnapshotCandidates({
          category: scopeQuery.filters.category,
          subcategory: scopeSubcategory,
          fieldOptions,
          singleFieldBehavior,
        })
          .map((candidate) => buildHostedOntologyPickerInitialSnapshot(model, candidate))
          .find((snapshot) => Boolean(snapshot)) ?? undefined;
      const rootDepth = initialSnapshot ? 1 : 0;
      registerHostedOntologyPickerContract(model, {
        applyHelpText: onReturn
          ? "apply the current query field selections and return to the query builder"
          : undefined,
        initialSnapshot,
        onReturn: onReturn
          ? () => {
              clearHostedOntologyPickerContract(model);
              setSelectionPickerSession(null);
              onReturn();
            }
          : undefined,
        rootBackHelpText: onReturn ? "return to the query builder" : undefined,
        rootBackLabel: onReturn ? "builder" : undefined,
        rootDepth,
        rootDetailBackHelpText: "return to the search semantics list",
        rootExitMode: onReturn ? "return" : rootDepth > 0 ? "apply" : undefined,
        rootFocusHelpText: "switch focus between search-semantics entries and detail",
        rootListTitle: "Search Semantics",
      });

      setSelectionPickerSession({
        model,
        initialSelections: initialSelections as OntologyPickerSelectionMap,
        applySelection: (selection) => {
          clearHostedOntologyPickerContract(model);
          onApply(selection as Pf2eTerminalQueryFieldSelectionMap);
          setSelectionPickerSession(null);
        },
      });
      return true;
    },
    [onUnavailable, query, services.ontology],
  );

  const closeQueryFieldPicker = React.useCallback(() => {
    if (selectionPickerSession) {
      clearHostedOntologyPickerContract(selectionPickerSession.model);
    }
    setSelectionPickerSession(null);
  }, [selectionPickerSession]);

  return {
    selectionPickerSession,
    openQueryFieldPicker,
    closeQueryFieldPicker,
  };
}
