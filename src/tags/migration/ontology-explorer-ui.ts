import type { Key } from "ink";

import type { SearchCategory } from "../../types.js";
import { buildOntologyExplorerEntityDetailLines, buildOntologyExplorerEntitySummary } from "./entity-page.js";
import {
  buildDerivedTagOntologyExplorerModel,
  filterOntologyExplorerNodes,
  type DerivedTagOntologyExplorerCategoryNode,
  type DerivedTagOntologyExplorerFamilyNode,
  type DerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerRecordNode,
  type DerivedTagOntologyExplorerTagNode,
} from "./ontology-explorer-data.js";
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
} from "./terminal-ui.js";

export type DerivedTagOntologyExplorerDepth = "category" | "family" | "tag" | "record";
type DerivedTagOntologyExplorerMotionStyle = "wrapped" | "clamped";

export type DerivedTagOntologyExplorerState = {
  depth: DerivedTagOntologyExplorerDepth;
  selectedCategoryKey?: SearchCategory;
  selectedFamilyKey?: string;
  selectedTagKey?: string;
  selectedRecordKey?: string;
  filter: string;
  detailScroll: number;
};

export type DerivedTagOntologyExplorerUiState = {
  activePane: "list" | "detail";
  explorerState: DerivedTagOntologyExplorerState;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  pendingListCommand: "g" | null;
  searchInput: string;
  searchMode: boolean;
};

type ExplorerNode =
  | DerivedTagOntologyExplorerCategoryNode
  | DerivedTagOntologyExplorerFamilyNode
  | DerivedTagOntologyExplorerTagNode
  | DerivedTagOntologyExplorerRecordNode;

export type DerivedTagOntologyExplorerSelection = {
  categories: DerivedTagOntologyExplorerCategoryNode[];
  category?: DerivedTagOntologyExplorerCategoryNode;
  families: DerivedTagOntologyExplorerFamilyNode[];
  family?: DerivedTagOntologyExplorerFamilyNode;
  tags: DerivedTagOntologyExplorerTagNode[];
  tag?: DerivedTagOntologyExplorerTagNode;
  records: DerivedTagOntologyExplorerRecordNode[];
  record?: DerivedTagOntologyExplorerRecordNode;
};

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

function findCategory(model: DerivedTagOntologyExplorerModel, key: SearchCategory | undefined): DerivedTagOntologyExplorerCategoryNode | undefined {
  return key ? model.categories.find((category) => category.key === key) : undefined;
}

function findFamily(category: DerivedTagOntologyExplorerCategoryNode | undefined, key: string | undefined): DerivedTagOntologyExplorerFamilyNode | undefined {
  return key ? category?.families.find((family) => family.key === key) : undefined;
}

function findTag(family: DerivedTagOntologyExplorerFamilyNode | undefined, key: string | undefined): DerivedTagOntologyExplorerTagNode | undefined {
  return key ? family?.tags.find((tag) => tag.key === key) : undefined;
}

function findRecord(tag: DerivedTagOntologyExplorerTagNode | undefined, key: string | undefined): DerivedTagOntologyExplorerRecordNode | undefined {
  return key ? tag?.records.find((record) => record.key === key) : undefined;
}

export function getDerivedTagOntologyExplorerSelection(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagOntologyExplorerSelection {
  const category = findCategory(model, state.selectedCategoryKey) ?? model.categories[0];
  const families = category?.families ?? [];
  const family = findFamily(category, state.selectedFamilyKey) ?? families[0];
  const tags = family?.tags ?? [];
  const tag = findTag(family, state.selectedTagKey) ?? tags[0];
  const records = tag?.records ?? [];
  const record = findRecord(tag, state.selectedRecordKey) ?? records[0];

  return {
    categories: model.categories,
    category,
    families,
    family,
    tags,
    tag,
    records,
    record,
  };
}

function findNearestFilteredNode(
  nodes: ExplorerNode[],
  filteredNodes: ExplorerNode[],
  selectedKey: string | undefined,
): ExplorerNode | undefined {
  if (filteredNodes.length === 0) {
    return undefined;
  }
  if (!selectedKey) {
    return filteredNodes[0];
  }

  const selectedIndex = nodes.findIndex((node) => node.key === selectedKey);
  if (selectedIndex < 0) {
    return filteredNodes[0];
  }

  const filteredKeys = new Set(filteredNodes.map((node) => node.key));
  let nearest: { node: ExplorerNode; distance: number; index: number } | null = null;

  for (let index = 0; index < nodes.length; index += 1) {
    const node = nodes[index]!;
    if (!filteredKeys.has(node.key)) {
      continue;
    }
    const distance = Math.abs(index - selectedIndex);
    if (!nearest || distance < nearest.distance || (distance === nearest.distance && index < nearest.index)) {
      nearest = { node, distance, index };
    }
  }

  return nearest?.node ?? filteredNodes[0];
}

function getNodesForDepth(selection: DerivedTagOntologyExplorerSelection, depth: DerivedTagOntologyExplorerDepth): ExplorerNode[] {
  if (depth === "category") {
    return selection.categories;
  }
  if (depth === "family") {
    return selection.families;
  }
  if (depth === "tag") {
    return selection.tags;
  }
  return selection.records;
}

function getSelectedKeyForDepth(selection: DerivedTagOntologyExplorerSelection, state: DerivedTagOntologyExplorerState): string | undefined {
  if (state.depth === "category") {
    return selection.category?.key;
  }
  if (state.depth === "family") {
    return selection.family?.key;
  }
  if (state.depth === "tag") {
    return selection.tag?.key;
  }
  return selection.record?.key;
}

export function createDerivedTagOntologyExplorerState(
  model: DerivedTagOntologyExplorerModel,
): DerivedTagOntologyExplorerState {
  return {
    depth: "category",
    selectedCategoryKey: model.categories[0]?.key,
    selectedFamilyKey: model.categories[0]?.families[0]?.key,
    selectedTagKey: model.categories[0]?.families[0]?.tags[0]?.key,
    selectedRecordKey: model.categories[0]?.families[0]?.tags[0]?.records[0]?.key,
    filter: "",
    detailScroll: 0,
  };
}

export function normalizeDerivedTagOntologyExplorerState(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagOntologyExplorerState {
  const selection = getDerivedTagOntologyExplorerSelection(model, state);
  let nextState: DerivedTagOntologyExplorerState = {
    ...state,
    selectedCategoryKey: selection.category?.key,
    selectedFamilyKey: selection.family?.key,
    selectedTagKey: selection.tag?.key,
    selectedRecordKey: selection.record?.key,
  };

  const allNodes = getNodesForDepth(selection, state.depth);
  const nodes = filterOntologyExplorerNodes(allNodes, state.filter);
  const selectedKey = getSelectedKeyForDepth(selection, nextState);
  if (nodes.length > 0 && !nodes.some((node) => node.key === selectedKey)) {
    const nearest = findNearestFilteredNode(allNodes, nodes, selectedKey) ?? nodes[0]!;
    if (nearest.kind === "category") {
      nextState = {
        ...nextState,
        selectedCategoryKey: nearest.key,
        selectedFamilyKey: nearest.families[0]?.key,
        selectedTagKey: nearest.families[0]?.tags[0]?.key,
        selectedRecordKey: nearest.families[0]?.tags[0]?.records[0]?.key,
      };
    } else if (nearest.kind === "family") {
      nextState = {
        ...nextState,
        selectedFamilyKey: nearest.key,
        selectedTagKey: nearest.tags[0]?.key,
        selectedRecordKey: nearest.tags[0]?.records[0]?.key,
      };
    } else if (nearest.kind === "tag") {
      nextState = {
        ...nextState,
        selectedTagKey: nearest.key,
        selectedRecordKey: nearest.records[0]?.key,
      };
    } else {
      nextState = {
        ...nextState,
        selectedRecordKey: nearest.key,
      };
    }
  }

  return nextState;
}

function getActiveNodes(model: DerivedTagOntologyExplorerModel, state: DerivedTagOntologyExplorerState): ExplorerNode[] {
  const selection = getDerivedTagOntologyExplorerSelection(model, state);
  return filterOntologyExplorerNodes(getNodesForDepth(selection, state.depth), state.filter);
}

export function moveDerivedTagOntologyExplorerSelection(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  delta: number,
): DerivedTagOntologyExplorerState {
  return moveDerivedTagOntologyExplorerSelectionWithStyle(model, state, delta, "wrapped");
}

export function jumpDerivedTagOntologyExplorerSelection(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  delta: number,
): DerivedTagOntologyExplorerState {
  return moveDerivedTagOntologyExplorerSelectionWithStyle(model, state, delta, "clamped");
}

function applyDerivedTagOntologyExplorerSelectionTarget(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  target: ExplorerNode,
): DerivedTagOntologyExplorerState {
  if (target.kind === "category") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      selectedCategoryKey: target.key,
      selectedFamilyKey: target.families[0]?.key,
      selectedTagKey: target.families[0]?.tags[0]?.key,
      selectedRecordKey: target.families[0]?.tags[0]?.records[0]?.key,
      detailScroll: 0,
    });
  }
  if (target.kind === "family") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      selectedFamilyKey: target.key,
      selectedTagKey: target.tags[0]?.key,
      selectedRecordKey: target.tags[0]?.records[0]?.key,
      detailScroll: 0,
    });
  }
  if (target.kind === "tag") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      selectedTagKey: target.key,
      selectedRecordKey: target.records[0]?.key,
      detailScroll: 0,
    });
  }
  return normalizeDerivedTagOntologyExplorerState(model, {
    ...state,
    selectedRecordKey: target.key,
    detailScroll: 0,
  });
}

function moveDerivedTagOntologyExplorerSelectionWithStyle(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  delta: number,
  motionStyle: DerivedTagOntologyExplorerMotionStyle,
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  const selection = getDerivedTagOntologyExplorerSelection(model, nextState);
  const nodes = getActiveNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const currentIndex = nodes.findIndex((node) => node.key === getSelectedKeyForDepth(selection, nextState));
  const targetIndex = motionStyle === "wrapped"
    ? moveSelectionWrapped(Math.max(0, currentIndex), delta, nodes.length)
    : moveSelection(Math.max(0, currentIndex), delta, nodes.length);
  const target = nodes[targetIndex];
  if (!target) {
    return nextState;
  }

  return applyDerivedTagOntologyExplorerSelectionTarget(model, nextState, target);
}

export function moveDerivedTagOntologyExplorerSelectionToBoundary(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  boundary: "start" | "end",
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  const nodes = getActiveNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const target = boundary === "start" ? nodes[0] : nodes[nodes.length - 1];
  if (!target) {
    return nextState;
  }

  return applyDerivedTagOntologyExplorerSelectionTarget(model, nextState, target);
}

export function drillIntoDerivedTagOntologyExplorer(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  if (nextState.depth === "category") {
    return { ...nextState, depth: "family", filter: "", detailScroll: 0 };
  }
  if (nextState.depth === "family") {
    return { ...nextState, depth: "tag", filter: "", detailScroll: 0 };
  }
  if (nextState.depth === "tag") {
    return { ...nextState, depth: "record", filter: "", detailScroll: 0 };
  }
  return nextState;
}

export function popDerivedTagOntologyExplorerDepth(
  state: DerivedTagOntologyExplorerState,
): DerivedTagOntologyExplorerState {
  if (state.depth === "record") {
    return { ...state, depth: "tag", filter: "", detailScroll: 0 };
  }
  if (state.depth === "tag") {
    return { ...state, depth: "family", filter: "", detailScroll: 0 };
  }
  if (state.depth === "family") {
    return { ...state, depth: "category", filter: "", detailScroll: 0 };
  }
  return state;
}

export function setDerivedTagOntologyExplorerFilter(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  filter: string,
): DerivedTagOntologyExplorerState {
  return normalizeDerivedTagOntologyExplorerState(model, {
    ...state,
    filter,
    detailScroll: 0,
  });
}

export function moveDerivedTagOntologyExplorerDetailScroll(
  state: DerivedTagOntologyExplorerState,
  delta: number,
  maxDetailScroll: number,
): DerivedTagOntologyExplorerState {
  return {
    ...state,
    detailScroll: Math.max(0, Math.min(maxDetailScroll, state.detailScroll + delta)),
  };
}

export function moveDerivedTagOntologyExplorerDetailScrollToBoundary(
  state: DerivedTagOntologyExplorerState,
  boundary: "start" | "end",
  maxDetailScroll: number,
): DerivedTagOntologyExplorerState {
  return {
    ...state,
    detailScroll: boundary === "start" ? 0 : maxDetailScroll,
  };
}

export function createDerivedTagOntologyExplorerUiState(
  model: DerivedTagOntologyExplorerModel,
): DerivedTagOntologyExplorerUiState {
  const explorerState = createDerivedTagOntologyExplorerState(model);
  return {
    activePane: "list",
    explorerState,
    layoutMode: "split",
    pendingListCommand: null,
    searchInput: explorerState.filter,
    searchMode: false,
  };
}

export function buildDerivedTagOntologyExplorerBreadcrumb(
  selection: DerivedTagOntologyExplorerSelection,
  state: DerivedTagOntologyExplorerState,
): string {
  const segments = ["ontology-search"];
  if (selection.category) {
    segments.push(selection.category.category);
  }
  if (state.depth !== "category" && selection.family) {
    segments.push(selection.family.family);
  }
  if (state.depth !== "category" && state.depth !== "family" && selection.tag) {
    segments.push(selection.tag.tag);
  }
  if (state.depth === "record" && selection.record) {
    segments.push(selection.record.record.name);
  }
  return segments.join(" > ");
}

export function buildDerivedTagOntologyExplorerListLines(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const selection = getDerivedTagOntologyExplorerSelection(model, state);
  const nodes = getActiveNodes(model, state);
  if (nodes.length === 0) {
    return [{ text: "No nodes match the current filter.", tone: "dim" }];
  }

  const selectedIndex = nodes.findIndex((node) => node.key === getSelectedKeyForDepth(selection, state));
  const windowStart = clampWindowStart(Math.max(0, selectedIndex), nodes.length, Math.max(1, bodyHeight));
  const visibleNodes = nodes.slice(windowStart, windowStart + Math.max(1, bodyHeight));

  if (state.depth === "family") {
    let lastAxis: string | undefined;
    const lines: DerivedTagTerminalLine[] = [];

    for (const [offset, node] of visibleNodes.entries()) {
      if (node.kind !== "family") {
        continue;
      }
      if (node.axis !== lastAxis) {
        if (lastAxis !== undefined) {
          lines.push({ text: "" });
        }
        lines.push({
          text: titleCaseLabel(node.axis),
          tone: "section",
          noWrap: true,
        });
        lastAxis = node.axis;
      }
      lines.push({
        text: `${node.family} | ${node.tagCount} tags | ${node.liveRecordCount} live records`,
        tone: windowStart + offset === Math.max(0, selectedIndex) ? "selected" : "default",
        noWrap: true,
      });
    }

    return lines;
  }

  return visibleNodes.map((node, offset) => {
    const isSelected = windowStart + offset === Math.max(0, selectedIndex);
    if (node.kind === "category") {
      return {
        text: `${node.category} | ${node.familyCount} families | ${node.tagCount} tags | ${node.taggedRecordCount} tagged records`,
        tone: isSelected ? "selected" : "default",
        noWrap: true,
      };
    }
    if (node.kind === "family") {
      return {
        text: `${node.family} | ${node.tagCount} tags | ${node.liveRecordCount} live records`,
        tone: isSelected ? "selected" : "default",
        noWrap: true,
      };
    }
    if (node.kind === "tag") {
      return {
        text: `${node.tag} | ${node.assignmentMode} | ${node.liveRecordCount} live records`,
        tone: isSelected ? "selected" : "default",
        noWrap: true,
      };
    }
    return {
      text: buildOntologyExplorerEntitySummary(node.record),
      tone: isSelected ? "selected" : "default",
      noWrap: true,
    };
  });
}

function buildCategoryDetailLines(category: DerivedTagOntologyExplorerCategoryNode): DerivedTagTerminalLine[] {
  return [
    { text: category.category, tone: "section" },
    { text: `Families: ${category.familyCount}` },
    { text: `Tags: ${category.tagCount}` },
    { text: `Tagged canonical records: ${category.taggedRecordCount}` },
  ];
}

function buildFamilyDetailLines(family: DerivedTagOntologyExplorerFamilyNode): DerivedTagTerminalLine[] {
  return [
    { text: family.family, tone: "section" },
    { text: family.description },
    { text: `Category: ${family.category}` },
    { text: `Axis: ${family.axis}` },
    { text: `Scope: ${family.subcategories?.join(", ") ?? "(all subcategories)"}` },
    { text: `Variant inheritance: ${family.variantInheritance ? "yes" : "no"}` },
    { text: `Tags: ${family.tagCount}` },
    { text: `Distinct live records: ${family.liveRecordCount}` },
  ];
}

function buildTagDetailLines(tag: DerivedTagOntologyExplorerTagNode): DerivedTagTerminalLine[] {
  return [
    { text: tag.tag, tone: "section" },
    { text: tag.description },
    { text: `Category: ${tag.category}` },
    { text: `Family: ${tag.family}` },
    { text: `Scope: ${tag.subcategories?.join(", ") ?? "(all subcategories)"}` },
    { text: `Assignment mode: ${tag.assignmentMode}` },
    { text: `Native ontology policy: ${tag.nativeOntologyPolicy ?? "(none)"}` },
    { text: `Variant inheritance override: ${tag.variantInheritance === undefined ? "(inherit family setting)" : tag.variantInheritance ? "yes" : "no"}` },
    { text: `Live canonical records: ${tag.liveRecordCount}` },
    { text: `Record pages: ${tag.records.length}` },
    { text: `Authored rules: ${tag.authoredRuleCount}` },
    { text: `Exemplars: +${tag.exemplarPositiveCount} / -${tag.exemplarNegativeCount}` },
    { text: `Legacy seed migrations: ${tag.legacyMigrationDefinitionCount} definitions across ${tag.legacyMigrationRecordCount} records` },
    { text: `Adjacent tags: ${tag.adjacentTags?.join(", ") ?? "(none)"}` },
    { text: `Composite children: ${tag.compositeOfAnyTags?.join(", ") ?? "(none)"}` },
    { text: "Applies when:", tone: "section" },
    { text: tag.appliesWhen?.join(" | ") ?? "(none)", indent: 2 },
    { text: "Does not apply when:", tone: "section" },
    { text: tag.doesNotApplyWhen?.join(" | ") ?? "(none)", indent: 2 },
    { text: "Positive signals:", tone: "section" },
    { text: tag.positiveSignals?.join(" | ") ?? "(none)", indent: 2 },
    { text: "Negative signals:", tone: "section" },
    { text: tag.negativeSignals?.join(" | ") ?? "(none)", indent: 2 },
  ];
}

export function buildDerivedTagOntologyExplorerDetailLines(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagTerminalLine[] {
  const selection = getDerivedTagOntologyExplorerSelection(model, state);
  if (state.depth === "category" && selection.category) {
    return buildCategoryDetailLines(selection.category);
  }
  if (state.depth === "family" && selection.family) {
    return buildFamilyDetailLines(selection.family);
  }
  if (state.depth === "tag" && selection.tag) {
    return buildTagDetailLines(selection.tag);
  }
  if (selection.record) {
    return buildOntologyExplorerEntityDetailLines(selection.record.record);
  }
  return [{ text: "No ontology entry selected.", tone: "dim" }];
}

export function getDerivedTagOntologyExplorerDetailPaneWidth(
  width: number,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
): number {
  return getTerminalTwoPaneDetailWidth(width, layoutMode, 46);
}

export function buildVisibleDerivedTagOntologyExplorerDetailLines(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  width: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  return sliceRenderedTerminalLines(
    buildDerivedTagOntologyExplorerDetailLines(model, state),
    getDerivedTagOntologyExplorerDetailPaneWidth(width, layoutMode),
    state.detailScroll,
    bodyHeight,
  );
}

export function getDerivedTagOntologyExplorerDetailMetrics(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  width: number,
  height: number,
): { bodyHeight: number; maxDetailScroll: number; detailJumpSize: number; detailPageSize: number; selectionJumpSize: number } {
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const detailWidth = getDerivedTagOntologyExplorerDetailPaneWidth(width, layoutMode);
  const detailLines = buildDerivedTagOntologyExplorerDetailLines(model, state);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  return {
    bodyHeight,
    detailJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    detailPageSize: Math.max(1, bodyHeight - 1),
    maxDetailScroll: Math.max(0, renderedDetailLineCount - bodyHeight),
    selectionJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
  };
}

export function buildDerivedTagOntologyExplorerHelpLines(): DerivedTagTerminalLine[] {
  return [
    { text: "Navigation", tone: "section" },
    { text: "Up / Down or j / k: move selection within the active depth" },
    { text: "Ctrl+U / Ctrl+D: jump up or down by half a pane without wrapping" },
    { text: "Space / b: page down or up without wrapping" },
    { text: "gg / G or Home / End: jump to the first or last item in the current depth" },
    { text: "Tab or w: switch focus between the list and detail panes" },
    { text: "z: toggle focused detail view while detail has focus" },
    { text: "With detail focus, j/k, Ctrl+U/D, Space/b, Home/End, and Page Up/Down scroll the current detail page" },
    { text: "Enter or Right / l: drill deeper into the hierarchy" },
    { text: "Left / h or Backspace: go up one level" },
    { text: "/: enter live inline search for the current depth" },
    { text: "Esc: clear search first, otherwise go back" },
    { text: "Page Up / Page Down: scroll long details" },
    { text: "q: return to the top-level area selector" },
  ];
}

export function getPrintableOntologyExplorerKeyCharacter(input: string, key: Key): string | undefined {
  if (key.ctrl || key.meta) {
    return undefined;
  }
  return getPrintableInput(input, key);
}

export function isExactPrintableOntologyExplorerKey(input: string, key: Key, value: string): boolean {
  return getPrintableOntologyExplorerKeyCharacter(input, key) === value;
}
