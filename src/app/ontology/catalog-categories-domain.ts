import { CATEGORY_SUBCATEGORY_MAP, SEARCH_CATEGORIES } from "../../domain/categories.js";
import type { Pf2eDataService } from "../../data/service.js";
import type { OntologyDomainModel, OntologyNode, SearchCategory } from "../../domain/index.js";
import { getOntologyDomainSummary } from "./domain-summaries.js";
import { buildFilterText, buildKeyValueDetailLines, titleCaseLabel } from "./node-helpers.js";

type CatalogCategoriesDataService = Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues">;

function buildCategorySubcategoryNode(
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

export function buildCatalogCategoriesDomain(dataService: CatalogCategoriesDataService): OntologyDomainModel {
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
    ...getOntologyDomainSummary("catalogCategories"),
    rootNodes: SEARCH_CATEGORIES.map((category) =>
      buildCategorySubcategoryNode(
        category,
        categoryCounts.get(category) ?? 0,
        subcategoryCountsByCategory.get(category) ?? new Map<string, number>(),
      ),
    ),
  };
}
