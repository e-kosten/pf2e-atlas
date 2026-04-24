import type { AppConfig } from "../domain/config-types.js";
import type { OntologyDomainModel } from "../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../domain/search-field-domains.js";
import type { Pf2eDataService } from "../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult, SearchVocabularyResult } from "../data/vocabulary.js";
import type { Pf2eApplicationSearchDiscoveryService } from "./search-discovery-service.js";
import { buildSearchSemanticsDomain } from "./ontology/search-semantics-domain.js";

export type Pf2eApplicationOntologyService = {
  loadSearchSemanticsDomain: (discoveryMode?: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
};

type OntologyDomainDataService = Pick<Pf2eDataService, "listRecords"> & {
  getSearchSemanticsBootstrapSummary?: (options?: {
    traitLimitPerCategory?: number;
  }) => SearchSemanticsBootstrapSummaryResult;
  getSearchVocabulary?: (options?: { traitLimitPerCategory?: number }) => SearchVocabularyResult;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: OntologyDomainDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
): Pf2eApplicationOntologyService {
  const searchSemanticsDomains = new Map<SearchFilterDiscoveryMode | "default", Promise<OntologyDomainModel>>();

  const loadSearchSemanticsDomain = (discoveryMode?: SearchFilterDiscoveryMode): Promise<OntologyDomainModel> => {
    const cacheKey = discoveryMode ?? "default";
    const cached = searchSemanticsDomains.get(cacheKey);
    if (cached) {
      return cached;
    }

    const domainPromise = Promise.resolve().then(() => buildSearchSemanticsDomain(config, dataService, discoveryService));
    searchSemanticsDomains.set(cacheKey, domainPromise);
    void domainPromise.catch(() => {
      if (searchSemanticsDomains.get(cacheKey) === domainPromise) {
        searchSemanticsDomains.delete(cacheKey);
      }
    });
    return domainPromise;
  };

  return {
    loadSearchSemanticsDomain,
  };
}
