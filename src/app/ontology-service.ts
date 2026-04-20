import type { AppConfig, OntologyDomainId, OntologyDomainModel, OntologyDomainSummary } from "../types.js";
import type { Pf2eDataService } from "../data/service.js";
import { buildCatalogCategoriesDomain } from "./ontology/catalog-categories-domain.js";
import { ONTOLOGY_DOMAINS } from "./ontology/domain-summaries.js";
import { buildDerivedTagsDomain } from "./ontology/derived-tags-domain.js";
import { buildSearchSemanticsDomain } from "./ontology/search-semantics-domain.js";

export type Pf2eApplicationOntologyService = {
  listDomains: () => OntologyDomainSummary[];
  loadDomain: (id: OntologyDomainId) => OntologyDomainModel;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords">,
): Pf2eApplicationOntologyService {
  const domainCache = new Map<OntologyDomainId, OntologyDomainModel>();
  const domainBuilders: Record<OntologyDomainId, () => OntologyDomainModel> = {
    derivedTags: () => buildDerivedTagsDomain(config),
    catalogCategories: () => buildCatalogCategoriesDomain(dataService),
    searchSemantics: () => buildSearchSemanticsDomain(config, dataService),
  };

  return {
    listDomains: () => ONTOLOGY_DOMAINS,
    loadDomain: (id) => {
      const cached = domainCache.get(id);
      if (cached) {
        return cached;
      }

      const domain = domainBuilders[id]();
      domainCache.set(id, domain);
      return domain;
    },
  };
}
