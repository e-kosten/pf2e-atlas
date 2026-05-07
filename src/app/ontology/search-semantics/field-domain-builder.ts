import {
  formatMetadataFieldLabel,
  formatMetadataFieldTypeLabel,
  formatOntologySearchVocabularyLabel,
} from "../../../domain/presentation-vocabulary.js";
import type { MetadataGlossaryArtifact } from "../../../domain/metadata-glossary-types.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchRequest } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type { SearchDiscoveryField, SearchSemanticsDiscoveryReader } from "../../search-discovery/service.js";
import { buildFilterText } from "../node-helpers.js";
import { buildPreparedFieldValueChildSource } from "./child-sources.js";
import { buildDerivedTagFamilyNodes, type DerivedTagFamilyNodeBuilder } from "./derived-tag-builder.js";
import type { SearchSemanticsRecordsDataService } from "./types.js";
import { buildFieldValueNodes } from "./value-node-builder.js";

export function buildMetadataFieldNodes(options: {
  dataService: SearchSemanticsRecordsDataService;
  searchSemanticsReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  getCategoryScopedFields: (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ) => readonly SearchDiscoveryField[];
  metadataGlossary: MetadataGlossaryArtifact | null;
  derivedTagCatalogByCategory: ReadonlyMap<
    SearchCategory,
    readonly import("../../../domain/record-types.js").DerivedTagCatalogEntry[]
  >;
  buildDerivedTagFamilyNode: DerivedTagFamilyNodeBuilder;
}): OntologyNode[] {
  const {
    dataService,
    searchSemanticsReader,
    category,
    subcategory,
    getCategoryScopedFields,
    metadataGlossary,
    derivedTagCatalogByCategory,
    buildDerivedTagFamilyNode,
  } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return getCategoryScopedFields(category, subcategory).map((fieldSemantics): OntologyNode => {
    const field = fieldSemantics.field;
    const fieldLabel = formatMetadataFieldLabel(field);
    const fieldTypeLabel = formatMetadataFieldTypeLabel(fieldSemantics.fieldType);
    const derivedTagChildren =
      field === "derivedTags"
        ? buildDerivedTagFamilyNodes({
            category,
            subcategory,
            idPrefix: `${idPrefix}:field:${field}`,
            derivedTagCatalogByCategory,
            buildDerivedTagFamilyNode,
          })
        : null;
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
        ? derivedTagChildren && derivedTagChildren.length > 0
          ? {
              childSource: { kind: "static", children: derivedTagChildren },
              childPresentation: {
                mode: "grouped",
                groupBy: "axis",
                render: "inline",
              } as const,
            }
          : {}
        : {
            childSource: fieldSemantics.discoverable
              ? {
                  kind: "sync",
                  load: () => {
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
                  },
                }
              : undefined,
          }),
    };
  });
}

export function buildPreparedMetadataFieldNodes(options: {
  dataService: SearchSemanticsRecordsDataService;
  preparedReader: SearchSemanticsDiscoveryReader;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  getCategoryScopedFields: (
    category: SearchCategory,
    subcategory: SearchSubcategory | null,
  ) => readonly SearchDiscoveryField[];
  metadataGlossary: MetadataGlossaryArtifact | null;
  derivedTagCatalogByCategory: ReadonlyMap<
    SearchCategory,
    readonly import("../../../domain/record-types.js").DerivedTagCatalogEntry[]
  >;
  buildDerivedTagFamilyNode: DerivedTagFamilyNodeBuilder;
  countLabel: string;
  matchingRequest?: Readonly<SearchRequest>;
  discoveryMode: "catalog" | "matching";
}): OntologyNode[] {
  const {
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
  } = options;
  const idPrefix = subcategory ? `${category}:${subcategory}` : category;
  return getCategoryScopedFields(category, subcategory).map((fieldSemantics): OntologyNode => {
    const field = fieldSemantics.field;
    const fieldLabel = formatMetadataFieldLabel(field);
    const fieldTypeLabel = formatMetadataFieldTypeLabel(fieldSemantics.fieldType);
    const derivedTagChildren =
      field === "derivedTags"
        ? buildDerivedTagFamilyNodes({
            category,
            subcategory,
            idPrefix: `${idPrefix}:field:${field}`,
            derivedTagCatalogByCategory,
            buildDerivedTagFamilyNode,
          })
        : null;
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
        ? derivedTagChildren && derivedTagChildren.length > 0
          ? {
              childSource: { kind: "static", children: derivedTagChildren },
              childPresentation: {
                mode: "grouped",
                groupBy: "axis",
                render: "inline",
              } as const,
            }
          : {}
        : {
            childSource: fieldSemantics.discoverable
              ? buildPreparedFieldValueChildSource({
                  dataService,
                  preparedReader,
                  category,
                  subcategory,
                  fieldSemantics,
                  metadataGlossary,
                  countLabel,
                  matchingRequest,
                })
              : undefined,
          }),
    };
  });
}
