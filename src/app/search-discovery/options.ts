import type { SearchFilterDiscoveryOption } from "../../domain/search-field-domains.js";

export function normalizeDiscoveryTargetField(field: string): string {
  if (field === "actorMetric" || field === "actorMetricCompare") {
    return "actorMetrics";
  }
  if (field === "itemMetric" || field === "itemMetricCompare") {
    return "itemMetrics";
  }
  if (field === "pack") {
    return "packs";
  }
  return field;
}

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
