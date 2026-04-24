import type { AppConfig } from "../domain/config-types.js";
import type { OntologyDomainModel } from "../domain/ontology-types.js";
import type { Pf2eDataService } from "../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult, SearchVocabularyResult } from "../data/vocabulary.js";
import type { Pf2eApplicationSearchDiscoveryService } from "./search-discovery-service.js";
import { buildSearchSemanticsDomain } from "./ontology/search-semantics-domain.js";

export type Pf2eApplicationOntologyService = {
  loadSearchSemanticsDomain: () => Promise<OntologyDomainModel>;
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
  let searchSemanticsDomainPromise: Promise<OntologyDomainModel> | null = null;

  const loadSearchSemanticsDomain = (): Promise<OntologyDomainModel> => {
    if (searchSemanticsDomainPromise) {
      return searchSemanticsDomainPromise;
    }

    searchSemanticsDomainPromise = Promise.resolve().then(() =>
      buildSearchSemanticsDomain(config, dataService, discoveryService),
    );
    void searchSemanticsDomainPromise.catch(() => {
      if (searchSemanticsDomainPromise) {
        searchSemanticsDomainPromise = null;
      }
    });
    return searchSemanticsDomainPromise;
  };

  return {
    loadSearchSemanticsDomain,
  };
}
