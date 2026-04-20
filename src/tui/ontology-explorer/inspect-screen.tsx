import React from "react";

import type { OntologyDomainId, OntologyDomainModel, OntologyNodeQuery } from "../../domain/ontology-types.js";
import { FilterExplorerScreen, type FilterExplorerOptions } from "../filter-explorer/index.js";

type OntologyInspectDomainSummary = {
  id: OntologyDomainId;
  label: string;
  description: string;
};

type OntologyInspectModelSource = {
  listDomains: () => readonly OntologyInspectDomainSummary[];
  loadDomain: (id: OntologyDomainId) => OntologyDomainModel;
};

export type OntologyInspectExplorerSnapshot = NonNullable<FilterExplorerOptions["initialSnapshot"]>;

function buildExplorerFilterText(...parts: Array<string | undefined>): string {
  return parts
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0)
    .join(" ")
    .toLowerCase();
}

function buildOntologyInspectDomainNode(
  domain: OntologyInspectDomainSummary,
  ontology: OntologyInspectModelSource,
) {
  return {
    id: `domain:${domain.id}`,
    kind: "domain" as const,
    label: domain.label,
    listLabel: domain.label,
    filterText: buildExplorerFilterText(domain.id, domain.label, domain.description),
    detailTitle: "Ontology Domain",
    detailLines: [
      { text: `Domain: ${domain.id}` },
      { text: "" },
      {
        text:
          domain.description || "Inspect the domain in the shared explorer and open search queries from matching entries.",
      },
    ],
    loadChildren: () => ontology.loadDomain(domain.id).rootNodes,
  };
}

export function buildOntologyInspectExplorerModel(ontology: OntologyInspectModelSource): OntologyDomainModel {
  const domains = ontology.listDomains();
  return {
    id: "searchSemantics",
    label: "Ontology Browser",
    description: "Browse ontology-backed domains and open shared browse/search queries from focused entries.",
    rootNodes: domains.map((domain) => buildOntologyInspectDomainNode(domain, ontology)),
  };
}

export function OntologyInspectScreen({
  initialSnapshot,
  model,
  onExit,
  onOpenQuery,
}: {
  initialSnapshot?: OntologyInspectExplorerSnapshot;
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => void;
}): React.JSX.Element {
  return (
    <FilterExplorerScreen
      title={model.label}
      model={model}
      initialSnapshot={initialSnapshot}
      onExit={onExit}
      mode={{
        kind: "inspect-and-open",
        onOpenQuery,
      }}
    />
  );
}
