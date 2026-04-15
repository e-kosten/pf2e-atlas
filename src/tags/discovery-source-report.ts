type SourceReportFields = {
  topSources: string[];
  topPublications: string[];
  topSourceSlices: string[];
  sourceScope: "source-slice" | "publication" | "pack" | null;
  dominantSourceShare: number;
};

function formatList(values: string[]): string {
  return values.join(", ") || "(none)";
}

export function formatDiscoverySourceContext(fields: SourceReportFields): string {
  const share = `${(fields.dominantSourceShare * 100).toFixed(1)}%`;
  return `source_scope=${fields.sourceScope ?? "(none)"} dominant_share=${share} top_source_slices=${formatList(fields.topSourceSlices)} top_publications=${formatList(fields.topPublications)} top_sources=${formatList(fields.topSources)}`;
}
