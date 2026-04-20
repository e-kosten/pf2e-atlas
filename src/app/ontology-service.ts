import type { AppConfig } from "../domain/config-types.js";
import type { OntologyDomainModel } from "../domain/ontology-types.js";
import type { Pf2eDataService } from "../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult, SearchVocabularyResult } from "../data/vocabulary.js";
import { buildDerivedTagsDomain } from "./ontology/derived-tags-domain.js";
import { buildSearchSemanticsDomain } from "./ontology/search-semantics-domain.js";
import { createPf2eApplicationStorageService, type Pf2eApplicationStorageService } from "./storage-service.js";

export type Pf2eApplicationOntologyService = {
  loadSearchSemanticsDomain: () => OntologyDomainModel;
};

type OntologyDomainDataService = Pick<Pf2eDataService, "listFilterValues" | "listRecords"> & {
  getSearchSemanticsBootstrapSummary?: (options?: { traitLimitPerCategory?: number }) => SearchSemanticsBootstrapSummaryResult;
  getSearchVocabulary?: (options?: { traitLimitPerCategory?: number }) => SearchVocabularyResult;
};

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: OntologyDomainDataService,
  storage: Pick<
    Pf2eApplicationStorageService,
    "loadDerivedTagOntologyExplorerModel"
  > = createPf2eApplicationStorageService(config),
): Pf2eApplicationOntologyService {
  let derivedTagsDomain: OntologyDomainModel | null = null;
  let searchSemanticsDomain: OntologyDomainModel | null = null;

  const loadDerivedTagsDomain = (): OntologyDomainModel => {
    if (!derivedTagsDomain) {
      derivedTagsDomain = buildDerivedTagsDomain(storage.loadDerivedTagOntologyExplorerModel());
    }
    return derivedTagsDomain;
  };

  const loadSearchSemanticsDomain = (): OntologyDomainModel => {
    if (!searchSemanticsDomain) {
      searchSemanticsDomain = buildSearchSemanticsDomain(config, dataService, loadDerivedTagsDomain);
    }
    return searchSemanticsDomain;
  };

  return {
    loadSearchSemanticsDomain,
  };
}
