import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import { getLoadedOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import type { FilterExplorerComposeTarget } from "../filter-explorer/types.js";
import type { SearchFilterExplorerFieldState } from "./filter-explorer-field-state.js";
import type { Pf2eTerminalQueryFieldOption } from "../search/service.js";
import {
  sortSearchFilterExplorerModel,
  type SearchFilterExplorerValueSortMode,
} from "./filter-explorer-value-sorting.js";

type DiscreteTarget = Extract<FilterExplorerComposeTarget, { kind?: "discrete" }>;

type ReconcileOptions = {
  currentModel: OntologyDomainModel;
  refreshedModel: OntologyDomainModel;
  fieldState: SearchFilterExplorerFieldState;
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  targetFields?: readonly string[];
  sortMode?: SearchFilterExplorerValueSortMode;
};

type TargetedRootReconcileOptions = Omit<ReconcileOptions, "targetFields"> & {
  selectedKeys: ReadonlySet<SelectedKey>;
  templates: ReadonlyMap<SelectedKey, OntologyNode>;
  targetFields: ReadonlySet<string>;
};

type SelectedKey = `${string}\u0000${string}`;

function getSelectedKey(field: string, value: string): SelectedKey {
  return `${field}\u0000${value}`;
}

function parseSelectedKey(key: SelectedKey): { field: string; value: string } {
  const [field = "", value = ""] = key.split("\0");
  return { field, value };
}

function isDiscreteTarget(target: FilterExplorerComposeTarget | undefined): target is DiscreteTarget {
  return target !== undefined && (!("kind" in target) || target.kind === "discrete");
}

function collectSelectedKeys(fieldState: SearchFilterExplorerFieldState): Set<SelectedKey> {
  const selected = new Set<SelectedKey>();
  for (const [field, selection] of Object.entries(fieldState.discreteSelections)) {
    for (const value of [...selection.include, ...selection.exclude]) {
      selected.add(getSelectedKey(field, value));
    }
  }
  return selected;
}

function collectNodeTemplates(
  nodes: readonly OntologyNode[],
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined,
  templates = new Map<SelectedKey, OntologyNode>(),
): Map<SelectedKey, OntologyNode> {
  for (const node of nodes) {
    const target = resolveSelectionTarget(node);
    if (isDiscreteTarget(target)) {
      templates.set(getSelectedKey(target.field, target.value), node);
    }
    collectNodeTemplates(getLoadedOntologyNodeChildren(node), resolveSelectionTarget, templates);
  }
  return templates;
}

function getNodeDiscreteField(
  node: OntologyNode,
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined,
): string | null {
  const target = resolveSelectionTarget(node);
  return isDiscreteTarget(target) ? target.field : null;
}

function getNodeFieldFromId(node: OntologyNode, fieldOptions: readonly Pf2eTerminalQueryFieldOption[]): string | null {
  return (
    fieldOptions.find(
      (fieldOption) => node.id.includes(`:field:${fieldOption.value}`) || node.id.endsWith(`:${fieldOption.value}`),
    )?.value ?? null
  );
}

function nodeContainsField(
  node: OntologyNode,
  field: string,
  resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined,
): boolean {
  if (getNodeDiscreteField(node, resolveSelectionTarget) === field) {
    return true;
  }
  if (node.id.includes(`:field:${field}`) || node.id.endsWith(`:${field}`)) {
    return true;
  }
  return getLoadedOntologyNodeChildren(node).some((child) => nodeContainsField(child, field, resolveSelectionTarget));
}

function cloneAsZeroCountSelectedNode(
  key: SelectedKey,
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[],
  templates: ReadonlyMap<SelectedKey, OntologyNode>,
): OntologyNode {
  const { field, value } = parseSelectedKey(key);
  const template = templates.get(key);
  const fieldLabel = fieldOptions.find((option) => option.value === field)?.label ?? field;
  const label = template?.label ?? value;
  const nodeId = template?.id ?? (field === "pack" ? `selected:${field}:${value}` : `selected:${field}:value:${value}`);

  return {
    id: nodeId,
    kind: template?.kind ?? "value",
    label,
    shortLabel: template?.shortLabel,
    filterText: template?.filterText ?? `${fieldLabel} ${value}`,
    listLabel: `${label} | 0`,
    detailTitle: template?.detailTitle ?? "Selected Filter Value",
    detailLines: [
      { text: label, tone: "section" },
      { text: `Field: ${fieldLabel}` },
      { text: `Value: ${value}` },
      { text: "Matching records: 0" },
    ],
    childSource: { kind: "static", children: [] },
    selection: template?.selection,
  };
}

function reconcileNode(
  refreshedNode: OntologyNode,
  previousNode: OntologyNode | undefined,
  options: {
    selectedKeys: ReadonlySet<SelectedKey>;
    fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
    templates: ReadonlyMap<SelectedKey, OntologyNode>;
    resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  },
): OntologyNode {
  const refreshedChildren = getLoadedOntologyNodeChildren(refreshedNode);
  const previousChildren = previousNode ? getLoadedOntologyNodeChildren(previousNode) : [];
  if (refreshedChildren.length === 0 && previousChildren.length === 0) {
    const nodeField = getNodeFieldFromId(refreshedNode, options.fieldOptions);
    const missingSelectedNodes = nodeField
      ? [...options.selectedKeys]
          .filter((key) => parseSelectedKey(key).field === nodeField)
          .map((key) => cloneAsZeroCountSelectedNode(key, options.fieldOptions, options.templates))
      : [];
    if (missingSelectedNodes.length > 0) {
      return {
        ...refreshedNode,
        childSource: { kind: "static", children: missingSelectedNodes },
      };
    }
    return refreshedNode;
  }

  const reconciledChildren = reconcileNodeLevel(refreshedChildren, previousChildren, options);
  return {
    ...refreshedNode,
    childSource: { kind: "static", children: reconciledChildren },
  };
}

function reconcileNodeLevel(
  refreshedNodes: readonly OntologyNode[],
  previousNodes: readonly OntologyNode[],
  options: {
    selectedKeys: ReadonlySet<SelectedKey>;
    fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
    templates: ReadonlyMap<SelectedKey, OntologyNode>;
    resolveSelectionTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  },
): OntologyNode[] {
  const previousById = new Map(previousNodes.map((node) => [node.id, node]));
  const refreshedBySelectedKey = new Set<SelectedKey>();
  const fieldsAtLevel = new Set<string>();
  const reconciled = refreshedNodes.map((node) => {
    const target = options.resolveSelectionTarget(node);
    if (isDiscreteTarget(target)) {
      refreshedBySelectedKey.add(getSelectedKey(target.field, target.value));
      fieldsAtLevel.add(target.field);
    }
    return reconcileNode(node, previousById.get(node.id), options);
  });

  for (const node of previousNodes) {
    const field = getNodeDiscreteField(node, options.resolveSelectionTarget);
    if (field) {
      fieldsAtLevel.add(field);
    }
  }

  const missingSelectedNodes = [...options.selectedKeys]
    .filter((key) => {
      const { field } = parseSelectedKey(key);
      return fieldsAtLevel.has(field) && !refreshedBySelectedKey.has(key);
    })
    .map((key) => cloneAsZeroCountSelectedNode(key, options.fieldOptions, options.templates));

  return [...reconciled, ...missingSelectedNodes];
}

function reconcileTargetedRootNodes(options: TargetedRootReconcileOptions): OntologyNode[] {
  const targetFields = options.targetFields;
  const currentTargetNodes = options.currentModel.rootNodes.filter((node) =>
    [...targetFields].some((field) => nodeContainsField(node, field, options.resolveSelectionTarget)),
  );
  const refreshedTargetNodes = options.refreshedModel.rootNodes.filter((node) =>
    [...targetFields].some((field) => nodeContainsField(node, field, options.resolveSelectionTarget)),
  );

  if (currentTargetNodes.length === options.currentModel.rootNodes.length) {
    return reconcileNodeLevel(refreshedTargetNodes, currentTargetNodes, options);
  }

  const refreshedById = new Map(refreshedTargetNodes.map((node) => [node.id, node]));
  return options.currentModel.rootNodes.flatMap((currentNode) => {
    const touchesTarget = [...targetFields].some((field) =>
      nodeContainsField(currentNode, field, options.resolveSelectionTarget),
    );
    if (!touchesTarget) {
      return [currentNode];
    }
    const refreshedNode = refreshedById.get(currentNode.id);
    return refreshedNode ? [reconcileNode(refreshedNode, currentNode, options)] : [];
  });
}

export function reconcileSearchFilterExplorerModel(options: ReconcileOptions): OntologyDomainModel {
  if (options.currentModel.id !== options.refreshedModel.id) {
    return sortSearchFilterExplorerModel(options.refreshedModel, {
      sortMode: options.sortMode ?? "semantic",
      fieldOptions: options.fieldOptions,
      resolveSelectionTarget: options.resolveSelectionTarget,
    });
  }

  const selectedKeys = collectSelectedKeys(options.fieldState);
  const templates = collectNodeTemplates(options.currentModel.rootNodes, options.resolveSelectionTarget);
  collectNodeTemplates(options.refreshedModel.rootNodes, options.resolveSelectionTarget, templates);
  const baseOptions = {
    selectedKeys,
    fieldOptions: options.fieldOptions,
    templates,
    resolveSelectionTarget: options.resolveSelectionTarget,
  };
  const targetFields = options.targetFields ? new Set(options.targetFields) : null;

  const reconciledModel = {
    ...options.refreshedModel,
    rootNodes: targetFields
      ? reconcileTargetedRootNodes({
          currentModel: options.currentModel,
          refreshedModel: options.refreshedModel,
          fieldState: options.fieldState,
          fieldOptions: options.fieldOptions,
          resolveSelectionTarget: options.resolveSelectionTarget,
          selectedKeys,
          templates,
          targetFields,
        })
      : reconcileNodeLevel(options.refreshedModel.rootNodes, options.currentModel.rootNodes, baseOptions),
  };

  return sortSearchFilterExplorerModel(reconciledModel, {
    sortMode: options.sortMode ?? "semantic",
    fieldOptions: options.fieldOptions,
    resolveSelectionTarget: options.resolveSelectionTarget,
  });
}
