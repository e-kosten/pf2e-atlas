import type {
  OntologyDomainModel,
  OntologyNode,
  OntologySelectionState,
} from "../../domain/ontology-types.js";
import type {
  SearchCategory,
  SearchSubcategory,
} from "../../domain/search-types.js";
import type { Pf2eTerminalFacetFieldOption } from "../search/service.js";
import { cloneOntologyNode, getOntologyNodeChildren, titleCaseLabel } from "../../app/ontology/node-helpers.js";

function isSelectableValueNode(node: OntologyNode, field: string): boolean {
  if (field === "derivedTags") {
    return node.kind === "tag";
  }
  return node.kind === "value" || node.kind === "trait";
}

function getAllowedStates(fieldType: Pf2eTerminalFacetFieldOption["fieldType"]): readonly OntologySelectionState[] {
  return fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"];
}

type OntologySelectionResolver = (node: OntologyNode) => OntologyNode["selection"];

function annotateSelectableNodes(node: OntologyNode, resolveSelection: OntologySelectionResolver): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const selection = resolveSelection(cloned);
  const children = cloned.children?.map((child) => annotateSelectableNodes(child, resolveSelection));
  const loadChildren = cloned.loadChildren
    ? () => cloned.loadChildren!().map((child) => annotateSelectableNodes(child, resolveSelection))
    : undefined;
  return {
    ...cloned,
    selection,
    children,
    loadChildren,
  };
}

function buildFieldSelectionResolver(fieldOption: Pf2eTerminalFacetFieldOption): OntologySelectionResolver {
  return (node) => {
    if (!isSelectableValueNode(node, fieldOption.value)) {
      return undefined;
    }
    return {
      field: fieldOption.value,
      fieldLabel: fieldOption.label,
      value: fieldOption.value === "derivedTags" ? node.label : (node.id.split(":").slice(-1)[0] ?? node.label),
      allowedStates: getAllowedStates(fieldOption.fieldType),
    };
  };
}

function findNodeById(nodes: readonly OntologyNode[], id: string): OntologyNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    const match = findNodeById(getOntologyNodeChildren(node), id);
    if (match) {
      return match;
    }
  }
  return undefined;
}

export function buildSearchFacetPickerModel(
  searchSemanticsDomain: OntologyDomainModel,
  options: {
    category: SearchCategory;
    subcategory: SearchSubcategory | null;
    fieldOptions: Pf2eTerminalFacetFieldOption[];
  },
): OntologyDomainModel {
  const categoryNode = findNodeById(searchSemanticsDomain.rootNodes, `searchSemantics:${options.category}`);
  const metadataFieldsNode =
    (options.subcategory
      ? findNodeById(getOntologyNodeChildren(categoryNode), `${options.category}:${options.subcategory}:metadataFields`)
      : undefined) ?? findNodeById(getOntologyNodeChildren(categoryNode), `${options.category}:metadataFields`);
  const allowedFields = new Map<string, Pf2eTerminalFacetFieldOption>(
    options.fieldOptions.map((field) => [field.value, field]),
  );
  const fieldNodesById = new Map(getOntologyNodeChildren(metadataFieldsNode).map((node) => [node.id, node]));

  const rootNodes = options.fieldOptions
    .map((fieldOption) => {
      const scopedFieldNodeId = options.subcategory
        ? `${options.category}:${options.subcategory}:field:${fieldOption.value}`
        : `${options.category}:field:${fieldOption.value}`;
      const fieldNode = fieldNodesById.get(scopedFieldNodeId) ?? fieldNodesById.get(`${options.category}:field:${fieldOption.value}`);
      return fieldNode ? annotateSelectableNodes(fieldNode, buildFieldSelectionResolver(fieldOption)) : null;
    })
    .filter((node): node is OntologyNode => Boolean(node))
    .map((node) => ({
      ...node,
      detailLines: [
        ...node.detailLines,
        ...(allowedFields.has(node.label)
          ? [{ text: `Facet picker label: ${allowedFields.get(node.label)!.label}` }]
          : []),
      ],
    }));

  return {
    id: "searchSemantics",
    label:
      options.fieldOptions.length === 1
        ? `${options.fieldOptions[0]!.label} Query`
        : `${titleCaseLabel(options.category)} Query Fields`,
    description: options.subcategory
      ? `Edit discoverable ${options.subcategory} query fields with hierarchy-first browsing.`
      : `Edit discoverable ${options.category} query fields with hierarchy-first browsing.`,
    rootNodes,
  };
}
