import { DatabaseSync } from "node:sqlite";

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
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  pauseForAnyKey,
  readTerminalKey,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  type DerivedTagTerminalKey,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSession,
} from "./terminal-ui.js";

export type DerivedTagOntologyExplorerDepth = "category" | "family" | "tag" | "record";
export type DerivedTagOntologyExplorerPaneFocus = "list" | "detail";

export type DerivedTagOntologyExplorerState = {
  depth: DerivedTagOntologyExplorerDepth;
  selectedCategoryKey?: SearchCategory;
  selectedFamilyKey?: string;
  selectedTagKey?: string;
  selectedRecordKey?: string;
  filter: string;
  detailScroll: number;
};

type ExplorerNode =
  | DerivedTagOntologyExplorerCategoryNode
  | DerivedTagOntologyExplorerFamilyNode
  | DerivedTagOntologyExplorerTagNode
  | DerivedTagOntologyExplorerRecordNode;

type ExplorerSelection = {
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

function getSelection(model: DerivedTagOntologyExplorerModel, state: DerivedTagOntologyExplorerState): ExplorerSelection {
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

function getNodesForDepth(selection: ExplorerSelection, depth: DerivedTagOntologyExplorerDepth): ExplorerNode[] {
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

function getSelectedKeyForDepth(selection: ExplorerSelection, state: DerivedTagOntologyExplorerState): string | undefined {
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
  const selection = getSelection(model, state);
  let nextState: DerivedTagOntologyExplorerState = {
    ...state,
    selectedCategoryKey: selection.category?.key,
    selectedFamilyKey: selection.family?.key,
    selectedTagKey: selection.tag?.key,
    selectedRecordKey: selection.record?.key,
  };

  const nodes = filterOntologyExplorerNodes(getNodesForDepth(selection, state.depth), state.filter);
  const selectedKey = getSelectedKeyForDepth(selection, nextState);
  if (nodes.length > 0 && !nodes.some((node) => node.key === selectedKey)) {
    const first = nodes[0]!;
    if (first.kind === "category") {
      nextState = {
        ...nextState,
        selectedCategoryKey: first.key,
        selectedFamilyKey: first.families[0]?.key,
        selectedTagKey: first.families[0]?.tags[0]?.key,
        selectedRecordKey: first.families[0]?.tags[0]?.records[0]?.key,
      };
    } else if (first.kind === "family") {
      nextState = {
        ...nextState,
        selectedFamilyKey: first.key,
        selectedTagKey: first.tags[0]?.key,
        selectedRecordKey: first.tags[0]?.records[0]?.key,
      };
    } else if (first.kind === "tag") {
      nextState = {
        ...nextState,
        selectedTagKey: first.key,
        selectedRecordKey: first.records[0]?.key,
      };
    } else {
      nextState = {
        ...nextState,
        selectedRecordKey: first.key,
      };
    }
  }

  return nextState;
}

function getActiveNodes(model: DerivedTagOntologyExplorerModel, state: DerivedTagOntologyExplorerState): ExplorerNode[] {
  const selection = getSelection(model, state);
  return filterOntologyExplorerNodes(getNodesForDepth(selection, state.depth), state.filter);
}

export function moveDerivedTagOntologyExplorerSelection(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  delta: number,
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  const selection = getSelection(model, nextState);
  const nodes = getActiveNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const currentIndex = nodes.findIndex((node) => node.key === getSelectedKeyForDepth(selection, nextState));
  const targetIndex = moveSelectionWrapped(Math.max(0, currentIndex), delta, nodes.length);
  const target = nodes[targetIndex];
  if (!target) {
    return nextState;
  }

  if (target.kind === "category") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...nextState,
      selectedCategoryKey: target.key,
      selectedFamilyKey: target.families[0]?.key,
      selectedTagKey: target.families[0]?.tags[0]?.key,
      selectedRecordKey: target.families[0]?.tags[0]?.records[0]?.key,
      detailScroll: 0,
    });
  }
  if (target.kind === "family") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...nextState,
      selectedFamilyKey: target.key,
      selectedTagKey: target.tags[0]?.key,
      selectedRecordKey: target.tags[0]?.records[0]?.key,
      detailScroll: 0,
    });
  }
  if (target.kind === "tag") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...nextState,
      selectedTagKey: target.key,
      selectedRecordKey: target.records[0]?.key,
      detailScroll: 0,
    });
  }
  return normalizeDerivedTagOntologyExplorerState(model, {
    ...nextState,
    selectedRecordKey: target.key,
    detailScroll: 0,
  });
}

export function moveDerivedTagOntologyExplorerSelectionToBoundary(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  boundary: "start" | "end",
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  const selection = getSelection(model, nextState);
  const nodes = getActiveNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const target = boundary === "start" ? nodes[0] : nodes[nodes.length - 1];
  if (!target) {
    return nextState;
  }

  return moveDerivedTagOntologyExplorerSelection(
    model,
    {
      ...nextState,
      selectedCategoryKey: target.kind === "category" ? target.key : nextState.selectedCategoryKey,
      selectedFamilyKey: target.kind === "family" ? target.key : nextState.selectedFamilyKey,
      selectedTagKey: target.kind === "tag" ? target.key : nextState.selectedTagKey,
      selectedRecordKey: target.kind === "record" ? target.key : nextState.selectedRecordKey,
    },
    0,
  );
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

function buildBreadcrumb(selection: ExplorerSelection, state: DerivedTagOntologyExplorerState): string {
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

function buildActiveListLines(
  terminalSession: DerivedTagTerminalSession,
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const selection = getSelection(model, state);
  const nodes = getActiveNodes(model, state);
  if (nodes.length === 0) {
    return [{ text: "No nodes match the current filter.", tone: "dim" }];
  }

  const selectedIndex = nodes.findIndex((node) => node.key === getSelectedKeyForDepth(selection, state));
  const windowStart = clampWindowStart(Math.max(0, selectedIndex), nodes.length, visibleCount);
  const visibleNodes = nodes.slice(windowStart, windowStart + visibleCount);

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

function getPrintableKeyCharacter(key: DerivedTagTerminalKey): string | undefined {
  if (!key.data.isCharacter) {
    return undefined;
  }
  if (typeof key.data.codepoint === "number") {
    return String.fromCodePoint(key.data.codepoint);
  }
  if (key.name === "space") {
    return " ";
  }
  return key.name.length === 1 ? key.name : undefined;
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

function buildDetailLines(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagTerminalLine[] {
  const selection = getSelection(model, state);
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

function buildVisibleDetailLines(
  terminalSession: DerivedTagTerminalSession,
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
): DerivedTagTerminalLine[] {
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const detailLines = buildDetailLines(model, state);
  return detailLines.slice(state.detailScroll, state.detailScroll + bodyHeight);
}

function renderOntologyExplorerHelp(terminalSession: DerivedTagTerminalSession): void {
  renderTerminalTextScreen(terminalSession, {
    title: "Ontology Search Help",
    body: [
      { text: "Navigation", tone: "section" },
      { text: "Up / Down or j / k: move selection within the active depth" },
      { text: "Ctrl+U / Ctrl+D: jump selection up or down by half a pane" },
      { text: "Home / End: jump to the first or last item in the current depth" },
      { text: "Tab: switch focus between the list and detail panes" },
      { text: "With detail focus, j/k, Ctrl+U/D, Home/End, and Page Up/Down scroll the current detail page" },
      { text: "Enter or Right / l: drill deeper into the hierarchy" },
      { text: "Left / h or Backspace: go up one level" },
      { text: "/: enter live inline search for the current depth" },
      { text: "Esc: clear search first, otherwise go back" },
      { text: "Page Up / Page Down: scroll long details" },
      { text: "q: return to the top-level area selector" },
    ],
    footer: [{ text: "Press any key to return.", tone: "dim" }],
  });
}

export async function runDerivedTagOntologyExplorerUi(
  db: DatabaseSync,
  terminalSession: DerivedTagTerminalSession,
): Promise<void> {
  const model = buildDerivedTagOntologyExplorerModel(db);
  let state = createDerivedTagOntologyExplorerState(model);
  let activePane: DerivedTagOntologyExplorerPaneFocus = "list";
  let searchMode = false;
  let searchInput = state.filter;

  while (true) {
    state = normalizeDerivedTagOntologyExplorerState(model, state);
    const selection = getSelection(model, state);
    const detailLines = buildDetailLines(model, state);
    const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
      hasSubtitle: true,
      footerLineCount: 2,
    }));
    const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
    const detailJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
    const detailPageSize = Math.max(1, bodyHeight - 1);
    const maxDetailScroll = Math.max(0, detailLines.length - bodyHeight);
    if (state.detailScroll > maxDetailScroll) {
      state = { ...state, detailScroll: maxDetailScroll };
    }

    renderTerminalTwoPaneScreen(terminalSession, {
      title: "Ontology Search",
      subtitle: `${buildBreadcrumb(selection, state)} | depth ${state.depth} | ${searchMode ? `/${searchInput}` : `/${state.filter}`}`,
      left: {
        title: `${activePane === "list" ? "> " : ""}${state.depth === "category"
          ? "Categories"
          : state.depth === "family"
            ? "Families"
            : state.depth === "tag"
              ? "Tags"
              : "Records"}`,
        lines: buildActiveListLines(terminalSession, model, state),
      },
      right: {
        title: `${activePane === "detail" ? "> " : ""}${state.depth === "category"
          ? "Category Details"
          : state.depth === "family"
            ? "Family Details"
            : state.depth === "tag"
              ? "Tag Details"
              : "Record Details"}`,
        lines: buildVisibleDetailLines(terminalSession, model, state),
      },
      footer: [
        {
          text: searchMode
            ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
            : "Tab focus  Up/Down or j/k move-scroll  Ctrl+U/D jump  Home/End edge  Enter/right drill  Left/backspace up  / search  Esc back/clear  ? help  q back",
          tone: "dim",
        },
        {
          text: searchMode
            ? `Search /${searchInput}`
            : `${activePane} focus | Detail scroll ${state.detailScroll}/${maxDetailScroll}`,
          tone: "accent",
        },
      ],
      leftWidth: 46,
    });

    const key = await readTerminalKey(terminalSession);
    const normalized = key.normalizedName;
    if (normalized === "ctrl_c" || normalized === "q") {
      return;
    }
    if (searchMode) {
      if (normalized === "enter" || normalized === "kp_enter") {
        searchMode = false;
        continue;
      }
      if (normalized === "backspace") {
        searchInput = searchInput.slice(0, -1);
        state = setDerivedTagOntologyExplorerFilter(model, state, searchInput);
        continue;
      }
      if (normalized === "escape") {
        searchMode = false;
        searchInput = "";
        state = setDerivedTagOntologyExplorerFilter(model, state, "");
        continue;
      }

      const printableCharacter = getPrintableKeyCharacter(key);
      if (printableCharacter) {
        searchInput += printableCharacter;
        state = setDerivedTagOntologyExplorerFilter(model, state, searchInput);
      }
      continue;
    }
    if (normalized === "tab" || normalized === "shift_tab") {
      activePane = activePane === "list" ? "detail" : "list";
      continue;
    }
    if (normalized === "?") {
      renderOntologyExplorerHelp(terminalSession);
      await readTerminalKey(terminalSession);
      continue;
    }
    if (activePane === "detail") {
      if (normalized === "up" || normalized === "k") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, -1, maxDetailScroll);
        continue;
      }
      if (normalized === "down" || normalized === "j") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, 1, maxDetailScroll);
        continue;
      }
      if (normalized === "ctrl_d") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, detailJumpSize, maxDetailScroll);
        continue;
      }
      if (normalized === "ctrl_u") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, -detailJumpSize, maxDetailScroll);
        continue;
      }
      if (normalized === "page_down") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, detailPageSize, maxDetailScroll);
        continue;
      }
      if (normalized === "page_up") {
        state = moveDerivedTagOntologyExplorerDetailScroll(state, -detailPageSize, maxDetailScroll);
        continue;
      }
      if (normalized === "home") {
        state = moveDerivedTagOntologyExplorerDetailScrollToBoundary(state, "start", maxDetailScroll);
        continue;
      }
      if (normalized === "end") {
        state = moveDerivedTagOntologyExplorerDetailScrollToBoundary(state, "end", maxDetailScroll);
        continue;
      }
      if (normalized === "left" || normalized === "h" || normalized === "backspace" || normalized === "escape") {
        activePane = "list";
        continue;
      }
      continue;
    }
    if (normalized === "up" || normalized === "k") {
      state = moveDerivedTagOntologyExplorerSelection(model, state, -1);
      continue;
    }
    if (normalized === "down" || normalized === "j") {
      state = moveDerivedTagOntologyExplorerSelection(model, state, 1);
      continue;
    }
    if (normalized === "ctrl_d") {
      state = moveDerivedTagOntologyExplorerSelection(model, state, selectionJumpSize);
      continue;
    }
    if (normalized === "ctrl_u") {
      state = moveDerivedTagOntologyExplorerSelection(model, state, -selectionJumpSize);
      continue;
    }
    if (normalized === "home") {
      state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "start");
      continue;
    }
    if (normalized === "end") {
      state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "end");
      continue;
    }
    if (normalized === "right" || normalized === "l" || normalized === "enter" || normalized === "kp_enter") {
      state = drillIntoDerivedTagOntologyExplorer(model, state);
      activePane = "list";
      continue;
    }
    if (normalized === "left" || normalized === "h" || normalized === "backspace") {
      const nextState = popDerivedTagOntologyExplorerDepth(state);
      if (nextState.depth === state.depth) {
        return;
      }
      state = nextState;
      activePane = "list";
      continue;
    }
    if (normalized === "escape") {
      if (state.filter) {
        searchInput = "";
        state = setDerivedTagOntologyExplorerFilter(model, state, "");
        continue;
      }
      const nextState = popDerivedTagOntologyExplorerDepth(state);
      if (nextState.depth === state.depth) {
        return;
      }
      state = nextState;
      activePane = "list";
      continue;
    }
    if (normalized === "/" || normalized === "slash") {
      searchMode = true;
      searchInput = state.filter;
      continue;
    }
    if (normalized === "page_down") {
      state = { ...state, detailScroll: Math.min(maxDetailScroll, state.detailScroll + Math.max(1, Math.floor(bodyHeight / 2))) };
      continue;
    }
    if (normalized === "page_up") {
      state = { ...state, detailScroll: Math.max(0, state.detailScroll - Math.max(1, Math.floor(bodyHeight / 2))) };
      continue;
    }
  }
}
