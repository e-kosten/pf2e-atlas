import type { OntologyDomainId, OntologyDomainSummary } from "../../domain/ontology-types.js";

const ONTOLOGY_DOMAIN_SUMMARIES: Record<OntologyDomainId, OntologyDomainSummary> = {
  derivedTags: {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Browse the authored derived-tag ontology with live record coverage and editorial detail.",
  },
  searchSemantics: {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Explore category-specific metadata fields, live value spaces, and advanced search predicates.",
  },
};

export function getOntologyDomainSummary(id: OntologyDomainId): OntologyDomainSummary {
  const summary = ONTOLOGY_DOMAIN_SUMMARIES[id];
  if (!summary) {
    throw new Error(`Unknown ontology domain: ${String(id)}`);
  }

  return { ...summary };
}
