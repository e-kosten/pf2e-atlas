import type { MetadataFieldSemantics } from "../../../domain/metadata-field-catalog.js";
import type { MetadataGlossaryArtifact, MetadataGlossaryEntry } from "../../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import { buildFilterText, buildKeyValueDetailLines, titleCaseLabel } from "../node-helpers.js";
import { formatSearchSemanticsMetricLabel } from "./labels.js";
import { buildMetadataValueQuery, buildMetricScalarMetadataQuery, buildValueScopedQuery } from "./query-builders.js";
import { buildQueryRecordChildSource } from "./record-child-sources.js";
import type { SearchSemanticsRecordsDataService, ValueNodeQueryOptions } from "./types.js";

export function getTraitGlossaryEntry(
  metadataGlossary: MetadataGlossaryArtifact | null,
  value: string,
): MetadataGlossaryEntry | undefined {
  return metadataGlossary?.fields.traits?.[value];
}

function buildTraitDetailLines(
  category: SearchCategory,
  value: string,
  liveRecordCount: number,
  metadataGlossary: MetadataGlossaryArtifact | null,
  countLabel = "Live canonical records",
): OntologyNode["detailLines"] {
  const glossaryEntry = getTraitGlossaryEntry(metadataGlossary, value);
  return [
    { text: glossaryEntry?.label ?? titleCaseLabel(value), tone: "section" },
    ...(glossaryEntry?.description ? [{ text: glossaryEntry.description }] : []),
    { text: `Trait: ${value}` },
    { text: `Category: ${category}` },
    { text: `${countLabel}: ${liveRecordCount}` },
  ];
}

export function buildResultReaderHint(): string {
  return "Press Enter or o to open the full matching set in the shared result reader.";
}

function getPackPresentationLabel(recordsService: SearchSemanticsRecordsDataService, packValue: string): string {
  const pack = recordsService.getPack?.(packValue);
  return pack?.label ?? pack?.name ?? packValue;
}

export function buildFieldValueNodes(
  dataService: SearchSemanticsRecordsDataService,
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  fieldSemantics: Pick<MetadataFieldSemantics, "field" | "fieldType">,
  values: ReadonlyArray<{ value: string; count: number }>,
  metadataGlossary: MetadataGlossaryArtifact | null,
  options: ValueNodeQueryOptions = {},
): readonly OntologyNode[] {
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const countLabel = options.countLabel ?? "Live canonical records";
  return values.map((entry): OntologyNode => {
    const metadata = buildMetadataValueQuery(fieldSemantics, entry.value);
    const traitGlossaryEntry =
      fieldSemantics.field === "traits" ? getTraitGlossaryEntry(metadataGlossary, entry.value) : undefined;
    const query = metadata
      ? buildValueScopedQuery(
          category,
          subcategory,
          fieldSemantics.field === "traits" ? "Browse records with this trait" : "Browse records with this value",
          metadata,
          options.matchingRequest,
        )
      : undefined;
    return {
      id: `${idPrefix}:${fieldSemantics.field}:${entry.value}`,
      kind: "value",
      label: traitGlossaryEntry?.label ?? entry.value,
      filterText: buildFilterText(
        category,
        subcategory ?? "",
        fieldSemantics.field,
        entry.value,
        traitGlossaryEntry?.label ?? "",
        traitGlossaryEntry?.description ?? "",
      ),
      listLabel: `${traitGlossaryEntry?.label ?? entry.value} | ${entry.count}`,
      detailTitle: fieldSemantics.field === "traits" ? "Trait Details" : "Filter Value",
      detailLines:
        fieldSemantics.field === "traits"
          ? [
              ...buildTraitDetailLines(category, entry.value, entry.count, metadataGlossary, countLabel),
              { text: buildResultReaderHint() },
            ]
          : buildKeyValueDetailLines(
              entry.value,
              [
                ["Category", category],
                ["Subcategory", subcategory ?? "(all)"],
                ["Field", fieldSemantics.field],
                ["Value", entry.value],
                [countLabel, entry.count],
              ],
              buildResultReaderHint(),
            ),
      query,
      childSource: query ? buildQueryRecordChildSource(dataService, query) : undefined,
    };
  });
}

export function buildPackValueNodes(
  dataService: SearchSemanticsRecordsDataService,
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  values: ReadonlyArray<{ value: string; count: number }>,
  options: ValueNodeQueryOptions = {},
): readonly OntologyNode[] {
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const countLabel = options.countLabel ?? "Live canonical records";

  return values.map((entry): OntologyNode => {
    const packLabel = getPackPresentationLabel(dataService, entry.value);
    const query = buildValueScopedQuery(
      category,
      subcategory,
      `Browse records from ${packLabel}`,
      {
        kind: "pack",
        value: entry.value,
      },
      options.matchingRequest,
    );

    return {
      id: `${idPrefix}:pack:${entry.value}`,
      kind: "value",
      label: packLabel,
      filterText: buildFilterText(category, subcategory ?? "", "pack", entry.value, packLabel),
      listLabel: `${packLabel} | ${entry.count}`,
      detailTitle: "Pack Details",
      detailLines: buildKeyValueDetailLines(
        packLabel,
        [
          ["Category", category],
          ["Subcategory", subcategory ?? "(all)"],
          ["Pack", entry.value],
          [countLabel, entry.count],
        ],
        buildResultReaderHint(),
      ),
      query,
      childSource: buildQueryRecordChildSource(dataService, query),
    };
  });
}

export function buildMetricValueNodes(
  recordsService: SearchSemanticsRecordsDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    values: ReadonlyArray<{ value: string; count: number }>;
    valueType: "text" | "boolean";
    countLabel?: string;
    matchingRequest?: ValueNodeQueryOptions["matchingRequest"];
  },
): readonly OntologyNode[] {
  const { category, subcategory, groupLabel, metricField, metadataField, metricKey, values, valueType } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const metricLabel = formatSearchSemanticsMetricLabel(metricKey);
  const countLabel = options.countLabel ?? "Live canonical records";
  return values.map((entry) => {
    const metadata = buildMetricScalarMetadataQuery(metadataField, metricKey, valueType, entry.value);
    const query = buildValueScopedQuery(
      category,
      subcategory,
      `Browse records where ${metricLabel} ${valueType === "boolean" ? "is" : "="} ${entry.value}`,
      metadata,
      options.matchingRequest,
    );
    return {
      id: `${idPrefix}:${metricField}:${metricKey}:${entry.value}`,
      kind: "value",
      label: entry.value,
      filterText: buildFilterText(
        category,
        subcategory ?? "",
        groupLabel,
        metricField,
        metricKey,
        metricLabel,
        entry.value,
      ),
      listLabel: `${entry.value} | ${entry.count}`,
      detailTitle: "Metric Value",
      detailLines: buildKeyValueDetailLines(
        entry.value,
        [
          ["Category", category],
          ["Subcategory", subcategory ?? "(all)"],
          ["Explorer group", groupLabel],
          ["Metric", metricLabel],
          ["Metric key", metricKey],
          ["Value type", valueType],
          [countLabel, entry.count],
        ],
        buildResultReaderHint(),
      ),
      query,
      childSource: buildQueryRecordChildSource(recordsService, query),
    };
  });
}
