import { DatabaseSync } from "node:sqlite";

import { CATEGORY_SUBCATEGORY_MAP, SEARCH_CATEGORIES } from "../domain/categories.js";
import {
  getMetadataFilterSemantics,
  type MetadataFieldSemantics,
} from "../domain/metadata-semantics.js";
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
    description: "Browse top-level catalog categories and subcategories with live record counts and ready-to-run browse scopes.",
  },
  {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Explore category-specific metadata fields, examples, common values, and advanced search predicates.",
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
    .flatMap((value) => value ? [value] : [])
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

function buildTagSampleDetailLines(
  tag: DerivedTagOntologyExplorerTagNode,
  limit = 4,
): OntologyNode["detailLines"] {
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
      { text: `Variant inheritance override: ${tag.variantInheritance === undefined ? "(inherit family setting)" : tag.variantInheritance ? "yes" : "no"}` },
      { text: `Live canonical records: ${tag.liveRecordCount}` },
      { text: `Record pages: ${tag.records.length}` },
      { text: `Authored rules: ${tag.authoredRuleCount}` },
      { text: `Exemplars: +${tag.exemplarPositiveCount} / -${tag.exemplarNegativeCount}` },
      { text: `Legacy seed migrations: ${tag.legacyMigrationDefinitionCount} definitions across ${tag.legacyMigrationRecordCount} records` },
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
    const model: DerivedTagOntologyExplorerModel = buildDerivedTagOntologyExplorerModel(db, { cacheKey: config.indexPath });
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
  const subcategoryNodes: OntologyNode[] = (CATEGORY_SUBCATEGORY_MAP[category] ?? []).map((subcategory): OntologyNode => ({
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
  }));

  return {
    id: category,
    kind: "category",
    label: titleCaseLabel(category),
    shortLabel: category,
    filterText: buildFilterText(category, ...(CATEGORY_SUBCATEGORY_MAP[category] ?? [])),
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
    rootNodes: SEARCH_CATEGORIES.map((category) => buildCategorySubcategoryNodes(
      category,
      categoryCounts.get(category) ?? 0,
      subcategoryCountsByCategory.get(category) ?? new Map<string, number>(),
    )),
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
        op: "contains",
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

function buildFieldValueNodes(
  category: SearchCategory,
  fieldSemantics: MetadataFieldSemantics,
  values: Array<{ value: string; count: number }>,
  metadataGlossary: MetadataGlossaryArtifact | null,
): OntologyNode[] {
  return values.map((entry): OntologyNode => {
    const metadata = buildMetadataValueQuery(fieldSemantics, entry.value);
    const traitGlossaryEntry = fieldSemantics.field === "traits"
      ? getTraitGlossaryEntry(metadataGlossary, entry.value)
      : undefined;
    return {
      id: `${category}:${fieldSemantics.field}:${entry.value}`,
      kind: "value",
      label: traitGlossaryEntry?.label ?? entry.value,
      filterText: buildFilterText(
        category,
        fieldSemantics.field,
        entry.value,
        traitGlossaryEntry?.label ?? "",
        traitGlossaryEntry?.description ?? "",
      ),
      listLabel: `${traitGlossaryEntry?.label ?? entry.value} | ${entry.count}`,
      detailTitle: fieldSemantics.field === "traits" ? "Trait Details" : "Filter Value",
      detailLines: fieldSemantics.field === "traits"
        ? [
          ...buildTraitDetailLines(category, entry.value, entry.count, metadataGlossary),
          { text: buildResultReaderHint() },
        ]
        : buildKeyValueDetailLines(entry.value, [
          ["Category", category],
          ["Field", fieldSemantics.field],
          ["Value", entry.value],
          ["Live canonical records", entry.count],
        ], buildResultReaderHint()),
      query: metadata
        ? {
          kind: "listRecords",
          label: fieldSemantics.field === "traits" ? "Browse records with this trait" : "Browse records with this value",
          filters: {
            category,
            metadata,
            limit: 20,
          },
        }
        : undefined,
    };
  });
}

function buildSearchSemanticsDomain(
  config: AppConfig,
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues">,
): OntologyDomainModel {
  const semantics = getMetadataFilterSemantics();
  const vocabulary = dataService.getSearchVocabulary();
  let derivedTagDomain: OntologyDomainModel | null = null;
  try {
    derivedTagDomain = buildDerivedTagsDomain(config);
  } catch {
    derivedTagDomain = null;
  }
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const metadataFieldsByName = new Map(semantics.metadataFields.map((entry) => [entry.field, entry]));
  const filterValuesCache = new Map<string, Array<{ value: string; count: number }>>();

  const getCachedFilterValues = (category: SearchCategory, field: MetadataFieldSemantics["field"]): Array<{ value: string; count: number }> => {
    const cacheKey = `${category}:${field}`;
    const cached = filterValuesCache.get(cacheKey);
    if (cached) {
      return cached;
    }
    const values = dataService.listFilterValues({ field, category }).values;
    filterValuesCache.set(cacheKey, values);
    return values;
  };

  const commonTraitsByCategory = new Map(vocabulary.commonTraitsByCategory.map((entry) => [entry.category, entry.traits]));
  const commonDerivedTagsByCategory = new Map(vocabulary.commonDerivedTagsByCategory.map((entry) => [entry.category, entry.tags]));
  const liveSubcategoryCountsByCategory = new Map(
    SEARCH_CATEGORIES.map((category) => [
      category,
      new Map(dataService.listFilterValues({ field: "subcategories", category }).values.map((entry) => [entry.value, entry.count])),
    ]),
  );
  const examplesByCategory = semantics.examplesByCategory;
  const derivedTagCategoryNodes = new Map(
    (derivedTagDomain?.rootNodes ?? [])
      .filter((node): node is OntologyNode => node.kind === "category" && Boolean(node.shortLabel))
      .map((node) => [node.shortLabel as SearchCategory, node] as const),
  );

  const rootNodes = SEARCH_CATEGORIES.map((category) => {
    const liveSubcategoryCounts = liveSubcategoryCountsByCategory.get(category) ?? new Map<string, number>();
    const categoryFields = semantics.metadataFieldsByCategory[category] ?? [];
    const metadataFieldNodes: OntologyNode[] = categoryFields.map((field): OntologyNode => {
      const fieldSemantics = metadataFieldsByName.get(field)!;
      const derivedTagCategoryNode = field === "derivedTags" ? derivedTagCategoryNodes.get(category) : undefined;
      return {
        id: `${category}:field:${field}`,
        kind: "field",
        label: field,
        filterText: buildFilterText(category, field, fieldSemantics.fieldType, fieldSemantics.notes ?? "", ...(fieldSemantics.subcategories ?? [])),
        listLabel: field,
        detailTitle: "Metadata Field Details",
        detailLines: [
          { text: field, tone: "section" },
          { text: `Category: ${category}` },
          { text: `Field type: ${fieldSemantics.fieldType}` },
          { text: `Discoverable: ${fieldSemantics.discoverable ? "yes" : "no"}` },
          { text: `Subcategory scope: ${fieldSemantics.subcategories?.join(", ") ?? "(all subcategories)"}` },
          { text: `Notes: ${fieldSemantics.notes ?? "(none)"}` },
          ...(fieldSemantics.discoverable
            ? [{ text: "Drill in to browse the full live value space for this field, then open matching records in the shared result reader." }]
            : []),
          ...(field === "derivedTags"
            ? [{ text: "This field exposes the full authored derived-tag hierarchy instead of a flat live-value list." }]
            : []),
        ],
        groupValues: {
          fieldType: fieldSemantics.fieldType,
        },
        ...(field === "derivedTags" && derivedTagCategoryNode?.children
          ? {
            children: derivedTagCategoryNode.children.map((node) => ({ ...node })),
            childPresentation: derivedTagCategoryNode.childPresentation,
          }
          : {
            loadChildren: fieldSemantics.discoverable
              ? () => {
                const liveValues = getCachedFilterValues(category, field);
                return liveValues.length > 0
                  ? buildFieldValueNodes(category, fieldSemantics, liveValues, metadataGlossary)
                  : [];
              }
              : undefined,
          }),
      };
    });

    const advancedPredicateNodes: OntologyNode[] = semantics.advancedPredicates
      .filter((predicate) => predicate.categories.includes(category))
      .map((predicate): OntologyNode => ({
        id: `${category}:advanced:${predicate.name}`,
        kind: "advancedPredicate",
        label: predicate.name,
        filterText: buildFilterText(category, predicate.name, predicate.description),
        listLabel: `${predicate.name} | ${predicate.operators.join(", ")}`,
        detailTitle: "Advanced Predicate Details",
        detailLines: [
          { text: predicate.name, tone: "section" },
          { text: predicate.description },
          { text: `Category: ${category}` },
          { text: `Operators: ${predicate.operators.join(", ")}` },
          { text: `Example: ${JSON.stringify(predicate.example)}` },
        ],
      }));

    const traitNodes: OntologyNode[] = (commonTraitsByCategory.get(category) ?? []).map((entry): OntologyNode => ({
      id: `${category}:trait:${entry.value}`,
      kind: "trait",
      label: getTraitGlossaryEntry(metadataGlossary, entry.value)?.label ?? entry.value,
      filterText: buildFilterText(
        category,
        entry.value,
        "trait",
        getTraitGlossaryEntry(metadataGlossary, entry.value)?.label ?? "",
        getTraitGlossaryEntry(metadataGlossary, entry.value)?.description ?? "",
      ),
      listLabel: `${getTraitGlossaryEntry(metadataGlossary, entry.value)?.label ?? entry.value} | ${entry.count}`,
      detailTitle: "Common Trait",
      detailLines: [
        ...buildTraitDetailLines(category, entry.value, entry.count, metadataGlossary),
        { text: buildResultReaderHint() },
      ],
      query: {
        kind: "listRecords",
        label: "Browse records with this trait",
        filters: {
          category,
          metadata: { field: "traits", op: "includesAny", values: [entry.value] },
          limit: 20,
        },
      },
    }));

    const commonDerivedTagNodes: OntologyNode[] = (commonDerivedTagsByCategory.get(category) ?? []).map((entry): OntologyNode => ({
      id: `${category}:derivedTag:${entry.value}`,
      kind: "derivedTagValue",
      label: entry.value,
      filterText: buildFilterText(category, entry.value, "derived tag"),
      listLabel: `${entry.value} | ${entry.count}`,
      detailTitle: "Common Derived Tag",
      detailLines: buildKeyValueDetailLines(entry.value, [
        ["Category", category],
        ["Derived tag", entry.value],
        ["Live canonical records", entry.count],
      ], buildResultReaderHint()),
      query: {
        kind: "listRecords",
        label: "Browse records with this derived tag",
        filters: {
          category,
          metadata: { field: "derivedTags", op: "includesAny", values: [entry.value] },
          limit: 20,
        },
      },
    }));

    const exampleNodes: OntologyNode[] = (examplesByCategory[category] ?? []).map((example, index): OntologyNode => ({
      id: `${category}:example:${index}`,
      kind: "example",
      label: example.label,
      filterText: buildFilterText(category, example.label, example.notes ?? ""),
      listLabel: example.label,
      detailTitle: "Example Predicate",
      detailLines: [
        { text: example.label, tone: "section" },
        { text: `Category: ${category}` },
        { text: `Predicate: ${JSON.stringify(example.metadata)}` },
        { text: `Notes: ${example.notes ?? "(none)"}` },
      ],
    }));

    const subcategoryNodes: OntologyNode[] = (CATEGORY_SUBCATEGORY_MAP[category] ?? []).map((subcategory): OntologyNode => ({
      id: `${category}:subcategory:${subcategory}`,
      kind: "subcategory",
      label: subcategory,
      filterText: buildFilterText(category, subcategory),
      listLabel: `${subcategory} | ${liveSubcategoryCounts.get(subcategory) ?? 0}`,
      detailTitle: "Subcategory Boundary",
      detailLines: buildKeyValueDetailLines(subcategory, [
        ["Category", category],
        ["Subcategory", subcategory],
        ["Live canonical records", liveSubcategoryCounts.get(subcategory) ?? 0],
      ], buildResultReaderHint()),
      query: {
        kind: "listRecords",
        label: "Browse this subcategory",
        filters: {
          category,
          subcategory,
          limit: 20,
        },
      },
    }));

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
    if (traitNodes.length > 0) {
      children.push({
        id: `${category}:commonTraits`,
        kind: "group",
        label: "Common Traits",
        filterText: buildFilterText(category, "common traits", ...traitNodes.map((node) => node.label)),
        listLabel: `Common traits | ${traitNodes.length}`,
        detailTitle: "Common Traits",
        detailLines: buildKeyValueDetailLines("Common Traits", [
          ["Category", category],
          ["Entries", traitNodes.length],
        ]),
        children: traitNodes,
      });
    }
    if (commonDerivedTagNodes.length > 0) {
      children.push({
        id: `${category}:commonDerivedTags`,
        kind: "group",
        label: "Common Derived Tags",
        filterText: buildFilterText(category, "common derived tags", ...commonDerivedTagNodes.map((node) => node.label)),
        listLabel: `Common derived tags | ${commonDerivedTagNodes.length}`,
        detailTitle: "Common Derived Tags",
        detailLines: buildKeyValueDetailLines("Common Derived Tags", [
          ["Category", category],
          ["Entries", commonDerivedTagNodes.length],
        ]),
        children: commonDerivedTagNodes,
      });
    }
    if (metadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(category, "metadata fields", ...categoryFields),
        listLabel: `Metadata fields | ${categoryFields.length}`,
        detailTitle: "Metadata Fields",
        detailLines: buildKeyValueDetailLines("Metadata Fields", [
          ["Category", category],
          ["Fields", categoryFields.length],
        ], "Use these typed fields after category and subcategory boundaries."),
        children: metadataFieldNodes,
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }
    if (advancedPredicateNodes.length > 0) {
      children.push({
        id: `${category}:advancedPredicates`,
        kind: "group",
        label: "Advanced Predicates",
        filterText: buildFilterText(category, "advanced predicates", ...advancedPredicateNodes.map((node) => node.label)),
        listLabel: `Advanced predicates | ${advancedPredicateNodes.length}`,
        detailTitle: "Advanced Predicates",
        detailLines: buildKeyValueDetailLines("Advanced Predicates", [
          ["Category", category],
          ["Predicates", advancedPredicateNodes.length],
        ]),
        children: advancedPredicateNodes,
      });
    }
    if (exampleNodes.length > 0) {
      children.push({
        id: `${category}:examples`,
        kind: "group",
        label: "Examples",
        filterText: buildFilterText(category, "examples", ...exampleNodes.map((node) => node.label)),
        listLabel: `Examples | ${exampleNodes.length}`,
        detailTitle: "Category Examples",
        detailLines: buildKeyValueDetailLines("Examples", [
          ["Category", category],
          ["Examples", exampleNodes.length],
        ]),
        children: exampleNodes,
      });
    }

    return {
      id: `searchSemantics:${category}`,
      kind: "category",
      label: titleCaseLabel(category),
      shortLabel: category,
      filterText: buildFilterText(category, ...categoryFields),
      listLabel: `${category} | ${children.length} groups`,
      detailTitle: "Search Semantics",
      detailLines: buildKeyValueDetailLines(titleCaseLabel(category), [
        ["Category", category],
        ["Subcategories", (CATEGORY_SUBCATEGORY_MAP[category] ?? []).length],
        ["Metadata fields", categoryFields.length],
        ["Advanced predicates", advancedPredicateNodes.length],
      ], "Explore category-specific search semantics, discoverable fields, and example predicates."),
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
  dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues">,
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
          throw new Error(`Unknown ontology domain: ${exhaustive}`);
        }
      }
      domainCache.set(id, domain);
      return domain;
    },
  };
}
