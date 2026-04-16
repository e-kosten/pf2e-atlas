#!/usr/bin/env node

import { createInterface, type Interface } from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import {
  openConfiguredIndex,
  parseInteger,
  writeDerivedTagMigrationSummary,
} from "../migration/cli-utils.js";
import { renderDerivedTagMigrationSessionSummary } from "../migration/render.js";
import { summarizeCurrentDerivedTagReviewQueue } from "../migration/runtime-state.js";
import { runDerivedTagMigrationReviewUi } from "../migration/review-ui.js";
import { writeDerivedTagMigrationSession } from "../migration/session-store.js";
import { buildDerivedTagMigrationSession } from "../migration/session-builder.js";
import {
  clearTerminalScreen,
  moveSelection,
  moveSelectionWrapped,
  pauseForAnyKey,
  readTerminalKey,
  terminalTheme,
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
    { kind: "quit", label: "Quit" },
  );
  return menuItems;
}

function renderWorkbenchHome(items: DerivedTagReviewQueueSummaryItem[], menuItems: WorkbenchMenuItem[], selectedIndex: number): string {
  const renderedMenu = menuItems.map((item, index) => {
    if (index === selectedIndex) {
      return `${terminalTheme.selectedMarker(">")} ${terminalTheme.selectedLine(item.label)}`;
    }
    return `  ${item.label}`;
  });

  return [
    terminalTheme.heading("Derived-Tag Migration Workbench"),
    "",
    terminalTheme.section("Pending review queue:"),
    ...(items.length > 0
      ? items.map((item) => {
        const confidence = item.confidence === "low"
          ? terminalTheme.danger(item.confidence)
          : item.confidence === "medium"
            ? terminalTheme.warning(item.confidence)
            : item.confidence === "mixed"
            ? terminalTheme.accent(item.confidence)
              : terminalTheme.success(item.confidence);
        const scope = item.kind === "assignment"
          ? `${item.category} ${item.family}.${item.tag}`
          : `${item.category} exemplar.${item.tag}`;
        return `- ${scope} confidence=${confidence} count=${item.count}`;
      })
      : [terminalTheme.dim("No pending authored review items.")]),
    "",
    terminalTheme.section("Menu:"),
    ...renderedMenu,
    "",
    terminalTheme.dim("Controls: Up/Down or j/k move  Enter select  ? help  q quit"),
  ].join("\n");
}

function renderWorkbenchHelp(): string {
  return [
    terminalTheme.heading("Workbench Help"),
    "",
    "Navigation:",
    "- Up / Down or j / k: move between workbench menu rows",
    "- Movement wraps at the top and bottom of the menu",
    "- Enter: open the selected row",
    "",
    "Queue review:",
    "- a: review all currently pending unresolved authored review items",
    "- Queue rows: open a focused review_queue session for one assignment or exemplar slice",
    "- The queue is selective review only: confident live-authored changes should not appear here",
    "",
    "Create session shortcuts:",
    "- s: create a legacy-seed review session",
    "  Review records currently carried forward from old seed-based tagging. Use this to decide two things per touched record: which tags should become explicit long-term assignments, and whether the record should remain an exemplar at all.",
    "- r: create a legacy-rule review session",
    "  Review records currently tagged by an old heuristic rule. Use this when deciding whether a legacy rule should be replaced by a future authored rule, converted into explicit assignments, or dropped as too noisy.",
    "- e: create an exemplar-cleanup review session",
    "  Review oversized exemplar sets and prune them down to the strongest teaching examples. This is about ontology explanation quality, not runtime tagging coverage.",
    "- n: create a new-tagging review session",
    "  Review currently untagged records and assign their future-state explicit tags. Use this for forward coverage work once legacy cleanup is not the source of truth anymore.",
    "",
    "Other:",
    "- q: quit",
    "",
    "Session prompts:",
    "- Create entries will ask for any scope they need, such as category, tag, or limit",
    "- category narrows the record type being reviewed",
    "- tag/family narrows the work to one ontology slice when applicable",
    "- limit keeps manual passes small and reviewable",
    "",
    "Exemplar review:",
    "- Exemplars are not runtime tagging state",
    "- Only uncertain exemplar decisions should go into review; confident exemplar changes should be written directly",
  ].join("\n");
}

function normalizeOptional(value: string): string | undefined {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

async function promptCategory(
  rl: Interface,
  required: boolean,
): Promise<SearchCategory | undefined> {
  const label = required
    ? `category (${MANAGED_CATEGORIES.join("/")})`
    : `category (${MANAGED_CATEGORIES.join("/")}, blank for all)`;

  while (true) {
    const answer = normalizeOptional(await rl.question(`${label}: `) ?? "");
    if (!answer) {
      if (!required) {
        return undefined;
      }
    } else if (MANAGED_CATEGORIES.includes(answer as DerivedTagManagedCategory)) {
      return answer as SearchCategory;
    }
    console.log(`Enter one of: ${MANAGED_CATEGORIES.join(", ")}.`);
  }
}

async function promptCustomSessionOptions(
  rl: Interface,
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
  const category = await promptCategory(rl, requireCategory);
  const subcategory = normalizeOptional(await rl.question("subcategory (blank to skip): ")) as SearchSubcategory | undefined;
  const family = mode === "review_queue"
    ? normalizeOptional(await rl.question("family (blank for any): "))
    : undefined;
  const tagRequired = mode === "legacy_rule";
  let tag: string | undefined;
  while (true) {
    tag = normalizeOptional(await rl.question(`tag${tagRequired ? "" : " (blank for any)"}: `));
    if (tag || !tagRequired) {
      break;
    }
    console.log("tag is required for legacy_rule sessions.");
  }
  const limitInput = normalizeOptional(await rl.question("limit (blank for default): "));
  const exemplarLimitInput = mode === "exemplar_cleanup"
    ? normalizeOptional(await rl.question("exemplar-limit (blank for none): "))
    : undefined;

  return {
    category,
    subcategory,
    family,
    tag,
    limit: parseInteger(limitInput, "--limit"),
    exemplarLimit: parseInteger(exemplarLimitInput, "--exemplar-limit"),
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
): Promise<void> {
  const { db } = await openConfiguredIndex(argv);
  try {
    const session = buildDerivedTagMigrationSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagMigrationSession(rootPath, session);
    await writeDerivedTagMigrationSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
    await runDerivedTagMigrationReviewUi(rootPath, session);
  } finally {
    db.close();
  }
}

async function main(): Promise<void> {
  const rootPath = process.cwd();
  const argv = process.argv.slice(2);
  let selectedIndex = 0;

  while (true) {
    const queueItems = summarizeCurrentDerivedTagReviewQueue();
    const menuItems = buildWorkbenchMenuItems(queueItems);
    selectedIndex = moveSelection(selectedIndex, 0, menuItems.length);
    clearTerminalScreen();
    console.log(renderWorkbenchHome(queueItems, menuItems, selectedIndex));

    const key = await readTerminalKey();
    const normalized = key.sequence.toLowerCase();

    if ((key.ctrl && key.name === "c") || normalized === "q") {
      return;
    }

    if (key.name === "up" || normalized === "k") {
      selectedIndex = moveSelectionWrapped(selectedIndex, -1, menuItems.length);
      continue;
    }
    if (key.name === "down" || normalized === "j") {
      selectedIndex = moveSelectionWrapped(selectedIndex, 1, menuItems.length);
      continue;
    }
    if (key.name !== "return" && key.name !== "enter") {
      if (normalized === "?") {
        await pauseForAnyKey(renderWorkbenchHelp());
        continue;
      }
      if (normalized === "a" && queueItems.length > 0) {
        await createAndRunSession(rootPath, argv, "review_queue", {});
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
        const optionsRl = createInterface({ input, output });
        let options: Awaited<ReturnType<typeof promptCustomSessionOptions>> | undefined;
        try {
          clearTerminalScreen();
          console.log(`Create ${mode} session\n`);
          options = await promptCustomSessionOptions(optionsRl, mode);
        } finally {
          optionsRl.close();
        }
        await createAndRunSession(rootPath, argv, mode, options ?? {});
      }
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
      await createAndRunSession(rootPath, argv, "review_queue", {});
      continue;
    }
    if (selectedItem.kind === "review_queue_item") {
      await createAndRunSession(rootPath, argv, "review_queue", {
        decisionKind: selectedItem.queueItem.kind,
        category: selectedItem.queueItem.category,
        family: selectedItem.queueItem.family,
        tag: selectedItem.queueItem.tag,
      });
      continue;
    }
    if (selectedItem.kind === "create_mode") {
      const optionsRl = createInterface({ input, output });
      let options: Awaited<ReturnType<typeof promptCustomSessionOptions>> | undefined;
      try {
        clearTerminalScreen();
        console.log(`Create ${selectedItem.mode} session\n`);
        options = await promptCustomSessionOptions(optionsRl, selectedItem.mode);
      } finally {
        optionsRl.close();
      }
      await createAndRunSession(rootPath, argv, selectedItem.mode, options ?? {});
      continue;
    }
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration workbench failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
