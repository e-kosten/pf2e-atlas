import type { OntologyChildPresentation, OntologyDomainModel, OntologyNode } from "../../types.js";
import { getOntologyNodeChildren, titleCaseLabel } from "../../app/ontology/node-helpers.js";
import {
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
} from "../framework/rendering.js";
import { moveSelection, moveSelectionWrapped } from "../framework/input.js";
import type { DerivedTagTerminalLine, DerivedTagTerminalTwoPaneLayoutMode } from "../framework/types.js";

export type OntologyBrowserState = {
  depth: number;
  selectedNodeIds: string[];
  filter: string;
  detailScroll: number;
};

export type OntologyBrowserUiState = {
  activePane: "list" | "detail";
  browserState: OntologyBrowserState;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  searchInput: string;
  searchMode: boolean;
};

export type OntologyBrowserSnapshot = OntologyBrowserUiState;

export type OntologyBrowserSelection = {
  ancestors: OntologyNode[];
  currentNodes: readonly OntologyNode[];
  currentNode?: OntologyNode;
  currentParent?: OntologyNode;
};

export type OntologyBrowserListRow = {
  kind: "group" | "node";
  node?: OntologyNode;
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

function filterNodes(nodes: readonly OntologyNode[], filter: string): readonly OntologyNode[] {
  const normalized = filter.trim().toLowerCase();
  if (!normalized) {
    return nodes;
  }
  return nodes.filter((node) => node.filterText.includes(normalized));
}

function findNodeById(nodes: readonly OntologyNode[], id: string | undefined): OntologyNode | undefined {
  return id ? nodes.find((node) => node.id === id) : undefined;
}

export function canDrillIntoOntologyNode(node: OntologyNode | undefined): boolean {
  return Boolean(node?.children?.length || node?.loadChildren);
}

function getChildrenOfSelectedParent(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): { ancestors: OntologyNode[]; parent?: OntologyNode; nodes: readonly OntologyNode[] } {
  const ancestors: OntologyNode[] = [];
  let nodes: readonly OntologyNode[] = model.rootNodes;
  let parent: OntologyNode | undefined;

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
  nodes: readonly OntologyNode[],
  filteredNodes: readonly OntologyNode[],
  selectedId: string | undefined,
): OntologyNode | undefined {
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
  let nearest: { node: OntologyNode; distance: number; index: number } | null = null;

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

export function createOntologyBrowserState(model: OntologyDomainModel): OntologyBrowserState {
  return {
    depth: 0,
    selectedNodeIds: [model.rootNodes[0]?.id ?? ""],
    filter: "",
    detailScroll: 0,
  };
}

export function cloneOntologyBrowserState(state: OntologyBrowserState): OntologyBrowserState {
  return {
    ...state,
    selectedNodeIds: [...state.selectedNodeIds],
  };
}

export function cloneOntologyBrowserSnapshot(snapshot: OntologyBrowserSnapshot): OntologyBrowserSnapshot {
  return {
    ...snapshot,
    browserState: cloneOntologyBrowserState(snapshot.browserState),
  };
}

export function normalizeOntologyBrowserState(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): OntologyBrowserState {
  const nextSelectedNodeIds = [...state.selectedNodeIds];
  const maxDepth = Math.max(0, nextSelectedNodeIds.length - 1);
  const nextDepth = Math.max(0, Math.min(state.depth, maxDepth));
  let nodes: readonly OntologyNode[] = model.rootNodes;

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

export function getOntologyBrowserSelection(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): OntologyBrowserSelection {
  const normalized = normalizeOntologyBrowserState(model, state);
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

function getCurrentNodes(model: OntologyDomainModel, state: OntologyBrowserState): readonly OntologyNode[] {
  return getOntologyBrowserSelection(model, state).currentNodes;
}

function shouldInlineGroups(
  presentation: OntologyChildPresentation | undefined,
  nodes: readonly OntologyNode[],
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

function getGroupRenderMode(parent: OntologyNode | undefined, nodes: readonly OntologyNode[]): GroupRenderMode {
  return shouldInlineGroups(parent?.childPresentation, nodes) ? "inline" : "flat";
}

export function moveOntologyBrowserSelection(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  delta: number,
): OntologyBrowserState {
  return moveOntologyBrowserSelectionWithStyle(model, state, delta, "wrapped");
}

export function jumpOntologyBrowserSelection(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  delta: number,
): OntologyBrowserState {
  return moveOntologyBrowserSelectionWithStyle(model, state, delta, "clamped");
}

function moveOntologyBrowserSelectionWithStyle(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  delta: number,
  motionStyle: "wrapped" | "clamped",
): OntologyBrowserState {
  const nextState = normalizeOntologyBrowserState(model, state);
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

export function moveOntologyBrowserSelectionToBoundary(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  boundary: "start" | "end",
): OntologyBrowserState {
  const nextState = normalizeOntologyBrowserState(model, state);
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

export function drillIntoOntologyBrowser(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): OntologyBrowserState {
  const nextState = normalizeOntologyBrowserState(model, state);
  const selection = getOntologyBrowserSelection(model, nextState);
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

export function popOntologyBrowserDepth(state: OntologyBrowserState): OntologyBrowserState {
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

export function setOntologyBrowserFilter(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  filter: string,
): OntologyBrowserState {
  return normalizeOntologyBrowserState(model, {
    ...state,
    filter,
    detailScroll: 0,
  });
}

export function moveOntologyBrowserDetailScroll(
  state: OntologyBrowserState,
  delta: number,
  maxDetailScroll: number,
): OntologyBrowserState {
  return {
    ...state,
    detailScroll: Math.max(0, Math.min(maxDetailScroll, state.detailScroll + delta)),
  };
}

export function moveOntologyBrowserDetailScrollToBoundary(
  state: OntologyBrowserState,
  boundary: "start" | "end",
  maxDetailScroll: number,
): OntologyBrowserState {
  return {
    ...state,
    detailScroll: boundary === "start" ? 0 : maxDetailScroll,
  };
}

export function createOntologyBrowserUiState(
  model: OntologyDomainModel,
  snapshot?: OntologyBrowserSnapshot,
): OntologyBrowserUiState {
  const browserState = normalizeOntologyBrowserState(
    model,
    snapshot ? cloneOntologyBrowserState(snapshot.browserState) : createOntologyBrowserState(model),
  );
  return {
    activePane: snapshot?.activePane ?? "list",
    browserState,
    layoutMode: snapshot?.layoutMode ?? "split",
    searchInput: snapshot?.searchInput ?? browserState.filter,
    searchMode: snapshot?.searchMode ?? false,
  };
}

export function buildOntologyBrowserBreadcrumb(model: OntologyDomainModel, state: OntologyBrowserState): string {
  const selection = getOntologyBrowserSelection(model, state);
  const segments = [model.label, ...selection.ancestors.map((node) => node.label)];
  if (selection.currentNode && selection.currentNodes.length > 0) {
    segments.push(selection.currentNode.label);
  }
  return segments.join(" > ");
}

export function buildOntologyBrowserListLines(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  return buildOntologyBrowserListRows(model, state, bodyHeight).map((row) => row.line);
}

export function buildOntologyBrowserListRows(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  bodyHeight: number,
  renderNodeLine: (node: OntologyNode, isSelected: boolean) => DerivedTagTerminalLine = (node, isSelected) => ({
    text: node.listLabel ?? node.label,
    tone: isSelected ? "selected" : "default",
    noWrap: true,
  }),
): OntologyBrowserListRow[] {
  const selection = getOntologyBrowserSelection(model, state);
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
    const rows: OntologyBrowserListRow[] = [];
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
            text: titleCaseLabel(groupValue),
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
    kind: "node" as const,
    node,
    line: renderNodeLine(node, windowStart + offset === Math.max(0, selectedIndex)),
  }));
}

export function buildOntologyBrowserDetailLines(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): DerivedTagTerminalLine[] {
  const selection = getOntologyBrowserSelection(model, state);
  return [...(selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }])];
}

export function getOntologyBrowserDetailTitle(model: OntologyDomainModel, state: OntologyBrowserState): string {
  const selection = getOntologyBrowserSelection(model, state);
  return selection.currentNode?.detailTitle ?? "Details";
}

export function buildVisibleOntologyBrowserDetailLines(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  width: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  return sliceRenderedTerminalLines(
    buildOntologyBrowserDetailLines(model, state),
    getTerminalTwoPaneDetailWidth(width, layoutMode, 46),
    state.detailScroll,
    bodyHeight,
  );
}

export function getOntologyBrowserDetailMetrics(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  width: number,
  height: number,
): {
  bodyHeight: number;
  maxDetailScroll: number;
  detailJumpSize: number;
  detailPageSize: number;
  selectionJumpSize: number;
} {
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );
  const detailWidth = getTerminalTwoPaneDetailWidth(width, layoutMode, 46);
  const detailLines = buildOntologyBrowserDetailLines(model, state);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  return {
    bodyHeight,
    detailJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    detailPageSize: Math.max(1, bodyHeight - 1),
    maxDetailScroll: Math.max(0, renderedDetailLineCount - bodyHeight),
    selectionJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
  };
}
