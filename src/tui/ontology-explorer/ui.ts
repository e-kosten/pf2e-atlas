import type { Key } from "ink";

import type {
  OntologyChildPresentation,
  OntologyDomainModel,
  OntologyNode,
} from "../../types.js";
import {
  getPrintableInput,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  moveSelectionWrapped,
  sliceRenderedTerminalLines,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "../terminal-ui.js";

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
  pendingListCommand: "g" | null;
  searchInput: string;
  searchMode: boolean;
};

export type OntologyBrowserSelection = {
  ancestors: OntologyNode[];
  currentNodes: OntologyNode[];
  currentNode?: OntologyNode;
  currentParent?: OntologyNode;
};

type GroupRenderMode = "flat" | "inline";

function titleCaseLabel(value: string): string {
  return value
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }
  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function filterNodes(nodes: OntologyNode[], filter: string): OntologyNode[] {
  const normalized = filter.trim().toLowerCase();
  if (!normalized) {
    return nodes;
  }
  return nodes.filter((node) => node.filterText.includes(normalized));
}

function findNodeById(nodes: OntologyNode[], id: string | undefined): OntologyNode | undefined {
  return id ? nodes.find((node) => node.id === id) : undefined;
}

function resolveOntologyNodeChildren(node: OntologyNode | undefined): OntologyNode[] {
  if (!node) {
    return [];
  }
  if (node.children) {
    return node.children;
  }
  if (!node.loadChildren) {
    return [];
  }
  node.children = node.loadChildren();
  return node.children;
}

export function canDrillIntoOntologyNode(node: OntologyNode | undefined): boolean {
  return Boolean(node?.children?.length || node?.loadChildren);
}

function getChildrenOfSelectedParent(model: OntologyDomainModel, state: OntologyBrowserState): { ancestors: OntologyNode[]; parent?: OntologyNode; nodes: OntologyNode[] } {
  const ancestors: OntologyNode[] = [];
  let nodes = model.rootNodes;
  let parent: OntologyNode | undefined;

  for (let level = 0; level < state.depth; level += 1) {
    const selected = findNodeById(nodes, state.selectedNodeIds[level]);
    const children = resolveOntologyNodeChildren(selected);
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

function findNearestFilteredNode(nodes: OntologyNode[], filteredNodes: OntologyNode[], selectedId: string | undefined): OntologyNode | undefined {
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

export function normalizeOntologyBrowserState(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): OntologyBrowserState {
  const nextSelectedNodeIds = [...state.selectedNodeIds];
  const maxDepth = Math.max(0, nextSelectedNodeIds.length - 1);
  const nextDepth = Math.max(0, Math.min(state.depth, maxDepth));
  let nodes = model.rootNodes;

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
    const normalized = level === nextDepth
      ? findNearestFilteredNode(nodes, filteredNodes, selected?.id)
      : selected ?? nodes[0];
    nextSelectedNodeIds[level] = normalized?.id ?? nodes[0]!.id;

    if (level === nextDepth) {
      return {
        ...state,
        depth: nextDepth,
        selectedNodeIds: nextSelectedNodeIds.slice(0, nextDepth + 1),
      };
    }

    nodes = normalized?.children ?? [];
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
  const currentNode = currentNodes.find((node) => node.id === normalized.selectedNodeIds[normalized.depth])
    ?? findNearestFilteredNode(nodes, currentNodes, normalized.selectedNodeIds[normalized.depth])
    ?? currentNodes[0];

  return {
    ancestors,
    currentNodes,
    currentNode,
    currentParent: parent,
  };
}

function getCurrentNodes(model: OntologyDomainModel, state: OntologyBrowserState): OntologyNode[] {
  return getOntologyBrowserSelection(model, state).currentNodes;
}

function shouldInlineGroups(
  presentation: OntologyChildPresentation | undefined,
  nodes: OntologyNode[],
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
  return groups.size > 0
    && groups.size <= (presentation.autoInlineMaxGroups ?? 6)
    && nodes.length <= (presentation.autoInlineMaxChildren ?? 30);
}

function getGroupRenderMode(parent: OntologyNode | undefined, nodes: OntologyNode[]): GroupRenderMode {
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
  const targetIndex = motionStyle === "wrapped"
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
  const children = resolveOntologyNodeChildren(currentNode);
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

export function popOntologyBrowserDepth(
  state: OntologyBrowserState,
): OntologyBrowserState {
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
): OntologyBrowserUiState {
  const browserState = createOntologyBrowserState(model);
  return {
    activePane: "list",
    browserState,
    layoutMode: "split",
    pendingListCommand: null,
    searchInput: browserState.filter,
    searchMode: false,
  };
}

export function buildOntologyBrowserBreadcrumb(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): string {
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
  const selection = getOntologyBrowserSelection(model, state);
  const nodes = selection.currentNodes;
  if (nodes.length === 0) {
    return [{ text: "No ontology entries match the current filter.", tone: "dim" }];
  }

  const selectedIndex = nodes.findIndex((node) => node.id === selection.currentNode?.id);
  const windowStart = clampWindowStart(Math.max(0, selectedIndex), nodes.length, Math.max(1, bodyHeight));
  const visibleNodes = nodes.slice(windowStart, windowStart + Math.max(1, bodyHeight));
  const renderMode = getGroupRenderMode(selection.currentParent, nodes);

  if (renderMode === "inline" && selection.currentParent?.childPresentation?.mode === "grouped") {
    const groupBy = selection.currentParent.childPresentation.groupBy;
    const lines: DerivedTagTerminalLine[] = [];
    let lastGroup: string | undefined;

    for (const [offset, node] of visibleNodes.entries()) {
      const groupValue = node.groupValues?.[groupBy];
      if (groupValue && groupValue !== lastGroup) {
        if (lastGroup !== undefined) {
          lines.push({ text: "" });
        }
        lines.push({
          text: titleCaseLabel(groupValue),
          tone: "section",
          noWrap: true,
        });
        lastGroup = groupValue;
      }
      lines.push({
        text: node.listLabel ?? node.label,
        tone: windowStart + offset === Math.max(0, selectedIndex) ? "selected" : "default",
        noWrap: true,
      });
    }
    return lines;
  }

  return visibleNodes.map((node, offset) => ({
    text: node.listLabel ?? node.label,
    tone: windowStart + offset === Math.max(0, selectedIndex) ? "selected" : "default",
    noWrap: true,
  }));
}

export function buildOntologyBrowserDetailLines(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): DerivedTagTerminalLine[] {
  const selection = getOntologyBrowserSelection(model, state);
  return selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }];
}

export function getOntologyBrowserDetailTitle(
  model: OntologyDomainModel,
  state: OntologyBrowserState,
): string {
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
): { bodyHeight: number; maxDetailScroll: number; detailJumpSize: number; detailPageSize: number; selectionJumpSize: number } {
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
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

export function buildOntologyBrowserHelpLines(): DerivedTagTerminalLine[] {
  return [
    { text: "Navigation", tone: "section" },
    { text: "Up / Down or j / k: move selection within the current ontology level" },
    { text: "Ctrl+U / Ctrl+D: jump up or down by half a pane without wrapping" },
    { text: "Space / b: page down or up without wrapping" },
    { text: "gg / G or Home / End: jump to the first or last entry in the current level" },
    { text: "Enter or Right / l: drill into the selected entry, or focus detail when it is a leaf" },
    { text: "o: open the selected entry's browse or lookup query in the search workspace when available" },
    { text: "Left / h or Backspace: go up one level" },
    { text: "Tab or w: switch focus between the list and detail panes" },
    { text: "z: toggle focused detail view while detail has focus" },
    { text: "/: enter live inline filtering for the current level" },
    { text: "Esc: clear the filter first, otherwise go back" },
    { text: "q: return to the ontology domain picker" },
  ];
}

export function getPrintableOntologyBrowserKeyCharacter(input: string, key: Key): string | undefined {
  if (key.ctrl || key.meta) {
    return undefined;
  }
  return getPrintableInput(input, key);
}

export function isExactPrintableOntologyBrowserKey(input: string, key: Key, value: string): boolean {
  return getPrintableOntologyBrowserKeyCharacter(input, key) === value;
}
