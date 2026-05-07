import {
  CATEGORY_SUBCATEGORY_MAP,
  SEARCH_CATEGORIES,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../../domain/categories.js";
import { getMetadataFilterSemantics } from "../../../domain/metadata-field-catalog.js";
import type { MetadataGlossaryArtifact } from "../../../domain/metadata-glossary-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../../domain/ontology-types.js";
import { getMetricDiscoveryGroupLabel } from "../../../domain/metric-discovery-group-label.js";
import { formatOntologySearchVocabularyLabel } from "../../../domain/presentation-vocabulary.js";
import type { DerivedTagCatalogEntry } from "../../../domain/record-types.js";
import { findSearchScopeFilter } from "../../../domain/search-request-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import { readMetadataGlossaryArtifact } from "../../../data/metadata-glossary.js";
import type { Pf2eApplicationSearchDiscoveryService, SearchDiscoveryField } from "../../search-discovery/service.js";
import { getOntologyDomainSummary } from "../domain-summaries.js";
import { buildFilterText, buildKeyValueDetailLines, cloneOntologyNode, titleCaseLabel } from "../node-helpers.js";
import {
  buildCommonDerivedTagShortcutGroup,
  createDerivedTagFamilyNodeBuilder,
  readDerivedTagCountsByScope,
} from "./derived-tag-builder.js";
import { buildMetadataFieldNodes, buildPreparedMetadataFieldNodes } from "./field-domain-builder.js";
import { buildMetricDiscoveryGroup } from "./metric-domain-builder.js";
import { buildSearchSemanticsScopeQuery } from "./query-builders.js";
import { buildPackFieldNode, buildPreparedPackFieldNode } from "./pack-domain-builder.js";
import type { SearchSemanticsDataService } from "./types.js";
import { buildFieldValueNodes } from "./value-node-builder.js";

type BroadSearchSemanticsDomainOptions = {
  discoveryMode?: SearchFilterDiscoveryMode;
};

function loadSearchSemanticsSummary(dataService: SearchSemanticsDataService) {
  return dataService.getSearchSemanticsBootstrapSummary();
}

function buildDerivedTagCatalogByCategory(
  entries: readonly DerivedTagCatalogEntry[],
): Map<SearchCategory, DerivedTagCatalogEntry[]> {
  return new Map(
    SEARCH_CATEGORIES.map((category) => [category, entries.filter((entry) => entry.category === category)]),
  );
}

function buildCommonTraitShortcutGroup(options: {
  dataService: SearchSemanticsDataService;
  category: SearchCategory;
  commonTraits: readonly { value: string; count: number }[];
  metadataGlossary: MetadataGlossaryArtifact | null;
  traitFieldSemantics: SearchDiscoveryField | undefined;
}): OntologyNode | null {
  if (!options.traitFieldSemantics) {
    return null;
  }

  const shortcutNodes = buildFieldValueNodes(
    options.dataService,
    options.category,
    null,
    options.traitFieldSemantics,
    options.commonTraits,
    options.metadataGlossary,
  ).map((node) => cloneOntologyNode(node, `${options.category}:commonTraitsShortcut`));

  if (shortcutNodes.length === 0) {
    return null;
  }

  return {
    id: `${options.category}:commonTraits`,
    kind: "group",
    label: "Common Traits",
    filterText: buildFilterText(options.category, "common traits", ...shortcutNodes.map((node) => node.label)),
    listLabel: `Common traits | ${shortcutNodes.length}`,
    detailTitle: "Common Traits",
    detailLines: buildKeyValueDetailLines(
      "Common Traits",
      [
        ["Category", options.category],
        ["Entries", shortcutNodes.length],
      ],
      "Shortcut into the canonical trait value space for this category.",
    ),
    childSource: { kind: "static", children: shortcutNodes },
  };
}

export function buildSearchSemanticsDomain(
  config: { indexPath: string },
  dataService: SearchSemanticsDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
  options: BroadSearchSemanticsDomainOptions = {},
): OntologyDomainModel {
  const searchSemanticsReader = discoveryService.createCatalogSearchSemanticsReader();
  const discoveryMode = options.discoveryMode ?? "matching";
  const summary = loadSearchSemanticsSummary(dataService);
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const metadataFieldsByName = new Map(
    getMetadataFilterSemantics().metadataFields.map((entry) => [entry.field, entry]),
  );
  const commonTraitsByCategory = new Map(summary.commonTraitsByCategory.map((entry) => [entry.category, entry.traits]));
  const commonDerivedTagsByCategory = new Map(
    summary.commonDerivedTagsByCategory.map((entry) => [entry.category, entry.tags]),
  );
  const derivedTagCatalogByCategory = buildDerivedTagCatalogByCategory(summary.derivedTagCatalog);
  const liveSubcategoryCountsByCategory = new Map(
    SEARCH_CATEGORIES.map((category) => {
      const subcategoryCounts =
        summary.subcategoryCountsByCategory.find((entry) => entry.category === category)?.subcategories ?? [];
      return [category, new Map(subcategoryCounts.map((entry) => [entry.value, entry.count]))] as const;
    }),
  );
  const derivedTagCountsByScopeCache = new Map<string, Map<string, number>>();

  const getCategoryScopedFields = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): readonly SearchDiscoveryField[] => discoveryService.getScopedMetadataFields({ category, subcategory });

  const getDerivedTagCountsByScope = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): Map<string, number> => {
    const cacheKey = `${category}:${subcategory ?? "*"}`;
    const cached = derivedTagCountsByScopeCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const counts = readDerivedTagCountsByScope(searchSemanticsReader, category, subcategory);
    derivedTagCountsByScopeCache.set(cacheKey, counts);
    return counts;
  };

  const buildDerivedTagFamilyNode = createDerivedTagFamilyNodeBuilder({
    derivedTagCatalogByCategory,
    getDerivedTagCountsByScope,
    visibilityMode: discoveryMode === "catalog" ? "allAuthored" : "nonzeroOnly",
    detailMode: "catalog",
  });

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
    const subcategoryMetadataFieldNodes = buildMetadataFieldNodes({
      dataService,
      searchSemanticsReader,
      category,
      subcategory,
      getCategoryScopedFields,
      metadataGlossary,
      derivedTagCatalogByCategory,
      buildDerivedTagFamilyNode,
    });
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
        childSource: { kind: "static", children: subcategoryMetadataFieldNodes },
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
      query: buildSearchSemanticsScopeQuery(category, subcategory, "Browse this subcategory"),
      childSource: { kind: "static", children },
    };
  }

  const rootNodes = SEARCH_CATEGORIES.map((category): OntologyNode => {
    const categoryFields = getCategoryScopedFields(category, null);
    const metadataFieldNodes = buildMetadataFieldNodes({
      dataService,
      searchSemanticsReader,
      category,
      subcategory: null,
      getCategoryScopedFields,
      metadataGlossary,
      derivedTagCatalogByCategory,
      buildDerivedTagFamilyNode,
    });
    const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, null);
    const packFieldNode = buildPackFieldNode({ dataService, searchSemanticsReader, category });

    const subcategoryNodes = CATEGORY_SUBCATEGORY_MAP[category].map((subcategory) =>
      buildSubcategoryNode(category, subcategory),
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
        childSource: { kind: "static", children: subcategoryNodes },
      });
    }

    const commonTraitGroup = buildCommonTraitShortcutGroup({
      dataService,
      category,
      commonTraits: commonTraitsByCategory.get(category) ?? [],
      metadataGlossary,
      traitFieldSemantics: metadataFieldsByName.get("traits"),
    });
    if (commonTraitGroup) {
      children.push(commonTraitGroup);
    }

    const commonDerivedTagGroup = buildCommonDerivedTagShortcutGroup({
      category,
      commonDerivedTags: commonDerivedTagsByCategory.get(category) ?? [],
      derivedTagCatalogByCategory,
      buildDerivedTagFamilyNode,
    });
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
        childSource: { kind: "static", children: metadataFieldNodes },
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }
    children.push(packFieldNode);
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
      childSource: { kind: "static", children },
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
  const subcategory = scope.subcategory.kind === "eq" ? normalizeSearchSubcategory(scope.subcategory.value) : null;
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

function normalizePreparedTargetField(field: string): string {
  if (field === "actorMetric" || field === "actorMetricCompare") {
    return "actorMetrics";
  }
  if (field === "itemMetric" || field === "itemMetricCompare") {
    return "itemMetrics";
  }
  if (field === "pack") {
    return "packs";
  }
  return field;
}

export async function buildPreparedSearchFilterExplorerDomain(
  config: { indexPath: string },
  dataService: SearchSemanticsDataService,
  discoveryService: Pf2eApplicationSearchDiscoveryService,
  options: {
    request: Readonly<SearchRequest>;
    discoveryMode: SearchFilterDiscoveryMode;
    targetFields?: readonly string[];
  },
): Promise<OntologyDomainModel> {
  const scope = resolvePreparedSearchFilterExplorerScope(options.request);
  if (!scope) {
    throw new Error("Search filter explorer requires a category-scoped canonical search request.");
  }

  const targetFields = options.targetFields
    ? new Set(options.targetFields.map((field) => normalizePreparedTargetField(field)))
    : null;
  const preparedReader = await discoveryService.prepareSearchSemanticsReader(options.request, options.discoveryMode, {
    targetFields: options.targetFields,
  });
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const summary = loadSearchSemanticsSummary(dataService);
  const countLabel = getPreparedSearchFilterExplorerCountLabel(options.discoveryMode);
  const matchingRequest = options.discoveryMode === "matching" ? options.request : undefined;
  const derivedTagCatalogByCategory = buildDerivedTagCatalogByCategory(summary.derivedTagCatalog);
  const derivedTagCountsByScopeCache = new Map<string, Map<string, number>>();

  const getCategoryScopedFields = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): readonly SearchDiscoveryField[] =>
    discoveryService
      .getScopedMetadataFields({ category, subcategory })
      .filter((field) => !targetFields || targetFields.has(field.field));

  const shouldIncludeMetricGroup = (metricField: "actorMetrics" | "itemMetrics"): boolean =>
    !targetFields || targetFields.has(metricField);

  const shouldIncludePackField = (): boolean => !targetFields || targetFields.has("packs");

  const getDerivedTagCountsByScope = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ): Map<string, number> => {
    const cacheKey = `${category}:${subcategory ?? "*"}`;
    const cached = derivedTagCountsByScopeCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const counts = readDerivedTagCountsByScope(preparedReader, category, subcategory);
    derivedTagCountsByScopeCache.set(cacheKey, counts);
    return counts;
  };

  const buildDerivedTagFamilyNode = createDerivedTagFamilyNodeBuilder({
    derivedTagCatalogByCategory,
    getDerivedTagCountsByScope,
    visibilityMode: "nonzeroOnly",
    detailMode: "prepared",
    countLabel,
  });

  function buildMetricDiscoveryGroups(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    return discoveryService
      .getMetricDiscoveryGroups({ category, subcategory })
      .filter((group) => shouldIncludeMetricGroup(group.metricField))
      .map((group) =>
        buildMetricDiscoveryGroup(dataService, preparedReader, {
          category,
          subcategory,
          metricField: group.metricField,
          metadataField: group.metadataField,
          label: getMetricDiscoveryGroupLabel(category, group.metricField),
          namespaces: group.namespaces,
          countLabel,
          matchingRequest,
        }),
      );
  }

  function buildSubcategoryNode(category: SearchCategory, subcategory: SearchSubcategory): OntologyNode {
    const metadataFieldNodes = buildPreparedMetadataFieldNodes({
      dataService,
      preparedReader,
      category,
      subcategory,
      getCategoryScopedFields,
      metadataGlossary,
      derivedTagCatalogByCategory,
      buildDerivedTagFamilyNode,
      countLabel,
      matchingRequest,
      discoveryMode: options.discoveryMode,
    });
    const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, subcategory);
    const packFieldNode = shouldIncludePackField()
      ? buildPreparedPackFieldNode({
          dataService,
          preparedReader,
          category,
          subcategory,
          countLabel,
          matchingRequest,
          discoveryMode: options.discoveryMode,
        })
      : null;
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
        childSource: { kind: "static", children: metadataFieldNodes },
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }

    children.push(...metricDiscoveryGroups);
    if (packFieldNode) {
      children.push(packFieldNode);
    }

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
      childSource: { kind: "static", children },
    };
  }

  const category = scope.category;
  const categoryFields = getCategoryScopedFields(category, null);
  const metadataFieldNodes = buildPreparedMetadataFieldNodes({
    dataService,
    preparedReader,
    category,
    subcategory: null,
    getCategoryScopedFields,
    metadataGlossary,
    derivedTagCatalogByCategory,
    buildDerivedTagFamilyNode,
    countLabel,
    matchingRequest,
    discoveryMode: options.discoveryMode,
  });
  const metricDiscoveryGroups = buildMetricDiscoveryGroups(category, null);
  const packFieldNode = shouldIncludePackField()
    ? buildPreparedPackFieldNode({
        dataService,
        preparedReader,
        category,
        subcategory: null,
        countLabel,
        matchingRequest,
        discoveryMode: options.discoveryMode,
      })
    : null;
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
      childSource: { kind: "static", children: [buildSubcategoryNode(category, scope.subcategory)] },
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
        childSource: { kind: "static", children: metadataFieldNodes },
        childPresentation: {
          mode: "grouped",
          groupBy: "fieldType",
          render: "inline",
        },
      });
    }
    children.push(...metricDiscoveryGroups);
    if (packFieldNode) {
      children.push(packFieldNode);
    }
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
        childSource: { kind: "static", children },
      },
    ],
  };
}
