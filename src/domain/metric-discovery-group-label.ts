import type { SearchCategory } from "./search-types.js";

export function getMetricDiscoveryGroupLabel(
  category: SearchCategory | null | undefined,
  metricField: "actorMetrics" | "itemMetrics",
): string {
  if (metricField === "actorMetrics") {
    if (category === "hazard") {
      return "Hazard Statistics";
    }
    return "Creature Statistics";
  }

  return "Item Properties";
}

export function getMetricQueryFieldLabel(
  field: "actorMetric" | "actorMetricCompare" | "itemMetric" | "itemMetricCompare",
  category: SearchCategory | null | undefined,
): string {
  return field === "actorMetric" || field === "actorMetricCompare"
    ? getMetricDiscoveryGroupLabel(category, "actorMetrics")
    : getMetricDiscoveryGroupLabel(category, "itemMetrics");
}
