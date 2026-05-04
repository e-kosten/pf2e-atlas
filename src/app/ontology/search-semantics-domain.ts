import {
  CATEGORY_SUBCATEGORY_MAP,
  SEARCH_CATEGORIES,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import {
  type Pf2eApplicationSearchDiscoveryService,
  type SearchDiscoveryField,
  type SearchSemanticsDiscoveryReader,
} from "../search-discovery-service.js";
import { getMetadataFilterSemantics } from "../../search/filters/semantics.js";
import { readMetadataGlossaryArtifact } from "../../data/metadata-glossary.js";
import type { Pf2eDataService } from "../../data/service.js";
import type { SearchSemanticsBootstrapSummaryResult } from "../../data/vocabulary.js";
import type { AppConfig } from "../../domain/config-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import {
  formatMetadataFieldLabel,
  formatMetadataFieldTypeLabel,
  formatOntologySearchVocabularyLabel,
} from "../../domain/presentation-vocabulary.js";
import type { DerivedTagCatalogEntry, DerivedTagCatalogTag } from "../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { buildScopeFilter, findSearchScopeFilter } from "../../domain/search-request-types.js";
import { normalizeText } from "../../shared/utils.js";
import { getOntologyDomainSummary } from "./domain-summaries.js";
import { buildFilterText, buildKeyValueDetailLines, cloneOntologyNode, titleCaseLabel } from "./node-helpers.js";
import {
  buildFieldValueNodes,
  buildMetricDiscoveryGroup,
  buildPackValueNodes,
  buildSearchSemanticsMetadataQuery,
} from "./search-semantics-helpers.js";

type SearchSemanticsDataService = Pick<Pf2eDataService, "getPack" | "listRecords"> & {
  getSearchSemanticsBootstrapSummary: (options?: { traitLimitPerCategory?: number }) => SearchSemanticsBootstrapSummaryResult;
};

type BroadSearchSemanticsDomainOptions = {
  discoveryMode?: SearchFilterDiscoveryMode;
};

type DerivedTagVisibilityMode = "nonzeroOnly" | "allAuthored";

type DerivedTagLabels = {
  categoryLabel: string;
  activeSubcategoryLabel: string;
  familyLabel: string;
  axisLabel: string;
  tagLabel: string;
  familyScopeLabel: string;
  assignmentModeLabel: string;
};

type DerivedTagTagCountEntry = {
  tag: DerivedTagCatalogTag;
  liveRecordCount: number;
};

function loadSearchSemanticsSummary(dataService: SearchSemanticsDataService): SearchSemanticsBootstrapSummaryResult {
  return dataService.getSearchSemanticsBootstrapSummary();
}

function buildDerivedTagQuery(
  category: SearchCategory,
  subcategory: SearchSubcategory | null,
  tag: string,
  label: string,
): OntologyNode["query"] {
  return buildSearchSemanticsMetadataQuery(
    category,
    subcategory,
    label,
    {
      kind: "metadataPredicate",
      predicate: {
        field: "derivedTags",
        op: "includes",
        value: tag,
      },
    },
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

function formatDerivedTagFamilyScopeLabel(familyEntry: DerivedTagCatalogEntry): string {
  return familyEntry.subcategories?.map((entry) => formatOntologySearchVocabularyLabel(entry)).join(", ") ??
    "(all subcategories)";
}

function getDerivedTagLabels(
  category: SearchCategory,
  activeSubcategory: SearchSubcategory | null,
  familyEntry: DerivedTagCatalogEntry,
  tag: DerivedTagCatalogTag,
): DerivedTagLabels {
  return {
    categoryLabel: formatOntologySearchVocabularyLabel(category),
    activeSubcategoryLabel: activeSubcategory ? formatOntologySearchVocabularyLabel(activeSubcategory) : "(all)",
    familyLabel: formatOntologySearchVocabularyLabel(familyEntry.family),
    axisLabel: formatOntologySearchVocabularyLabel(familyEntry.axis),
    tagLabel: formatOntologySearchVocabularyLabel(tag.value),
    familyScopeLabel: formatDerivedTagFamilyScopeLabel(familyEntry),
    assignmentModeLabel: tag.assignmentMode
      ? formatOntologySearchVocabularyLabel(tag.assignmentMode)
      : familyEntry.assignmentMode
        ? formatOntologySearchVocabularyLabel(familyEntry.assignmentMode)
        : "(unspecified)",
  };
}

function readDerivedTagCountsByScope(
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
}): OntologyNode {
  const { category, activeSubcategory, familyEntry, tagEntry, familyNodeId, detailLines } = options;
  const querySubcategory = resolveDerivedTagNodeSubcategory(activeSubcategory, familyEntry);
  const labels = getDerivedTagLabels(category, activeSubcategory, familyEntry, tagEntry.tag);
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
  const { category, subcategory, familyEntry, idPrefix, tagNodes, visibleTagCount, detailLines, childPresentation } = options;
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
    children: tagNodes,
    ...(childPresentation ? { childPresentation } : {}),
  };
}

export function buildSearchSemanticsDomain(
  config: AppConfig,
  dataService: SearchSemanticsDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
  options: BroadSearchSemanticsDomainOptions = {},
): OntologyDomainModel {
  const searchSemanticsReader = discoveryService.createCatalogSearchSemanticsReader();
  const discoveryMode = options.discoveryMode ?? "matching";
  const semantics = getMetadataFilterSemantics();
  const summary = loadSearchSemanticsSummary(dataService);
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const metadataFieldsByName = new Map(semantics.metadataFields.map((entry) => [entry.field, entry]));

  const getCategoryScopedFields = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): readonly SearchDiscoveryField[] => discoveryService.getScopedMetadataFields({ category, subcategory });

  const commonTraitsByCategory = new Map(
    summary.commonTraitsByCategory.map((entry) => [entry.category, entry.traits]),
  );
  const commonDerivedTagsByCategory = new Map(
    summary.commonDerivedTagsByCategory.map((entry) => [entry.category, entry.tags]),
  );
  const derivedTagCatalogByCategory = new Map<SearchCategory, DerivedTagCatalogEntry[]>(
    SEARCH_CATEGORIES.map((category) => [
      category,
      summary.derivedTagCatalog.filter((entry) => entry.category === category),
    ]),
  );
  const liveSubcategoryCountsByCategory = new Map(
    SEARCH_CATEGORIES.map((category) => {
      const subcategoryCounts =
        summary.subcategoryCountsByCategory.find((entry) => entry.category === category)?.subcategories ?? [];
      return [category, new Map(subcategoryCounts.map((entry) => [entry.value, entry.count]))] as const;
    }),
  );
  const derivedTagCountsByScopeCache = new Map<string, Map<string, number>>();

  function getDerivedTagCountsByScope(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): Map<string, number> {
    const cacheKey = `${category}:${subcategory ?? "*"}`;
    const cached = derivedTagCountsByScopeCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const counts = readDerivedTagCountsByScope(searchSemanticsReader, category, subcategory);
    derivedTagCountsByScopeCache.set(cacheKey, counts);
    return counts;
  }

  function buildDerivedTagFamilyNode(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    familyEntry: DerivedTagCatalogEntry,
    idPrefix: string,
  ): OntologyNode | null {
    if (!isDerivedTagFamilyInScope(subcategory, familyEntry)) {
      return null;
    }

    if (familyEntry.tags.length === 0) {
      return null;
    }

    const liveCountsByTag = getDerivedTagCountsByScope(category, subcategory);
    const visibilityMode: DerivedTagVisibilityMode = discoveryMode === "catalog" ? "allAuthored" : "nonzeroOnly";
    const tagEntries = buildDerivedTagTagCountEntries(familyEntry, liveCountsByTag, visibilityMode);
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
        detailLines: [
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
      detailLines: buildKeyValueDetailLines(
        formatOntologySearchVocabularyLabel(familyEntry.family),
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
          ["Active subcategory", subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"],
          ["Axis", formatOntologySearchVocabularyLabel(familyEntry.axis)],
          ["Family scope", formatDerivedTagFamilyScopeLabel(familyEntry)],
          ["Tags", tagEntries.length],
        ],
        familyEntry.description,
      ),
    });
  }

  function buildDerivedTagFamilyNodes(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    idPrefix: string,
  ): OntologyNode[] {
    return (derivedTagCatalogByCategory.get(category) ?? [])
      .map((entry) => buildDerivedTagFamilyNode(category, subcategory, entry, idPrefix))
      .filter((node): node is OntologyNode => Boolean(node));
  }

  function buildMetadataFieldNodes(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return getCategoryScopedFields(category, subcategory).map((fieldSemantics): OntologyNode => {
      const field = fieldSemantics.field;
      const fieldLabel = formatMetadataFieldLabel(field);
      const fieldTypeLabel = formatMetadataFieldTypeLabel(fieldSemantics.fieldType);
      const derivedTagChildren =
        field === "derivedTags" ? buildDerivedTagFamilyNodes(category, subcategory, `${idPrefix}:field:${field}`) : null;
      return {
        id: `${idPrefix}:field:${field}`,
        kind: "field",
        label: fieldLabel,
        filterText: buildFilterText(
          category,
          subcategory ?? "",
          field,
          fieldLabel,
          fieldTypeLabel,
          fieldSemantics.fieldType,
          fieldSemantics.notes ?? "",
          ...(fieldSemantics.subcategories ?? []),
        ),
        listLabel: fieldLabel,
        detailTitle: "Metadata Field Details",
        detailLines: [
          { text: fieldLabel, tone: "section" },
          { text: `Category: ${category}` },
          { text: `Subcategory: ${subcategory ?? "(all)"}` },
          { text: `Field type: ${fieldTypeLabel}` },
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
                  text: "This field exposes a lightweight derived-tag family and tag navigator without embedding record subtrees under each tag.",
                },
              ]
            : []),
        ],
        groupValues: {
          fieldType: fieldTypeLabel,
        },
        ...(field === "derivedTags"
          ? (derivedTagChildren && derivedTagChildren.length > 0
              ? {
                  children: derivedTagChildren,
                  childPresentation: {
                    mode: "grouped",
                    groupBy: "axis",
                    render: "inline",
                  } as const,
                }
              : {})
          : {
              loadChildren: fieldSemantics.discoverable
                ? () => {
                    const liveValues = searchSemanticsReader
                      .discoverFieldValues({
                        category,
                        subcategory,
                        field,
                      })
                      .map((entry) => ({
                        value: String(entry.value),
                        count: entry.count,
                      }));
                    return liveValues.length > 0
                      ? buildFieldValueNodes(
                          dataService,
                          category,
                          subcategory,
                          fieldSemantics,
                          liveValues,
                          metadataGlossary,
                        )
                      : [];
                  }
                : undefined,
            }),
      };
    });
  }

  function buildCommonTraitShortcutGroup(category: SearchCategory): OntologyNode | null {
    const traitFieldSemantics = metadataFieldsByName.get("traits");
    if (!traitFieldSemantics) {
      return null;
    }

    const shortcutNodes = buildFieldValueNodes(
      dataService,
      category,
      null,
      traitFieldSemantics,
      commonTraitsByCategory.get(category) ?? [],
      metadataGlossary,
    ).map((node) => cloneOntologyNode(node, `${category}:commonTraitsShortcut`));

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

  function buildCommonDerivedTagShortcutGroup(category: SearchCategory): OntologyNode | null {
    const desiredTags = new Set(
      (commonDerivedTagsByCategory.get(category) ?? []).map((entry) => entry.value.toLowerCase()),
    );
    const matchingFamilies = (derivedTagCatalogByCategory.get(category) ?? [])
      .map((familyEntry) => {
        const matchingTags = familyEntry.tags.filter((tag) => desiredTags.has(tag.value.toLowerCase()));
        if (matchingTags.length === 0) {
          return null;
        }
        return buildDerivedTagFamilyNode(
          category,
          null,
          {
            ...familyEntry,
            tags: matchingTags,
          },
          `${category}:commonDerivedTags`,
        );
      })
      .filter((node): node is OntologyNode => Boolean(node));

    if (matchingFamilies.length === 0) {
      return null;
    }

    const tagCount = matchingFamilies.reduce((total, familyNode) => total + (familyNode.children?.length ?? 0), 0);

    return {
      id: `${category}:commonDerivedTags`,
      kind: "group",
      label: "Common Derived Tags",
      filterText: buildFilterText(category, "common derived tags", ...matchingFamilies.map((node) => node.label)),
      listLabel: `Common derived tags | ${tagCount}`,
      detailTitle: "Common Derived Tags",
      detailLines: buildKeyValueDetailLines(
        "Common Derived Tags",
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
          ["Families", matchingFamilies.length],
          ["Entries", tagCount],
        ],
        "Shortcut into the category's derived-tag family and tag navigator.",
      ),
      children: matchingFamilies,
    };
  }

  function buildMetricDiscoveryGroups(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    return discoveryService.getMetricDiscoveryGroups({ category, subcategory }).map((group) =>
      buildMetricDiscoveryGroup(dataService, searchSemanticsReader, {
        category,
        subcategory,
        metricField: group.metricField,
        metadataField: group.metadataField,
        label: getMetricDiscoveryGroupLabel(category, group.metricField),
        namespaces: group.namespaces,
      }),
    );
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
      label: formatOntologySearchVocabularyLabel(subcategory),
      filterText: buildFilterText(category, subcategory, ...children.map((node) => node.label)),
      listLabel: `${formatOntologySearchVocabularyLabel(subcategory)} | ${liveSubcategoryCountsByCategory.get(category)?.get(subcategory) ?? 0}`,
      detailTitle: "Subcategory Boundary",
      detailLines: buildKeyValueDetailLines(
        formatOntologySearchVocabularyLabel(subcategory),
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
          ["Subcategory", formatOntologySearchVocabularyLabel(subcategory)],
          ["Live canonical records", liveSubcategoryCountsByCategory.get(category)?.get(subcategory) ?? 0],
          ["Metadata fields", subcategoryMetadataFieldNodes.length],
        ],
        "Browse this subcategory directly, or drill in to its scoped metadata and metric surfaces.",
      ),
      query: {
        label: "Browse this subcategory",
        request: {
          mode: "browse",
          filter: buildScopeFilter(category, subcategory),
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
    const commonTraitGroup = buildCommonTraitShortcutGroup(category);
    if (commonTraitGroup) {
      children.push(commonTraitGroup);
    }
    const commonDerivedTagGroup = buildCommonDerivedTagShortcutGroup(category);
    if (commonDerivedTagGroup) {
      children.push(commonDerivedTagGroup);
    }
    if (metadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(category, "metadata fields", ...categoryFields.map((field) => field.field)),
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
      filterText: buildFilterText(category, ...categoryFields.map((field) => field.field)),
      listLabel: `${formatOntologySearchVocabularyLabel(category)} | ${children.length} groups`,
      detailTitle: "Search Semantics",
      detailLines: buildKeyValueDetailLines(
        titleCaseLabel(category),
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
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
    ...getOntologyDomainSummary("searchSemantics"),
    rootNodes,
  };
}

function resolvePreparedSearchFilterExplorerScope(
  request: Readonly<SearchRequest>,
): { category: SearchCategory; subcategory: SearchSubcategory | null } | null {
  const scope = findSearchScopeFilter(request.filter);
  if (!scope) {
    return null;
  }

  const category = normalizeSearchCategory(scope.category);
  const subcategory =
    scope.subcategory.kind === "eq" ? normalizeSearchSubcategory(scope.subcategory.value) : null;
  if (!category || (scope.subcategory.kind === "eq" && !subcategory)) {
    return null;
  }

  return {
    category,
    subcategory,
  };
}

function getPreparedSearchFilterExplorerCountLabel(mode: SearchFilterDiscoveryMode): string {
  return mode === "matching" ? "Matching records" : "Applicable records";
}

export async function buildPreparedSearchFilterExplorerDomain(
  config: AppConfig,
  dataService: SearchSemanticsDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
  options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
  },
): Promise<OntologyDomainModel> {
  const scope = resolvePreparedSearchFilterExplorerScope(options.request);
  if (!scope) {
    throw new Error("Search filter explorer requires a category-scoped canonical search request.");
  }

  const preparedReader = await discoveryService.prepareSearchSemanticsReader(options.request, options.discoveryMode);
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const summary = loadSearchSemanticsSummary(dataService);
  const countLabel = getPreparedSearchFilterExplorerCountLabel(options.discoveryMode);
  const derivedTagCatalogByCategory = new Map<SearchCategory, DerivedTagCatalogEntry[]>(
    SEARCH_CATEGORIES.map((category) => [
      category,
      summary.derivedTagCatalog.filter((entry) => entry.category === category),
    ]),
  );
  const derivedTagCountsByScopeCache = new Map<string, Map<string, number>>();

  const getCategoryScopedFields = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): readonly SearchDiscoveryField[] => discoveryService.getScopedMetadataFields({ category, subcategory });

  function getDerivedTagCountsByScope(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): Map<string, number> {
    const cacheKey = `${category}:${subcategory ?? "*"}`;
    const cached = derivedTagCountsByScopeCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const counts = readDerivedTagCountsByScope(preparedReader, category, subcategory);
    derivedTagCountsByScopeCache.set(cacheKey, counts);
    return counts;
  }

  function buildDerivedTagFamilyNode(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    familyEntry: DerivedTagCatalogEntry,
    idPrefix: string,
  ): OntologyNode | null {
    if (!isDerivedTagFamilyInScope(subcategory, familyEntry)) {
      return null;
    }

    const liveCountsByTag = getDerivedTagCountsByScope(category, subcategory);
    const tagEntries = buildDerivedTagTagCountEntries(familyEntry, liveCountsByTag, "nonzeroOnly");
    const familyNodeId = getDerivedTagFamilyNodeId(idPrefix, familyEntry);
    const tagNodes = tagEntries.map((tagEntry) => {
      const labels = getDerivedTagLabels(category, subcategory, familyEntry, tagEntry.tag);
      return buildDerivedTagTagNode({
        category,
        activeSubcategory: subcategory,
        familyEntry,
        tagEntry,
        familyNodeId,
        detailLines: [
          { text: labels.tagLabel, tone: "section" },
          ...(tagEntry.tag.description ? [{ text: tagEntry.tag.description }] : []),
          { text: `Family: ${labels.familyLabel}` },
          { text: `Axis: ${labels.axisLabel}` },
          { text: `${countLabel}: ${tagEntry.liveRecordCount}` },
        ],
      });
    });
    if (tagNodes.length === 0) {
      return null;
    }

    return buildDerivedTagFamilyBrowseNode({
      category,
      subcategory,
      familyEntry,
      idPrefix,
      tagNodes,
      visibleTagCount: tagNodes.length,
      detailLines: buildKeyValueDetailLines(
        formatOntologySearchVocabularyLabel(familyEntry.family),
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
          ["Subcategory", subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"],
          ["Axis", formatOntologySearchVocabularyLabel(familyEntry.axis)],
          ["Tags", tagNodes.length],
        ],
        familyEntry.description,
      ),
      childPresentation: {
        mode: "grouped",
        groupBy: "axis",
        render: "inline",
      },
    });
  }

  function buildDerivedTagFamilyNodes(
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    idPrefix: string,
  ): OntologyNode[] {
    return (derivedTagCatalogByCategory.get(category) ?? [])
      .map((entry) => buildDerivedTagFamilyNode(category, subcategory, entry, idPrefix))
      .filter((node): node is OntologyNode => Boolean(node));
  }

  function buildMetadataFieldNodes(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return getCategoryScopedFields(category, subcategory).map((fieldSemantics): OntologyNode => {
      const field = fieldSemantics.field;
      const fieldLabel = formatMetadataFieldLabel(field);
      const fieldTypeLabel = formatMetadataFieldTypeLabel(fieldSemantics.fieldType);
      const derivedTagChildren =
        field === "derivedTags" ? buildDerivedTagFamilyNodes(category, subcategory, `${idPrefix}:field:${field}`) : null;
      return {
        id: `${idPrefix}:field:${field}`,
        kind: "field",
        label: fieldLabel,
        filterText: buildFilterText(category, subcategory ?? "", field, fieldLabel, fieldTypeLabel),
        listLabel: fieldLabel,
        detailTitle: "Metadata Field Details",
        detailLines: [
          { text: fieldLabel, tone: "section" },
          { text: `Category: ${formatOntologySearchVocabularyLabel(category)}` },
          { text: `Subcategory: ${subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"}` },
          { text: `Field type: ${fieldTypeLabel}` },
          {
            text:
              options.discoveryMode === "matching"
                ? "Drill in to browse values from the current matching query context."
                : "Drill in to browse values from the current applicability slice.",
          },
        ],
        groupValues: {
          fieldType: fieldTypeLabel,
        },
        ...(field === "derivedTags"
          ? (derivedTagChildren && derivedTagChildren.length > 0
              ? {
                  children: derivedTagChildren,
                  childPresentation: {
                    mode: "grouped",
                    groupBy: "axis",
                    render: "inline",
                  } as const,
                }
              : {})
          : {
              loadChildren: fieldSemantics.discoverable
                ? () =>
                    buildFieldValueNodes(
                      dataService,
                      category,
                      subcategory,
                      fieldSemantics,
                      preparedReader.discoverFieldValues({
                        category,
                        subcategory,
                        field,
                      }).map((entry) => ({
                        value: String(entry.value),
                        count: entry.count,
                      })),
                      metadataGlossary,
                      { countLabel },
                    )
                : undefined,
            }),
      };
    });
  }

  function buildMetricDiscoveryGroups(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    return discoveryService.getMetricDiscoveryGroups({ category, subcategory }).map((group) =>
      buildMetricDiscoveryGroup(dataService, preparedReader, {
        category,
        subcategory,
        metricField: group.metricField,
        metadataField: group.metadataField,
        label: getMetricDiscoveryGroupLabel(category, group.metricField),
        namespaces: group.namespaces,
        countLabel,
      }),
    );
  }

  function buildPackFieldNode(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return {
      id: `${idPrefix}:pack`,
      kind: "field",
      label: "Pack",
      filterText: buildFilterText(category, subcategory ?? "", "pack", "source", "compendium"),
      listLabel: "Pack",
      detailTitle: "Pack Details",
      detailLines: [
        { text: "Pack", tone: "section" },
        { text: `Category: ${formatOntologySearchVocabularyLabel(category)}` },
        { text: `Subcategory: ${subcategory ? formatOntologySearchVocabularyLabel(subcategory) : "(all)"}` },
        {
          text:
            options.discoveryMode === "matching"
              ? "Drill in to browse packs from the current matching query context."
              : "Drill in to browse packs from the current applicability slice.",
        },
      ],
      loadChildren: () =>
        buildPackValueNodes(
          dataService,
          category,
          subcategory,
          preparedReader.discoverFieldValues({
            category,
            subcategory,
            field: "packs",
          }).map((entry) => ({
            value: String(entry.value),
            count: entry.count,
          })),
          { countLabel },
        ),
    };
  }

  function buildSubcategoryNode(category: SearchCategory, subcategory: SearchSubcategory): OntologyNode {
    const metadataFieldNodes = buildMetadataFieldNodes(category, subcategory);
    const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, subcategory);
    const packFieldNode = buildPackFieldNode(category, subcategory);
    const children: OntologyNode[] = [];

    if (metadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:${subcategory}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(category, subcategory, "metadata fields"),
        listLabel: `Metadata fields | ${metadataFieldNodes.length}`,
        detailTitle: "Metadata Fields",
        detailLines: buildKeyValueDetailLines(
          "Metadata Fields",
          [
            ["Category", formatOntologySearchVocabularyLabel(category)],
            ["Subcategory", formatOntologySearchVocabularyLabel(subcategory)],
            ["Fields", metadataFieldNodes.length],
          ],
          "Use these typed fields within the active subcategory scope.",
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
    children.push(packFieldNode);

    return {
      id: `${category}:subcategory:${subcategory}`,
      kind: "subcategory",
      label: formatOntologySearchVocabularyLabel(subcategory),
      filterText: buildFilterText(category, subcategory, ...children.map((node) => node.label)),
      listLabel: formatOntologySearchVocabularyLabel(subcategory),
      detailTitle: "Subcategory Boundary",
      detailLines: buildKeyValueDetailLines(
        formatOntologySearchVocabularyLabel(subcategory),
        [
          ["Category", formatOntologySearchVocabularyLabel(category)],
          ["Subcategory", formatOntologySearchVocabularyLabel(subcategory)],
        ],
        options.discoveryMode === "matching"
          ? "This explorer is showing values from the current matching query context."
          : "This explorer is showing values from the current applicability slice.",
      ),
      children,
    };
  }

  const category = scope.category;
  const categoryFields = getCategoryScopedFields(category, null);
  const metadataFieldNodes = buildMetadataFieldNodes(category, null);
  const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, null);
  const packFieldNode = buildPackFieldNode(category, null);
  const children: OntologyNode[] = [];

  if (scope.subcategory) {
    children.push({
      id: `${category}:subcategories`,
      kind: "group",
      label: "Subcategories",
      filterText: buildFilterText(category, scope.subcategory, "subcategories"),
      listLabel: "Subcategories | 1",
      detailTitle: "Category Boundaries",
      detailLines: buildKeyValueDetailLines("Subcategories", [
        ["Category", formatOntologySearchVocabularyLabel(category)],
        ["Subcategories", 1],
      ]),
      children: [buildSubcategoryNode(category, scope.subcategory)],
    });
  } else {
    if (metadataFieldNodes.length > 0) {
      children.push({
        id: `${category}:metadataFields`,
        kind: "group",
        label: "Metadata Fields",
        filterText: buildFilterText(category, "metadata fields", ...categoryFields.map((field) => field.field)),
        listLabel: `Metadata fields | ${categoryFields.length}`,
        detailTitle: "Metadata Fields",
        detailLines: buildKeyValueDetailLines(
          "Metadata Fields",
          [
            ["Category", formatOntologySearchVocabularyLabel(category)],
            ["Fields", categoryFields.length],
          ],
          options.discoveryMode === "matching"
            ? "Showing values from the current matching query context."
            : "Showing values from the current applicability slice.",
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
    children.push(packFieldNode);
  }

  return {
    ...getOntologyDomainSummary("searchSemantics"),
    rootNodes: [
      {
        id: `searchSemantics:${category}`,
        kind: "category",
        label: titleCaseLabel(category),
        shortLabel: category,
        filterText: buildFilterText(category, ...children.map((node) => node.label)),
        listLabel: `${formatOntologySearchVocabularyLabel(category)} | ${children.length} groups`,
        detailTitle: "Search Semantics",
        detailLines: buildKeyValueDetailLines(
          titleCaseLabel(category),
          [
            ["Category", formatOntologySearchVocabularyLabel(category)],
            ["Metadata fields", categoryFields.length],
            ["Metric discovery groups", metricDiscoveryGroups.length],
            ["Discovery mode", options.discoveryMode],
          ],
          "Use the shared explorer to compose scoped filters from this prepared value space.",
        ),
        children,
      },
    ],
  };
}
