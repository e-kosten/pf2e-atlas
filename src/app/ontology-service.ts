import type { AppConfig, OntologyDomainId, OntologyDomainModel, OntologyDomainSummary } from "../domain/index.js";
import type { Pf2eDataService } from "../data/service.js";
import { buildCatalogCategoriesDomain } from "./ontology/catalog-categories-domain.js";
import { ONTOLOGY_DOMAINS } from "./ontology/domain-summaries.js";
import { buildDerivedTagsDomain } from "./ontology/derived-tags-domain.js";
import { buildSearchSemanticsDomain } from "./ontology/search-semantics-domain.js";
import { createPf2eApplicationStorageService, type Pf2eApplicationStorageService } from "./storage-service.js";

export type Pf2eApplicationOntologyService = {
  listDomains: () => readonly OntologyDomainSummary[];
  loadDomain: (id: OntologyDomainId) => OntologyDomainModel;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords">,
  storage: Pick<
    Pf2eApplicationStorageService,
    "loadDerivedTagOntologyExplorerModel"
  > = createPf2eApplicationStorageService(config),
): Pf2eApplicationOntologyService {
  const domainCache = new Map<OntologyDomainId, OntologyDomainModel>();
  const buildDomain = (id: OntologyDomainId): OntologyDomainModel => {
    switch (id) {
      case "derivedTags":
        return buildDerivedTagsDomain(storage.loadDerivedTagOntologyExplorerModel());
      case "catalogCategories":
        return buildCatalogCategoriesDomain(dataService);
      case "searchSemantics":
        return buildSearchSemanticsDomain(config, dataService, () => loadDomain("derivedTags"));
    }
  };

  const loadDomain = (id: OntologyDomainId): OntologyDomainModel => {
    const cached = domainCache.get(id);
    if (cached) {
      return cached;
    }

    const domain = buildDomain(id);
    domainCache.set(id, domain);
    return domain;
  };

  return {
    listDomains: () => ONTOLOGY_DOMAINS.map((domain) => ({ ...domain })),
    loadDomain,
  };
}
