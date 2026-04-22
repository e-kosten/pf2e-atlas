import { getOntologyNodeChildren, titleCaseLabel } from "../../app/ontology/node-helpers.js";
import { formatMetadataFieldTypeLabel } from "../../domain/presentation-vocabulary.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";
import { moveSelection, moveSelectionWrapped } from "../framework/input.js";
import { formatTerminalBreadcrumb } from "../list-detail-formatting.js";
import type {
  FilterExplorerBrowserSelection,
  FilterExplorerBrowserSnapshot,
  FilterExplorerBrowserState,
  FilterExplorerBrowserUiState,
  FilterExplorerModel,
  FilterExplorerNode,
} from "./types.js";

export type FilterExplorerListRow = {
  kind: "group" | "node";
  node?: FilterExplorerNode;
  line: DerivedTagTerminalLine;
};

type GroupRenderMode = "flat" | "inline";

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }

  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function filterNodes(nodes: readonly FilterExplorerNode[], filter: string): readonly FilterExplorerNode[] {
  const normalized = filter.trim().toLowerCase();
  if (!normalized) {
    return nodes;
  }

  return nodes.filter((node) => node.filterText.includes(normalized));
}

function findNodeById(nodes: readonly FilterExplorerNode[], id: string | undefined): FilterExplorerNode | undefined {
  return id ? nodes.find((node) => node.id === id) : undefined;
}

export function canDrillIntoFilterExplorerNode(node: FilterExplorerNode | undefined): boolean {
  return Boolean(node?.children?.length || node?.loadChildren);
}

function getChildrenOfSelectedParent(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): { ancestors: FilterExplorerNode[]; parent?: FilterExplorerNode; nodes: readonly FilterExplorerNode[] } {
  const ancestors: FilterExplorerNode[] = [];
  let nodes: readonly FilterExplorerNode[] = model.rootNodes;
  let parent: FilterExplorerNode | undefined;

  for (let level = 0; level < state.depth; level += 1) {
    const selected = findNodeById(nodes, state.selectedNodeIds[level]);
    const children = getOntologyNodeChildren(selected);
    if (!selected || children.length === 0) {
      return {
        ancestors,
        parent,
        nodes,
      };
    }

    ancestors.push(selected);
    parent = selected;
    nodes = children;
  }

  return {
    ancestors,
    parent,
    nodes,
  };
}

function findNearestFilteredNode(
  nodes: readonly FilterExplorerNode[],
  filteredNodes: readonly FilterExplorerNode[],
  selectedId: string | undefined,
): FilterExplorerNode | undefined {
  if (filteredNodes.length === 0) {
    return undefined;
  }
  if (!selectedId) {
    return filteredNodes[0];
  }

  const selectedIndex = nodes.findIndex((node) => node.id === selectedId);
  if (selectedIndex < 0) {
    return filteredNodes[0];
  }

  const filteredIds = new Set(filteredNodes.map((node) => node.id));
  let nearest: { node: FilterExplorerNode; distance: number; index: number } | null = null;

  for (let index = 0; index < nodes.length; index += 1) {
    const node = nodes[index]!;
    if (!filteredIds.has(node.id)) {
      continue;
    }

    const distance = Math.abs(index - selectedIndex);
    if (!nearest || distance < nearest.distance || (distance === nearest.distance && index < nearest.index)) {
      nearest = { node, distance, index };
    }
  }

  return nearest?.node ?? filteredNodes[0];
}

export function createFilterExplorerBrowserState(model: FilterExplorerModel): FilterExplorerBrowserState {
  return {
    depth: 0,
    selectedNodeIds: [model.rootNodes[0]?.id ?? ""],
    filter: "",
    detailScroll: 0,
  };
}

export function cloneFilterExplorerBrowserState(state: FilterExplorerBrowserState): FilterExplorerBrowserState {
  return {
    ...state,
    selectedNodeIds: [...state.selectedNodeIds],
  };
}

export function cloneFilterExplorerBrowserSnapshot(
  snapshot: FilterExplorerBrowserSnapshot,
): FilterExplorerBrowserSnapshot {
  return {
    ...snapshot,
    browserState: cloneFilterExplorerBrowserState(snapshot.browserState),
  };
}

export function normalizeFilterExplorerBrowserState(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): FilterExplorerBrowserState {
  const nextSelectedNodeIds = [...state.selectedNodeIds];
  const maxDepth = Math.max(0, nextSelectedNodeIds.length - 1);
  const nextDepth = Math.max(0, Math.min(state.depth, maxDepth));
  let nodes: readonly FilterExplorerNode[] = model.rootNodes;

  for (let level = 0; level <= nextDepth; level += 1) {
    if (nodes.length === 0) {
      return {
        ...state,
        depth: Math.max(0, level - 1),
        selectedNodeIds: nextSelectedNodeIds.slice(0, level),
      };
    }

    const filteredNodes = level === nextDepth ? filterNodes(nodes, state.filter) : nodes;
    const selected = nodes.find((node) => node.id === nextSelectedNodeIds[level]);
    const normalized =
      level === nextDepth ? findNearestFilteredNode(nodes, filteredNodes, selected?.id) : (selected ?? nodes[0]);
    nextSelectedNodeIds[level] = normalized?.id ?? nodes[0]!.id;

    if (level === nextDepth) {
      return {
        ...state,
        depth: nextDepth,
        selectedNodeIds: nextSelectedNodeIds.slice(0, nextDepth + 1),
      };
    }

    nodes = getOntologyNodeChildren(normalized);
  }

  return {
    ...state,
    depth: nextDepth,
    selectedNodeIds: nextSelectedNodeIds.slice(0, nextDepth + 1),
  };
}

export function getFilterExplorerBrowserSelection(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): FilterExplorerBrowserSelection {
  const normalized = normalizeFilterExplorerBrowserState(model, state);
  const { ancestors, parent, nodes } = getChildrenOfSelectedParent(model, normalized);
  const currentNodes = filterNodes(nodes, normalized.filter);
  const currentNode =
    currentNodes.find((node) => node.id === normalized.selectedNodeIds[normalized.depth]) ??
    findNearestFilteredNode(nodes, currentNodes, normalized.selectedNodeIds[normalized.depth]) ??
    currentNodes[0];

  return {
    ancestors,
    currentNodes,
    currentNode,
    currentParent: parent,
  };
}

function getCurrentNodes(model: FilterExplorerModel, state: FilterExplorerBrowserState): readonly FilterExplorerNode[] {
  return getFilterExplorerBrowserSelection(model, state).currentNodes;
}

function shouldInlineGroups(
  presentation: FilterExplorerNode["childPresentation"] | undefined,
  nodes: readonly FilterExplorerNode[],
): boolean {
  if (!presentation || presentation.mode !== "grouped") {
    return false;
  }
  if (presentation.render === "inline") {
    return true;
  }
  if (presentation.render !== "auto") {
    return false;
  }

  const groups = new Set(
    nodes.map((node) => node.groupValues?.[presentation.groupBy]).filter((value): value is string => Boolean(value)),
  );
  return (
    groups.size > 0 &&
    groups.size <= (presentation.autoInlineMaxGroups ?? 6) &&
    nodes.length <= (presentation.autoInlineMaxChildren ?? 30)
  );
}

function getGroupRenderMode(parent: FilterExplorerNode | undefined, nodes: readonly FilterExplorerNode[]): GroupRenderMode {
  return shouldInlineGroups(parent?.childPresentation, nodes) ? "inline" : "flat";
}

export function moveFilterExplorerSelection(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  delta: number,
): FilterExplorerBrowserState {
  return moveFilterExplorerSelectionWithStyle(model, state, delta, "wrapped");
}

export function jumpFilterExplorerSelection(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  delta: number,
): FilterExplorerBrowserState {
  return moveFilterExplorerSelectionWithStyle(model, state, delta, "clamped");
}

function moveFilterExplorerSelectionWithStyle(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  delta: number,
  motionStyle: "wrapped" | "clamped",
): FilterExplorerBrowserState {
  const nextState = normalizeFilterExplorerBrowserState(model, state);
  const nodes = getCurrentNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const currentIndex = nodes.findIndex((node) => node.id === nextState.selectedNodeIds[nextState.depth]);
  const targetIndex =
    motionStyle === "wrapped"
      ? moveSelectionWrapped(Math.max(0, currentIndex), delta, nodes.length)
      : moveSelection(Math.max(0, currentIndex), delta, nodes.length);
  const target = nodes[targetIndex];
  if (!target) {
    return nextState;
  }

  const selectedNodeIds = [...nextState.selectedNodeIds];
  selectedNodeIds[nextState.depth] = target.id;
  return {
    ...nextState,
    detailScroll: 0,
    selectedNodeIds,
  };
}

export function moveFilterExplorerSelectionToBoundary(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  boundary: "start" | "end",
): FilterExplorerBrowserState {
  const nextState = normalizeFilterExplorerBrowserState(model, state);
  const nodes = getCurrentNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const target = boundary === "start" ? nodes[0] : nodes[nodes.length - 1];
  if (!target) {
    return nextState;
  }

  const selectedNodeIds = [...nextState.selectedNodeIds];
  selectedNodeIds[nextState.depth] = target.id;
  return {
    ...nextState,
    detailScroll: 0,
    selectedNodeIds,
  };
}

export function drillIntoFilterExplorerBrowser(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): FilterExplorerBrowserState {
  const nextState = normalizeFilterExplorerBrowserState(model, state);
  const selection = getFilterExplorerBrowserSelection(model, nextState);
  const currentNode = selection.currentNode;
  const children = getOntologyNodeChildren(currentNode);
  if (children.length === 0) {
    return nextState;
  }

  const selectedNodeIds = [...nextState.selectedNodeIds];
  selectedNodeIds[nextState.depth + 1] = children[0]?.id ?? "";
  return {
    ...nextState,
    depth: nextState.depth + 1,
    detailScroll: 0,
    filter: "",
    selectedNodeIds: selectedNodeIds.slice(0, nextState.depth + 2),
  };
}

export function popFilterExplorerDepth(state: FilterExplorerBrowserState): FilterExplorerBrowserState {
  if (state.depth === 0) {
    return state;
  }

  return {
    ...state,
    depth: state.depth - 1,
    detailScroll: 0,
    filter: "",
    selectedNodeIds: state.selectedNodeIds.slice(0, state.depth),
  };
}

export function setFilterExplorerFilter(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  filter: string,
): FilterExplorerBrowserState {
  return normalizeFilterExplorerBrowserState(model, {
    ...state,
    filter,
    detailScroll: 0,
  });
}

export function moveFilterExplorerDetailScroll(
  state: FilterExplorerBrowserState,
  delta: number,
  maxDetailScroll: number,
): FilterExplorerBrowserState {
  return {
    ...state,
    detailScroll: Math.max(0, Math.min(maxDetailScroll, state.detailScroll + delta)),
  };
}

export function moveFilterExplorerDetailScrollToBoundary(
  state: FilterExplorerBrowserState,
  boundary: "start" | "end",
  maxDetailScroll: number,
): FilterExplorerBrowserState {
  return {
    ...state,
    detailScroll: boundary === "start" ? 0 : maxDetailScroll,
  };
}

export function createFilterExplorerBrowserUiState(
  model: FilterExplorerModel,
  snapshot?: FilterExplorerBrowserSnapshot,
): FilterExplorerBrowserUiState {
  const browserState = normalizeFilterExplorerBrowserState(
    model,
    snapshot ? cloneFilterExplorerBrowserState(snapshot.browserState) : createFilterExplorerBrowserState(model),
  );

  return {
    activePane: snapshot?.activePane ?? "list",
    browserState,
    layoutMode: snapshot?.layoutMode ?? "split",
    searchInput: snapshot?.searchInput ?? browserState.filter,
    searchMode: snapshot?.searchMode ?? false,
  };
}

export function buildFilterExplorerBreadcrumb(
  model: FilterExplorerModel,
  selection: FilterExplorerBrowserSelection,
): string {
  const segments = [model.label, ...selection.ancestors.map((node) => node.label)];
  if (selection.currentNode && selection.currentNodes.length > 0) {
    segments.push(selection.currentNode.label);
  }

  return formatTerminalBreadcrumb(segments);
}

function formatFilterExplorerGroupValue(groupBy: string, value: string): string {
  if (groupBy === "fieldType") {
    return formatMetadataFieldTypeLabel(value);
  }

  return titleCaseLabel(value);
}

export function buildFilterExplorerListRows(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
  bodyHeight: number,
  renderNodeLine: (node: FilterExplorerNode, isSelected: boolean) => DerivedTagTerminalLine = (node, isSelected) => ({
    text: node.listLabel ?? node.label,
    tone: isSelected ? "selected" : "default",
    noWrap: true,
  }),
): FilterExplorerListRow[] {
  const selection = getFilterExplorerBrowserSelection(model, state);
  const nodes = selection.currentNodes;
  if (nodes.length === 0) {
    return [
      {
        kind: "group",
        line: { text: "No ontology entries match the current filter.", tone: "dim" },
      },
    ];
  }

  const selectedIndex = nodes.findIndex((node) => node.id === selection.currentNode?.id);
  const windowStart = clampWindowStart(Math.max(0, selectedIndex), nodes.length, Math.max(1, bodyHeight));
  const visibleNodes = nodes.slice(windowStart, windowStart + Math.max(1, bodyHeight));
  const renderMode = getGroupRenderMode(selection.currentParent, nodes);

  if (renderMode === "inline" && selection.currentParent?.childPresentation?.mode === "grouped") {
    const groupBy = selection.currentParent.childPresentation.groupBy;
    const rows: FilterExplorerListRow[] = [];
    let lastGroup: string | undefined;

    for (const [offset, node] of visibleNodes.entries()) {
      const groupValue = node.groupValues?.[groupBy];
      if (groupValue && groupValue !== lastGroup) {
        if (lastGroup !== undefined) {
          rows.push({
            kind: "group",
            line: { text: "" },
          });
        }

        rows.push({
          kind: "group",
          line: {
            text: formatFilterExplorerGroupValue(groupBy, groupValue),
            tone: "section",
            noWrap: true,
          },
        });
        lastGroup = groupValue;
      }

      rows.push({
        kind: "node",
        node,
        line: renderNodeLine(node, windowStart + offset === Math.max(0, selectedIndex)),
      });
    }

    return rows;
  }

  return visibleNodes.map((node, offset) => ({
    kind: "node",
    node,
    line: renderNodeLine(node, windowStart + offset === Math.max(0, selectedIndex)),
  }));
}

export function buildFilterExplorerDetailLines(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): DerivedTagTerminalLine[] {
  const selection = getFilterExplorerBrowserSelection(model, state);
  return [...(selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }])];
}

export function getFilterExplorerDetailTitle(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserState,
): string {
  const selection = getFilterExplorerBrowserSelection(model, state);
  return selection.currentNode?.detailTitle ?? "Details";
}
