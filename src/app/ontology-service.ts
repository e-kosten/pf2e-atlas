import type { AppConfig } from "../domain/config-types.js";
import type { OntologyDomainModel } from "../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../domain/search-field-domains.js";
import type { SearchRequest } from "../domain/search-request-types.js";
import type { Pf2eDataService } from "../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult } from "../data/vocabulary.js";
import type { Pf2eApplicationSearchDiscoveryService } from "./search-discovery-service.js";
import {
  buildPreparedSearchFilterExplorerDomain,
  buildSearchSemanticsDomain,
} from "./ontology/search-semantics-domain.js";

export type Pf2eApplicationOntologyService = {
  loadSearchSemanticsDomain: () => Promise<OntologyDomainModel>;
  loadSearchFilterExplorerDomain: (options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
  }) => Promise<OntologyDomainModel>;
};

type OntologyDomainDataService = Pick<Pf2eDataService, "listRecords"> & {
  getSearchSemanticsBootstrapSummary: (options?: {
    traitLimitPerCategory?: number;
  }) => SearchSemanticsBootstrapSummaryResult;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: OntologyDomainDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
): Pf2eApplicationOntologyService {
  let searchSemanticsDomainPromise: Promise<OntologyDomainModel> | null = null;
  const searchFilterExplorerPromiseCache = new Map<string, Promise<OntologyDomainModel>>();

  function buildSearchFilterExplorerCacheKey(options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
  }): string {
    return `${options.discoveryMode}|${JSON.stringify(options.request)}`;
  }

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

  const loadSearchFilterExplorerDomain = (options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
  }): Promise<OntologyDomainModel> => {
    const cacheKey = buildSearchFilterExplorerCacheKey(options);
    const cached = searchFilterExplorerPromiseCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const promise = buildPreparedSearchFilterExplorerDomain(config, dataService, discoveryService, options);
    searchFilterExplorerPromiseCache.set(cacheKey, promise);
    void promise.catch(() => {
      searchFilterExplorerPromiseCache.delete(cacheKey);
    });
    return promise;
  };

  return {
    loadSearchSemanticsDomain,
    loadSearchFilterExplorerDomain,
  };
}
