import {
  compareFilterValues,
  type FilterValueOrdering,
} from "../../domain/filter-value-ordering.js";
import {
  getSearchPromotedFieldValueOrdering,
  isSearchPromotedFieldDomainKey,
} from "../../domain/search-field-domains.js";
import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import { getLoadedOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import type { FilterExplorerComposeTarget } from "../filter-explorer/types.js";
import type { Pf2eTerminalQueryFieldOption } from "../search/service.js";

export type SearchFilterExplorerValueSortMode = "semantic" | "frequency";

type DiscreteTarget = Extract<FilterExplorerComposeTarget, { kind?: "discrete" }>;

type SortableNode = {
  node: OntologyNode;
  target: DiscreteTarget;
  count: number;
};

export type SearchFilterExplorerValueSortOptions = {
  readonly sortMode: SearchFilterExplorerValueSortMode;
  readonly fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
  readonly resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
};

function isDiscreteTarget(target: FilterExplorerComposeTarget | undefined): target is DiscreteTarget {
  return target !== undefined && (!("kind" in target) || target.kind === "discrete");
}

function parseNodeCount(node: OntologyNode): number {
  const countText = node.listLabel?.match(/\|\s*([0-9,]+)\s*$/)?.[1];
  if (!countText) {
    return 0;
  }
  const count = Number.parseInt(countText.replaceAll(",", ""), 10);
  return Number.isFinite(count) ? count : 0;
}

function getSemanticValueOrdering(fieldOption: Pf2eTerminalQueryFieldOption | undefined): FilterValueOrdering {
  if (fieldOption?.valueOrdering) {
    return fieldOption.valueOrdering;
  }

  if (fieldOption && isSearchPromotedFieldDomainKey(fieldOption.value)) {
    return getSearchPromotedFieldValueOrdering(fieldOption.value) ?? { kind: "alpha" };
  }

  if (!fieldOption) {
    return { kind: "alpha" };
  }

  switch (fieldOption.fieldType) {
    case "number":
      return { kind: "numericAsc" };
    case "boolean":
      return { kind: "booleanTrueFirst" };
    case "set":
    case "enumString":
    case "text":
    default:
      return { kind: "alpha" };
  }
}

function supportsFrequencySort(fieldOption: Pf2eTerminalQueryFieldOption | undefined): boolean {
  if (!fieldOption) {
    return false;
  }
  if (getSemanticValueOrdering(fieldOption).kind !== "alpha") {
    return false;
  }
  return ["set", "enumString", "text"].includes(fieldOption.fieldType);
}

function getFieldOptionMap(
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[],
): ReadonlyMap<string, Pf2eTerminalQueryFieldOption> {
  return new Map(fieldOptions.map((fieldOption) => [fieldOption.value, fieldOption]));
}

function getSortableNodes(
  nodes: readonly OntologyNode[],
  resolveSelectionTarget: SearchFilterExplorerValueSortOptions["resolveSelectionTarget"],
): SortableNode[] | null {
  const sortableNodes = nodes.map((node) => {
    const target = resolveSelectionTarget(node);
    return isDiscreteTarget(target)
      ? {
          node,
          target,
          count: parseNodeCount(node),
        }
      : null;
  });

  if (sortableNodes.some((entry) => entry === null)) {
    return null;
  }

  const concreteNodes = sortableNodes.filter((entry): entry is SortableNode => entry !== null);
  if (concreteNodes.length === 0) {
    return null;
  }

  const fields = new Set(concreteNodes.map((entry) => entry.target.field));
  return fields.size === 1 ? concreteNodes : null;
}

function sortNodeLevel(
  nodes: readonly OntologyNode[],
  options: SearchFilterExplorerValueSortOptions,
): OntologyNode[] {
  const fieldOptions = getFieldOptionMap(options.fieldOptions);
  const sortableNodes = getSortableNodes(nodes, options.resolveSelectionTarget);

  if (!sortableNodes) {
    return [...nodes];
  }

  const fieldOption = fieldOptions.get(sortableNodes[0]!.target.field);
  const ordering =
    options.sortMode === "frequency" && supportsFrequencySort(fieldOption)
      ? { kind: "countDescThenAlpha" as const }
      : getSemanticValueOrdering(fieldOption);
  const valueForOrdering = (entry: SortableNode): string =>
    ordering.kind === "alpha" || ordering.kind === "countDescThenAlpha"
      ? entry.target.valueLabel ?? entry.node.label
      : entry.target.value;

  return [...sortableNodes]
    .sort((left, right) =>
      compareFilterValues(
        { value: valueForOrdering(left), count: left.count },
        { value: valueForOrdering(right), count: right.count },
        ordering,
      ),
    )
    .map((entry) => entry.node);
}

function sortNodeTree(node: OntologyNode, options: SearchFilterExplorerValueSortOptions): OntologyNode {
  const children = getLoadedOntologyNodeChildren(node);
  if (children.length === 0) {
    return node;
  }

  const sortedChildren = sortNodeLevel(
    children.map((child) => sortNodeTree(child, options)),
    options,
  );

  return {
    ...node,
    childSource: { kind: "static", children: sortedChildren },
  };
}

export function sortSearchFilterExplorerModel(
  model: OntologyDomainModel,
  options: SearchFilterExplorerValueSortOptions,
): OntologyDomainModel {
  return {
    ...model,
    rootNodes: sortNodeLevel(
      model.rootNodes.map((node) => sortNodeTree(node, options)),
      options,
    ),
  };
}

export function levelSupportsSearchFilterExplorerFrequencySort(
  nodes: readonly OntologyNode[],
  options: Pick<SearchFilterExplorerValueSortOptions, "fieldOptions" | "resolveSelectionTarget">,
): boolean {
  const sortableNodes = getSortableNodes(nodes, options.resolveSelectionTarget);
  if (!sortableNodes) {
    return false;
  }
  const fieldOption = getFieldOptionMap(options.fieldOptions).get(sortableNodes[0]!.target.field);
  return supportsFrequencySort(fieldOption);
}
