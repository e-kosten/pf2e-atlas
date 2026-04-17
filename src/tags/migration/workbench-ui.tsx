import React from "react";
import { DatabaseSync } from "node:sqlite";

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { normalizeDerivedTag } from "../index.js";
import {
  openConfiguredIndex,
  parseInteger,
  writeDerivedTagMigrationSummary,
} from "./cli-utils.js";
import { getActionableSessionScopeKeys } from "./actionable-session-scope.js";
import { renderDerivedTagMigrationSessionSummary } from "./render.js";
import {
  getPublishedDerivedTagMigrationOntology,
  summarizeCurrentDerivedTagReviewQueue,
} from "./runtime-state.js";
import {
  compareDisplayText,
  compareManagedCategory,
  DERIVED_TAG_MANAGED_CATEGORIES,
} from "./list-sorting.js";
import { summarizeDerivedTagCategoryScopes } from "./category-scope-summary.js";
import { DerivedTagMigrationReviewScreen } from "./review-ui.js";
import { writeDerivedTagMigrationSession } from "./session-store.js";
import { buildDerivedTagMigrationSession } from "./session-builder.js";
import {
  TerminalTextScreen,
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSelectOption,
} from "./terminal-ui.js";
import { DerivedTagOntologyExplorerScreen } from "./ontology-explorer-screen.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
  DerivedTagReviewQueueSummaryItem,
} from "./types.js";

const ANY_CATEGORY = "__all_categories__";
const ANY_SUBCATEGORY = "__all_subcategories__";
const ANY_FAMILY = "__all_families__";
const ANY_TAG = "__all_tags__";

type TopLevelAreaId = "tag_refinement" | "ontology_search" | "search";

type WorkbenchArea = {
  id: TopLevelAreaId;
  label: string;
  description: string;
};

type TagRefinementMenuItem =
  | { kind: "review_queue_item"; label: string; queueItem: DerivedTagReviewQueueSummaryItem }
  | { kind: "review_all"; label: string }
  | { kind: "create_mode"; label: string; mode: DerivedTagMigrationMode }
  | { kind: "back"; label: string };

type WorkbenchRoute =
  | { kind: "areas" }
  | { kind: "tag_refinement" }
  | { kind: "search" }
  | { kind: "ontology"; db: DatabaseSync; cacheKey?: string }
  | { kind: "review"; session: DerivedTagMigrationSession };

export type WorkbenchState = {
  route: WorkbenchRoute;
  selectedAreaIndex: number;
  tagRefinementSelectedIndex: number;
};

export type WorkbenchAction =
  | { type: "move_area"; delta: number }
  | { type: "move_tag_refinement"; delta: number; itemCount: number }
  | { type: "set_route"; route: WorkbenchRoute }
  | { type: "set_tag_refinement_index"; index: number; itemCount: number };

type WorkbenchServices = {
  buildSession: typeof buildDerivedTagMigrationSession;
  openIndex: typeof openConfiguredIndex;
  summarizeQueue: typeof summarizeCurrentDerivedTagReviewQueue;
  writeSession: typeof writeDerivedTagMigrationSession;
  writeSummary: typeof writeDerivedTagMigrationSummary;
};

const DEFAULT_WORKBENCH_SERVICES: WorkbenchServices = {
  buildSession: buildDerivedTagMigrationSession,
  openIndex: openConfiguredIndex,
  summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
  writeSession: writeDerivedTagMigrationSession,
  writeSummary: writeDerivedTagMigrationSummary,
};

const WORKBENCH_AREAS: WorkbenchArea[] = [
  {
    id: "tag_refinement",
    label: "Tag Refinement",
    description: "Review authored queue items and create AI proposal, legacy-seed, legacy-rule, and exemplar-cleanup sessions.",
  },
  {
    id: "ontology_search",
    label: "Ontology Search",
    description: "Browse category -> family -> tag -> record and inspect how derived tags map onto live indexed records.",
  },
  {
    id: "search",
    label: "Search",
    description: "Future first-class search surface for exact lookup, hard-filter browsing, and ranked/semantic retrieval.",
  },
];

export function workbenchReducer(state: WorkbenchState, action: WorkbenchAction): WorkbenchState {
  switch (action.type) {
    case "move_area":
      return {
        ...state,
        selectedAreaIndex: moveSelectionWrapped(state.selectedAreaIndex, action.delta, WORKBENCH_AREAS.length),
      };
    case "move_tag_refinement":
      return {
        ...state,
        tagRefinementSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelectionWrapped(state.tagRefinementSelectedIndex, action.delta, action.itemCount),
      };
    case "set_tag_refinement_index":
      return {
        ...state,
        tagRefinementSelectedIndex: action.itemCount <= 0 ? 0 : Math.max(0, Math.min(action.index, action.itemCount - 1)),
      };
    case "set_route":
      return {
        ...state,
        route: action.route,
      };
    default:
      return state;
  }
}

export function createWorkbenchState(initialRoute: WorkbenchRoute = { kind: "areas" }): WorkbenchState {
  return {
    route: initialRoute,
    selectedAreaIndex: 0,
    tagRefinementSelectedIndex: 0,
  };
}

function formatModeLabel(mode: DerivedTagMigrationMode): string {
  if (mode === "proposal_review") {
    return "AI proposal review";
  }
  if (mode === "review_queue") {
    return "review queue";
  }
  return mode.replaceAll("_", " ");
}

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }
  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function buildScrollableLines<T extends { label: string }>(
  items: T[],
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, bodyHeight);
  const windowStart = clampWindowStart(selectedIndex, items.length, visibleCount);

  return items.slice(windowStart, windowStart + visibleCount).map((item, offset) => ({
    text: item.label,
    tone: windowStart + offset === selectedIndex ? "selected" : "default",
    noWrap: true,
  }));
}

function buildAreaDetailLines(queueItems: DerivedTagReviewQueueSummaryItem[]): DerivedTagTerminalLine[] {
  return [
    { text: "Tag Refinement", tone: "section" },
    { text: `${queueItems.length} pending review queue slice${queueItems.length === 1 ? "" : "s"}` },
    { text: "" },
    { text: "Ontology Search", tone: "section" },
    { text: "Browse the published ontology and drill from tags into live records." },
    { text: "" },
    { text: "Search", tone: "section" },
    { text: "Reserved for the future TUI search surface powered by the same project search capabilities as the MCP server." },
  ];
}

function buildTopLevelHelpLines(): DerivedTagTerminalLine[] {
  return [
    { text: "Top-Level Help", tone: "section" },
    { text: "Up / Down or j / k: move between areas" },
    { text: "Enter: open the selected area" },
    { text: "q: exit the TUI" },
  ];
}

function buildTagRefinementMenuItems(items: DerivedTagReviewQueueSummaryItem[]): TagRefinementMenuItem[] {
  const menuItems: TagRefinementMenuItem[] = [];
  if (items.length > 0) {
    menuItems.push({ kind: "review_all", label: "Review all pending queue items" });
    for (const item of items) {
      const kindLabel = item.kind === "assignment" ? "assignment" : "exemplar";
      const scope = item.kind === "assignment"
        ? `${item.category} ${item.family}.${item.tag}`
        : `${item.category} exemplar.${item.tag}`;
      menuItems.push({
        kind: "review_queue_item",
        label: `Review ${kindLabel} ${scope}  confidence=${item.confidence}  count=${item.count}`,
        queueItem: item,
      });
    }
  }
  menuItems.push(
    { kind: "create_mode", label: "Create legacy-seed review session", mode: "legacy_seed" },
    { kind: "create_mode", label: "Create legacy-rule review session", mode: "legacy_rule" },
    { kind: "create_mode", label: "Create exemplar-cleanup review session", mode: "exemplar_cleanup" },
    { kind: "create_mode", label: "Create AI proposal review session", mode: "proposal_review" },
    { kind: "back", label: "Back to top level" },
  );
  return menuItems;
}

function buildQueueLines(queueItems: DerivedTagReviewQueueSummaryItem[]): DerivedTagTerminalLine[] {
  if (queueItems.length === 0) {
    return [{ text: "No pending authored review items.", tone: "dim" }];
  }

  return queueItems.flatMap((item) => {
    const scope = item.kind === "assignment"
      ? `${item.category} ${item.family}.${item.tag}`
      : `${item.category} exemplar.${item.tag}`;
    return [
      { text: scope, tone: "section" as const },
      { text: `confidence=${item.confidence} count=${item.count}`, indent: 2 },
    ];
  });
}

function normalizeOptional(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function uniqueSorted(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => compareDisplayText(left, right) || left.localeCompare(right));
}

function getSessionScopeOntology() {
  return getPublishedDerivedTagMigrationOntology();
}

function buildCategorySelectOptions(
  mode: DerivedTagMigrationMode,
  db: DatabaseSync,
  required: boolean,
): DerivedTagTerminalSelectOption<string>[] {
  const scopeSummary = summarizeDerivedTagCategoryScopes(db, mode);
  const categoryOptions = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const detailLines = scopeSummary.categories.find((entry) => entry.category === category)?.detailLines ?? [];
    return {
      value: category,
      label: category,
      detailLines: [
        { text: category, tone: "section" },
        ...detailLines.map((line) => ({ text: line })),
      ],
    } satisfies DerivedTagTerminalSelectOption<string>;
  });

  if (required) {
    return categoryOptions;
  }

  return [
    {
      value: ANY_CATEGORY,
      label: "All categories",
      detailLines: [
        { text: "All categories", tone: "section" },
        ...scopeSummary.allCategoriesDetailLines.map((line) => ({ text: line })),
      ],
    },
    ...categoryOptions,
  ];
}

function listSubcategoriesForCategory(category: SearchCategory): SearchSubcategory[] {
  return uniqueSorted(
    getSessionScopeOntology().families
      .filter((family) => family.category === category)
      .flatMap((family) => family.subcategories ?? []),
  ) as SearchSubcategory[];
}

function buildSubcategorySelectOptions(category: SearchCategory): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const subcategories = listSubcategoriesForCategory(category);
  return [
    {
      value: ANY_SUBCATEGORY,
      label: "All subcategories",
      detailLines: [
        { text: `${category} / all subcategories`, tone: "section" },
        { text: "Keep the session scoped to the full category." },
      ],
    },
    ...subcategories.map((subcategory) => {
      const matchingFamilies = ontology.families.filter((family) =>
        family.category === category && (family.subcategories?.includes(subcategory) ?? false));
      const matchingTags = ontology.tags.filter((tag) => {
        if (tag.category !== category) {
          return false;
        }
        const family = ontology.familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
        return family?.subcategories?.includes(subcategory) ?? false;
      });
      return {
        value: subcategory,
        label: subcategory,
        detailLines: [
          { text: `${category}/${subcategory}`, tone: "section" },
          { text: `${matchingFamilies.length} families apply` },
          { text: `${matchingTags.length} tags apply` },
        ],
      } satisfies DerivedTagTerminalSelectOption<string>;
    }),
  ];
}

function familyMatchesScope(
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: { category: SearchCategory; family: string; subcategories?: SearchSubcategory[]; axis: string },
): boolean {
  if (category && family.category !== category) {
    return false;
  }
  if (!subcategory) {
    return true;
  }
  if (!family.subcategories || family.subcategories.length === 0) {
    return true;
  }
  return family.subcategories.includes(subcategory);
}

function buildFamilySelectOptions(
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  const familyOptions = ontology.families
    .filter((family) => familyMatchesScope(category, subcategory, family))
    .filter((family) => !actionableScope || actionableScope.familyKeys.has(`${family.category}:${normalizeDerivedTag(family.family)}` as `${SearchCategory}:${string}`))
    .sort((left, right) =>
      compareManagedCategory(left.category, right.category)
      || compareDisplayText(left.axis, right.axis)
      || compareDisplayText(left.family, right.family)
      || left.family.localeCompare(right.family))
    .map((family) => ({
      value: category ? family.family : `${family.category}:${family.family}`,
      label: category ? `${family.axis} / ${family.family}` : `${family.category} / ${family.axis} / ${family.family}`,
      detailLines: [
        { text: family.family, tone: "section" },
        { text: family.description },
        { text: `Category: ${family.category}` },
        { text: `Axis: ${family.axis}` },
        { text: `Scope: ${family.subcategories?.join(", ") ?? "(all subcategories)"}` },
        { text: `Variant inheritance: ${family.variantInheritance ? "yes" : "no"}` },
      ],
    } satisfies DerivedTagTerminalSelectOption<string>));

  return [
    {
      value: ANY_FAMILY,
      label: "All families",
      detailLines: [
        { text: "All families", tone: "section" },
        { text: "Keep family unspecified and review the wider queue slice." },
      ],
    } satisfies DerivedTagTerminalSelectOption<string>,
    ...familyOptions,
  ];
}

function tagMatchesScope(
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  familyKey: string | undefined,
  tag: { category: SearchCategory; family: string },
): boolean {
  if (category && tag.category !== category) {
    return false;
  }
  if (familyKey && normalizeDerivedTag(tag.family) !== normalizeDerivedTag(familyKey)) {
    return false;
  }
  if (!subcategory) {
    return true;
  }

  const family = getSessionScopeOntology().familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
  if (!family?.subcategories || family.subcategories.length === 0) {
    return true;
  }

  return family.subcategories.includes(subcategory);
}

function buildTagSelectOptions(
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
  required: boolean,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  const tagOptions = ontology.tags
    .filter((tag) => tagMatchesScope(category, subcategory, family, tag))
    .filter((tag) => !actionableScope || actionableScope.tagKeys.has(`${tag.category}:${normalizeDerivedTag(tag.tag)}` as `${SearchCategory}:${string}`))
    .sort((left, right) =>
      compareManagedCategory(left.category, right.category)
      || compareDisplayText(left.family, right.family)
      || compareDisplayText(left.tag, right.tag)
      || left.tag.localeCompare(right.tag))
    .map((tag) => {
      const family = ontology.familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
      return {
        value: category ? tag.tag : `${tag.category}:${tag.tag}`,
        label: category ? `${tag.family} / ${tag.tag}` : `${tag.category} / ${tag.family} / ${tag.tag}`,
        detailLines: [
          { text: tag.tag, tone: "section" },
          { text: tag.description },
          { text: `Category: ${tag.category}` },
          { text: `Family: ${tag.family}` },
          { text: `Axis: ${family?.axis ?? "(unknown)"}` },
          { text: `Scope: ${family?.subcategories?.join(", ") ?? "(all subcategories)"}` },
          { text: `Assignment mode: ${tag.assignmentMode}` },
        ],
      } satisfies DerivedTagTerminalSelectOption<string>;
    });

  if (required) {
    return tagOptions;
  }

  return [
    {
      value: ANY_TAG,
      label: "All tags",
      detailLines: [
        { text: "All tags", tone: "section" },
        { text: "Keep tag unspecified and create a broader review session." },
      ],
    },
    ...tagOptions,
  ];
}

async function promptCategory(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  db: DatabaseSync,
  mode: DerivedTagMigrationMode,
  required: boolean,
): Promise<SearchCategory | null | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: "Choose a category boundary for the session",
    prompt: "Categories",
    entries: buildCategorySelectOptions(mode, db, required),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_CATEGORY) {
    return null;
  }
  return value as SearchCategory;
}

async function promptSubcategory(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  category: SearchCategory,
): Promise<SearchSubcategory | null | undefined> {
  const options = buildSubcategorySelectOptions(category);
  if (options.length <= 1) {
    return null;
  }

  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: `Optionally narrow ${category} to a subcategory`,
    prompt: "Subcategories",
    entries: options,
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_SUBCATEGORY) {
    return null;
  }
  return value as SearchSubcategory;
}

async function promptTag(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
  required: boolean,
): Promise<{ category?: SearchCategory; tag?: string } | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: required
      ? "Choose the tag to review"
      : "Optionally narrow the session to one tag",
    prompt: "Tags",
    entries: buildTagSelectOptions(mode, category, subcategory, family, exemplarLimit, required),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_TAG) {
    return {};
  }
  if (!category) {
    const [resolvedCategory, resolvedTag] = value.split(":", 2);
    if (resolvedCategory && resolvedTag) {
      return {
        category: resolvedCategory as SearchCategory,
        tag: resolvedTag,
      };
    }
  }
  return { tag: value };
}

async function promptFamily(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): Promise<{ category?: SearchCategory; family?: string } | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: "Optionally narrow the queue to one ontology family",
    prompt: "Families",
    entries: buildFamilySelectOptions(mode, category, subcategory, exemplarLimit),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_FAMILY) {
    return {};
  }
  if (!category) {
    const [resolvedCategory, resolvedFamily] = value.split(":", 2);
    if (resolvedCategory && resolvedFamily) {
      return {
        category: resolvedCategory as SearchCategory,
        family: resolvedFamily,
      };
    }
  }
  return { family: value };
}

async function promptInteger(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  prompt: string,
  flagName: string,
): Promise<number | undefined> {
  while (true) {
    const value = normalizeOptional(await terminal.promptTextInput({
      title: "Session Scope",
      prompt,
    }));

    try {
      return parseInteger(value, flagName);
    } catch (error) {
      await terminal.pauseForAnyKey((error as Error).message);
    }
  }
}

async function promptCustomSessionOptions(
  terminal: ReturnType<typeof useDerivedTagTerminalApp>,
  db: DatabaseSync,
  mode: DerivedTagMigrationMode,
): Promise<{
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
} | undefined> {
  const requireCategory = mode === "legacy_rule";
  const categorySelection = await promptCategory(terminal, db, mode, requireCategory);
  if (categorySelection === undefined) {
    return undefined;
  }
  const category = categorySelection ?? undefined;

  const subcategorySelection = category ? await promptSubcategory(terminal, category) : null;
  if (subcategorySelection === undefined) {
    return undefined;
  }
  const subcategory = subcategorySelection ?? undefined;

  const exemplarLimit = mode === "exemplar_cleanup"
    ? await promptInteger(terminal, "exemplar-limit (blank for none)", "--exemplar-limit")
    : undefined;

  const familySelection = mode === "review_queue" || mode === "proposal_review" || mode === "exemplar_cleanup"
    ? await promptFamily(terminal, mode, category, subcategory, exemplarLimit)
    : {};
  if (familySelection === undefined) {
    return undefined;
  }
  const resolvedCategory = category ?? familySelection.category;
  const family = familySelection.family;

  const tagSelection = await promptTag(
    terminal,
    mode,
    resolvedCategory,
    subcategory,
    family,
    exemplarLimit,
    mode === "legacy_rule",
  );
  if (tagSelection === undefined) {
    return undefined;
  }
  const resolvedTagCategory = resolvedCategory ?? tagSelection.category;
  const tag = tagSelection.tag;

  const limit = await promptInteger(terminal, "limit (blank for default)", "--limit");

  return {
    category: resolvedTagCategory,
    subcategory,
    family,
    tag,
    limit,
    exemplarLimit,
  };
}

function TopLevelAreaScreen({
  selectedAreaIndex,
  queueItems,
  onOpenSelectedArea,
  onMove,
  onQuit,
}: {
  selectedAreaIndex: number;
  queueItems: DerivedTagReviewQueueSummaryItem[];
  onOpenSelectedArea: () => void;
  onMove: (delta: number) => void;
  onQuit: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (normalized === "ctrl_c" || normalized === "q" || normalized === "escape") {
      onQuit();
      return;
    }
    if (normalized === "up" || normalized === "k") {
      onMove(-1);
      return;
    }
    if (normalized === "down" || normalized === "j") {
      onMove(1);
      return;
    }
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Top-Level Help",
        body: buildTopLevelHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }
    if (normalized === "enter") {
      onOpenSelectedArea();
    }
  });

  return (
    <TerminalTwoPaneScreen
      title="PF2E Tag TUI"
      subtitle="Choose a first-class TUI area"
      left={{
        title: "Areas",
        lines: buildScrollableLines(WORKBENCH_AREAS, selectedAreaIndex, bodyHeight),
      }}
      right={{
        title: "Selected Area",
        lines: [
          { text: WORKBENCH_AREAS[selectedAreaIndex]?.label ?? "", tone: "section" },
          { text: WORKBENCH_AREAS[selectedAreaIndex]?.description ?? "" },
          { text: "" },
          ...buildAreaDetailLines(queueItems),
        ],
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter select  ? help  q quit", tone: "dim" },
        { text: `${queueItems.length} pending queue slice${queueItems.length === 1 ? "" : "s"}`, tone: "accent" },
      ]}
      leftWidth={32}
    />
  );
}

function SearchPlaceholderScreen({
  onBack,
}: {
  onBack: () => void;
}): React.JSX.Element {
  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (normalized === "q" || normalized === "backspace" || normalized === "left" || normalized === "escape" || normalized === "ctrl_c") {
      onBack();
    }
  });

  return (
    <TerminalTextScreen
      title="Search"
      body={[
        { text: "This area is reserved for the future first-class TUI search surface.", tone: "section" },
        { text: "" },
        { text: "Planned capabilities:" },
        { text: "Exact name lookup, category-aware hard filters, deterministic listing, and ranked/semantic search over the same indexed PF2E data surfaced by the MCP server.", indent: 2 },
        { text: "" },
        { text: "Entity pages in Ontology Search are the groundwork for this future surface.", tone: "dim" },
      ]}
      footer={[{ text: "q or Backspace return to top level", tone: "dim" }]}
    />
  );
}

function TagRefinementScreen({
  selectedIndex,
  queueItems,
  onBack,
  onMove,
  onOpenSelected,
  onQuickAction,
}: {
  selectedIndex: number;
  queueItems: DerivedTagReviewQueueSummaryItem[];
  onBack: () => void;
  onMove: (delta: number, itemCount: number) => void;
  onOpenSelected: (menuItems: TagRefinementMenuItem[]) => void;
  onQuickAction: (mode: "review_all" | DerivedTagMigrationMode) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const menuItems = buildTagRefinementMenuItems(queueItems);
  const clampedSelectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

  React.useEffect(() => {
    if (clampedSelectedIndex !== selectedIndex) {
      onMove(0, menuItems.length);
    }
  }, [clampedSelectedIndex, menuItems.length, onMove, selectedIndex]);

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);

    if (normalized === "ctrl_c" || normalized === "q" || normalized === "backspace" || normalized === "left" || normalized === "escape") {
      onBack();
      return;
    }
    if (normalized === "up" || normalized === "k") {
      onMove(-1, menuItems.length);
      return;
    }
    if (normalized === "down" || normalized === "j") {
      onMove(1, menuItems.length);
      return;
    }
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Tag Refinement Help",
        body: [
          { text: "Navigation", tone: "section" },
          { text: "Up / Down or j / k: move between tag-refinement rows" },
          { text: "Enter: open the selected row" },
          { text: "q or Backspace: return to top level" },
          { text: "" },
          { text: "Shortcuts", tone: "section" },
          { text: "a review all queue items" },
          { text: "s create a legacy-seed session" },
          { text: "r create a legacy-rule session" },
          { text: "e create an exemplar-cleanup session" },
          { text: "p create an AI proposal review session" },
        ],
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }
    if (normalized === "a" && queueItems.length > 0) {
      onQuickAction("review_all");
      return;
    }
    if (normalized === "s") {
      onQuickAction("legacy_seed");
      return;
    }
    if (normalized === "r") {
      onQuickAction("legacy_rule");
      return;
    }
    if (normalized === "e") {
      onQuickAction("exemplar_cleanup");
      return;
    }
    if (normalized === "p" || normalized === "n") {
      onQuickAction("proposal_review");
      return;
    }
    if (normalized === "enter") {
      onOpenSelected(menuItems);
    }
  });

  return (
    <TerminalTwoPaneScreen
      title="Tag Refinement"
      subtitle={`${queueItems.length} queue slice${queueItems.length === 1 ? "" : "s"} pending review`}
      left={{
        title: "Menu",
        lines: buildScrollableLines(menuItems, clampedSelectedIndex, bodyHeight),
      }}
      right={{
        title: "Pending Review Queue",
        lines: buildQueueLines(queueItems),
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter select  ? help  a review all  s/r/e/p create sessions  q/back top level", tone: "dim" },
        { text: `Selected: ${menuItems[clampedSelectedIndex]?.label ?? "(none)"}`, tone: "accent" },
      ]}
      leftWidth={48}
    />
  );
}

export function DerivedTagMigrationWorkbenchApp({
  rootPath,
  argv,
  onExit,
  initialRoute = { kind: "areas" } satisfies WorkbenchRoute,
  services = DEFAULT_WORKBENCH_SERVICES,
}: {
  rootPath: string;
  argv: string[];
  onExit: () => void;
  initialRoute?: WorkbenchRoute;
  services?: WorkbenchServices;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [state, dispatch] = React.useReducer(workbenchReducer, initialRoute, createWorkbenchState);
  const [busyMessage, setBusyMessage] = React.useState<string | null>(null);

  const queueItems = services.summarizeQueue();

  const runWithBusyState = React.useCallback(async <T,>(message: string, task: () => Promise<T>): Promise<T> => {
    setBusyMessage(message);
    try {
      return await task();
    } finally {
      setBusyMessage(null);
    }
  }, []);

  const openReviewSession = React.useCallback((session: DerivedTagMigrationSession) => {
    dispatch({ type: "set_route", route: { kind: "review", session } });
  }, []);

  const createSessionAndOpenReview = React.useCallback(async (
    mode: DerivedTagMigrationMode,
    options: {
      category?: SearchCategory;
      subcategory?: SearchSubcategory;
      decisionKind?: DerivedTagMigrationReviewDecisionKind;
      family?: string;
      tag?: string;
      limit?: number;
      exemplarLimit?: number;
    },
  ) => {
    await runWithBusyState(`Preparing ${formatModeLabel(mode)} session...`, async () => {
      const { db } = await services.openIndex(argv);
      try {
        const session = services.buildSession(db, {
          mode,
          ...options,
        });
        await services.writeSession(rootPath, session);
        await services.writeSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
        openReviewSession(session);
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatModeLabel(mode)} session.\n\n${(error as Error).message}`);
      } finally {
        db.close();
      }
    });
  }, [argv, openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const startCustomSession = React.useCallback(async (mode: DerivedTagMigrationMode) => {
    await runWithBusyState(`Loading ${formatModeLabel(mode)} session options...`, async () => {
      const { db } = await services.openIndex(argv);
      try {
        const options = await promptCustomSessionOptions(terminal, db, mode);
        if (options) {
          const session = services.buildSession(db, {
            mode,
            ...options,
          });
          await services.writeSession(rootPath, session);
          await services.writeSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
          openReviewSession(session);
        }
      } catch (error) {
        await terminal.pauseForAnyKey(`Could not create the ${formatModeLabel(mode)} session.\n\n${(error as Error).message}`);
      } finally {
        db.close();
      }
    });
  }, [argv, openReviewSession, rootPath, runWithBusyState, services, terminal]);

  const openOntology = React.useCallback(async () => {
    await runWithBusyState("Opening ontology explorer...", async () => {
      const { db, config } = await services.openIndex(argv);
      dispatch({
        type: "set_route",
        route: {
          kind: "ontology",
          cacheKey: config.indexPath,
          db,
        },
      });
    });
  }, [argv, runWithBusyState, services]);

  const openSelectedArea = React.useCallback(() => {
    const selectedArea = WORKBENCH_AREAS[state.selectedAreaIndex];
    if (!selectedArea) {
      return;
    }
    if (selectedArea.id === "tag_refinement") {
      dispatch({ type: "set_route", route: { kind: "tag_refinement" } });
      return;
    }
    if (selectedArea.id === "ontology_search") {
      void openOntology();
      return;
    }
    dispatch({ type: "set_route", route: { kind: "search" } });
  }, [openOntology, state.selectedAreaIndex]);

  const openSelectedTagRefinementItem = React.useCallback((menuItems: TagRefinementMenuItem[]) => {
    const selectedItem = menuItems[state.tagRefinementSelectedIndex];
    if (!selectedItem) {
      return;
    }
    if (selectedItem.kind === "back") {
      dispatch({ type: "set_route", route: { kind: "areas" } });
      return;
    }
    if (selectedItem.kind === "review_all") {
      void createSessionAndOpenReview("review_queue", {});
      return;
    }
    if (selectedItem.kind === "review_queue_item") {
      void createSessionAndOpenReview("review_queue", {
        decisionKind: selectedItem.queueItem.kind,
        category: selectedItem.queueItem.category,
        family: selectedItem.queueItem.family,
        tag: selectedItem.queueItem.tag,
      });
      return;
    }
    if (selectedItem.kind === "create_mode") {
      void startCustomSession(selectedItem.mode);
    }
  }, [createSessionAndOpenReview, startCustomSession, state.tagRefinementSelectedIndex]);

  const runQuickTagRefinementAction = React.useCallback((mode: "review_all" | DerivedTagMigrationMode) => {
    if (mode === "review_all") {
      void createSessionAndOpenReview("review_queue", {});
      return;
    }
    void startCustomSession(mode);
  }, [createSessionAndOpenReview, startCustomSession]);

  if (busyMessage) {
    return (
      <TerminalTextScreen
        title="PF2E Tag TUI"
        body={[{ text: busyMessage, tone: "section" }]}
        footer={[{ text: "Working...", tone: "dim" }]}
      />
    );
  }

  if (state.route.kind === "ontology") {
    const route = state.route;
    return (
      <DerivedTagOntologyExplorerScreen
        db={route.db}
        options={{ cacheKey: route.cacheKey }}
        onExit={() => {
          route.db.close();
          dispatch({ type: "set_route", route: { kind: "areas" } });
        }}
      />
    );
  }

  if (state.route.kind === "review") {
    return (
      <DerivedTagMigrationReviewScreen
        rootPath={rootPath}
        initialSession={state.route.session}
        onComplete={() => {
          dispatch({ type: "set_route", route: { kind: "tag_refinement" } });
        }}
      />
    );
  }

  if (state.route.kind === "search") {
    return (
      <SearchPlaceholderScreen
        onBack={() => dispatch({ type: "set_route", route: { kind: "areas" } })}
      />
    );
  }

  if (state.route.kind === "tag_refinement") {
    return (
      <TagRefinementScreen
        selectedIndex={state.tagRefinementSelectedIndex}
        queueItems={queueItems}
        onBack={() => dispatch({ type: "set_route", route: { kind: "areas" } })}
        onMove={(delta, itemCount) => dispatch(delta === 0
          ? { type: "set_tag_refinement_index", index: Math.max(0, Math.min(state.tagRefinementSelectedIndex, Math.max(0, itemCount - 1))), itemCount }
          : { type: "move_tag_refinement", delta, itemCount })}
        onOpenSelected={openSelectedTagRefinementItem}
        onQuickAction={runQuickTagRefinementAction}
      />
    );
  }

  return (
    <TopLevelAreaScreen
      selectedAreaIndex={state.selectedAreaIndex}
      queueItems={queueItems}
      onMove={(delta) => dispatch({ type: "move_area", delta })}
      onOpenSelectedArea={openSelectedArea}
      onQuit={onExit}
    />
  );
}

function WorkbenchRunner({
  rootPath,
  argv,
}: {
  rootPath: string;
  argv: string[];
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return (
    <DerivedTagMigrationWorkbenchApp
      rootPath={rootPath}
      argv={argv}
      onExit={() => terminal.exitApp()}
    />
  );
}

export async function runDerivedTagMigrationWorkbenchApp(
  rootPath: string,
  argv: string[],
): Promise<void> {
  await runDerivedTagTerminalApp(
    <WorkbenchRunner rootPath={rootPath} argv={argv} />,
  );
}
