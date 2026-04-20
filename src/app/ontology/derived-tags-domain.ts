import type { OntologyDomainModel, OntologyNode } from "../../types.js";
import {
  type DerivedTagOntologyExplorerCategoryNode,
  type DerivedTagOntologyExplorerFamilyNode,
  type DerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerRecordNode,
  type DerivedTagOntologyExplorerTagNode,
} from "./derived-tag-explorer.js";
import { getOntologyDomainSummary } from "./domain-summaries.js";
import { buildFilterText, titleCaseLabel } from "./node-helpers.js";
import {
  buildOntologyExplorerEntityDetailLines,
  buildOntologyExplorerEntitySummary,
} from "./presenter.js";

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

export function buildDerivedTagsDomain(model: DerivedTagOntologyExplorerModel): OntologyDomainModel {
  return {
    ...getOntologyDomainSummary("derivedTags"),
    rootNodes: model.categories.map(buildCategoryNode),
  };
}
