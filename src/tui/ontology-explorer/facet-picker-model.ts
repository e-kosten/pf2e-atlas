import type {
  OntologyDomainModel,
  OntologyNode,
  OntologySelectionState,
  SearchCategory,
  SearchSubcategory,
} from "../../types.js";
import type { Pf2eTerminalFacetFieldOption } from "../search-service.js";

function titleCaseLabel(value: string): string {
  return value
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function cloneOntologyNode(node: OntologyNode): OntologyNode {
  return {
    ...node,
    children: node.children?.map(cloneOntologyNode),
    loadChildren: node.loadChildren ? () => node.loadChildren!().map(cloneOntologyNode) : undefined,
    childPresentation: node.childPresentation ? { ...node.childPresentation } : undefined,
    groupValues: node.groupValues ? { ...node.groupValues } : undefined,
    query: node.query ? { ...node.query, filters: { ...node.query.filters } } : undefined,
    selection: node.selection ? { ...node.selection } : undefined,
  };
}

function isSelectableValueNode(node: OntologyNode, field: string): boolean {
  if (field === "derivedTags") {
    return node.kind === "tag";
  }
  return node.kind === "value" || node.kind === "trait";
}

function getAllowedStates(fieldType: Pf2eTerminalFacetFieldOption["fieldType"]): OntologySelectionState[] {
  return fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"];
}

type OntologySelectionResolver = (node: OntologyNode) => OntologyNode["selection"];

function annotateSelectableNodes(node: OntologyNode, resolveSelection: OntologySelectionResolver): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const selection = resolveSelection(cloned);
  if (selection) {
    cloned.selection = selection;
  }
  if (cloned.children) {
    cloned.children = cloned.children.map((child) => annotateSelectableNodes(child, resolveSelection));
  }
  if (cloned.loadChildren) {
    const loadChildren = cloned.loadChildren;
    cloned.loadChildren = () => loadChildren().map((child) => annotateSelectableNodes(child, resolveSelection));
  }
  return cloned;
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

function findNodeById(nodes: OntologyNode[], id: string): OntologyNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    if (node.children) {
      const match = findNodeById(node.children, id);
      if (match) {
        return match;
      }
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
  const metadataFieldsNode = findNodeById(categoryNode?.children ?? [], `${options.category}:metadataFields`);
  const allowedFields = new Map<string, Pf2eTerminalFacetFieldOption>(
    options.fieldOptions.map((field) => [field.value, field]),
  );
  const fieldNodesById = new Map((metadataFieldsNode?.children ?? []).map((node) => [node.id, node]));

  const rootNodes = options.fieldOptions
    .map((fieldOption) => {
      const fieldNode = fieldNodesById.get(`${options.category}:field:${fieldOption.value}`);
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
