import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { Pf2eDataService } from "../../data/service.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../domain/metadata-filter-types.js";
import type { MetadataGlossaryArtifact, MetadataGlossaryEntry } from "../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../domain/ontology-types.js";
import type { MetadataAtomicPredicate } from "../../domain/search-filter-metadata.js";
import { buildAllOfFilter, buildScopeFilter, type SearchFilterNode } from "../../domain/search-request-types.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
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
  metadata: MetadataFilterNode,
): OntologyNode["query"] {
  return {
    label,
    request: {
      mode: "browse",
      filter: buildAllOfFilter([buildScopeFilter(category, subcategory), buildSearchFilterFromMetadataNode(metadata)]),
      limit: 20,
    },
  };
}

export function buildSearchFilterFromMetadataNode(metadata: MetadataFilterNode): SearchFilterNode {
  if ("and" in metadata) {
    return {
      kind: "allOf",
      children: metadata.and.map((child) => buildSearchFilterFromMetadataNode(child)),
    };
  }

  if ("or" in metadata) {
    return {
      kind: "anyOf",
      children: metadata.or.map((child) => buildSearchFilterFromMetadataNode(child)),
    };
  }

  if ("not" in metadata) {
    return {
      kind: "not",
      child: buildSearchFilterFromMetadataNode(metadata.not),
    };
  }

  if (metadata.field === "actorMetric" || metadata.field === "itemMetric") {
    const op =
      metadata.op === "=="
        ? "eq"
        : metadata.op === "!="
          ? "notEq"
          : metadata.op === ">"
            ? "gt"
            : metadata.op === ">="
              ? "gte"
              : metadata.op === "<"
                ? "lt"
                : "lte";
    return {
      kind: "metric",
      metric: metadata.metric,
      op,
      value: metadata.value,
    };
  }

  if (metadata.field === "actorMetricCompare" || metadata.field === "itemMetricCompare") {
    const op =
      metadata.op === "=="
        ? "eq"
        : metadata.op === "!="
          ? "notEq"
          : metadata.op === ">"
            ? "gt"
            : metadata.op === ">="
              ? "gte"
              : metadata.op === "<"
                ? "lt"
                : "lte";
    return {
      kind: "metricCompare",
      leftMetric: metadata.leftMetric,
      op,
      rightMetric: metadata.rightMetric,
    };
  }

  if ("values" in metadata) {
    if (metadata.op === "includesAny") {
      return metadata.values.length === 1
        ? {
            kind: "metadataPredicate",
            predicate: { field: metadata.field, op: "includes", value: metadata.values[0]! } as MetadataAtomicPredicate,
          }
        : {
            kind: "anyOf",
            children: metadata.values.map((value) => ({
              kind: "metadataPredicate",
              predicate: { field: metadata.field, op: "includes", value } as MetadataAtomicPredicate,
            })),
          };
    }

    if (metadata.op === "includesAll") {
      return metadata.values.length === 1
        ? {
            kind: "metadataPredicate",
            predicate: { field: metadata.field, op: "includes", value: metadata.values[0]! } as MetadataAtomicPredicate,
          }
        : {
            kind: "allOf",
            children: metadata.values.map((value) => ({
              kind: "metadataPredicate",
              predicate: { field: metadata.field, op: "includes", value } as MetadataAtomicPredicate,
            })),
          };
    }

    return {
      kind: "not",
      child:
        metadata.values.length === 1
          ? {
              kind: "metadataPredicate",
              predicate: { field: metadata.field, op: "includes", value: metadata.values[0]! } as MetadataAtomicPredicate,
            }
          : {
              kind: "anyOf",
              children: metadata.values.map((value) => ({
                kind: "metadataPredicate",
                predicate: { field: metadata.field, op: "includes", value } as MetadataAtomicPredicate,
              })),
            },
    };
  }

  if ("min" in metadata && "max" in metadata) {
    return {
      kind: "metadataPredicate",
      predicate: { field: metadata.field, op: "between", min: metadata.min, max: metadata.max },
    };
  }

  const op =
    metadata.op === "eq"
      ? "eq"
      : metadata.op === "notEq"
        ? "notEq"
        : metadata.op === "gte"
          ? "gte"
          : metadata.op === "lte"
            ? "lte"
            : metadata.op === "contains"
              ? "contains"
              : metadata.op === "notContains"
                ? "notContains"
                : "eq";

  return {
    kind: "metadataPredicate",
    predicate:
      {
        field: metadata.field,
        op,
        ...("value" in metadata ? { value: metadata.value } : null),
      } as MetadataAtomicPredicate,
  };
}

function buildMetricInspectMetadataQuery(
  metricField: "actorMetrics" | "itemMetrics",
  metricKey: string,
): MetadataFilterNode {
  return metricField === "actorMetrics"
    ? {
        field: "actorMetricCompare",
        leftMetric: metricKey,
        op: ">=",
        rightMetric: metricKey,
      }
    : {
        field: "itemMetricCompare",
        leftMetric: metricKey,
        op: ">=",
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
  if (!query) {
    return [];
  }

  const request = query.request;
  if (request.mode !== "browse") {
    return [];
  }

  return dataService.listRecords(request).records.map(buildNormalizedRecordNode);
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
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    values: ReadonlyArray<{ value: string; count: number }>;
    valueType: "text" | "boolean";
  },
): readonly OntologyNode[] {
  const { category, subcategory, groupLabel, metricField, metadataField, metricKey, values, valueType } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const metricLabel = formatSearchSemanticsMetricLabel(metricKey);
  return values.map((entry) => {
    const metadata = buildMetricScalarMetadataQuery(metadataField, metricKey, valueType, entry.value);
    const query = buildSearchSemanticsMetadataQuery(
      category,
      subcategory,
      `Browse records where ${metricLabel} ${valueType === "boolean" ? "is" : "="} ${entry.value}`,
      metadata,
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
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    liveRecordCount: number;
  },
): OntologyNode {
  const { category, subcategory, groupLabel, metricField, metadataField, metricKey, liveRecordCount } = options;
  const valueType =
    metricField === "actorMetrics" ? inferActorMetricValueType(metricKey) : inferItemMetricValueType(metricKey);
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  const metricLabel = formatSearchSemanticsMetricLabel(metricKey);
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
      { text: `Live canonical records: ${liveRecordCount}` },
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
    loadChildren:
      valueType === "text" || valueType === "boolean"
        ? () =>
            buildMetricValueNodes(dataService, {
              category,
              subcategory,
              groupLabel,
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
        : undefined,
  };
}

function buildMetricNamespaceNode(
  dataService: SearchSemanticsDiscoveryDataService,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    groupLabel: string;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    prefix: string;
    description: string;
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
            groupLabel,
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
    label: string;
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
      `Explore live ${label.toLowerCase()} namespaces, keys, and exact scalar values from the indexed corpus.`,
    ),
    children: namespaces.map((namespace) =>
      buildMetricNamespaceNode(dataService, {
        category,
        subcategory,
        groupLabel: label,
        metricField,
        metadataField,
        prefix: namespace.prefix,
        description: namespace.description,
      }),
    ),
  };
}
