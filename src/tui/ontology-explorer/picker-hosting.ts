import { getOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import type { OntologyDomainModel } from "../../types.js";
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
  rootDepth: number,
): OntologyBrowserSnapshot | undefined {
  if (rootDepth !== 1 || model.rootNodes.length !== 1) {
    return undefined;
  }

  const rootNode = model.rootNodes[0];
  const firstChild = getOntologyNodeChildren(rootNode)[0];
  if (!rootNode || !firstChild) {
    return undefined;
  }

  return {
    activePane: "list",
    browserState: {
      depth: 1,
      selectedNodeIds: [rootNode.id, firstChild.id],
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
