#!/usr/bin/env node

import { DatabaseSync } from "node:sqlite";

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { normalizeDerivedTag } from "../index.js";
import {
  openConfiguredIndex,
  parseInteger,
  writeDerivedTagMigrationSummary,
} from "../migration/cli-utils.js";
import { runDerivedTagOntologyExplorerUi } from "../migration/ontology-explorer-ui.js";
import { renderDerivedTagMigrationSessionSummary } from "../migration/render.js";
import {
  getPublishedDerivedTagMigrationOntology,
  summarizeCurrentDerivedTagReviewQueue,
} from "../migration/runtime-state.js";
import {
  compareDisplayText,
  compareManagedCategory,
  DERIVED_TAG_MANAGED_CATEGORIES,
} from "../migration/list-sorting.js";
import { summarizeDerivedTagCategoryScopes } from "../migration/category-scope-summary.js";
import { runDerivedTagMigrationReviewUi } from "../migration/review-ui.js";
import { writeDerivedTagMigrationSession } from "../migration/session-store.js";
import { buildDerivedTagMigrationSession } from "../migration/session-builder.js";
import {
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  pauseForAnyKey,
  promptTerminalSelectOption,
  promptTerminalTextInput,
  readTerminalKey,
  readTerminalKeyOrResize,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  runWithDerivedTagTerminalSession,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSelectOption,
  type DerivedTagTerminalSession,
} from "../migration/terminal-ui.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagReviewQueueSummaryItem,
} from "../migration/types.js";

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

function formatModeLabel(mode: DerivedTagMigrationMode): string {
  if (mode === "proposal_review") {
    return "AI proposal review";
  }
  if (mode === "review_queue") {
    return "review queue";
  }
  return mode.replaceAll("_", " ");
}

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

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }
  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function buildScrollableLines<T extends { label: string }>(
  terminalSession: DerivedTagTerminalSession,
  items: T[],
  selectedIndex: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
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

function renderWorkbenchHelp(terminalSession: DerivedTagTerminalSession): void {
  renderTerminalTextScreen(terminalSession, {
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
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const familyOptions = ontology.families
    .filter((family) => familyMatchesScope(category, subcategory, family))
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
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  required: boolean,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const tagOptions = ontology.tags
    .filter((tag) => tagMatchesScope(category, subcategory, family, tag))
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
  terminalSession: DerivedTagTerminalSession,
  db: DatabaseSync,
  mode: DerivedTagMigrationMode,
  required: boolean,
): Promise<SearchCategory | null | undefined> {
  const value = await promptTerminalSelectOption(terminalSession, {
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
  terminalSession: DerivedTagTerminalSession,
  category: SearchCategory,
): Promise<SearchSubcategory | null | undefined> {
  const options = buildSubcategorySelectOptions(category);
  if (options.length <= 1) {
    return null;
  }

  const value = await promptTerminalSelectOption(terminalSession, {
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
  terminalSession: DerivedTagTerminalSession,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  required: boolean,
): Promise<{ category?: SearchCategory; tag?: string } | undefined> {
  const value = await promptTerminalSelectOption(terminalSession, {
    title: "Session Scope",
    subtitle: required
      ? "Choose the tag to review"
      : "Optionally narrow the session to one tag",
    prompt: "Tags",
    entries: buildTagSelectOptions(category, subcategory, family, required),
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
  terminalSession: DerivedTagTerminalSession,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
): Promise<{ category?: SearchCategory; family?: string } | undefined> {
  const value = await promptTerminalSelectOption(terminalSession, {
    title: "Session Scope",
    subtitle: "Optionally narrow the queue to one ontology family",
    prompt: "Families",
    entries: buildFamilySelectOptions(category, subcategory),
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
  terminalSession: DerivedTagTerminalSession,
  prompt: string,
  flagName: string,
): Promise<number | undefined> {
  while (true) {
    const value = normalizeOptional(await promptTerminalTextInput(terminalSession, {
      title: "Session Scope",
      prompt,
    }));

    try {
      return parseInteger(value, flagName);
    } catch (error) {
      await pauseForAnyKey(terminalSession, (error as Error).message);
    }
  }
}

async function promptCustomSessionOptions(
  terminalSession: DerivedTagTerminalSession,
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
  const categorySelection = await promptCategory(terminalSession, db, mode, requireCategory);
  if (categorySelection === undefined) {
    return undefined;
  }
  const category = categorySelection ?? undefined;

  const subcategorySelection = category ? await promptSubcategory(terminalSession, category) : null;
  if (subcategorySelection === undefined) {
    return undefined;
  }
  const subcategory = subcategorySelection ?? undefined;

  const familySelection = mode === "review_queue" || mode === "proposal_review"
    ? await promptFamily(terminalSession, category, subcategory)
    : {};
  if (familySelection === undefined) {
    return undefined;
  }
  const resolvedCategory = category ?? familySelection.category;
  const family = familySelection.family;

  const tagSelection = await promptTag(terminalSession, resolvedCategory, subcategory, family, mode === "legacy_rule");
  if (tagSelection === undefined) {
    return undefined;
  }
  const resolvedTagCategory = resolvedCategory ?? tagSelection.category;
  const tag = tagSelection.tag;

  const limit = await promptInteger(terminalSession, "limit (blank for default)", "--limit");
  const exemplarLimit = mode === "exemplar_cleanup"
    ? await promptInteger(terminalSession, "exemplar-limit (blank for none)", "--exemplar-limit")
    : undefined;

  return {
    category: resolvedTagCategory,
    subcategory,
    family,
    tag,
    limit,
    exemplarLimit,
  };
}

async function createAndRunSession(
  rootPath: string,
  db: DatabaseSync,
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
  terminalSession: DerivedTagTerminalSession,
): Promise<void> {
  try {
    const session = buildDerivedTagMigrationSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagMigrationSession(rootPath, session);
    await writeDerivedTagMigrationSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
    await runDerivedTagMigrationReviewUi(rootPath, session, terminalSession);
  } catch (error) {
    await pauseForAnyKey(
      terminalSession,
      `Could not create the ${formatModeLabel(mode)} session.\n\n${(error as Error).message}`,
    );
  }
}

async function openOntologySearch(
  argv: string[],
  terminalSession: DerivedTagTerminalSession,
): Promise<void> {
  const { db, config } = await openConfiguredIndex(argv);
  try {
    await runDerivedTagOntologyExplorerUi(db, terminalSession, {
      cacheKey: config.indexPath,
    });
  } finally {
    db.close();
  }
}

async function runSearchPlaceholder(
  terminalSession: DerivedTagTerminalSession,
): Promise<void> {
  while (true) {
    renderTerminalTextScreen(terminalSession, {
      title: "Search",
      body: [
        { text: "This area is reserved for the future first-class TUI search surface.", tone: "section" },
        { text: "" },
        { text: "Planned capabilities:" },
        { text: "Exact name lookup, category-aware hard filters, deterministic listing, and ranked/semantic search over the same indexed PF2E data surfaced by the MCP server.", indent: 2 },
        { text: "" },
        { text: "Entity pages in Ontology Search are the groundwork for this future surface.", tone: "dim" },
      ],
      footer: [{ text: "q or Backspace return to top level", tone: "dim" }],
    });

    const key = await readTerminalKeyOrResize(terminalSession);
    if (key.normalizedName === "q" || key.normalizedName === "backspace" || key.normalizedName === "left" || key.normalizedName === "escape" || key.normalizedName === "ctrl_c") {
      return;
    }
  }
}

async function runTagRefinementWorkbench(
  terminalSession: DerivedTagTerminalSession,
  rootPath: string,
  argv: string[],
): Promise<void> {
  let selectedIndex = 0;

  while (true) {
    const queueItems = summarizeCurrentDerivedTagReviewQueue();
    const menuItems = buildTagRefinementMenuItems(queueItems);
    selectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

    renderTerminalTwoPaneScreen(terminalSession, {
      title: "Tag Refinement",
      subtitle: `${queueItems.length} queue slice${queueItems.length === 1 ? "" : "s"} pending review`,
      left: {
        title: "Menu",
        lines: buildScrollableLines(terminalSession, menuItems, selectedIndex),
      },
      right: {
        title: "Pending Review Queue",
        lines: buildQueueLines(queueItems),
      },
      footer: [
        { text: "Up/Down or j/k move  Enter select  ? help  a review all  s/r/e/p create sessions  q/back top level", tone: "dim" },
        { text: `Selected: ${menuItems[selectedIndex]?.label ?? "(none)"}`, tone: "accent" },
      ],
      leftWidth: 48,
    });

    const key = await readTerminalKeyOrResize(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c" || normalized === "q" || normalized === "backspace" || normalized === "left" || normalized === "escape") {
      return;
    }

    if (normalized === "up" || normalized === "k") {
      selectedIndex = moveSelectionWrapped(selectedIndex, -1, menuItems.length);
      continue;
    }
    if (normalized === "down" || normalized === "j") {
      selectedIndex = moveSelectionWrapped(selectedIndex, 1, menuItems.length);
      continue;
    }
    if (normalized === "?") {
      renderWorkbenchHelp(terminalSession);
      await readTerminalKey(terminalSession);
      continue;
    }
    if (normalized === "a" && queueItems.length > 0) {
      const { db } = await openConfiguredIndex(argv);
      try {
        await createAndRunSession(rootPath, db, "review_queue", {}, terminalSession);
      } finally {
        db.close();
      }
      continue;
    }
    if (normalized === "s" || normalized === "r" || normalized === "e" || normalized === "p" || normalized === "n") {
      const mode: DerivedTagMigrationMode = normalized === "s"
        ? "legacy_seed"
        : normalized === "r"
          ? "legacy_rule"
          : normalized === "e"
            ? "exemplar_cleanup"
            : "proposal_review";
      const { db } = await openConfiguredIndex(argv);
      try {
        const options = await promptCustomSessionOptions(terminalSession, db, mode);
        if (options) {
          await createAndRunSession(rootPath, db, mode, options, terminalSession);
        }
      } finally {
        db.close();
      }
      continue;
    }
    if (normalized !== "enter" && normalized !== "kp_enter") {
      continue;
    }

    const selectedItem = menuItems[selectedIndex];
    if (!selectedItem) {
      continue;
    }

    if (selectedItem.kind === "back") {
      return;
    }
    if (selectedItem.kind === "review_all") {
      const { db } = await openConfiguredIndex(argv);
      try {
        await createAndRunSession(rootPath, db, "review_queue", {}, terminalSession);
      } finally {
        db.close();
      }
      continue;
    }
    if (selectedItem.kind === "review_queue_item") {
      const { db } = await openConfiguredIndex(argv);
      try {
        await createAndRunSession(rootPath, db, "review_queue", {
          decisionKind: selectedItem.queueItem.kind,
          category: selectedItem.queueItem.category,
          family: selectedItem.queueItem.family,
          tag: selectedItem.queueItem.tag,
        }, terminalSession);
      } finally {
        db.close();
      }
      continue;
    }
    if (selectedItem.kind === "create_mode") {
      const { db } = await openConfiguredIndex(argv);
      try {
        const options = await promptCustomSessionOptions(terminalSession, db, selectedItem.mode);
        if (options) {
          await createAndRunSession(rootPath, db, selectedItem.mode, options, terminalSession);
        }
      } finally {
        db.close();
      }
    }
  }
}

async function runWorkbench(
  terminalSession: DerivedTagTerminalSession,
  rootPath: string,
  argv: string[],
): Promise<void> {
  let selectedAreaIndex = 0;

  while (true) {
    const queueItems = summarizeCurrentDerivedTagReviewQueue();

    renderTerminalTwoPaneScreen(terminalSession, {
      title: "PF2E Tag TUI",
      subtitle: "Choose a first-class TUI area",
      left: {
        title: "Areas",
        lines: buildScrollableLines(terminalSession, WORKBENCH_AREAS, selectedAreaIndex),
      },
      right: {
        title: "Selected Area",
        lines: [
          { text: WORKBENCH_AREAS[selectedAreaIndex]?.label ?? "", tone: "section" },
          { text: WORKBENCH_AREAS[selectedAreaIndex]?.description ?? "" },
          { text: "" },
          ...buildAreaDetailLines(queueItems),
        ],
      },
      footer: [
        { text: "Up/Down or j/k move  Enter select  ? help  q quit", tone: "dim" },
        { text: `${queueItems.length} pending queue slice${queueItems.length === 1 ? "" : "s"}`, tone: "accent" },
      ],
      leftWidth: 32,
    });

    const key = await readTerminalKeyOrResize(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c" || normalized === "q" || normalized === "escape") {
      return;
    }
    if (normalized === "up" || normalized === "k") {
      selectedAreaIndex = moveSelectionWrapped(selectedAreaIndex, -1, WORKBENCH_AREAS.length);
      continue;
    }
    if (normalized === "down" || normalized === "j") {
      selectedAreaIndex = moveSelectionWrapped(selectedAreaIndex, 1, WORKBENCH_AREAS.length);
      continue;
    }
    if (normalized === "?") {
      renderTerminalTextScreen(terminalSession, {
        title: "Top-Level Help",
        body: buildTopLevelHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      await readTerminalKey(terminalSession);
      continue;
    }
    if (normalized !== "enter" && normalized !== "kp_enter") {
      continue;
    }

    const selectedArea = WORKBENCH_AREAS[selectedAreaIndex];
    if (!selectedArea) {
      continue;
    }

    if (selectedArea.id === "tag_refinement") {
      await runTagRefinementWorkbench(terminalSession, rootPath, argv);
      continue;
    }
    if (selectedArea.id === "ontology_search") {
      await openOntologySearch(argv, terminalSession);
      continue;
    }
    await runSearchPlaceholder(terminalSession);
  }
}

async function main(): Promise<void> {
  const rootPath = process.cwd();
  const argv = process.argv.slice(2);
  await runWithDerivedTagTerminalSession((terminalSession) => runWorkbench(terminalSession, rootPath, argv));
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration workbench failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
