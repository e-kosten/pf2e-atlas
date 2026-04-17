import { DatabaseSync } from "node:sqlite";

import type { SearchCategory } from "../../types.js";
import {
  buildDerivedTagOntologyExplorerModel,
  filterOntologyExplorerNodes,
  type DerivedTagOntologyExplorerCategoryNode,
  type DerivedTagOntologyExplorerFamilyNode,
  type DerivedTagOntologyExplorerModel,
  type DerivedTagOntologyExplorerTagNode,
} from "./ontology-explorer-data.js";
import {
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  pauseForAnyKey,
  promptTerminalTextInput,
  readTerminalKey,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSession,
} from "./terminal-ui.js";

export type DerivedTagOntologyExplorerDepth = "category" | "family" | "tag";

export type DerivedTagOntologyExplorerState = {
  depth: DerivedTagOntologyExplorerDepth;
  selectedCategoryKey?: SearchCategory;
  selectedFamilyKey?: string;
  selectedTagKey?: string;
  filter: string;
  detailScroll: number;
};

type ExplorerSelection = {
  categories: DerivedTagOntologyExplorerCategoryNode[];
  category?: DerivedTagOntologyExplorerCategoryNode;
  families: DerivedTagOntologyExplorerFamilyNode[];
  family?: DerivedTagOntologyExplorerFamilyNode;
  tags: DerivedTagOntologyExplorerTagNode[];
  tag?: DerivedTagOntologyExplorerTagNode;
};

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

function getSelection(model: DerivedTagOntologyExplorerModel, state: DerivedTagOntologyExplorerState): ExplorerSelection {
  const category = findCategory(model, state.selectedCategoryKey) ?? model.categories[0];
  const families = category?.families ?? [];
  const family = findFamily(category, state.selectedFamilyKey) ?? families[0];
  const tags = family?.tags ?? [];
  const tag = findTag(family, state.selectedTagKey) ?? tags[0];

  return {
    categories: model.categories,
    category,
    families,
    family,
    tags,
    tag,
  };
}

export function createDerivedTagOntologyExplorerState(
  model: DerivedTagOntologyExplorerModel,
): DerivedTagOntologyExplorerState {
  return {
    depth: "category",
    selectedCategoryKey: model.categories[0]?.key,
    selectedFamilyKey: model.categories[0]?.families[0]?.key,
    selectedTagKey: model.categories[0]?.families[0]?.tags[0]?.key,
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
  };

  if (state.depth === "category") {
    const visibleCategories = filterOntologyExplorerNodes(selection.categories, state.filter);
    if (visibleCategories.length > 0 && !visibleCategories.some((category) => category.key === nextState.selectedCategoryKey)) {
      nextState = {
        ...nextState,
        selectedCategoryKey: visibleCategories[0]!.key,
        selectedFamilyKey: visibleCategories[0]!.families[0]?.key,
        selectedTagKey: visibleCategories[0]!.families[0]?.tags[0]?.key,
      };
    }
  } else if (state.depth === "family") {
    const visibleFamilies = filterOntologyExplorerNodes(selection.families, state.filter);
    if (visibleFamilies.length > 0 && !visibleFamilies.some((family) => family.key === nextState.selectedFamilyKey)) {
      nextState = {
        ...nextState,
        selectedFamilyKey: visibleFamilies[0]!.key,
        selectedTagKey: visibleFamilies[0]!.tags[0]?.key,
      };
    }
  } else {
    const visibleTags = filterOntologyExplorerNodes(selection.tags, state.filter);
    if (visibleTags.length > 0 && !visibleTags.some((tag) => tag.key === nextState.selectedTagKey)) {
      nextState = {
        ...nextState,
        selectedTagKey: visibleTags[0]!.key,
      };
    }
  }

  return nextState;
}

function getActiveNodes(model: DerivedTagOntologyExplorerModel, state: DerivedTagOntologyExplorerState): Array<
  DerivedTagOntologyExplorerCategoryNode | DerivedTagOntologyExplorerFamilyNode | DerivedTagOntologyExplorerTagNode
> {
  const selection = getSelection(model, state);
  if (state.depth === "category") {
    return filterOntologyExplorerNodes(selection.categories, state.filter);
  }
  if (state.depth === "family") {
    return filterOntologyExplorerNodes(selection.families, state.filter);
  }
  return filterOntologyExplorerNodes(selection.tags, state.filter);
}

export function moveDerivedTagOntologyExplorerSelection(
  model: DerivedTagOntologyExplorerModel,
  state: DerivedTagOntologyExplorerState,
  delta: number,
): DerivedTagOntologyExplorerState {
  const nextState = normalizeDerivedTagOntologyExplorerState(model, state);
  const nodes = getActiveNodes(model, nextState);
  if (nodes.length === 0) {
    return nextState;
  }

  const currentIndex = nextState.depth === "category"
    ? nodes.findIndex((node) => node.kind === "category" && node.key === nextState.selectedCategoryKey)
    : nextState.depth === "family"
      ? nodes.findIndex((node) => node.kind === "family" && node.key === nextState.selectedFamilyKey)
      : nodes.findIndex((node) => node.kind === "tag" && node.key === nextState.selectedTagKey);
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
      detailScroll: 0,
    });
  }
  if (target.kind === "family") {
    return normalizeDerivedTagOntologyExplorerState(model, {
      ...nextState,
      selectedFamilyKey: target.key,
      selectedTagKey: target.tags[0]?.key,
      detailScroll: 0,
    });
  }
  return normalizeDerivedTagOntologyExplorerState(model, {
    ...nextState,
    selectedTagKey: target.key,
    detailScroll: 0,
  });
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
  return nextState;
}

export function popDerivedTagOntologyExplorerDepth(
  state: DerivedTagOntologyExplorerState,
): DerivedTagOntologyExplorerState {
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

function buildBreadcrumb(selection: ExplorerSelection, state: DerivedTagOntologyExplorerState): string {
  const segments = ["ontology"];
  if (selection.category) {
    segments.push(selection.category.category);
  }
  if (state.depth !== "category" && selection.family) {
    segments.push(selection.family.family);
  }
  if (state.depth === "tag" && selection.tag) {
    segments.push(selection.tag.tag);
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

  const selectedIndex = state.depth === "category"
    ? nodes.findIndex((node) => node.kind === "category" && node.key === selection.category?.key)
    : state.depth === "family"
      ? nodes.findIndex((node) => node.kind === "family" && node.key === selection.family?.key)
      : nodes.findIndex((node) => node.kind === "tag" && node.key === selection.tag?.key);
  const windowStart = clampWindowStart(Math.max(0, selectedIndex), nodes.length, visibleCount);

  return nodes.slice(windowStart, windowStart + visibleCount).map((node, offset) => {
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
    return {
      text: `${node.tag} | ${node.assignmentMode} | ${node.liveRecordCount} live records`,
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
  if (selection.tag) {
    return buildTagDetailLines(selection.tag);
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
    title: "Ontology Explorer Help",
    body: [
      { text: "Navigation", tone: "section" },
      { text: "Up / Down or j / k: move selection within the active depth" },
      { text: "Enter or Right / l: drill deeper into the hierarchy" },
      { text: "Left / h or Backspace: go up one level" },
      { text: "/: set an inline filter for the current depth" },
      { text: "Esc: clear the current filter" },
      { text: "Page Up / Page Down or Ctrl+U / Ctrl+D: scroll long details" },
      { text: "q: return to the workbench" },
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

  while (true) {
    state = normalizeDerivedTagOntologyExplorerState(model, state);
    const selection = getSelection(model, state);
    const detailLines = buildDetailLines(model, state);
    const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
      hasSubtitle: true,
      footerLineCount: 2,
    }));
    const maxDetailScroll = Math.max(0, detailLines.length - bodyHeight);
    if (state.detailScroll > maxDetailScroll) {
      state = { ...state, detailScroll: maxDetailScroll };
    }

    renderTerminalTwoPaneScreen(terminalSession, {
      title: "Derived-Tag Ontology Explorer",
      subtitle: `${buildBreadcrumb(selection, state)} | depth ${state.depth} | filter ${state.filter || "(none)"}`,
      left: {
        title: state.depth === "category" ? "Categories" : state.depth === "family" ? "Families" : "Tags",
        lines: buildActiveListLines(terminalSession, model, state),
      },
      right: {
        title: state.depth === "category" ? "Category Details" : state.depth === "family" ? "Family Details" : "Tag Details",
        lines: buildVisibleDetailLines(terminalSession, model, state),
      },
      footer: [
        { text: "Up/Down or j/k move  Enter/right drill  Left/backspace up  / filter  Esc clear  ? help  q quit", tone: "dim" },
        { text: `Detail scroll ${state.detailScroll}/${maxDetailScroll}`, tone: "accent" },
      ],
      leftWidth: 46,
    });

    const key = await readTerminalKey(terminalSession);
    const normalized = key.normalizedName;
    if (normalized === "ctrl_c" || normalized === "q") {
      return;
    }
    if (normalized === "?") {
      renderOntologyExplorerHelp(terminalSession);
      await readTerminalKey(terminalSession);
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
    if (normalized === "right" || normalized === "l" || normalized === "enter" || normalized === "kp_enter") {
      state = drillIntoDerivedTagOntologyExplorer(model, state);
      continue;
    }
    if (normalized === "left" || normalized === "h" || normalized === "backspace") {
      state = popDerivedTagOntologyExplorerDepth(state);
      continue;
    }
    if (normalized === "escape") {
      state = { ...state, filter: "", detailScroll: 0 };
      continue;
    }
    if (normalized === "/" || normalized === "slash") {
      const filter = await promptTerminalTextInput(terminalSession, {
        title: "Ontology Explorer Filter",
        prompt: `Filter ${state.depth} entries`,
        defaultValue: state.filter,
      });
      state = setDerivedTagOntologyExplorerFilter(model, state, filter ?? "");
      continue;
    }
    if (normalized === "page_down" || normalized === "ctrl_d") {
      state = { ...state, detailScroll: Math.min(maxDetailScroll, state.detailScroll + Math.max(1, Math.floor(bodyHeight / 2))) };
      continue;
    }
    if (normalized === "page_up" || normalized === "ctrl_u") {
      state = { ...state, detailScroll: Math.max(0, state.detailScroll - Math.max(1, Math.floor(bodyHeight / 2))) };
      continue;
    }
    if (normalized === "home") {
      await pauseForAnyKey(terminalSession, "Use / to filter, Enter to drill in, and q to return to the workbench.");
    }
  }
}
