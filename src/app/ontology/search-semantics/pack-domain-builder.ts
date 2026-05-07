import { formatOntologySearchVocabularyLabel } from "../../../domain/presentation-vocabulary.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type { SearchSemanticsDiscoveryReader } from "../../search-discovery/service.js";
import { buildFilterText } from "../node-helpers.js";
import { buildPreparedPackChildSource } from "./child-sources.js";
import type { SearchSemanticsRecordsDataService } from "./types.js";
import { buildPackValueNodes } from "./value-node-builder.js";

export function buildPackFieldNode(options: {
  dataService: SearchSemanticsRecordsDataService;
  searchSemanticsReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
}): OntologyNode {
  const { dataService, searchSemanticsReader, category } = options;
  return {
    id: `${category}:pack`,
    kind: "field",
    label: "Pack",
    filterText: buildFilterText(category, "pack", "source", "compendium"),
    listLabel: "Pack",
    detailTitle: "Pack Details",
    detailLines: [
      { text: "Pack", tone: "section" },
      { text: `Category: ${formatOntologySearchVocabularyLabel(category)}` },
      { text: "Drill in to browse packs from this category." },
    ],
    childSource: {
      kind: "sync",
      load: () =>
        buildPackValueNodes(
          dataService,
          category,
          null,
          searchSemanticsReader
            .discoverFieldValues({
              category,
              subcategory: null,
              field: "packs",
            })
            .map((entry) => ({
              value: String(entry.value),
              count: entry.count,
            })),
        ),
    },
  };
}

export function buildPreparedPackFieldNode(options: {
  dataService: SearchSemanticsRecordsDataService;
  preparedReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  countLabel: string;
  matchingRequest?: Readonly<SearchRequest>;
  discoveryMode: "catalog" | "matching";
}): OntologyNode {
  const { dataService, preparedReader, category, subcategory, countLabel, matchingRequest } = options;
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
    childSource: buildPreparedPackChildSource({
      dataService,
      preparedReader,
      category,
      subcategory,
      countLabel,
      matchingRequest,
    }),
  };
}
