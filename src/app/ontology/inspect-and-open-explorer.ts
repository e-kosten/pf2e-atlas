import type { Pf2eDataService } from "../../data/service.js";
import type { OntologyDomainModel, OntologyNode, OntologyNodeQuery } from "../../domain/ontology-types.js";
import type { Pf2eApplicationOntologyService } from "../ontology-service.js";
import {
  buildFilterText,
  buildKeyValueDetailLines,
  buildNormalizedRecordNode,
  cloneOntologyNode,
} from "./node-helpers.js";

type OntologyExplorerDataService = Pick<Pf2eDataService, "listRecords">;
type OntologyExplorerOntologyService = Pick<Pf2eApplicationOntologyService, "listDomains" | "loadDomain">;

const ONTOLOGY_EXPLORER_ROOT_ID = "searchSemantics";

function buildOntologyQueryRecordChildren(
  dataService: OntologyExplorerDataService,
  query: OntologyNodeQuery | undefined,
): readonly OntologyNode[] {
  if (!query || query.kind !== "listRecords") {
    return [];
  }

  return dataService.listRecords(query.filters).records.map(buildNormalizedRecordNode);
}

function decorateNodeForInspectAndOpen(
  node: OntologyNode,
  dataService: OntologyExplorerDataService,
): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const children = cloned.children?.map((child) => decorateNodeForInspectAndOpen(child, dataService));

  if (children) {
    return {
      ...cloned,
      children,
    };
  }

  if (cloned.loadChildren) {
    return {
      ...cloned,
      loadChildren: () => cloned.loadChildren!().map((child) => decorateNodeForInspectAndOpen(child, dataService)),
    };
  }

  if (cloned.query?.kind === "listRecords") {
    return {
      ...cloned,
      loadChildren: () => buildOntologyQueryRecordChildren(dataService, cloned.query),
    };
  }

  return cloned;
}

function buildOntologyDomainNode(
  domain: ReturnType<OntologyExplorerOntologyService["listDomains"]>[number],
  ontology: OntologyExplorerOntologyService,
  dataService: OntologyExplorerDataService,
): OntologyNode {
  return {
    id: `domain:${domain.id}`,
    kind: "domain",
    label: domain.label,
    filterText: buildFilterText(domain.id, domain.label, domain.description),
    listLabel: domain.label,
    detailTitle: "Ontology Domain",
    detailLines: buildKeyValueDetailLines(
      domain.label,
      [["Domain", domain.id]],
      `${domain.description} Drill in to inspect the domain in the shared explorer.`,
    ),
    loadChildren: () =>
      ontology.loadDomain(domain.id).rootNodes.map((node) => decorateNodeForInspectAndOpen(node, dataService)),
  };
}

export function buildInspectAndOpenOntologyExplorerModel(
  ontology: OntologyExplorerOntologyService,
  dataService: OntologyExplorerDataService,
): OntologyDomainModel {
  const domains = ontology.listDomains();
  return {
    id: ONTOLOGY_EXPLORER_ROOT_ID,
    label: "Ontology Browser",
    description: "Browse ontology-backed domains, inspect matching records inline, and open search results when needed.",
    rootNodes: domains.map((domain) => buildOntologyDomainNode(domain, ontology, dataService)),
  };
}
