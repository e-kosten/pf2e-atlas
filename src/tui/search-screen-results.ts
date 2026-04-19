import type { DerivedTagTerminalCommandOption, DerivedTagTerminalLine } from "./terminal-ui.js";
import type { SearchScreenState } from "./search-screen-state.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import { formatResultPosition, formatSort, getSessionBufferRange } from "./search-screen-state.js";
import type { Pf2eTerminalSearchSession } from "./search-service.js";
import { clampWindowStart } from "./list-utils.js";
import { buildOntologyExplorerEntityDetailLines } from "./ontology-explorer/entity-page.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./ontology-explorer/entity-record.js";

export type SearchResultCommandId = "jumpToResult" | "sortResults" | "openSetup";

function buildSearchResultLabel(record: Pf2eTerminalSearchSession["results"][number]): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level} | ${record.packLabel}`;
}

export function parseJumpToResultInput(input: string, total: number): number | string {
  const normalized = input.replace(/[,_\s]+/g, "");
  if (!/^\d+$/.test(normalized)) {
    return "Enter a result number such as `6000`.";
  }

  const parsed = Number.parseInt(normalized, 10);
  if (!Number.isFinite(parsed) || parsed < 1) {
    return "Result numbers start at 1.";
  }
  if (parsed > total) {
    return `Result ${parsed} is out of range. Valid positions are 1-${total}.`;
  }

  return parsed - 1;
}

export function buildResultLines(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
  bodyHeight: number,
  loadingMore: boolean,
): DerivedTagTerminalLine[] {
  if (!session || session.results.length === 0) {
    return [
      { text: "No applied results yet.", tone: "section" },
      { text: "Execute the current query setup to switch into the result reader.", tone: "dim" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const statusRows = loadingMore || session.loadedCount < session.total ? 1 : 0;
  const resultWindowCount = Math.max(1, visibleCount - statusRows);
  const localSelectedIndex = selectedIndex - session.windowOffset;
  const safeIndex = Math.max(0, Math.min(localSelectedIndex, session.results.length - 1));
  const windowStart = clampWindowStart(safeIndex, session.results.length, resultWindowCount);

  const lines: DerivedTagTerminalLine[] = session.results
    .slice(windowStart, windowStart + resultWindowCount)
    .map((record, offset) => ({
      text: buildSearchResultLabel(record),
      tone:
        localSelectedIndex >= 0 &&
        localSelectedIndex < session.results.length &&
        windowStart + offset === localSelectedIndex
          ? "selected"
          : "default",
      noWrap: true,
    }));

  if (loadingMore) {
    lines.push({ text: `Loading around ${formatResultPosition(selectedIndex, session.total)}...`, tone: "accent" });
  }

  return lines;
}

export function buildPendingResultDetailLines(
  session: Pf2eTerminalSearchSession,
  resultIndex: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Result Preview", tone: "section" },
    { text: `Showing result ${formatResultPosition(resultIndex, session.total)}` },
    { text: `Sort: ${formatSort(session.sort)}` },
    { text: "" },
    { text: "Loading the result window around the current selection.", tone: "accent" },
    { text: `Current buffer: ${getSessionBufferRange(session)}`, tone: "dim" },
  ];
}

export function buildResultDetailLines(
  record: Pf2eTerminalSearchSession["results"][number],
  session: Pf2eTerminalSearchSession,
  resultIndex: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Result Preview", tone: "section" },
    { text: `Showing result ${formatResultPosition(resultIndex, session.total)}` },
    { text: `Sort: ${formatSort(session.sort)}` },
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

export function buildResultCommandPaletteEntries(
  state: SearchScreenState,
  origin: SearchScreenOrigin,
): DerivedTagTerminalCommandOption<SearchResultCommandId>[] {
  return [
    {
      value: "jumpToResult",
      label: "Jump to Result",
      description: "Jump to an absolute result position in the active result set.",
      keywords: ["position", "goto"],
    },
    {
      value: "sortResults",
      label: "Change Sort",
      description: state.session
        ? `Switch result ordering from ${formatSort(state.session.sort)}.`
        : "Change the active result ordering.",
      keywords: ["order", "ranking"],
    },
    ...(origin === "ontology"
      ? [
          {
            value: "openSetup" as const,
            label: "Open Query Setup",
            description: "Switch from the result reader into the search setup workspace without leaving the search flow.",
            keywords: ["setup", "filters", "workspace"],
          },
        ]
      : []),
  ];
}
