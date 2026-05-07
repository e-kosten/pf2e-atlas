import type { SearchSemanticsDiscoveryReader } from "../../search-discovery/service.js";
import type { SearchDiscoveryField } from "../../search-discovery/types.js";
import type { MetadataGlossaryArtifact } from "../../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type { SearchSemanticsRecordsDataService } from "./types.js";
import { buildFieldValueNodes, buildPackValueNodes } from "./value-node-builder.js";

export function buildPreparedFieldValueChildSource(options: {
  dataService: SearchSemanticsRecordsDataService;
  preparedReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  fieldSemantics: SearchDiscoveryField;
  metadataGlossary: MetadataGlossaryArtifact | null;
  countLabel: string;
  matchingRequest?: Readonly<SearchRequest>;
}): OntologyNode["childSource"] {
  const {
    dataService,
    preparedReader,
    category,
    subcategory,
    fieldSemantics,
    metadataGlossary,
    countLabel,
    matchingRequest,
  } = options;
  const buildNodes = (entries: readonly { value: unknown; count: number }[]) =>
    buildFieldValueNodes(
      dataService,
      category,
      subcategory,
      fieldSemantics,
      entries.map((entry) => ({
        value: String(entry.value),
        count: entry.count,
      })),
      metadataGlossary,
      { countLabel, matchingRequest },
    );
  const cachedOptions = preparedReader.discoverFieldValues({
    category,
    subcategory,
    field: fieldSemantics.field,
  });

  if (cachedOptions.length > 0) {
    return { kind: "sync", load: () => buildNodes(cachedOptions) };
  }

  return {
    kind: "lazy",
    load: async () =>
      buildNodes(
        await preparedReader.discoverFieldValuesAsync({
          category,
          subcategory,
          field: fieldSemantics.field,
        }),
      ),
  };
}

export function buildPreparedPackChildSource(options: {
  dataService: SearchSemanticsRecordsDataService;
  preparedReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  countLabel: string;
  matchingRequest?: Readonly<SearchRequest>;
}): OntologyNode["childSource"] {
  const { dataService, preparedReader, category, subcategory, countLabel, matchingRequest } = options;
  const buildNodes = (entries: readonly { value: unknown; count: number }[]) =>
    buildPackValueNodes(
      dataService,
      category,
      subcategory,
      entries.map((entry) => ({
        value: String(entry.value),
        count: entry.count,
      })),
      { countLabel, matchingRequest },
    );
  const cachedOptions = preparedReader.discoverFieldValues({
    category,
    subcategory,
    field: "packs",
  });

  if (cachedOptions.length > 0) {
    return { kind: "sync", load: () => buildNodes(cachedOptions) };
  }

  return {
    kind: "lazy",
    load: async () =>
      buildNodes(
        await preparedReader.discoverFieldValuesAsync({
          category,
          subcategory,
          field: "packs",
        }),
      ),
  };
}
