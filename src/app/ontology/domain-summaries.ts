import type { OntologyDomainId, OntologyDomainSummary } from "../../types.js";

export const ONTOLOGY_DOMAINS: OntologyDomainSummary[] = [
  {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Browse the authored derived-tag ontology with live record coverage and editorial detail.",
  },
  {
    id: "catalogCategories",
    label: "Categories",
    description:
      "Browse top-level catalog categories and subcategories with live record counts and ready-to-run browse scopes.",
  },
  {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Explore category-specific metadata fields, live value spaces, and advanced search predicates.",
  },
];

export function getOntologyDomainSummary(id: OntologyDomainId): OntologyDomainSummary {
  const domain = ONTOLOGY_DOMAINS.find((entry) => entry.id === id);
  if (!domain) {
    throw new Error(`Unknown ontology domain: ${id}`);
  }
  return domain;
}
