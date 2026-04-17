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

type WorkbenchMenuItem =
  | { kind: "review_queue_item"; label: string; queueItem: DerivedTagReviewQueueSummaryItem }
  | { kind: "review_all"; label: string }
  | { kind: "create_mode"; label: string; mode: DerivedTagMigrationMode }
  | { kind: "explore_ontology"; label: string }
  | { kind: "quit"; label: string };

function buildWorkbenchMenuItems(items: DerivedTagReviewQueueSummaryItem[]): WorkbenchMenuItem[] {
  const menuItems: WorkbenchMenuItem[] = [];
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
    { kind: "explore_ontology", label: "Explore ontology" },
    { kind: "quit", label: "Quit" },
  );
  return menuItems;
}

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }
  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function buildMenuLines(
  terminalSession: DerivedTagTerminalSession,
  menuItems: WorkbenchMenuItem[],
  selectedIndex: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const windowStart = clampWindowStart(selectedIndex, menuItems.length, visibleCount);

  return menuItems.slice(windowStart, windowStart + visibleCount).map((item, offset) => ({
    text: item.label,
    tone: windowStart + offset === selectedIndex ? "selected" : "default",
    noWrap: true,
  }));
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
    title: "Derived-Tag Workbench Help",
    body: [
      { text: "Navigation", tone: "section" },
      { text: "Up / Down or j / k: move between workbench menu rows" },
      { text: "Enter: open the selected row" },
      { text: "" },
      { text: "Shortcuts", tone: "section" },
      { text: "a review all queue items" },
      { text: "s create a legacy-seed session" },
      { text: "r create a legacy-rule session" },
      { text: "e create an exemplar-cleanup session" },
      { text: "n create a new-tagging session" },
      { text: "o open the ontology explorer" },
      { text: "q quit" },
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

async function openOntologyExplorer(
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

async function runWorkbench(
  terminalSession: DerivedTagTerminalSession,
  rootPath: string,
  argv: string[],
): Promise<void> {
  let selectedIndex = 0;

  while (true) {
    const queueItems = summarizeCurrentDerivedTagReviewQueue();
    const menuItems = buildWorkbenchMenuItems(queueItems);
    selectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

    renderTerminalTwoPaneScreen(terminalSession, {
      title: "Derived-Tag Migration Workbench",
      subtitle: `${queueItems.length} queue slice${queueItems.length === 1 ? "" : "s"} pending review`,
      left: {
        title: "Menu",
        lines: buildMenuLines(terminalSession, menuItems, selectedIndex),
      },
      right: {
        title: "Pending Review Queue",
        lines: buildQueueLines(queueItems),
      },
      footer: [
        { text: "Up/Down or j/k move  Enter select  ? help  a review all  s/r/e/n create sessions  o explore ontology  q quit", tone: "dim" },
        { text: `Selected: ${menuItems[selectedIndex]?.label ?? "(none)"}`, tone: "accent" },
      ],
      leftWidth: 48,
    });

    const key = await readTerminalKey(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c" || normalized === "q") {
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
    if (normalized === "o") {
      await openOntologyExplorer(argv, terminalSession);
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

    if (selectedItem.kind === "quit") {
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
      continue;
    }
    if (selectedItem.kind === "explore_ontology") {
      await openOntologyExplorer(argv, terminalSession);
    }
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
