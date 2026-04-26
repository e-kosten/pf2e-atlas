import type { DerivedTagTerminalLine } from "../framework/types.js";
import type { DerivedTagTerminalActionTargetOption } from "../action-target.js";
import type { SearchScreenState } from "./state.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import { formatResultPosition, formatSort, getSessionBufferRange } from "./state.js";
import type { Pf2eTerminalSearchSession } from "../search/service.js";
import { clampWindowStart } from "../list-utils.js";
import {
  buildLookupMatchTypeGroup,
  buildSearchResultRowLine,
  buildTerminalListDetailMetadataLines,
  formatLookupMatchTypeBadge,
  formatLookupMatchTypeLabel,
} from "../list-detail-formatting.js";
import { buildTerminalGroupedListLines } from "../list-detail-presentation.js";
import { buildOntologyExplorerEntityDetailLines } from "../../app/ontology/presenter.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "../../app/ontology/entity-record.js";

export type SearchResultCommandId = "jumpToResult" | "sortResults" | "openEditor";

export function canChangeResultSort(session: Pf2eTerminalSearchSession | null): boolean {
  return Boolean(session && session.query.mode !== "search");
}

function getLookupPresentation(
  session: Pf2eTerminalSearchSession | null,
): { policy: "tiered" | "global" } | null {
  if (!session || session.query.mode !== "lookup") {
    return null;
  }

  return {
    policy: session.sort.endsWith("Global") ? "global" : "tiered",
  };
}

function getResultLookupMatchType(
  record: Pf2eTerminalSearchSession["results"][number],
  presentation: { policy: "tiered" | "global" } | null,
) {
  if (!presentation) {
    return "none" as const;
  }

  return record.matchType ?? "none";
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
      { text: "Execute the current query to switch into the result reader.", tone: "dim" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const statusRows = loadingMore || session.loadedCount < session.total ? 1 : 0;
  const resultWindowCount = Math.max(1, visibleCount - statusRows);
  const localSelectedIndex = selectedIndex - session.windowOffset;
  const safeIndex = Math.max(0, Math.min(localSelectedIndex, session.results.length - 1));
  const windowStart = clampWindowStart(safeIndex, session.results.length, resultWindowCount);
  const visibleRecords = session.results.slice(windowStart, windowStart + resultWindowCount);
  const lookupPresentation = getLookupPresentation(session);
  const lines: DerivedTagTerminalLine[] = buildTerminalGroupedListLines({
    items: visibleRecords,
    selectedIndex:
      localSelectedIndex >= 0 && localSelectedIndex < session.results.length ? localSelectedIndex - windowStart : 0,
    getGroup:
      lookupPresentation?.policy === "tiered"
        ? (record) => {
            const matchType = getResultLookupMatchType(record, lookupPresentation);
            return matchType === "none" ? null : buildLookupMatchTypeGroup(matchType);
          }
        : undefined,
    buildItemLine: (record, options) => {
      const matchType = getResultLookupMatchType(record, lookupPresentation);
      return buildSearchResultRowLine(record, {
        selected: options.selected,
        badges:
          lookupPresentation?.policy === "global" && matchType !== "none"
            ? [formatLookupMatchTypeBadge(matchType)]
            : undefined,
      });
    },
  });

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
    ...buildTerminalListDetailMetadataLines([
      { label: "Showing", value: `result ${formatResultPosition(resultIndex, session.total)}` },
      { label: "Sort", value: formatSort(session.sort) },
    ]),
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
  const lookupPresentation = getLookupPresentation(session);
  const lookupMatchType = getResultLookupMatchType(record, lookupPresentation);
  const matchType =
    lookupMatchType === "none"
      ? null
      : lookupMatchType === "exact" || lookupMatchType === "normalized_exact" || lookupMatchType === "fuzzy"
        ? formatLookupMatchTypeLabel(lookupMatchType)
        : null;
  return [
    { text: "Result Preview", tone: "section" },
    ...buildTerminalListDetailMetadataLines([
      { label: "Showing", value: `result ${formatResultPosition(resultIndex, session.total)}` },
      { label: "Sort", value: formatSort(session.sort) },
      ...(matchType ? [{ label: "Match", value: matchType }] : []),
      { label: "Source", value: record.packLabel },
    ]),
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

export function buildResultActionEntries(
  state: SearchScreenState,
  origin: SearchScreenOrigin,
): DerivedTagTerminalActionTargetOption<SearchResultCommandId>[] {
  return [
    {
      id: "jumpToResult",
      label: "Jump to Result",
      description: "Jump to an absolute result position in the active result set.",
    },
    ...(canChangeResultSort(state.session)
      ? [
          {
            id: "sortResults" as const,
            label: "Change Sort",
            description: state.session
              ? `Switch result ordering from ${formatSort(state.session.sort)}.`
                : "Choose the active result ordering for this result reader.",
          },
        ]
      : []),
    ...(origin === "ontology"
      ? [
          {
            id: "openEditor" as const,
            label: "Open Query Editor",
            description: "Switch from the result reader into the query editor without leaving the search flow.",
          },
        ]
      : []),
  ];
}
