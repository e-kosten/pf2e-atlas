import { getMetadataFilterSemantics } from "../../domain/metadata-field-catalog.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { MetadataFieldName } from "../../domain/metadata-field-types.js";
import { buildDiscoveryCacheKey } from "./cache-keys.js";
import type { SearchDiscoveryField, SearchDiscoveryScope } from "./types.js";

export type SearchDiscoveryMetadataFieldRegistry = {
  getFieldValueOrdering: (field: string) => FilterValueOrdering | undefined;
  getScopedMetadataFields: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryField[];
};

export function createSearchDiscoveryMetadataFieldRegistry(): SearchDiscoveryMetadataFieldRegistry {
  const semantics = getMetadataFilterSemantics();
  const metadataFieldsByName = new Map<MetadataFieldName, SearchDiscoveryField>(
    semantics.metadataFields.map((field) => [
      field.field,
      {
        field: field.field,
        fieldType: field.fieldType,
        discoverable: field.discoverable,
        notes: field.notes,
        subcategories: field.subcategories,
        valueOrdering: field.valueOrdering,
      },
    ]),
  );
  const scopedFieldCache = new Map<string, readonly SearchDiscoveryField[]>();

  return {
    getFieldValueOrdering: (field) => metadataFieldsByName.get(field as MetadataFieldName)?.valueOrdering,
    getScopedMetadataFields: (scope) => {
      if (!scope) {
        return [];
      }

      const cacheKey = buildDiscoveryCacheKey([scope.category, scope.subcategory ?? "all"]);
      const cached = scopedFieldCache.get(cacheKey);
      if (cached) {
        return cached;
      }

      const fields = semantics.metadataFieldsByCategory[scope.category]
        .map((field) => metadataFieldsByName.get(field))
        .filter((field): field is SearchDiscoveryField => Boolean(field))
        .filter(
          (field) => !scope.subcategory || !field.subcategories || field.subcategories.includes(scope.subcategory),
        );
      scopedFieldCache.set(cacheKey, fields);
      return fields;
    },
  };
}
