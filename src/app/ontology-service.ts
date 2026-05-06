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
  loadSearchSemanticsDomain: (options: { discoveryMode: SearchFilterDiscoveryMode }) => Promise<OntologyDomainModel>;
  loadSearchFilterExplorerDomain: (options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
    targetFields?: readonly string[];
  }) => Promise<OntologyDomainModel>;
};

type OntologyDomainDataService = Pick<Pf2eDataService, "getPack" | "listRecords"> & {
  getSearchSemanticsBootstrapSummary: (options?: {
    traitLimitPerCategory?: number;
  }) => SearchSemanticsBootstrapSummaryResult;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: OntologyDomainDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
): Pf2eApplicationOntologyService {
  const searchSemanticsDomainPromiseCache = new Map<SearchFilterDiscoveryMode, Promise<OntologyDomainModel>>();
  const searchFilterExplorerPromiseCache = new Map<string, Promise<OntologyDomainModel>>();

  function buildSearchFilterExplorerCacheKey(options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
    targetFields?: readonly string[];
  }): string {
    return `${options.discoveryMode}|${JSON.stringify(options.request)}|${[...(options.targetFields ?? [])].sort().join(",")}`;
  }

  const loadSearchSemanticsDomain = (options: {
    discoveryMode: SearchFilterDiscoveryMode;
  }): Promise<OntologyDomainModel> => {
    const cached = searchSemanticsDomainPromiseCache.get(options.discoveryMode);
    if (cached) {
      return cached;
    }

    const promise = Promise.resolve().then(() =>
      buildSearchSemanticsDomain(config, dataService, discoveryService, options),
    );
    searchSemanticsDomainPromiseCache.set(options.discoveryMode, promise);
    void promise.catch(() => {
      searchSemanticsDomainPromiseCache.delete(options.discoveryMode);
    });
    return promise;
  };

  const loadSearchFilterExplorerDomain = (options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
    targetFields?: readonly string[];
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
