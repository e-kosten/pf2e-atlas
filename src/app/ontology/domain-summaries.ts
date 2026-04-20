import type { OntologyDomainId, OntologyDomainSummary } from "../../domain/ontology-types.js";

const DERIVED_TAGS_DOMAIN_SUMMARY = {
  id: "derivedTags",
  label: "Derived Tags",
  description: "Browse the authored derived-tag ontology with live record coverage and editorial detail.",
} as const satisfies OntologyDomainSummary;

const SEARCH_SEMANTICS_DOMAIN_SUMMARY = {
  id: "searchSemantics",
  label: "Search Semantics",
  description: "Explore category-specific metadata fields, live value spaces, and advanced search predicates.",
} as const satisfies OntologyDomainSummary;

export function getOntologyDomainSummary(id: OntologyDomainId): OntologyDomainSummary {
  switch (id) {
    case "derivedTags":
      return { ...DERIVED_TAGS_DOMAIN_SUMMARY };
    case "searchSemantics":
      return { ...SEARCH_SEMANTICS_DOMAIN_SUMMARY };
  }

  throw new Error(`Unknown ontology domain: ${String(id)}`);
}
