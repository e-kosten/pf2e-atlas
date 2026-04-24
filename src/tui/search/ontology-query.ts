import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import {
  normalizeSearchQuery,
} from "./query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchQuery,
} from "./service-types.js";

export function createSearchQueryFromOntologyQuery(
  query: OntologyNodeQuery,
  _dependencies: unknown,
  _fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  return normalizeSearchQuery(query.request);
}
