import type { OntologyNode } from "../../../domain/ontology-types.js";
import { formatOntologySearchVocabularyLabel } from "../../../domain/presentation-vocabulary.js";
import type { DerivedTagCatalogEntry, DerivedTagCatalogTag } from "../../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type { SearchSemanticsDiscoveryReader } from "../../search-discovery/service.js";
import { normalizeText } from "../../../shared/utils.js";
import { buildFilterText, buildKeyValueDetailLines } from "../node-helpers.js";
import { formatDerivedTagFamilyScopeLabel, getDerivedTagLabels, type DerivedTagLabels } from "./labels.js";
import { buildDerivedTagQuery } from "./query-builders.js";

type DerivedTagVisibilityMode = "nonzeroOnly" | "allAuthored";

type DerivedTagTagCountEntry = {
  tag: DerivedTagCatalogTag;
  liveRecordCount: number;
};

export type DerivedTagFamilyNodeBuilder = (
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  familyEntry: DerivedTagCatalogEntry,
  idPrefix: string,
) => OntologyNode | null;

export function readDerivedTagCountsByScope(
  reader: SearchSemanticsDiscoveryReader,
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
): Map<string, number> {
  return new Map(
    reader
      .discoverFieldValues({
        category,
        subcategory,
        field: "derivedTags",
      })
      .map((entry) => [String(entry.value), entry.count]),
  );
}

function isDerivedTagFamilyInScope(
  subcategory: SearchSubcategory | null,
  familyEntry: DerivedTagCatalogEntry,
): boolean {
  return !subcategory || !familyEntry.subcategories?.length || familyEntry.subcategories.includes(subcategory);
}

function getDerivedTagFamilyNodeId(idPrefix: string, familyEntry: DerivedTagCatalogEntry): string {
  return `${idPrefix}:family:${normalizeText(familyEntry.family)}`;
}

function getDerivedTagTagNodeId(familyNodeId: string, tag: DerivedTagCatalogTag): string {
  return `${familyNodeId}:tag:${tag.value}`;
}

function resolveDerivedTagNodeSubcategory(
  activeSubcategory: SearchSubcategory | null,
  familyEntry: DerivedTagCatalogEntry,
): SearchSubcategory | null {
  if (activeSubcategory) {
    return activeSubcategory;
  }
  return familyEntry.subcategories?.length === 1 ? familyEntry.subcategories[0]! : null;
}

function buildDerivedTagTagCountEntries(
  familyEntry: DerivedTagCatalogEntry,
  liveCountsByTag: ReadonlyMap<string, number>,
  visibilityMode: DerivedTagVisibilityMode,
): DerivedTagTagCountEntry[] {
  return familyEntry.tags
    .map((tag) => ({
      tag,
      liveRecordCount: liveCountsByTag.get(tag.value) ?? 0,
    }))
    .filter((entry) => visibilityMode === "allAuthored" || entry.liveRecordCount > 0);
}

function buildDerivedTagTagNode(options: {
  category: SearchCategory;
  activeSubcategory: SearchSubcategory | null;
  familyEntry: DerivedTagCatalogEntry;
  tagEntry: DerivedTagTagCountEntry;
  familyNodeId: string;
  detailLines: OntologyNode["detailLines"];
  labels: DerivedTagLabels;
}): OntologyNode {
  const { category, activeSubcategory, familyEntry, tagEntry, familyNodeId, detailLines, labels } = options;
  const querySubcategory = resolveDerivedTagNodeSubcategory(activeSubcategory, familyEntry);
  return {
    id: getDerivedTagTagNodeId(familyNodeId, tagEntry.tag),
    kind: "tag",
    label: labels.tagLabel,
    filterText: buildFilterText(
      category,
      activeSubcategory ?? "",
      familyEntry.axis,
      familyEntry.family,
      tagEntry.tag.value,
      tagEntry.tag.description ?? "",
    ),
    detailTitle: "Derived Tag",
    detailLines,
    query: buildDerivedTagQuery(
      category,
      querySubcategory,
      tagEntry.tag.value,
      `Browse records with the ${labels.tagLabel} derived tag`,
    ),
    listLabel: `${labels.tagLabel} | ${tagEntry.liveRecordCount}`,
  };
}

function buildDerivedTagFamilyBrowseNode(options: {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  familyEntry: DerivedTagCatalogEntry;
  idPrefix: string;
  tagNodes: OntologyNode[];
  visibleTagCount: number;
  detailLines: OntologyNode["detailLines"];
  childPresentation?: OntologyNode["childPresentation"];
}): OntologyNode {
  const { category, subcategory, familyEntry, idPrefix, tagNodes, visibleTagCount, detailLines, childPresentation } =
    options;
  const familyLabel = formatOntologySearchVocabularyLabel(familyEntry.family);
  return {
    id: getDerivedTagFamilyNodeId(idPrefix, familyEntry),
    kind: "family",
    label: familyLabel,
    filterText: buildFilterText(
      category,
      subcategory ?? "",
      familyEntry.axis,
      familyEntry.family,
      familyEntry.description,
      ...(familyEntry.subcategories ?? []),
      ...familyEntry.tags.map((tag) => tag.value),
    ),
    listLabel: `${familyLabel} | ${visibleTagCount} tags`,
    detailTitle: "Derived Tag Family",
    detailLines,
    groupValues: {
      axis: familyEntry.axis,
    },
    childSource: { kind: "static", children: tagNodes },
    ...(childPresentation ? { childPresentation } : {}),
  };
}

export function createDerivedTagFamilyNodeBuilder(options: {
  derivedTagCatalogByCategory: ReadonlyMap<SearchCategory, readonly DerivedTagCatalogEntry[]>;
  getDerivedTagCountsByScope: (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ) => ReadonlyMap<string, number>;
  visibilityMode: DerivedTagVisibilityMode;
  detailMode: "catalog" | "prepared";
  countLabel?: string;
}): DerivedTagFamilyNodeBuilder {
  return (category, subcategory, familyEntry, idPrefix) => {
    if (!isDerivedTagFamilyInScope(subcategory, familyEntry)) {
      return null;
    }

    if (familyEntry.tags.length === 0) {
      return null;
    }

    const liveCountsByTag = options.getDerivedTagCountsByScope(category, subcategory);
    const tagEntries = buildDerivedTagTagCountEntries(familyEntry, liveCountsByTag, options.visibilityMode);
    if (tagEntries.length === 0) {
      return null;
    }

    const familyNodeId = getDerivedTagFamilyNodeId(idPrefix, familyEntry);
    const tagNodes = tagEntries.map((tagEntry) => {
      const labels = getDerivedTagLabels(category, subcategory, familyEntry, tagEntry.tag);
      return buildDerivedTagTagNode({
        category,
        activeSubcategory: subcategory,
        familyEntry,
        tagEntry,
        familyNodeId,
        labels,
        detailLines:
          options.detailMode === "catalog"
            ? [
                { text: labels.tagLabel, tone: "section" },
                ...(tagEntry.tag.description ? [{ text: tagEntry.tag.description }] : []),
                { text: `Category: ${labels.categoryLabel}` },
                { text: `Active subcategory: ${labels.activeSubcategoryLabel}` },
                { text: `Family: ${labels.familyLabel}` },
                { text: `Axis: ${labels.axisLabel}` },
                { text: `Family scope: ${labels.familyScopeLabel}` },
                { text: `Assignment mode: ${labels.assignmentModeLabel}` },
                { text: `Live canonical records: ${tagEntry.liveRecordCount}` },
                { text: "Press Enter or o to open the full matching set in the shared result reader." },
              ]
            : [
                { text: labels.tagLabel, tone: "section" },
                ...(tagEntry.tag.description ? [{ text: tagEntry.tag.description }] : []),
                { text: `Family: ${labels.familyLabel}` },
                { text: `Axis: ${labels.axisLabel}` },
                { text: `${options.countLabel ?? "Matching records"}: ${tagEntry.liveRecordCount}` },
              ],
      });
    });

    return buildDerivedTagFamilyBrowseNode({
      category,
      subcategory,
      familyEntry,
      idPrefix,
      tagNodes,
      visibleTagCount: tagEntries.length,
      detailLines:
        options.detailMode === "catalog"
          ? buildKeyValueDetailLines(
              formatOntologySearchVocabularyLabel(familyEntry.family),
              [
                ["Category", formatOntologySearchVocabularyLabel(category)],
                ["Active subcategory", subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"],
                ["Axis", formatOntologySearchVocabularyLabel(familyEntry.axis)],
                ["Family scope", formatDerivedTagFamilyScopeLabel(familyEntry)],
                ["Tags", tagEntries.length],
              ],
              familyEntry.description,
            )
          : buildKeyValueDetailLines(
              formatOntologySearchVocabularyLabel(familyEntry.family),
              [
                ["Category", formatOntologySearchVocabularyLabel(category)],
                ["Subcategory", subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"],
                ["Axis", formatOntologySearchVocabularyLabel(familyEntry.axis)],
                ["Tags", tagNodes.length],
              ],
              familyEntry.description,
            ),
      childPresentation:
        options.detailMode === "prepared"
          ? {
              mode: "grouped",
              groupBy: "axis",
              render: "inline",
            }
          : undefined,
    });
  };
}

export function buildDerivedTagFamilyNodes(options: {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  idPrefix: string;
  derivedTagCatalogByCategory: ReadonlyMap<SearchCategory, readonly DerivedTagCatalogEntry[]>;
  buildDerivedTagFamilyNode: DerivedTagFamilyNodeBuilder;
}): OntologyNode[] {
  return (options.derivedTagCatalogByCategory.get(options.category) ?? [])
    .map((entry) => options.buildDerivedTagFamilyNode(options.category, options.subcategory, entry, options.idPrefix))
    .filter((node): node is OntologyNode => Boolean(node));
}

export function buildCommonDerivedTagShortcutGroup(options: {
  category: SearchCategory;
  commonDerivedTags: readonly { value: string }[];
  derivedTagCatalogByCategory: ReadonlyMap<SearchCategory, readonly DerivedTagCatalogEntry[]>;
  buildDerivedTagFamilyNode: DerivedTagFamilyNodeBuilder;
}): OntologyNode | null {
  const desiredTags = new Set(options.commonDerivedTags.map((entry) => entry.value.toLowerCase()));
  const matchingFamilies = (options.derivedTagCatalogByCategory.get(options.category) ?? [])
    .map((familyEntry) => {
      const matchingTags = familyEntry.tags.filter((tag) => desiredTags.has(tag.value.toLowerCase()));
      if (matchingTags.length === 0) {
        return null;
      }
      return options.buildDerivedTagFamilyNode(
        options.category,
        null,
        {
          ...familyEntry,
          tags: matchingTags,
        },
        `${options.category}:commonDerivedTags`,
      );
    })
    .filter((node): node is OntologyNode => Boolean(node));

  if (matchingFamilies.length === 0) {
    return null;
  }

  const tagCount = matchingFamilies.reduce(
    (total, familyNode) =>
      total + (familyNode.childSource?.kind === "static" ? familyNode.childSource.children.length : 0),
    0,
  );

  return {
    id: `${options.category}:commonDerivedTags`,
    kind: "group",
    label: "Common Derived Tags",
    filterText: buildFilterText(options.category, "common derived tags", ...matchingFamilies.map((node) => node.label)),
    listLabel: `Common derived tags | ${tagCount}`,
    detailTitle: "Common Derived Tags",
    detailLines: buildKeyValueDetailLines(
      "Common Derived Tags",
      [
        ["Category", formatOntologySearchVocabularyLabel(options.category)],
        ["Families", matchingFamilies.length],
        ["Entries", tagCount],
      ],
      "Shortcut into the category's derived-tag family and tag navigator.",
    ),
    childSource: { kind: "static", children: matchingFamilies },
  };
}
