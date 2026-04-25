import type { OntologyDomainModel } from "../../domain/ontology-types.js";

const SEARCH_FILTER_EXPLORER_LOADING_NODE_ID = "searchFilterExplorer:loading";

export function createSearchFilterExplorerLoadingModel(title: string): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: title,
    description: "Loading search explorer entries.",
    rootNodes: [
      {
        id: SEARCH_FILTER_EXPLORER_LOADING_NODE_ID,
        kind: "group",
        label: "Loading explorer entries...",
        listLabel: "Loading explorer entries...",
        filterText: "loading explorer entries",
        detailTitle: title,
        detailLines: [
          { text: "Loading explorer entries...", tone: "section" },
          { text: "Refreshing the scoped explorer tree in the background.", tone: "dim" },
        ],
      },
    ],
  };
}

export function isSearchFilterExplorerLoadingModel(model: OntologyDomainModel): boolean {
  return model.rootNodes.length === 0 || model.rootNodes[0]?.id === SEARCH_FILTER_EXPLORER_LOADING_NODE_ID;
}
