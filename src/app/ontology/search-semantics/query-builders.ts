import type { MetadataFieldSemantics } from "../../../domain/metadata-field-catalog.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../../domain/metadata-field-types.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import { buildAllOfFilter, buildScopeFilter, type SearchFilterNode } from "../../../domain/search-request-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import { normalizeText } from "../../../shared/utils.js";

export function buildSearchSemanticsMetadataQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  label: string,
  filter: SearchFilterNode,
): NonNullable<OntologyNode["query"]> {
  return {
    label,
    request: {
      mode: "browse",
      filter: buildAllOfFilter([buildScopeFilter(category, subcategory), filter]),
      limit: 20,
    },
  };
}

export function buildSearchSemanticsScopeQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  label: string,
): NonNullable<OntologyNode["query"]> {
  return {
    label,
    request: {
      mode: "browse",
      filter: buildScopeFilter(category, subcategory),
      limit: 20,
    },
  };
}

export function buildValueScopedQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  label: string,
  valueFilter: SearchFilterNode,
  matchingRequest: Readonly<SearchRequest> | undefined,
): NonNullable<OntologyNode["query"]> {
  if (!matchingRequest) {
    return buildSearchSemanticsMetadataQuery(category, subcategory, label, valueFilter);
  }

  const { offset: _offset, ...requestWithoutOffset } = matchingRequest;
  return {
    label,
    request: {
      ...requestWithoutOffset,
      filter: buildAllOfFilter([matchingRequest.filter, valueFilter]),
      limit: 20,
    } as SearchRequest,
  };
}

export function buildDerivedTagQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  tag: string,
  label: string,
): NonNullable<OntologyNode["query"]> {
  return buildSearchSemanticsMetadataQuery(category, subcategory, label, {
    kind: "metadataPredicate",
    predicate: {
      field: "derivedTags",
      op: "includes",
      value: tag,
    },
  });
}

export function buildPackValueQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  label: string,
  packValue: string,
  matchingRequest: Readonly<SearchRequest> | undefined,
): NonNullable<OntologyNode["query"]> {
  return buildValueScopedQuery(
    category,
    subcategory,
    label,
    {
      kind: "pack",
      value: packValue,
    },
    matchingRequest,
  );
}

export function buildFieldValueQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  fieldSemantics: Pick<MetadataFieldSemantics, "field" | "fieldType">,
  value: string,
  label: string,
  matchingRequest: Readonly<SearchRequest> | undefined,
): NonNullable<OntologyNode["query"]> | undefined {
  const valueFilter = buildMetadataValueQuery(fieldSemantics, value);
  return valueFilter ? buildValueScopedQuery(category, subcategory, label, valueFilter, matchingRequest) : undefined;
}

export function buildMetricValueQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  metricKey: string,
  valueType: "text" | "boolean",
  value: string,
  label: string,
  matchingRequest: Readonly<SearchRequest> | undefined,
): NonNullable<OntologyNode["query"]> {
  return buildValueScopedQuery(
    category,
    subcategory,
    label,
    buildMetricScalarMetadataQuery(metricKey, valueType, value),
    matchingRequest,
  );
}

export function buildMetricInspectQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  metricKey: string,
  metricLabel: string,
): OntologyNode["query"] {
  return buildSearchSemanticsMetadataQuery(category, subcategory, `Browse records with ${metricLabel}`, {
    kind: "metricCompare",
    leftMetric: metricKey,
    op: "gte",
    rightMetric: metricKey,
  });
}

function parseOntologyBooleanValue(value: string): boolean | undefined {
  switch (normalizeText(value)) {
    case "true":
    case "yes":
    case "1":
      return true;
    case "false":
    case "no":
    case "0":
      return false;
    default:
      return undefined;
  }
}

export function buildMetadataValueQuery(
  fieldSemantics: Pick<MetadataFieldSemantics, "field" | "fieldType">,
  value: string,
): SearchFilterNode | undefined {
  switch (fieldSemantics.fieldType) {
    case "set":
      return {
        kind: "metadataPredicate",
        predicate: {
          field: fieldSemantics.field as MetadataSetField,
          op: "includes",
          value,
        },
      };
    case "enumString":
      return {
        kind: "metadataPredicate",
        predicate: {
          field: fieldSemantics.field as MetadataEnumStringField,
          op: "eq",
          value,
        },
      };
    case "text":
      return {
        kind: "metadataPredicate",
        predicate: {
          field: fieldSemantics.field as MetadataTextStringField,
          op: "eq",
          value,
        },
      };
    case "number": {
      const numericValue = Number(value);
      return Number.isFinite(numericValue)
        ? {
            kind: "metadataPredicate",
            predicate: {
              field: fieldSemantics.field as MetadataNumberField,
              op: "eq",
              value: numericValue,
            },
          }
        : undefined;
    }
    case "boolean": {
      const booleanValue = parseOntologyBooleanValue(value);
      return booleanValue === undefined
        ? undefined
        : {
            kind: "metadataPredicate",
            predicate: {
              field: fieldSemantics.field as MetadataBooleanField,
              op: "eq",
              value: booleanValue,
            },
          };
    }
  }
}

function buildMetricScalarMetadataQuery(
  metric: string,
  valueType: "text" | "boolean",
  value: string,
): SearchFilterNode {
  return {
    kind: "metric",
    metric,
    op: "eq",
    value: valueType === "boolean" ? value === "true" : value,
  };
}
