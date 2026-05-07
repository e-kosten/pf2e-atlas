import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type { SearchSemanticsDiscoveryReader } from "../../search-discovery/service.js";
import { buildFilterText, buildKeyValueDetailLines } from "../node-helpers.js";
import { formatSearchSemanticsMetricLabel } from "./labels.js";
import { buildMetricInspectQuery } from "./query-builders.js";
import type { SearchSemanticsRecordsDataService } from "./types.js";
import { buildMetricValueNodes } from "./value-node-builder.js";

function buildMetricKeyNode(
  recordsService: SearchSemanticsRecordsDataService,
  discoveryReader: SearchSemanticsDiscoveryReader,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    liveRecordCount: number;
    numericMin?: number | null;
    numericMax?: number | null;
    countLabel?: string;
    matchingRequest?: Readonly<SearchRequest>;
  },
): OntologyNode {
  const { category, subcategory, groupLabel, metricField, metadataField, metricKey, liveRecordCount } = options;
  const valueType =
    metricField === "actorMetrics" ? inferActorMetricValueType(metricKey) : inferItemMetricValueType(metricKey);
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const metricLabel = formatSearchSemanticsMetricLabel(metricKey);
  const countLabel = options.countLabel ?? "Live canonical records";
  const inspectQuery =
    valueType === "number" ? buildMetricInspectQuery(category, subcategory, metricKey, metricLabel) : undefined;

  return {
    id: `${idPrefix}:${metricField}:${metricKey}`,
    kind: "metric",
    label: metricLabel,
    shortLabel: metricKey,
    filterText: buildFilterText(
      category,
      subcategory ?? "",
      groupLabel,
      metricField,
      metricKey,
      metricLabel,
      valueType ?? "",
    ),
    listLabel: `${metricLabel} | ${liveRecordCount}`,
    detailTitle: "Metric Details",
    detailLines: [
      { text: metricLabel, tone: "section" },
      { text: `Category: ${category}` },
      { text: `Subcategory: ${subcategory ?? "(all)"}` },
      { text: `Explorer group: ${groupLabel}` },
      { text: `Metric key: ${metricKey}` },
      { text: `Value type: ${valueType ?? "unknown"}` },
      ...(valueType === "number" && (options.numericMin !== undefined || options.numericMax !== undefined)
        ? [{ text: `Catalog range: ${options.numericMin ?? "?"} to ${options.numericMax ?? "?"}` }]
        : []),
      { text: `${countLabel}: ${liveRecordCount}` },
      ...(valueType === "text" || valueType === "boolean"
        ? [
            {
              text: "Drill in to browse exact live scalar values for this metric, then inspect matching records in the shared result reader.",
            },
          ]
        : [
            {
              text: "Use the shared filter explorer to author numeric literal filters on this metric. Inspect mode will open the exact live records for this metric key, while scalar enumeration stays limited to text and boolean metrics.",
            },
          ]),
    ],
    query: inspectQuery,
    childSource:
      valueType === "text" || valueType === "boolean"
        ? {
            kind: "lazy",
            load: async () =>
              buildMetricValueNodes(recordsService, {
                category,
                subcategory,
                groupLabel,
                metricField,
                metadataField,
                metricKey,
                valueType,
                values: (
                  await discoveryReader.discoverMetricValues({
                    category,
                    subcategory,
                    metricField,
                    metricKey,
                  })
                ).map((entry) => ({
                  value: String(entry.value),
                  count: entry.count,
                })),
                countLabel,
                matchingRequest: options.matchingRequest,
              }),
          }
        : undefined,
  };
}

function buildMetricNamespaceNode(
  recordsService: SearchSemanticsRecordsDataService,
  discoveryReader: SearchSemanticsDiscoveryReader,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    prefix: string;
    description: string;
    countLabel?: string;
    matchingRequest?: Readonly<SearchRequest>;
  },
): OntologyNode {
  const { category, subcategory, groupLabel, metricField, metadataField, prefix, description } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return {
    id: `${idPrefix}:${metricField}:namespace:${prefix}`,
    kind: "metricNamespace",
    label: prefix,
    filterText: buildFilterText(category, subcategory ?? "", groupLabel, metricField, prefix, description),
    listLabel: prefix,
    detailTitle: "Metric Namespace",
    detailLines: [
      { text: prefix, tone: "section" },
      { text: description },
      { text: `Category: ${category}` },
      { text: `Subcategory: ${subcategory ?? "(all)"}` },
      { text: `Explorer group: ${groupLabel}` },
      {
        text: "Drill in to browse live metric keys, then inspect exact scalar values where the backend can enumerate them.",
      },
    ],
    childSource: {
      kind: "lazy",
      load: async () =>
        (
          await discoveryReader.discoverMetricKeys({
            category,
            subcategory,
            metricField,
            metricPrefix: prefix,
          })
        ).map((entry) =>
          buildMetricKeyNode(recordsService, discoveryReader, {
            category,
            subcategory,
            groupLabel,
            metricField,
            metadataField,
            metricKey: String(entry.value),
            liveRecordCount: entry.count,
            numericMin: entry.numericMin,
            numericMax: entry.numericMax,
            countLabel: options.countLabel,
            matchingRequest: options.matchingRequest,
          }),
        ),
    },
  };
}

export function buildMetricDiscoveryGroup(
  recordsService: SearchSemanticsRecordsDataService,
  discoveryReader: SearchSemanticsDiscoveryReader,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    label: string;
    namespaces: ReadonlyArray<{ prefix: string; description: string }>;
    countLabel?: string;
    matchingRequest?: Readonly<SearchRequest>;
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
      `Explore live ${label.toLowerCase()} namespaces, keys, and exact scalar values from the indexed corpus.`,
    ),
    childSource: {
      kind: "static",
      children: namespaces.map((namespace) =>
        buildMetricNamespaceNode(recordsService, discoveryReader, {
          category,
          subcategory,
          groupLabel: label,
          metricField,
          metadataField,
          prefix: namespace.prefix,
          description: namespace.description,
          countLabel: options.countLabel,
          matchingRequest: options.matchingRequest,
        }),
      ),
    },
  };
}
