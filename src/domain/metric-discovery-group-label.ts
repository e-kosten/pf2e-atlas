import type { SearchCategory } from "./search-types.js";

export function getMetricDiscoveryGroupLabel(
  category: SearchCategory,
  metricField: "actorMetrics" | "itemMetrics",
): string {
  if (metricField === "actorMetrics") {
    return category === "hazard" ? "Hazard Statistics" : "Creature Statistics";
  }

  return "Item Properties";
}
