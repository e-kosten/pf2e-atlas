import { getOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import type { OntologyBrowserSnapshot } from "./ui.js";

export type HostedOntologyPickerContract = {
  applyHelpText?: string;
  initialSnapshot?: OntologyBrowserSnapshot;
  onReturn?: () => void;
  rootBackHelpText?: string;
  rootBackLabel?: string;
  rootDepth?: number;
  rootDetailBackHelpText?: string;
  rootExitMode?: "apply" | "return";
  rootFocusHelpText?: string;
  rootListTitle?: string;
};

const hostedPickerContracts = new WeakMap<OntologyDomainModel, HostedOntologyPickerContract>();

export function buildHostedOntologyPickerInitialSnapshot(
  model: OntologyDomainModel,
  options: {
    drillToFirstChild?: boolean;
    selectedNodeIds: string[];
  },
): OntologyBrowserSnapshot | undefined {
  if (options.selectedNodeIds.length === 0) {
    return undefined;
  }

  let nodes = model.rootNodes;
  const selectedNodeIds: string[] = [];
  let currentNode;

  for (const selectedId of options.selectedNodeIds) {
    currentNode = nodes.find((node) => node.id === selectedId);
    if (!currentNode) {
      return undefined;
    }
    selectedNodeIds.push(currentNode.id);
    nodes = getOntologyNodeChildren(currentNode);
  }

  if (options.drillToFirstChild) {
    const firstChild = currentNode ? getOntologyNodeChildren(currentNode)[0] : undefined;
    if (!firstChild) {
      return undefined;
    }
    selectedNodeIds.push(firstChild.id);
  }

  return {
    activePane: "list",
    browserState: {
      depth: selectedNodeIds.length - 1,
      selectedNodeIds,
      filter: "",
      detailScroll: 0,
    },
    layoutMode: "split",
    searchInput: "",
    searchMode: false,
  };
}

export function registerHostedOntologyPickerContract(
  model: OntologyDomainModel,
  contract: HostedOntologyPickerContract,
): void {
  hostedPickerContracts.set(model, contract);
}

export function getHostedOntologyPickerContract(model: OntologyDomainModel): HostedOntologyPickerContract | undefined {
  return hostedPickerContracts.get(model);
}

export function clearHostedOntologyPickerContract(model: OntologyDomainModel): void {
  hostedPickerContracts.delete(model);
}
