import { DatabaseSync } from "node:sqlite";

import { CATEGORY_SUBCATEGORY_MAP, SEARCH_CATEGORIES, normalizeSearchCategory } from "../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../domain/metadata-semantics.js";
import { inferActorMetricValueType } from "../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../domain/item-metrics.js";
import { readMetadataGlossaryArtifact } from "../data/metadata-glossary.js";
import { Pf2eDataService } from "../data/service.js";
import type {
  AppConfig,
  MetadataGlossaryArtifact,
  MetadataGlossaryEntry,
  MetadataBooleanField,
  MetadataFilterNode,
  MetadataEnumStringField,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
  NormalizedRecord,
  OntologyDomainId,
  OntologyDomainModel,
  OntologyDomainSummary,
  OntologyNode,
  SearchCategory,
  SearchSubcategory,
} from "../types.js";
import { normalizeText } from "../utils.js";
import {
  buildDerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerCategoryNode,
  type DerivedTagOntologyExplorerFamilyNode,
  type DerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerRecordNode,
  type DerivedTagOntologyExplorerTagNode,
} from "../tui/ontology-explorer/data.js";
import {
  buildOntologyExplorerEntityDetailLines,
  buildOntologyExplorerEntitySummary,
} from "../tui/ontology-explorer/entity-page.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "../tui/ontology-explorer/entity-record.js";

export type Pf2eApplicationOntologyService = {
  listDomains: () => OntologyDomainSummary[];
  loadDomain: (id: OntologyDomainId) => OntologyDomainModel;
};

const ONTOLOGY_DOMAINS: OntologyDomainSummary[] = [
  {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Browse the authored derived-tag ontology with live record coverage and editorial detail.",
  },
  {
    id: "catalogCategories",
    label: "Categories",
    description:
      "Browse top-level catalog categories and subcategories with live record counts and ready-to-run browse scopes.",
  },
  {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Explore category-specific metadata fields, live value spaces, and advanced search predicates.",
  },
];

function titleCaseLabel(value: string): string {
  return value
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function buildFilterText(...values: Array<string | null | undefined>): string {
  return values
    .flatMap((value) => (value ? [value] : []))
    .map((value) => normalizeText(value))
    .filter(Boolean)
    .join(" ");
}

function buildKeyValueDetailLines(
  title: string,
  entries: Array<[string, string | number | null | undefined]>,
  description?: string,
): OntologyNode["detailLines"] {
  const lines: OntologyNode["detailLines"] = [{ text: title, tone: "section" }];
  if (description) {
    lines.push({ text: description });
  }
  for (const [label, value] of entries) {
    const rendered = value === null || value === undefined || value === "" ? "(none)" : String(value);
    lines.push({ text: `${label}: ${rendered}` });
  }
  return lines;
}

function getTraitGlossaryEntry(
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

function cloneMetadataFilterNode(metadata: MetadataFilterNode): MetadataFilterNode {
  return structuredClone(metadata);
}

function cloneOntologyNode(node: OntologyNode, idPrefix?: string): OntologyNode {
  return {
    ...node,
    id: idPrefix ? `${idPrefix}:${node.id}` : node.id,
    children: node.children?.map((child) => cloneOntologyNode(child, idPrefix)),
    loadChildren: node.loadChildren
      ? () => node.loadChildren!().map((child) => cloneOntologyNode(child, idPrefix))
      : undefined,
    childPresentation: node.childPresentation ? { ...node.childPresentation } : undefined,
    groupValues: node.groupValues ? { ...node.groupValues } : undefined,
    query: node.query ? { ...node.query, filters: { ...node.query.filters } } : undefined,
    selection: node.selection ? { ...node.selection } : undefined,
  };
}

function buildNormalizedRecordNode(record: NormalizedRecord): OntologyNode {
  const entityRecord = mapNormalizedRecordToOntologyExplorerEntityRecord(record);
  return {
    id: record.recordKey,
    kind: "record",
    label: record.name,
    filterText: buildFilterText(
      record.recordKey,
      record.name,
      record.category,
      record.subcategory ?? "",
      record.descriptionText ?? "",
      record.blurbText ?? "",
    ),
    listLabel: buildOntologyExplorerEntitySummary(entityRecord),
    detailTitle: "Record Details",
    detailLines: buildOntologyExplorerEntityDetailLines(entityRecord),
    query: {
      kind: "lookup",
      label: "Open exact record lookup",
      filters: {
        nameQuery: record.name,
        category: record.category,
        subcategory: record.subcategory ?? undefined,
        limit: 5,
      },
    },
  };
}

function buildSearchSemanticsMetadataQuery(
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

function buildRecordNode(recordNode: DerivedTagOntologyExplorerRecordNode): OntologyNode {
  return {
    id: recordNode.key,
    kind: "record",
    label: recordNode.record.name,
    filterText: buildFilterText(
      recordNode.tag,
      recordNode.record.name,
      recordNode.record.recordKey,
      recordNode.record.category,
      recordNode.record.subcategory ?? "",
      recordNode.record.descriptionText ?? "",
      recordNode.record.blurbText ?? "",
    ),
    listLabel: buildOntologyExplorerEntitySummary(recordNode.record),
    detailTitle: "Record Details",
    detailLines: buildOntologyExplorerEntityDetailLines(recordNode.record),
    query: {
      kind: "lookup",
      label: "Open exact record lookup",
      filters: {
        nameQuery: recordNode.record.name,
        category: recordNode.record.category,
        subcategory: recordNode.record.subcategory ?? undefined,
        limit: 5,
      },
    },
  };
}

function buildTagSampleDetailLines(tag: DerivedTagOntologyExplorerTagNode, limit = 4): OntologyNode["detailLines"] {
  const sampleRecords = tag.records.slice(0, limit);
  if (sampleRecords.length === 0) {
    return [
      { text: "Live sample records:", tone: "section" },
      { text: "(none)", indent: 2 },
    ];
  }

  return [
    { text: "Live sample records:", tone: "section" },
    ...sampleRecords.flatMap((recordNode) => {
      const preview = recordNode.record.blurbText ?? recordNode.record.descriptionText ?? "";
      return [
        { text: buildOntologyExplorerEntitySummary(recordNode.record), indent: 2 },
        ...(preview ? [{ text: preview, indent: 4 }] : []),
      ];
    }),
  ];
}

function buildTagNode(tag: DerivedTagOntologyExplorerTagNode): OntologyNode {
  return {
    id: tag.key,
    kind: "tag",
    label: tag.tag,
    filterText: buildFilterText(
      tag.category,
      tag.family,
      tag.tag,
      tag.description,
      ...(tag.appliesWhen ?? []),
      ...(tag.doesNotApplyWhen ?? []),
      ...(tag.positiveSignals ?? []),
      ...(tag.negativeSignals ?? []),
      ...(tag.adjacentTags ?? []),
      ...(tag.compositeOfAnyTags ?? []),
    ),
    listLabel: `${tag.tag} | ${tag.assignmentMode} | ${tag.liveRecordCount} live records`,
    detailTitle: "Tag Details",
    detailLines: [
      { text: tag.tag, tone: "section" },
      { text: tag.description },
      { text: `Category: ${tag.category}` },
      { text: `Family: ${tag.family}` },
      { text: `Scope: ${tag.subcategories?.join(", ") ?? "(all subcategories)"}` },
      { text: `Assignment mode: ${tag.assignmentMode}` },
      { text: `Native ontology policy: ${tag.nativeOntologyPolicy ?? "(none)"}` },
      {
        text: `Variant inheritance override: ${tag.variantInheritance === undefined ? "(inherit family setting)" : tag.variantInheritance ? "yes" : "no"}`,
      },
      { text: `Live canonical records: ${tag.liveRecordCount}` },
      { text: `Record pages: ${tag.records.length}` },
      { text: `Authored rules: ${tag.authoredRuleCount}` },
      { text: `Exemplars: +${tag.exemplarPositiveCount} / -${tag.exemplarNegativeCount}` },
      {
        text: `Legacy seed migrations: ${tag.legacyMigrationDefinitionCount} definitions across ${tag.legacyMigrationRecordCount} records`,
      },
      { text: `Adjacent tags: ${tag.adjacentTags?.join(", ") ?? "(none)"}` },
      { text: `Composite children: ${tag.compositeOfAnyTags?.join(", ") ?? "(none)"}` },
      { text: "Applies when:", tone: "section" },
      { text: tag.appliesWhen?.join(" | ") ?? "(none)", indent: 2 },
      { text: "Does not apply when:", tone: "section" },
      { text: tag.doesNotApplyWhen?.join(" | ") ?? "(none)", indent: 2 },
      { text: "Positive signals:", tone: "section" },
      { text: tag.positiveSignals?.join(" | ") ?? "(none)", indent: 2 },
      { text: "Negative signals:", tone: "section" },
      { text: tag.negativeSignals?.join(" | ") ?? "(none)", indent: 2 },
      ...buildTagSampleDetailLines(tag),
    ],
    children: tag.records.map(buildRecordNode),
    query: {
      kind: "listRecords",
      label: "List records with this derived tag",
      filters: {
        category: tag.category,
        subcategory: tag.subcategories?.length === 1 ? tag.subcategories[0] : undefined,
        metadata: { field: "derivedTags", op: "includesAny", values: [tag.tag] },
        limit: 20,
      },
    },
  };
}

function buildFamilyNode(family: DerivedTagOntologyExplorerFamilyNode): OntologyNode {
  return {
    id: family.key,
    kind: "family",
    label: family.family,
    filterText: buildFilterText(
      family.category,
      family.axis,
      family.family,
      family.description,
      ...(family.subcategories ?? []),
      ...family.tags.map((tag) => tag.tag),
    ),
    listLabel: `${family.family} | ${family.tagCount} tags | ${family.liveRecordCount} live records`,
    detailTitle: "Family Details",
    detailLines: [
      { text: family.family, tone: "section" },
      { text: family.description },
      { text: `Category: ${family.category}` },
      { text: `Axis: ${family.axis}` },
      { text: `Scope: ${family.subcategories?.join(", ") ?? "(all subcategories)"}` },
      { text: `Variant inheritance: ${family.variantInheritance ? "yes" : "no"}` },
      { text: `Tags: ${family.tagCount}` },
      { text: `Distinct live records: ${family.liveRecordCount}` },
    ],
    children: family.tags.map(buildTagNode),
    groupValues: {
      axis: family.axis,
    },
  };
}

function buildCategoryNode(category: DerivedTagOntologyExplorerCategoryNode): OntologyNode {
  return {
    id: category.key,
    kind: "category",
    label: titleCaseLabel(category.category),
    shortLabel: category.category,
    filterText: buildFilterText(
      category.category,
      ...category.families.map((family) => `${family.axis} ${family.family} ${family.description}`),
    ),
    listLabel: `${category.category} | ${category.familyCount} families | ${category.tagCount} tags | ${category.taggedRecordCount} tagged records`,
    detailTitle: "Category Details",
    detailLines: [
      { text: category.category, tone: "section" },
      { text: `Families: ${category.familyCount}` },
      { text: `Tags: ${category.tagCount}` },
      { text: `Tagged canonical records: ${category.taggedRecordCount}` },
    ],
    children: category.families.map(buildFamilyNode),
    childPresentation: {
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    },
  };
}

function buildDerivedTagsDomain(config: AppConfig): OntologyDomainModel {
  const db = new DatabaseSync(config.indexPath);
  try {
    const model: DerivedTagOntologyExplorerModel = buildDerivedTagOntologyExplorerModel(db, {
      cacheKey: config.indexPath,
    });
    return {
      ...ONTOLOGY_DOMAINS.find((domain) => domain.id === "derivedTags")!,
      rootNodes: model.categories.map(buildCategoryNode),
    };
  } finally {
    db.close();
  }
}

function buildCategorySubcategoryNodes(
  category: SearchCategory,
  categoryCount: number,
  liveSubcategoryCounts: Map<string, number>,
): OntologyNode {
  const subcategoryNodes: OntologyNode[] = CATEGORY_SUBCATEGORY_MAP[category].map(
    (subcategory): OntologyNode => ({
      id: `${category}:${subcategory}`,
      kind: "subcategory",
      label: titleCaseLabel(subcategory),
      shortLabel: subcategory,
      filterText: buildFilterText(category, subcategory),
      listLabel: `${subcategory} | ${liveSubcategoryCounts.get(subcategory) ?? 0} live records`,
      detailTitle: "Subcategory Details",
      detailLines: buildKeyValueDetailLines(titleCaseLabel(subcategory), [
        ["Category", category],
        ["Subcategory", subcategory],
        ["Live canonical records", liveSubcategoryCounts.get(subcategory) ?? 0],
      ]),
      query: {
        kind: "listRecords",
        label: "Browse this subcategory",
        filters: {
          category,
          subcategory,
          limit: 20,
        },
      },
    }),
  );

  return {
    id: category,
    kind: "category",
    label: titleCaseLabel(category),
    shortLabel: category,
    filterText: buildFilterText(category, ...CATEGORY_SUBCATEGORY_MAP[category]),
    listLabel: `${category} | ${categoryCount} live records`,
    detailTitle: "Category Details",
    detailLines: buildKeyValueDetailLines(titleCaseLabel(category), [
      ["Category", category],
      ["Live canonical records", categoryCount],
      ["Subcategories", subcategoryNodes.length],
    ]),
    children: subcategoryNodes,
    query: {
      kind: "listRecords",
      label: "Browse this category",
      filters: {
        category,
        limit: 20,
      },
    },
  };
}

function buildCatalogCategoriesDomain(
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues">,
): OntologyDomainModel {
  const vocabulary = dataService.getSearchVocabulary();
  const categoryCounts = new Map(vocabulary.categories.map((entry) => [entry.value, entry.count]));
  const subcategoryCountsByCategory = new Map<SearchCategory, Map<string, number>>();

  for (const category of SEARCH_CATEGORIES) {
    const liveSubcategories = dataService.listFilterValues({
      field: "subcategories",
      category,
    }).values;
    subcategoryCountsByCategory.set(category, new Map(liveSubcategories.map((entry) => [entry.value, entry.count])));
  }

  return {
    ...ONTOLOGY_DOMAINS.find((domain) => domain.id === "catalogCategories")!,
    rootNodes: SEARCH_CATEGORIES.map((category) =>
      buildCategorySubcategoryNodes(
        category,
        categoryCounts.get(category) ?? 0,
        subcategoryCountsByCategory.get(category) ?? new Map<string, number>(),
      ),
    ),
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
  dataService: Pick<Pf2eDataService, "listRecords">,
  query: OntologyNode["query"] | undefined,
): OntologyNode[] {
  if (!query || query.kind !== "listRecords") {
    return [];
  }

  return dataService.listRecords(query.filters).records.map(buildNormalizedRecordNode);
}

function buildFieldValueNodes(
  dataService: Pick<Pf2eDataService, "listRecords">,
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  fieldSemantics: MetadataFieldSemantics,
  values: Array<{ value: string; count: number }>,
  metadataGlossary: MetadataGlossaryArtifact | null,
): OntologyNode[] {
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
  dataService: Pick<Pf2eDataService, "listFilterValues" | "listRecords">,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    metricKey: string;
    values: Array<{ value: string; count: number }>;
    valueType: "text" | "boolean";
  },
): OntologyNode[] {
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
  dataService: Pick<Pf2eDataService, "listFilterValues" | "listRecords">,
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
  dataService: Pick<Pf2eDataService, "listFilterValues" | "listRecords">,
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
      { text: "Drill in to browse live metric keys, then inspect exact scalar values where the backend can enumerate them." },
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

function buildMetricDiscoveryGroup(
  dataService: Pick<Pf2eDataService, "listFilterValues" | "listRecords">,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    metricField: "actorMetrics" | "itemMetrics";
    metadataField: "actorMetric" | "itemMetric";
    label: "Actor Metrics" | "Item Metrics";
    namespaces: Array<{ prefix: string; description: string }>;
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

function buildSearchSemanticsDomain(
  config: AppConfig,
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords">,
): OntologyDomainModel {
  const semantics = getMetadataFilterSemantics();
  const vocabulary = dataService.getSearchVocabulary();
  let derivedTagDomain: OntologyDomainModel | null;
  try {
    derivedTagDomain = buildDerivedTagsDomain(config);
  } catch {
    derivedTagDomain = null;
  }
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const metadataFieldsByName = new Map(semantics.metadataFields.map((entry) => [entry.field, entry]));
  const filterValuesCache = new Map<string, Array<{ value: string; count: number }>>();

  const getCachedFilterValues = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    field: MetadataFieldSemantics["field"],
  ): Array<{ value: string; count: number }> => {
    const cacheKey = `${category}:${subcategory ?? "all"}:${field}`;
    const cached = filterValuesCache.get(cacheKey);
    if (cached) {
      return cached;
    }
    const values = dataService.listFilterValues({ field, category, subcategory: subcategory ?? undefined }).values;
    filterValuesCache.set(cacheKey, values);
    return values;
  };

  const getCategoryScopedFields = (category: SearchCategory, subcategory: SearchSubcategory | null) =>
    semantics.metadataFieldsByCategory[category].filter((field) => {
      const fieldSemantics = metadataFieldsByName.get(field);
      if (!fieldSemantics) {
        return false;
      }
      return !subcategory || !fieldSemantics.subcategories || fieldSemantics.subcategories.includes(subcategory);
    });

  const commonTraitsByCategory = new Map(
    vocabulary.commonTraitsByCategory.map((entry) => [entry.category, entry.traits]),
  );
  const commonDerivedTagsByCategory = new Map(
    vocabulary.commonDerivedTagsByCategory.map((entry) => [entry.category, entry.tags]),
  );
  const liveSubcategoryCountsByCategory = new Map(
    SEARCH_CATEGORIES.map((category) => [
      category,
      new Map(
        dataService
          .listFilterValues({ field: "subcategories", category })
          .values.map((entry) => [entry.value, entry.count]),
      ),
    ]),
  );
  const derivedTagCategoryNodes = new Map(
    (derivedTagDomain?.rootNodes ?? [])
      .filter((node): node is OntologyNode => node.kind === "category" && Boolean(node.shortLabel))
      .flatMap((node) => {
        const category = normalizeSearchCategory(node.shortLabel);
        return category ? ([[category, node]] as const) : [];
      }),
  );

  function buildMetadataFieldNodes(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return getCategoryScopedFields(category, subcategory).map((field): OntologyNode => {
      const fieldSemantics = metadataFieldsByName.get(field)!;
      const derivedTagCategoryNode = field === "derivedTags" ? derivedTagCategoryNodes.get(category) : undefined;
      return {
        id: `${idPrefix}:field:${field}`,
        kind: "field",
        label: field,
        filterText: buildFilterText(
          category,
          subcategory ?? "",
          field,
          fieldSemantics.fieldType,
          fieldSemantics.notes ?? "",
          ...(fieldSemantics.subcategories ?? []),
        ),
        listLabel: field,
        detailTitle: "Metadata Field Details",
        detailLines: [
          { text: field, tone: "section" },
          { text: `Category: ${category}` },
          { text: `Subcategory: ${subcategory ?? "(all)"}` },
          { text: `Field type: ${fieldSemantics.fieldType}` },
          { text: `Discoverable: ${fieldSemantics.discoverable ? "yes" : "no"}` },
          { text: `Subcategory scope: ${fieldSemantics.subcategories?.join(", ") ?? "(all subcategories)"}` },
          { text: `Notes: ${fieldSemantics.notes ?? "(none)"}` },
          ...(fieldSemantics.discoverable
            ? [
                {
                  text: "Drill in to browse the full live value space for this field, inspect exact matching records inline, or launch the seeded search/editor flow.",
                },
              ]
            : []),
          ...(field === "derivedTags"
            ? [
                {
                  text: "This field exposes the full authored derived-tag hierarchy instead of a flat live-value list.",
                },
              ]
            : []),
        ],
        groupValues: {
          fieldType: fieldSemantics.fieldType,
        },
        ...(field === "derivedTags" && derivedTagCategoryNode?.children
          ? {
              children: derivedTagCategoryNode.children.map((node) =>
                cloneOntologyNode(node, `${idPrefix}:field:${field}`),
              ),
              childPresentation: derivedTagCategoryNode.childPresentation
                ? { ...derivedTagCategoryNode.childPresentation }
                : undefined,
            }
          : {
              loadChildren: fieldSemantics.discoverable
                ? () => {
                    const liveValues = getCachedFilterValues(category, subcategory, field);
                    return liveValues.length > 0
                      ? buildFieldValueNodes(dataService, category, subcategory, fieldSemantics, liveValues, metadataGlossary)
                      : [];
                  }
                : undefined,
            }),
      };
    });
  }

  function buildCommonTraitShortcutGroup(category: SearchCategory, metadataFieldNodes: OntologyNode[]): OntologyNode | null {
    const traitFieldNode = metadataFieldNodes.find((node) => node.label === "traits");
    const traitNodesByLabel = new Map(
      (traitFieldNode?.loadChildren?.() ?? []).map((node) => [normalizeText(node.label), node]),
    );
    const shortcutNodes = (commonTraitsByCategory.get(category) ?? [])
      .map((entry) => traitNodesByLabel.get(normalizeText(getTraitGlossaryEntry(metadataGlossary, entry.value)?.label ?? entry.value)))
      .filter((node): node is OntologyNode => Boolean(node))
      .map((node) => cloneOntologyNode(node, `${category}:commonTraitsShortcut`));

    if (shortcutNodes.length === 0) {
      return null;
    }

    return {
      id: `${category}:commonTraits`,
      kind: "group",
      label: "Common Traits",
      filterText: buildFilterText(category, "common traits", ...shortcutNodes.map((node) => node.label)),
      listLabel: `Common traits | ${shortcutNodes.length}`,
      detailTitle: "Common Traits",
      detailLines: buildKeyValueDetailLines(
        "Common Traits",
        [
          ["Category", category],
          ["Entries", shortcutNodes.length],
        ],
        "Shortcut into the canonical trait value space for this category.",
      ),
      children: shortcutNodes,
    };
  }

  function buildCommonDerivedTagShortcutGroup(
    category: SearchCategory,
    metadataFieldNodes: OntologyNode[],
  ): OntologyNode | null {
    const derivedTagFieldNode = metadataFieldNodes.find((node) => node.label === "derivedTags");
    if (!derivedTagFieldNode?.children) {
      return null;
    }

    const desiredTags = new Set(
      (commonDerivedTagsByCategory.get(category) ?? []).map((entry) => normalizeText(entry.value)),
    );
    const matchingFamilies = derivedTagFieldNode.children.flatMap((familyNode) => {
        const matchingTags =
          familyNode.children?.filter((tagNode) => desiredTags.has(normalizeText(tagNode.label))) ?? [];
        if (matchingTags.length === 0) {
          return [];
        }
        return [
          {
            ...cloneOntologyNode(familyNode, `${category}:commonDerivedTagsShortcut`),
            children: matchingTags.map((tagNode) =>
              cloneOntologyNode(tagNode, `${category}:commonDerivedTagsShortcut`),
            ),
          } satisfies OntologyNode,
        ];
      });

    if (matchingFamilies.length === 0) {
      return null;
    }

    return {
      id: `${category}:commonDerivedTags`,
      kind: "group",
      label: "Common Derived Tags",
      filterText: buildFilterText(category, "common derived tags", ...matchingFamilies.map((node) => node.label)),
      listLabel: `Common derived tags | ${desiredTags.size}`,
      detailTitle: "Common Derived Tags",
      detailLines: buildKeyValueDetailLines(
        "Common Derived Tags",
        [
          ["Category", category],
          ["Families", matchingFamilies.length],
          ["Entries", desiredTags.size],
        ],
        "Shortcut into the canonical derived-tag family/tag hierarchy for this category.",
      ),
      children: matchingFamilies,
    };
  }

  function buildMetricDiscoveryGroups(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): OntologyNode[] {
    const groups: OntologyNode[] = [];
    if (
      semantics.actorMetricDiscovery &&
      semantics.advancedPredicates.some((predicate) => predicate.name === "actorMetric" && predicate.categories.includes(category))
    ) {
      groups.push(
        buildMetricDiscoveryGroup(dataService, {
          category,
          subcategory,
          metricField: "actorMetrics",
          metadataField: "actorMetric",
          label: "Actor Metrics",
          namespaces: semantics.actorMetricDiscovery.namespaces,
        }),
      );
    }
    if (
      semantics.itemMetricDiscovery &&
      semantics.advancedPredicates.some((predicate) => predicate.name === "itemMetric" && predicate.categories.includes(category))
    ) {
      groups.push(
        buildMetricDiscoveryGroup(dataService, {
          category,
          subcategory,
          metricField: "itemMetrics",
          metadataField: "itemMetric",
          label: "Item Metrics",
          namespaces: semantics.itemMetricDiscovery.namespaces,
        }),
      );
    }
    return groups;
  }

  function buildSubcategoryNode(category: SearchCategory, subcategory: SearchSubcategory): OntologyNode {
    const subcategoryMetadataFieldNodes = buildMetadataFieldNodes(category, subcategory);
    const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, subcategory);
    const children: OntologyNode[] = [];

    if (subcategoryMetadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:${subcategory}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(
          category,
          subcategory,
          "metadata fields",
          ...subcategoryMetadataFieldNodes.map((node) => node.label),
        ),
        listLabel: `Metadata fields | ${subcategoryMetadataFieldNodes.length}`,
        detailTitle: "Metadata Fields",
        detailLines: buildKeyValueDetailLines(
          "Metadata Fields",
          [
            ["Category", category],
            ["Subcategory", subcategory],
            ["Fields", subcategoryMetadataFieldNodes.length],
          ],
          "Use these typed fields within this subcategory boundary.",
        ),
        children: subcategoryMetadataFieldNodes,
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }

    children.push(...metricDiscoveryGroups);

    return {
      id: `${category}:subcategory:${subcategory}`,
      kind: "subcategory",
      label: subcategory,
      filterText: buildFilterText(category, subcategory, ...children.map((node) => node.label)),
      listLabel: `${subcategory} | ${liveSubcategoryCountsByCategory.get(category)?.get(subcategory) ?? 0}`,
      detailTitle: "Subcategory Boundary",
      detailLines: buildKeyValueDetailLines(
        subcategory,
        [
          ["Category", category],
          ["Subcategory", subcategory],
          ["Live canonical records", liveSubcategoryCountsByCategory.get(category)?.get(subcategory) ?? 0],
          ["Metadata fields", subcategoryMetadataFieldNodes.length],
        ],
        "Browse this subcategory directly, or drill in to its scoped metadata and metric surfaces.",
      ),
      query: {
        kind: "listRecords",
        label: "Browse this subcategory",
        filters: {
          category,
          subcategory,
          limit: 20,
        },
      },
      children,
    };
  }

  const rootNodes = SEARCH_CATEGORIES.map((category) => {
    const categoryFields = getCategoryScopedFields(category, null);
    const metadataFieldNodes = buildMetadataFieldNodes(category, null);
    const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, null);

    const subcategoryNodes: OntologyNode[] = CATEGORY_SUBCATEGORY_MAP[category].map(
      (subcategory): OntologyNode => buildSubcategoryNode(category, subcategory),
    );

    const children: OntologyNode[] = [];
    if (subcategoryNodes.length > 0) {
      children.push({
        id: `${category}:subcategories`,
        kind: "group",
        label: "Subcategories",
        filterText: buildFilterText(category, "subcategories", ...subcategoryNodes.map((node) => node.label)),
        listLabel: `Subcategories | ${subcategoryNodes.length}`,
        detailTitle: "Category Boundaries",
        detailLines: buildKeyValueDetailLines("Subcategories", [
          ["Category", category],
          ["Subcategories", subcategoryNodes.length],
        ]),
        children: subcategoryNodes,
      });
    }
    const commonTraitGroup = buildCommonTraitShortcutGroup(category, metadataFieldNodes);
    if (commonTraitGroup) {
      children.push(commonTraitGroup);
    }
    const commonDerivedTagGroup = buildCommonDerivedTagShortcutGroup(category, metadataFieldNodes);
    if (commonDerivedTagGroup) {
      children.push(commonDerivedTagGroup);
    }
    if (metadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(category, "metadata fields", ...categoryFields),
        listLabel: `Metadata fields | ${categoryFields.length}`,
        detailTitle: "Metadata Fields",
        detailLines: buildKeyValueDetailLines(
          "Metadata Fields",
          [
            ["Category", category],
            ["Fields", categoryFields.length],
          ],
          "Use these typed fields after category and subcategory boundaries.",
        ),
        children: metadataFieldNodes,
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }
    children.push(...metricDiscoveryGroups);
    return {
      id: `searchSemantics:${category}`,
      kind: "category",
      label: titleCaseLabel(category),
      shortLabel: category,
      filterText: buildFilterText(category, ...categoryFields),
      listLabel: `${category} | ${children.length} groups`,
      detailTitle: "Search Semantics",
      detailLines: buildKeyValueDetailLines(
        titleCaseLabel(category),
        [
          ["Category", category],
          ["Subcategories", CATEGORY_SUBCATEGORY_MAP[category].length],
          ["Metadata fields", categoryFields.length],
          ["Metric discovery groups", metricDiscoveryGroups.length],
        ],
        "Explore category-specific search semantics, live value spaces, metric discovery, and canonical browse surfaces.",
      ),
      children,
    };
  });

  return {
    ...ONTOLOGY_DOMAINS.find((domain) => domain.id === "searchSemantics")!,
    rootNodes,
  };
}

export function createPf2eApplicationOntologyService(
  config: AppConfig,
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords">,
): Pf2eApplicationOntologyService {
  const domainCache = new Map<OntologyDomainId, OntologyDomainModel>();

  return {
    listDomains: () => ONTOLOGY_DOMAINS,
    loadDomain: (id) => {
      const cached = domainCache.get(id);
      if (cached) {
        return cached;
      }

      let domain: OntologyDomainModel;
      switch (id) {
        case "derivedTags":
          domain = buildDerivedTagsDomain(config);
          break;
        case "catalogCategories":
          domain = buildCatalogCategoriesDomain(dataService);
          break;
        case "searchSemantics":
          domain = buildSearchSemanticsDomain(config, dataService);
          break;
        default: {
          const exhaustive: never = id;
          throw new Error(`Unknown ontology domain: ${String(exhaustive)}`);
        }
      }
      domainCache.set(id, domain);
      return domain;
    },
  };
}
