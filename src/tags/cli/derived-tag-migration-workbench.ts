#!/usr/bin/env node

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import {
  openConfiguredIndex,
  parseInteger,
  writeDerivedTagMigrationSummary,
} from "../migration/cli-utils.js";
import { runDerivedTagOntologyExplorerUi } from "../migration/ontology-explorer-ui.js";
import { renderDerivedTagMigrationSessionSummary } from "../migration/render.js";
import { summarizeCurrentDerivedTagReviewQueue } from "../migration/runtime-state.js";
import { runDerivedTagMigrationReviewUi } from "../migration/review-ui.js";
import { writeDerivedTagMigrationSession } from "../migration/session-store.js";
import { buildDerivedTagMigrationSession } from "../migration/session-builder.js";
import {
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  pauseForAnyKey,
  promptTerminalTextInput,
  readTerminalKey,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  runWithDerivedTagTerminalSession,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSession,
} from "../migration/terminal-ui.js";
import type {
  DerivedTagManagedCategory,
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagReviewQueueSummaryItem,
} from "../migration/types.js";

const MANAGED_CATEGORIES: DerivedTagManagedCategory[] = ["affliction", "creature", "equipment", "hazard", "spell"];

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

const WORKBENCH_AREAS: WorkbenchArea[] = [
  {
    id: "tag_refinement",
    label: "Tag Refinement",
    description: "Review authored queue items and create legacy-seed, legacy-rule, exemplar-cleanup, and new-tagging sessions.",
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
    { kind: "create_mode", label: "Create new-tagging review session", mode: "new_tagging" },
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
      { text: "n create a new-tagging session" },
    ],
    footer: [{ text: "Press any key to return.", tone: "dim" }],
  });
}

function normalizeOptional(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

async function promptCategory(
  terminalSession: DerivedTagTerminalSession,
  required: boolean,
): Promise<SearchCategory | undefined> {
  while (true) {
    const answer = normalizeOptional(await promptTerminalTextInput(terminalSession, {
      title: "Session Scope",
      prompt: required
        ? `category (${MANAGED_CATEGORIES.join("/")})`
        : `category (${MANAGED_CATEGORIES.join("/")}, blank for all)`,
    }));

    if (!answer) {
      if (!required) {
        return undefined;
      }
    } else if (MANAGED_CATEGORIES.includes(answer as DerivedTagManagedCategory)) {
      return answer as SearchCategory;
    }

    await pauseForAnyKey(terminalSession, `Enter one of: ${MANAGED_CATEGORIES.join(", ")}.`);
  }
}

async function promptInteger(
  terminalSession: DerivedTagTerminalSession,
  prompt: string,
  flagName: string,
): Promise<number | undefined> {
  const value = normalizeOptional(await promptTerminalTextInput(terminalSession, {
    title: "Session Scope",
    prompt,
  }));
  return parseInteger(value, flagName);
}

async function promptCustomSessionOptions(
  terminalSession: DerivedTagTerminalSession,
  mode: DerivedTagMigrationMode,
): Promise<{
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
}> {
  const requireCategory = mode === "legacy_rule" || mode === "new_tagging";
  const category = await promptCategory(terminalSession, requireCategory);
  const subcategory = normalizeOptional(await promptTerminalTextInput(terminalSession, {
    title: "Session Scope",
    prompt: "subcategory (blank to skip)",
  })) as SearchSubcategory | undefined;
  const family = mode === "review_queue"
    ? normalizeOptional(await promptTerminalTextInput(terminalSession, {
      title: "Session Scope",
      prompt: "family (blank for any)",
    }))
    : undefined;
  const tagRequired = mode === "legacy_rule";
  let tag: string | undefined;
  while (true) {
    tag = normalizeOptional(await promptTerminalTextInput(terminalSession, {
      title: "Session Scope",
      prompt: `tag${tagRequired ? "" : " (blank for any)"}`,
    }));
    if (tag || !tagRequired) {
      break;
    }
    await pauseForAnyKey(terminalSession, "tag is required for legacy_rule sessions.");
  }

  const limit = await promptInteger(terminalSession, "limit (blank for default)", "--limit");
  const exemplarLimit = mode === "exemplar_cleanup"
    ? await promptInteger(terminalSession, "exemplar-limit (blank for none)", "--exemplar-limit")
    : undefined;

  return {
    category,
    subcategory,
    family,
    tag,
    limit,
    exemplarLimit,
  };
}

async function createAndRunSession(
  rootPath: string,
  argv: string[],
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
  const { db } = await openConfiguredIndex(argv);
  try {
    const session = buildDerivedTagMigrationSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagMigrationSession(rootPath, session);
    await writeDerivedTagMigrationSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
    await runDerivedTagMigrationReviewUi(rootPath, session, terminalSession);
  } finally {
    db.close();
  }
}

async function openOntologySearch(
  argv: string[],
  terminalSession: DerivedTagTerminalSession,
): Promise<void> {
  const { db } = await openConfiguredIndex(argv);
  try {
    await runDerivedTagOntologyExplorerUi(db, terminalSession);
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

    const key = await readTerminalKey(terminalSession);
    if (key.normalizedName === "q" || key.normalizedName === "backspace" || key.normalizedName === "left" || key.normalizedName === "ctrl_c") {
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
        { text: "Up/Down or j/k move  Enter select  ? help  a review all  s/r/e/n create sessions  q/back top level", tone: "dim" },
        { text: `Selected: ${menuItems[selectedIndex]?.label ?? "(none)"}`, tone: "accent" },
      ],
      leftWidth: 48,
    });

    const key = await readTerminalKey(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c" || normalized === "q" || normalized === "backspace" || normalized === "left") {
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
      await createAndRunSession(rootPath, argv, "review_queue", {}, terminalSession);
      continue;
    }
    if (normalized === "s" || normalized === "r" || normalized === "e" || normalized === "n") {
      const mode: DerivedTagMigrationMode = normalized === "s"
        ? "legacy_seed"
        : normalized === "r"
          ? "legacy_rule"
          : normalized === "e"
            ? "exemplar_cleanup"
            : "new_tagging";
      const options = await promptCustomSessionOptions(terminalSession, mode);
      await createAndRunSession(rootPath, argv, mode, options, terminalSession);
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
      await createAndRunSession(rootPath, argv, "review_queue", {}, terminalSession);
      continue;
    }
    if (selectedItem.kind === "review_queue_item") {
      await createAndRunSession(rootPath, argv, "review_queue", {
        decisionKind: selectedItem.queueItem.kind,
        category: selectedItem.queueItem.category,
        family: selectedItem.queueItem.family,
        tag: selectedItem.queueItem.tag,
      }, terminalSession);
      continue;
    }
    if (selectedItem.kind === "create_mode") {
      const options = await promptCustomSessionOptions(terminalSession, selectedItem.mode);
      await createAndRunSession(rootPath, argv, selectedItem.mode, options, terminalSession);
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

    const key = await readTerminalKey(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c" || normalized === "q") {
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
