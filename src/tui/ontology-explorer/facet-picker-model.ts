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

function resolveSelectableFieldOption(
  node: OntologyNode,
  currentFieldOption: Pf2eTerminalFacetFieldOption | null,
  allowedFields: ReadonlyMap<string, Pf2eTerminalFacetFieldOption>,
): Pf2eTerminalFacetFieldOption | null {
  if (currentFieldOption && isSelectableValueNode(node, currentFieldOption.value)) {
    return currentFieldOption;
  }
  if (node.kind === "tag") {
    return allowedFields.get("derivedTags") ?? null;
  }
  if (node.kind === "trait") {
    return allowedFields.get("traits") ?? null;
  }
  return null;
}

function annotateSelectableNodes(
  node: OntologyNode,
  allowedFields: ReadonlyMap<string, Pf2eTerminalFacetFieldOption>,
  currentFieldOption: Pf2eTerminalFacetFieldOption | null = null,
): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const nextFieldOption = cloned.kind === "field" ? (allowedFields.get(cloned.label) ?? null) : currentFieldOption;
  const selectionFieldOption = resolveSelectableFieldOption(cloned, nextFieldOption, allowedFields);
  const selection = selectionFieldOption ? buildFieldSelectionResolver(selectionFieldOption)(cloned) : undefined;
  const children = cloned.children?.map((child) => annotateSelectableNodes(child, allowedFields, nextFieldOption));
  const loadChildren = cloned.loadChildren
    ? () => cloned.loadChildren!().map((child) => annotateSelectableNodes(child, allowedFields, nextFieldOption))
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
  const allowedFields = new Map<string, Pf2eTerminalFacetFieldOption>(
    options.fieldOptions.map((field) => [field.value, field]),
  );
  const rootNodes = categoryNode
    ? [
        {
          ...annotateSelectableNodes(categoryNode, allowedFields),
          detailLines: [
            ...categoryNode.detailLines,
            { text: `Query scope: ${options.category}${options.subcategory ? ` / ${options.subcategory}` : ""}` },
            {
              text: `Selectable fields: ${options.fieldOptions.map((field) => field.label).join(", ")}`,
            },
          ],
        },
      ]
    : [];

  return {
    id: "searchSemantics",
    label:
      options.fieldOptions.length === 1
        ? `${options.fieldOptions[0]!.label} Query`
        : `${titleCaseLabel(options.category)} Query Fields`,
    description: options.subcategory
      ? `Browse ${options.category} search semantics from the ${options.subcategory} query scope and apply discoverable field selections.`
      : `Browse ${options.category} search semantics and apply discoverable field selections from the explorer.`,
    rootNodes,
  };
}
