import type { Pf2eDataService } from "../../data/service.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { MetadataFieldName, MetadataFieldType } from "../../domain/metadata-field-types.js";
import type {
  SearchFilterDiscoveryApplicability,
  SearchFilterDiscoveryMode,
  SearchFilterDiscoveryOption,
  SearchFilterDiscoveryRequest,
  SearchFilterDiscoveryResult,
  SearchPromotedFieldDomainKey,
} from "../../domain/search-field-domains.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";

export type SearchDiscoveryDataService = Pick<
  Pf2eDataService,
  "discoverFilterValues" | "getPack" | "listFilterValues"
> &
  Partial<Pick<Pf2eDataService, "listMetricCatalogKeys" | "listMetricCatalogValues">>;

export type SearchDiscoveryScope = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
};

export type SearchDiscoveryField = {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  discoverable: boolean;
  notes?: string;
  subcategories?: readonly SearchSubcategory[];
  valueOrdering?: FilterValueOrdering;
};

export type SearchDiscoveryMetricGroup = {
  metricField: "actorMetrics" | "itemMetrics";
  metadataField: "actorMetric" | "itemMetric";
  namespaces: ReadonlyArray<{ prefix: string; description: string }>;
};

export type SearchSemanticsDiscoveryReader = {
  scope: SearchDiscoveryScope | null;
  mode: SearchFilterDiscoveryMode;
  discoverFieldValues: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    field: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverFieldValuesAsync: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    field: string;
  }) => Promise<readonly SearchFilterDiscoveryOption[]>;
  discoverMetricKeys: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }) => Promise<readonly SearchFilterDiscoveryOption[]>;
  discoverMetricValues: (options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }) => Promise<readonly SearchFilterDiscoveryOption[]>;
};

export type Pf2eApplicationSearchDiscoveryService = {
  discoverFilterValues: (request: SearchFilterDiscoveryRequest) => Promise<SearchFilterDiscoveryResult>;
  discoverCatalogFilterValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    target: { field: string };
  }) => SearchFilterDiscoveryResult;
  discoverMetricKeys: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverMetricValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }) => readonly SearchFilterDiscoveryOption[];
  getMetricDiscoveryGroups: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryMetricGroup[];
  getScopedMetadataFields: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryField[];
  isPromotedFieldAvailable: (
    field: SearchPromotedFieldDomainKey,
    applicability: SearchFilterDiscoveryApplicability,
  ) => boolean;
  createCatalogSearchSemanticsReader: () => SearchSemanticsDiscoveryReader;
  prepareSearchSemanticsReader: (
    request: Readonly<SearchRequest>,
    mode: SearchFilterDiscoveryMode,
    options?: { targetFields?: readonly string[] },
  ) => Promise<SearchSemanticsDiscoveryReader>;
  resolvePackName: (packValue: string) => string | undefined;
};
