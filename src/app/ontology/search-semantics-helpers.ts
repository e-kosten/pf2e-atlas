import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import type { Pf2eDataService } from "../../data/service.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../domain/metadata-types.js";
import type { MetadataGlossaryArtifact, MetadataGlossaryEntry } from "../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { normalizeText } from "../../shared/utils.js";
import {
  buildFilterText,
  buildKeyValueDetailLines,
  buildNormalizedRecordNode,
  cloneMetadataFilterNode,
  titleCaseLabel,
} from "./node-helpers.js";

type SearchSemanticsRecordsDataService = Pick<Pf2eDataService, "listRecords">;
type SearchSemanticsDiscoveryDataService = Pick<Pf2eDataService, "listFilterValues" | "listRecords">;

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
): OntologyNode["detailLines"] {
  const glossaryEntry = getTraitGlossaryEntry(metadataGlossary, value);
  return [
    { text: glossaryEntry?.label ?? titleCaseLabel(value), tone: "section" },
    ...(glossaryEntry?.description ? [{ text: glossaryEntry.description }] : []),
    { text: `Trait: ${value}` },
    { text: `Category: ${category}` },
    { text: `Live canonical records: ${liveRecordCount}` },
  ];
}

function buildResultReaderHint(): string {
  return "Press Enter or o to open the full matching set in the shared result reader.";
}

export function buildSearchSemanticsMetadataQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  label: string,
  metadata: MetadataFilterNode,
): OntologyNode["query"] {
  return {
    kind: "listRecords",
    label,
    filters: {
      category,
      subcategory: subcategory ?? undefined,
      metadata: cloneMetadataFilterNode(metadata),
      limit: 20,
    },
  };
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

function buildMetadataValueQuery(
  fieldSemantics: Pick<MetadataFieldSemantics, "field" | "fieldType">,
  value: string,
): MetadataFilterNode | undefined {
  switch (fieldSemantics.fieldType) {
    case "set":
      return {
        field: fieldSemantics.field as MetadataSetField,
        op: "includesAny",
        values: [value],
      };
    case "enumString":
      return {
        field: fieldSemantics.field as MetadataEnumStringField,
        op: "eq",
        value,
      };
    case "text":
      return {
        field: fieldSemantics.field as MetadataTextStringField,
        op: "eq",
        value,
      };
    case "number": {
      const numericValue = Number(value);
      return Number.isFinite(numericValue)
        ? {
            field: fieldSemantics.field as MetadataNumberField,
            op: "eq",
            value: numericValue,
          }
        : undefined;
    }
    case "boolean": {
      const booleanValue = parseOntologyBooleanValue(value);
      return booleanValue === undefined
        ? undefined
        : {
            field: fieldSemantics.field as MetadataBooleanField,
            op: "eq",
            value: booleanValue,
          };
    }
  }
}

function buildMetricScalarMetadataQuery(
  field: "actorMetric" | "itemMetric",
  metric: string,
  valueType: "text" | "boolean",
  value: string,
): MetadataFilterNode {
  return {
    field,
    metric,
    op: "==",
    value: valueType === "boolean" ? value === "true" : value,
  };
}

function buildQueryRecordChildren(
  dataService: SearchSemanticsRecordsDataService,
  query: OntologyNode["query"] | undefined,
): readonly OntologyNode[] {
  if (!query || query.kind !== "listRecords") {
    return [];
  }

  return dataService.listRecords(query.filters).records.map(buildNormalizedRecordNode);
}

export function buildFieldValueNodes(
  dataService: SearchSemanticsRecordsDataService,
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  fieldSemantics: MetadataFieldSemantics,
  values: ReadonlyArray<{ value: string; count: number }>,
  metadataGlossary: MetadataGlossaryArtifact | null,
): readonly OntologyNode[] {
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return values.map((entry): OntologyNode => {
    const metadata = buildMetadataValueQuery(fieldSemantics, entry.value);
    const traitGlossaryEntry =
      fieldSemantics.field === "traits" ? getTraitGlossaryEntry(metadataGlossary, entry.value) : undefined;
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
              ...buildTraitDetailLines(category, entry.value, entry.count, metadataGlossary),
              { text: buildResultReaderHint() },
            ]
          : buildKeyValueDetailLines(
              entry.value,
              [
                ["Category", category],
                ["Subcategory", subcategory ?? "(all)"],
                ["Field", fieldSemantics.field],
                ["Value", entry.value],
                ["Live canonical records", entry.count],
              ],
              buildResultReaderHint(),
            ),
      query: metadata
        ? buildSearchSemanticsMetadataQuery(
            category,
            subcategory,
            fieldSemantics.field === "traits" ? "Browse records with this trait" : "Browse records with this value",
            metadata,
          )
        : undefined,
      loadChildren: metadata
        ? () =>
            buildQueryRecordChildren(
              dataService,
              buildSearchSemanticsMetadataQuery(
                category,
                subcategory,
                fieldSemantics.field === "traits" ? "Browse records with this trait" : "Browse records with this value",
                metadata,
              ),
            )
        : undefined,
    };
  });
}

function buildMetricValueNodes(
  dataService: SearchSemanticsDiscoveryDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    values: ReadonlyArray<{ value: string; count: number }>;
    valueType: "text" | "boolean";
  },
): readonly OntologyNode[] {
  const { category, subcategory, metricField, metadataField, metricKey, values, valueType } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return values.map((entry) => {
    const metadata = buildMetricScalarMetadataQuery(metadataField, metricKey, valueType, entry.value);
    const query = buildSearchSemanticsMetadataQuery(
      category,
      subcategory,
      `Browse records where ${metricKey} ${valueType === "boolean" ? "is" : "="} ${entry.value}`,
      metadata,
    );
    return {
      id: `${idPrefix}:${metricField}:${metricKey}:${entry.value}`,
      kind: "value",
      label: entry.value,
      filterText: buildFilterText(category, subcategory ?? "", metricField, metricKey, entry.value),
      listLabel: `${entry.value} | ${entry.count}`,
      detailTitle: "Metric Value",
      detailLines: buildKeyValueDetailLines(
        entry.value,
        [
          ["Category", category],
          ["Subcategory", subcategory ?? "(all)"],
          ["Metric field", metricField],
          ["Metric key", metricKey],
          ["Value type", valueType],
          ["Live canonical records", entry.count],
        ],
        buildResultReaderHint(),
      ),
      query,
      loadChildren: () => buildQueryRecordChildren(dataService, query),
    };
  });
}

function buildMetricKeyNode(
  dataService: SearchSemanticsDiscoveryDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    liveRecordCount: number;
  },
): OntologyNode {
  const { category, subcategory, metricField, metadataField, metricKey, liveRecordCount } = options;
  const valueType =
    metricField === "actorMetrics" ? inferActorMetricValueType(metricKey) : inferItemMetricValueType(metricKey);
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const browseQuery =
    valueType === "text" || valueType === "boolean"
      ? undefined
      : {
          kind: "listRecords" as const,
          label: `Browse ${category} records`,
          filters: {
            category,
            subcategory: subcategory ?? undefined,
            limit: 20,
          },
        };

  return {
    id: `${idPrefix}:${metricField}:${metricKey}`,
    kind: "metric",
    label: metricKey,
    filterText: buildFilterText(category, subcategory ?? "", metricField, metricKey, valueType ?? ""),
    listLabel: `${metricKey} | ${liveRecordCount}`,
    detailTitle: "Metric Details",
    detailLines: [
      { text: metricKey, tone: "section" },
      { text: `Category: ${category}` },
      { text: `Subcategory: ${subcategory ?? "(all)"}` },
      { text: `Metric field: ${metricField}` },
      { text: `Value type: ${valueType ?? "unknown"}` },
      { text: `Live canonical records: ${liveRecordCount}` },
      ...(valueType === "text" || valueType === "boolean"
        ? [
            {
              text: "Drill in to browse exact live scalar values for this metric, then inspect matching records in the shared result reader.",
            },
          ]
        : [
            {
              text: "Use the structured query editor for numeric comparisons on this metric. Live scalar enumeration is only available for text and boolean metrics.",
            },
          ]),
    ],
    query: browseQuery,
    loadChildren:
      valueType === "text" || valueType === "boolean"
        ? () =>
            buildMetricValueNodes(dataService, {
              category,
              subcategory,
              metricField,
              metadataField,
              metricKey,
              valueType,
              values: dataService.listFilterValues({
                field: metricField,
                category,
                subcategory: subcategory ?? undefined,
                metric: metricKey,
              }).values,
            })
        : () => buildQueryRecordChildren(dataService, browseQuery),
  };
}

function buildMetricNamespaceNode(
  dataService: SearchSemanticsDiscoveryDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    prefix: string;
    description: string;
  },
): OntologyNode {
  const { category, subcategory, metricField, metadataField, prefix, description } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return {
    id: `${idPrefix}:${metricField}:namespace:${prefix}`,
    kind: "metricNamespace",
    label: prefix,
    filterText: buildFilterText(category, subcategory ?? "", metricField, prefix, description),
    listLabel: prefix,
    detailTitle: "Metric Namespace",
    detailLines: [
      { text: prefix, tone: "section" },
      { text: description },
      { text: `Category: ${category}` },
      { text: `Subcategory: ${subcategory ?? "(all)"}` },
      { text: `Metric field: ${metricField}` },
      {
        text: "Drill in to browse live metric keys, then inspect exact scalar values where the backend can enumerate them.",
      },
    ],
    loadChildren: () =>
      dataService
        .listFilterValues({
          field: metricField,
          category,
          subcategory: subcategory ?? undefined,
          metricPrefix: prefix,
        })
        .values.map((entry) =>
          buildMetricKeyNode(dataService, {
            category,
            subcategory,
            metricField,
            metadataField,
            metricKey: entry.value,
            liveRecordCount: entry.count,
          }),
        ),
  };
}

export function buildMetricDiscoveryGroup(
  dataService: SearchSemanticsDiscoveryDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    label: "Actor Metrics" | "Item Metrics";
    namespaces: ReadonlyArray<{ prefix: string; description: string }>;
  },
): OntologyNode {
  const { category, subcategory, metricField, metadataField, label, namespaces } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return {
    id: `${idPrefix}:${metricField}:discovery`,
    kind: "group",
    label,
    filterText: buildFilterText(category, subcategory ?? "", label, ...namespaces.map((entry) => entry.prefix)),
    listLabel: `${label} | ${namespaces.length} namespaces`,
    detailTitle: label,
    detailLines: buildKeyValueDetailLines(
      label,
      [
        ["Category", category],
        ["Subcategory", subcategory ?? "(all)"],
        ["Namespaces", namespaces.length],
      ],
      "Explore live metric namespaces, keys, and exact scalar values from the indexed corpus.",
    ),
    children: namespaces.map((namespace) =>
      buildMetricNamespaceNode(dataService, {
        category,
        subcategory,
        metricField,
        metadataField,
        prefix: namespace.prefix,
        description: namespace.description,
      }),
    ),
  };
}
