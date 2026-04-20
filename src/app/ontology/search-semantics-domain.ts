import { CATEGORY_SUBCATEGORY_MAP, SEARCH_CATEGORIES, normalizeSearchCategory } from "../../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import { readMetadataGlossaryArtifact } from "../../data/metadata-glossary.js";
import type { Pf2eDataService } from "../../data/service.js";
import type { AppConfig } from "../../domain/config-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { normalizeText } from "../../shared/utils.js";
import { getOntologyDomainSummary } from "./domain-summaries.js";
import { buildFilterText, buildKeyValueDetailLines, cloneOntologyNode, titleCaseLabel } from "./node-helpers.js";
import {
  buildFieldValueNodes,
  buildMetricDiscoveryGroup,
  buildSearchSemanticsMetadataQuery,
  getTraitGlossaryEntry,
} from "./search-semantics-helpers.js";

type SearchSemanticsDataService = Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords">;

export function buildSearchSemanticsDomain(
  config: AppConfig,
  dataService: SearchSemanticsDataService,
  loadDerivedTagsDomain: () => OntologyDomainModel,
): OntologyDomainModel {
  const semantics = getMetadataFilterSemantics();
  const vocabulary = dataService.getSearchVocabulary();
  let derivedTagDomain: OntologyDomainModel | null;
  try {
    derivedTagDomain = loadDerivedTagsDomain();
  } catch {
    derivedTagDomain = null;
  }
  const metadataGlossary = readMetadataGlossaryArtifact(config.indexPath);
  const metadataFieldsByName = new Map(semantics.metadataFields.map((entry) => [entry.field, entry]));
  const filterValuesCache = new Map<string, readonly { value: string; count: number }[]>();

  const getCachedFilterValues = (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
    field: MetadataFieldSemantics["field"],
  ): readonly { value: string; count: number }[] => {
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
  const derivedTagCatalogByCategoryAndFamily = new Map<string, (typeof vocabulary.derivedTagCatalog)[number]>(
    vocabulary.derivedTagCatalog.map((entry) => [`${entry.category}:${normalizeText(entry.family)}`, entry] as const),
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

  function buildScopedDerivedTagNode(
    node: OntologyNode,
    options: {
      category: SearchCategory;
      subcategory: SearchSubcategory | null;
    },
  ): OntologyNode | null {
    const { category, subcategory } = options;

    if (subcategory && node.kind === "record") {
      const recordSubcategory = node.query?.filters.subcategory ?? null;
      if (recordSubcategory !== subcategory) {
        return null;
      }
    }

    const familyCatalogEntry =
      node.kind === "family"
        ? derivedTagCatalogByCategoryAndFamily.get(`${category}:${normalizeText(node.label)}`)
        : undefined;
    if (
      subcategory &&
      familyCatalogEntry?.subcategories?.length &&
      !familyCatalogEntry.subcategories.includes(subcategory)
    ) {
      return null;
    }

    const cloned = cloneOntologyNode(node);
    const query =
      subcategory && cloned.query?.kind === "listRecords" && cloned.query.filters.category === category
        ? {
            ...cloned.query,
            filters: {
              ...cloned.query.filters,
              subcategory,
            },
          }
        : cloned.query;

    if (!node.children) {
      return {
        ...cloned,
        query,
      };
    }

    const allowedFamilyTags = familyCatalogEntry
      ? new Set(familyCatalogEntry.tags.map((tag) => normalizeText(tag.value)))
      : null;
    const scopedChildren = node.children
      .filter(
        (child) =>
          !(node.kind === "family" && allowedFamilyTags && child.kind === "tag") ||
          allowedFamilyTags.has(normalizeText(child.label)),
      )
      .map((child) => buildScopedDerivedTagNode(child, options))
      .filter((child): child is OntologyNode => Boolean(child));

    if (node.kind === "family" && scopedChildren.length === 0) {
      return null;
    }

    return {
      ...cloned,
      query,
      children: scopedChildren,
    };
  }

  function buildMetadataFieldNodes(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return getCategoryScopedFields(category, subcategory).map((field): OntologyNode => {
      const fieldSemantics = metadataFieldsByName.get(field)!;
      const derivedTagCategoryNode = field === "derivedTags" ? derivedTagCategoryNodes.get(category) : undefined;
      const derivedTagChildren =
        field === "derivedTags" && derivedTagCategoryNode?.children
          ? derivedTagCategoryNode.children
              .map((node) => buildScopedDerivedTagNode(node, { category, subcategory }))
              .filter((node): node is OntologyNode => Boolean(node))
              .map((node) => cloneOntologyNode(node, `${idPrefix}:field:${field}`))
          : null;
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
        ...(field === "derivedTags" && derivedTagCategoryNode && derivedTagChildren
          ? {
              children: derivedTagChildren,
              childPresentation: derivedTagCategoryNode.childPresentation
                ? { ...derivedTagCategoryNode.childPresentation }
                : undefined,
            }
          : {
              loadChildren: fieldSemantics.discoverable
                ? () => {
                    const liveValues = getCachedFilterValues(category, subcategory, field);
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

  function buildCommonTraitShortcutGroup(
    category: SearchCategory,
    metadataFieldNodes: OntologyNode[],
  ): OntologyNode | null {
    const traitFieldNode = metadataFieldNodes.find((node) => node.label === "traits");
    const traitNodesByLabel = new Map(
      (traitFieldNode?.loadChildren?.() ?? []).map((node) => [normalizeText(node.label), node]),
    );
    const shortcutNodes = (commonTraitsByCategory.get(category) ?? [])
      .map((entry) => {
        const glossaryLabel = getTraitGlossaryEntry(metadataGlossary, entry.value)?.label ?? entry.value;
        return traitNodesByLabel.get(normalizeText(glossaryLabel));
      })
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
      (commonDerivedTagsByCategory.get(category) ?? []).map((entry) => entry.value.toLowerCase()),
    );
    const matchingFamilies = derivedTagFieldNode.children.flatMap((familyNode) => {
      const matchingTags = familyNode.children?.filter((tagNode) => desiredTags.has(tagNode.label.toLowerCase())) ?? [];
      if (matchingTags.length === 0) {
        return [];
      }
      return [
        {
          ...cloneOntologyNode(familyNode, `${category}:commonDerivedTagsShortcut`),
          children: matchingTags.map((tagNode) => cloneOntologyNode(tagNode, `${category}:commonDerivedTagsShortcut`)),
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

  function buildBooleanGroupNodes(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return (Object.entries(semantics.booleanGroups) as Array<[keyof typeof semantics.booleanGroups, string]>).map(
      ([groupName, description]) => ({
        id: `${idPrefix}:booleanGroup:${groupName}`,
        kind: "booleanGroup",
        label: groupName,
        filterText: buildFilterText(category, subcategory ?? "", groupName, description),
        listLabel: groupName,
        detailTitle: "Boolean Group Details",
        detailLines: buildKeyValueDetailLines(
          titleCaseLabel(groupName),
          [
            ["Category", category],
            ["Subcategory", subcategory ?? "(all)"],
          ],
          description,
        ),
      }),
    );
  }

  function buildAdvancedPredicateNodes(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const idPrefix = subcategory ? `${category}:${subcategory}` : category;
    return semantics.advancedPredicates
      .filter((predicate) => predicate.categories.includes(category))
      .map((predicate) => ({
        id: `${idPrefix}:advanced:${predicate.name}`,
        kind: "advancedPredicate",
        label: predicate.name,
        filterText: buildFilterText(
          category,
          subcategory ?? "",
          predicate.name,
          predicate.description,
          ...predicate.operators,
        ),
        listLabel: `${predicate.name} | ${predicate.operators.join(", ")}`,
        detailTitle: "Advanced Predicate Details",
        detailLines: [
          { text: predicate.name, tone: "section" },
          { text: predicate.description },
          { text: `Category: ${category}` },
          { text: `Subcategory: ${subcategory ?? "(all)"}` },
          { text: `Operators: ${predicate.operators.join(", ")}` },
          { text: "Press Enter or o to open the full matching set in the shared result reader." },
        ],
        query: buildSearchSemanticsMetadataQuery(
          category,
          subcategory,
          `Browse records matching the ${predicate.name} example`,
          predicate.example,
        ),
      }));
  }

  function buildMetricDiscoveryGroups(category: SearchCategory, subcategory: SearchSubcategory | null): OntologyNode[] {
    const groups: OntologyNode[] = [];
    if (
      semantics.actorMetricDiscovery &&
      semantics.advancedPredicates.some(
        (predicate) => predicate.name === "actorMetric" && predicate.categories.includes(category),
      )
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
      semantics.advancedPredicates.some(
        (predicate) => predicate.name === "itemMetric" && predicate.categories.includes(category),
      )
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
    const booleanGroupNodes = buildBooleanGroupNodes(category, subcategory);
    const advancedPredicateNodes = buildAdvancedPredicateNodes(category, subcategory);
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

    if (booleanGroupNodes.length > 0) {
      children.push({
        id: `${category}:${subcategory}:booleanGroups`,
        kind: "group",
        label: "Boolean Groups",
        filterText: buildFilterText(category, subcategory, "boolean groups"),
        listLabel: `Boolean groups | ${booleanGroupNodes.length}`,
        detailTitle: "Boolean Groups",
        detailLines: buildKeyValueDetailLines(
          "Boolean Groups",
          [
            ["Category", category],
            ["Subcategory", subcategory],
            ["Groups", booleanGroupNodes.length],
          ],
          "Explore the logical group operators available when building nested metadata predicates.",
        ),
        children: booleanGroupNodes,
      });
    }

    if (advancedPredicateNodes.length > 0) {
      children.push({
        id: `${category}:${subcategory}:advancedPredicates`,
        kind: "group",
        label: "Advanced Predicates",
        filterText: buildFilterText(category, subcategory, "advanced predicates"),
        listLabel: `Advanced predicates | ${advancedPredicateNodes.length}`,
        detailTitle: "Advanced Predicates",
        detailLines: buildKeyValueDetailLines(
          "Advanced Predicates",
          [
            ["Category", category],
            ["Subcategory", subcategory],
            ["Predicates", advancedPredicateNodes.length],
          ],
          "Explore the metadata predicates that operate on keyed metrics and comparisons beyond simple field/value filtering.",
        ),
        children: advancedPredicateNodes,
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
    const booleanGroupNodes = buildBooleanGroupNodes(category, null);
    const advancedPredicateNodes = buildAdvancedPredicateNodes(category, null);
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
    if (booleanGroupNodes.length > 0) {
      children.push({
        id: `${category}:booleanGroups`,
        kind: "group",
        label: "Boolean Groups",
        filterText: buildFilterText(category, "boolean groups"),
        listLabel: `Boolean groups | ${booleanGroupNodes.length}`,
        detailTitle: "Boolean Groups",
        detailLines: buildKeyValueDetailLines(
          "Boolean Groups",
          [
            ["Category", category],
            ["Groups", booleanGroupNodes.length],
          ],
          "Explore the logical group operators available when building nested metadata predicates.",
        ),
        children: booleanGroupNodes,
      });
    }
    if (advancedPredicateNodes.length > 0) {
      children.push({
        id: `${category}:advancedPredicates`,
        kind: "group",
        label: "Advanced Predicates",
        filterText: buildFilterText(category, "advanced predicates"),
        listLabel: `Advanced predicates | ${advancedPredicateNodes.length}`,
        detailTitle: "Advanced Predicates",
        detailLines: buildKeyValueDetailLines(
          "Advanced Predicates",
          [
            ["Category", category],
            ["Predicates", advancedPredicateNodes.length],
          ],
          "Explore the metadata predicates that operate on keyed metrics and comparisons beyond simple field/value filtering.",
        ),
        children: advancedPredicateNodes,
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
          ["Boolean groups", booleanGroupNodes.length],
          ["Advanced predicates", advancedPredicateNodes.length],
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
