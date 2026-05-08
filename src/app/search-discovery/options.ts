import type { SearchFilterDiscoveryOption } from "../../domain/search-field-domains.js";
import { normalizeSearchDiscoveryTargetField } from "../../domain/search-field-domains.js";

export function mapFilterValueDiscoveryOption(entry: {
  value: string;
  count: number;
  valueType?: "number" | "text" | "boolean";
  numericMin?: number | null;
  numericMax?: number | null;
}): SearchFilterDiscoveryOption {
  return {
    id: entry.value,
    value: entry.value,
    count: entry.count,
    ...(entry.valueType ? { valueType: entry.valueType } : {}),
    ...(entry.numericMin !== undefined ? { numericMin: entry.numericMin } : {}),
    ...(entry.numericMax !== undefined ? { numericMax: entry.numericMax } : {}),
  };
}

export const normalizeDiscoveryTargetField = normalizeSearchDiscoveryTargetField;
