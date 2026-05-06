import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { SearchSemanticsDiscoveryReader } from "../search-discovery-service.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../domain/metadata-field-types.js";
import type { MetadataGlossaryArtifact, MetadataGlossaryEntry } from "../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../domain/ontology-types.js";
import { buildAllOfFilter, buildScopeFilter, type SearchFilterNode } from "../../domain/search-request-types.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import type { SearchCategory, SearchResult, SearchSubcategory } from "../../domain/search-types.js";
import { normalizeText } from "../../shared/utils.js";
import {
  buildFilterText,
  buildKeyValueDetailLines,
  buildNormalizedRecordNode,
  titleCaseLabel,
} from "./node-helpers.js";

type SearchSemanticsRecordsDataService = {
  listRecords: (request: SearchRequest) => SearchResult;
  search?: (request: SearchRequest) => Promise<SearchResult>;
  getPack?: (packValue: string) => { name: string; label?: string } | undefined;
};

type ValueNodeQueryOptions = {
  countLabel?: string;
  matchingRequest?: Readonly<SearchRequest>;
};

const METRIC_SEGMENT_LABELS: Readonly<Record<string, string>> = {
  ac: "AC",
  ac_bonus: "AC Bonus",
  arcana: "Arcana",
  athletics: "Athletics",
  best: "Best",
  bt: "Broken Threshold",
  cha: "Charisma",
  check_penalty: "Check Penalty",
  con: "Constitution",
  crafting: "Crafting",
  damage_dice: "Damage Dice",
  damage_die_faces: "Damage Die Faces",
  dc: "DC",
  dex: "Dexterity",
  dex_cap: "Dex Cap",
  faces: "Faces",
  fly: "Fly",
  fort: "Fortitude",
  hardness: "Hardness",
  hp: "HP",
  int: "Intelligence",
  land: "Land",
  max: "Maximum",
  min: "Minimum",
  mod: "Modifier",
  perception: "Perception",
  proficient: "Proficient",
  range: "Range",
  range_increment: "Range Increment",
  rank: "Rank",
  ref: "Reflex",
  religion: "Religion",
  reload: "Reload",
  scent: "Scent",
  speed_penalty: "Speed Penalty",
  str: "Strength",
  strength: "Strength",
  thievery: "Thievery",
  value: "Value",
  will: "Will",
  wis: "Wisdom",
  worst: "Worst",
};

export function getTraitGlossaryEntry(
  metadataGlossary: MetadataGlossaryArtifact | null,
  value: string,
): MetadataGlossaryEntry | undefined {
  return metadataGlossary?.fields.traits?.[value];
}

export { getMetricDiscoveryGroupLabel };

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

function buildResultReaderHint(): string {
  return "Press Enter or o to open the full matching set in the shared result reader.";
}

function buildValueScopedQuery(
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

function humanizeMetricSegment(segment: string): string {
  return METRIC_SEGMENT_LABELS[segment] ?? titleCaseLabel(segment);
}

export function formatSearchSemanticsMetricLabel(metricKey: string): string {
  const segments = metricKey.split(".");
  const [first, second, third, fourth] = segments;

  if (first === "ability" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Modifier`;
  }
  if (first === "perception" && second === "mod") {
    return "Perception Modifier";
  }
  if (first === "ac" && second === "value") {
    return "Armor Class";
  }
  if (first === "hardness" && second === "value") {
    return "Hardness";
  }
  if (first === "hp" && second === "value") {
    return "Hit Points";
  }
  if (first === "hp" && second === "max") {
    return "Maximum Hit Points";
  }
  if (first === "hp" && second === "bt") {
    return "Broken Threshold";
  }
  if (first === "save" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Save Modifier`;
  }
  if (first === "save" && second === "best") {
    return "Best Save";
  }
  if (first === "save" && second === "worst") {
    return "Worst Save";
  }
  if (first === "skill" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Modifier`;
  }
  if (first === "skill" && second && third === "rank") {
    return `${humanizeMetricSegment(second)} Rank`;
  }
  if (first === "skill" && second && third === "proficient") {
    return `${humanizeMetricSegment(second)} Proficient`;
  }
  if (first === "stealth" && second === "mod") {
    return "Stealth Modifier";
  }
  if (first === "stealth" && second === "dc") {
    return "Stealth DC";
  }
  if (first === "speed" && second && third === "value") {
    return `${humanizeMetricSegment(second)} Speed`;
  }
  if (first === "sense" && second && third === "range") {
    return `${humanizeMetricSegment(second)} Range`;
  }
  if (first === "disable" && second === "dc" && third === "min") {
    return "Minimum Disable DC";
  }
  if (first === "disable" && second === "dc" && third === "max") {
    return "Maximum Disable DC";
  }
  if (first === "disable" && second && third === "dc" && fourth === "min") {
    return `Minimum ${humanizeMetricSegment(second)} Disable DC`;
  }
  if (first === "disable" && second && third === "dc" && fourth === "max") {
    return `Maximum ${humanizeMetricSegment(second)} Disable DC`;
  }
  if (first === "disable" && second && third === "rank" && fourth === "min") {
    return `Minimum ${humanizeMetricSegment(second)} Disable Rank`;
  }
  if (first === "weapon" && second) {
    return `Weapon ${humanizeMetricSegment(second)}`;
  }
  if (first === "armor" && second) {
    return `Armor ${humanizeMetricSegment(second)}`;
  }
  if (first === "shield" && second) {
    return `Shield ${humanizeMetricSegment(second)}`;
  }

  return segments.map((segment) => humanizeMetricSegment(segment)).join(" ");
}

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

function buildMetricInspectMetadataQuery(
  metricField: "actorMetrics" | "itemMetrics",
  metricKey: string,
): SearchFilterNode {
  return metricField === "actorMetrics"
    ? {
        kind: "metricCompare",
        leftMetric: metricKey,
        op: "gte",
        rightMetric: metricKey,
      }
    : {
        kind: "metricCompare",
        leftMetric: metricKey,
        op: "gte",
        rightMetric: metricKey,
      };
}

function buildMetricInspectQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  metricField: "actorMetrics" | "itemMetrics",
  metricKey: string,
  metricLabel: string,
): OntologyNode["query"] {
  return buildSearchSemanticsMetadataQuery(
    category,
    subcategory,
    `Browse records with ${metricLabel}`,
    buildMetricInspectMetadataQuery(metricField, metricKey),
  );
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

function getPackPresentationLabel(recordsService: SearchSemanticsRecordsDataService, packValue: string): string {
  const pack = recordsService.getPack?.(packValue);
  return pack?.label ?? pack?.name ?? packValue;
}

function buildMetricScalarMetadataQuery(
  field: "actorMetric" | "itemMetric",
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

function buildQueryRecordChildren(
  dataService: SearchSemanticsRecordsDataService,
  query: OntologyNode["query"] | undefined,
): readonly OntologyNode[] {
  if (!query) {
    return [];
  }

  const request = query.request;
  if (request.mode !== "browse") {
    return [];
  }

  return dataService.listRecords(request).records.map(buildNormalizedRecordNode);
}

async function buildQueryRecordChildrenAsync(
  dataService: SearchSemanticsRecordsDataService,
  query: OntologyNode["query"] | undefined,
): Promise<readonly OntologyNode[]> {
  if (!query) {
    return [];
  }

  if (query.request.mode === "browse") {
    return buildQueryRecordChildren(dataService, query);
  }

  const search = dataService.search;
  if (!search) {
    return [];
  }

  return (await search(query.request)).records.map(buildNormalizedRecordNode);
}

function buildQueryRecordChildSource(
  dataService: SearchSemanticsRecordsDataService,
  query: NonNullable<OntologyNode["query"]>,
): OntologyNode["childSource"] {
  return { kind: "lazy", load: () => buildQueryRecordChildrenAsync(dataService, query) };
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

function buildMetricValueNodes(
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
    matchingRequest?: Readonly<SearchRequest>;
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
    valueType === "number"
      ? buildMetricInspectQuery(category, subcategory, metricField, metricKey, metricLabel)
      : undefined;

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
