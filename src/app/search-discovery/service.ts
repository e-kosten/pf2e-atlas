import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { createScopedSearchDiscoveryApplicability } from "./applicability.js";
import { createSearchDiscoveryMetadataFieldRegistry } from "./metadata-fields.js";
import { createSearchDiscoveryMetricCatalog } from "./metric-discovery.js";
import { createCatalogSearchSemanticsReader, prepareSearchSemanticsReader } from "./readers.js";
import { createSearchDiscoveryValueResolver } from "./value-discovery.js";
import type {
  Pf2eApplicationSearchDiscoveryService,
  SearchDiscoveryDataService,
  SearchDiscoveryField,
  SearchDiscoveryMetricGroup,
  SearchDiscoveryScope,
  SearchSemanticsDiscoveryReader,
} from "./types.js";

export { createScopedSearchDiscoveryApplicability };
export type {
  Pf2eApplicationSearchDiscoveryService,
  SearchDiscoveryField,
  SearchDiscoveryMetricGroup,
  SearchDiscoveryScope,
  SearchSemanticsDiscoveryReader,
};

export function createPf2eApplicationSearchDiscoveryService(
  dataService: SearchDiscoveryDataService,
): Pf2eApplicationSearchDiscoveryService {
  const metadataFields = createSearchDiscoveryMetadataFieldRegistry();
  const metricCatalog = createSearchDiscoveryMetricCatalog(dataService);
  const valueResolver = createSearchDiscoveryValueResolver({
    dataService,
    getFieldValueOrdering: metadataFields.getFieldValueOrdering,
  });

  return {
    discoverFilterValues: valueResolver.discoverFilterValues,
    discoverCatalogFilterValues: valueResolver.discoverCatalogFilterValues,
    discoverMetricKeys: metricCatalog.discoverMetricKeys,
    discoverMetricValues: metricCatalog.discoverMetricValues,
    getMetricDiscoveryGroups: metricCatalog.getMetricDiscoveryGroups,
    getScopedMetadataFields: metadataFields.getScopedMetadataFields,
    isPromotedFieldAvailable: (field, applicability) =>
      valueResolver.discoverCatalogFilterValues({
        applicability,
        target: { field },
      }).options.length > 0,
    createCatalogSearchSemanticsReader: () =>
      createCatalogSearchSemanticsReader({
        metricCatalog,
        valueResolver,
      }),
    prepareSearchSemanticsReader: (
      request: Readonly<SearchRequest>,
      mode: SearchFilterDiscoveryMode,
      options: { targetFields?: readonly string[] } = {},
    ) =>
      prepareSearchSemanticsReader({
        request,
        mode,
        targetFields: options.targetFields,
        discoverFieldOptions: valueResolver.discoverFieldOptions,
        getScopedMetadataFields: metadataFields.getScopedMetadataFields,
      }),
    resolvePackName: (packValue) => dataService.getPack(packValue)?.name,
  };
}
