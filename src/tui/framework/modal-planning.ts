import { TERMINAL_DIALOG_CONTINUE_FOOTER } from "../interaction-bindings.js";
import {
  createTerminalChoiceSizingDescriptor,
  createTerminalMessageSizingDescriptor,
  createTerminalTextInputSizingDescriptor,
  planTerminalModalLayout,
  type TerminalModalLayoutResult,
} from "../terminal-modal-layout.js";
import { getRenderedTerminalLineCount } from "./line-rendering.js";
import {
  buildCommandPaletteDetailLines,
  buildPromptDetailLines,
  clampPromptSelectionIndex,
  filterCommandPaletteEntries,
} from "./modal-helpers.js";
import { getPolicyStateForValue } from "./modal-policy-state.js";
import {
  buildPolicySummaryLines,
  buildTextPromptBodyLines,
} from "./modal-prompt-bodies.js";
import type { DerivedTagTerminalLine, TerminalModalState } from "./types.js";

export type { TerminalModalLayoutResult } from "../terminal-modal-layout.js";

function getPreferredPromptWindowSize(itemCount: number): number {
  return Math.max(1, Math.min(itemCount, 8));
}

function getPreferredPromptDetailSize(lineCount: number): number {
  return Math.max(2, Math.min(lineCount, 8));
}

function getChoiceDetailMeasureWidth(terminalWidth: number, preferredLeftPaneWidth = 38): number {
  const separatorWidth = 1;
  const minimumRightPaneWidth = 20;
  const preferredRightWidth = terminalWidth - preferredLeftPaneWidth - separatorWidth;
  if (preferredRightWidth < minimumRightPaneWidth) {
    return terminalWidth;
  }
  return Math.max(minimumRightPaneWidth, preferredRightWidth);
}

function createChoicePromptDescriptor(terminalWidth: number, itemCount: number, detailLines: DerivedTagTerminalLine[]) {
  const detailLineCount = getRenderedTerminalLineCount(detailLines, getChoiceDetailMeasureWidth(terminalWidth));
  return createTerminalChoiceSizingDescriptor({
    staticBodyLineCount: 0,
    preferredLeftPaneWidth: 38,
    list: {
      itemCount,
      chromeRows: 2,
      minVisibleRowCount: Math.min(itemCount, 4),
      preferredVisibleRowCount: getPreferredPromptWindowSize(itemCount),
      maxVisibleRowCount: 12,
    },
    detail: {
      lineCount: detailLineCount,
      chromeRows: 2,
      minVisibleLineCount: Math.min(detailLineCount, 2),
      preferredVisibleLineCount: getPreferredPromptDetailSize(detailLineCount),
      maxVisibleLineCount: 10,
    },
  });
}

export function planTerminalModalStateLayout(
  modal: TerminalModalState,
  terminalWidth: number,
  terminalHeight: number,
): TerminalModalLayoutResult | null {
  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    const footer = modal.options.footer ?? [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }];
    return planTerminalModalLayout({
      kind: "dialog",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "inline",
      headerRows: 3,
      footerRows: footer.length,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(modal.options.body, terminalWidth),
      }),
    });
  }

  if (modal.kind === "text") {
    return planTerminalModalLayout({
      kind: "text",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "inline",
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalTextInputSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(buildTextPromptBodyLines(modal.options, modal.value), terminalWidth),
      }),
    });
  }

  if (modal.kind === "command") {
    const filteredEntries = filterCommandPaletteEntries(modal.options.entries, modal.filterText);
    if (filteredEntries.length === 0) {
      return planTerminalModalLayout({
        kind: "command",
        terminalWidth,
        terminalHeight,
        forcedPresentation: modal.options.presentation ?? "inline",
        headerRows: 3,
        footerRows: 1,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(
            [
              { text: modal.options.prompt, tone: "section" },
              { text: `Filter: ${modal.filterText || "(none)"}`, tone: "accent" },
              { text: "No commands match the current filter.", tone: "warning" },
            ],
            terminalWidth,
          ),
        }),
      });
    }

    const clampedSelectedIndex = clampPromptSelectionIndex(modal.selectedIndex, filteredEntries.length);
    const selectedOption = filteredEntries[clampedSelectedIndex];
    return planTerminalModalLayout({
      kind: "command",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "inline",
      headerRows: 3,
      footerRows: 3,
      descriptor: createChoicePromptDescriptor(
        terminalWidth,
        filteredEntries.length,
        buildCommandPaletteDetailLines(selectedOption, modal.filterText),
      ),
    });
  }

  if (modal.kind === "select") {
    if (modal.options.entries.length === 0) {
      return planTerminalModalLayout({
        kind: "select",
        terminalWidth,
        terminalHeight,
        forcedPresentation: modal.options.presentation ?? "screen",
        headerRows: 3,
        footerRows: 1,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(
            [
              { text: modal.options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ],
            terminalWidth,
          ),
        }),
      });
    }

    const selectedOption = modal.options.entries[modal.selectedIndex];
    return planTerminalModalLayout({
      kind: "select",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "screen",
      headerRows: 3,
      footerRows: 3,
      descriptor: createChoicePromptDescriptor(terminalWidth, modal.options.entries.length, [
        ...buildPromptDetailLines(selectedOption),
        { text: "" },
        { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]),
    });
  }

  if (modal.kind === "multiselect") {
    if (modal.options.entries.length === 0) {
      return planTerminalModalLayout({
        kind: "multiselect",
        terminalWidth,
        terminalHeight,
        forcedPresentation: modal.options.presentation ?? "screen",
        headerRows: 3,
        footerRows: 1,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(
            [
              { text: modal.options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ],
            terminalWidth,
          ),
        }),
      });
    }

    const selectedOption = modal.options.entries[modal.selectedIndex];
    const selectedSet = new Set(modal.selectedValues);
    const selectedLabels = modal.options.entries
      .filter((entry) => selectedSet.has(entry.value))
      .map((entry) => entry.label);
    return planTerminalModalLayout({
      kind: "multiselect",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "screen",
      headerRows: 3,
      footerRows: 3,
      descriptor: createChoicePromptDescriptor(terminalWidth, modal.options.entries.length, [
        ...buildPromptDetailLines(selectedOption),
        { text: "" },
        { text: "Current Selection", tone: "section" },
        { text: selectedLabels.length > 0 ? selectedLabels.join(", ") : "(none)" },
      ]),
    });
  }

  if (modal.options.entries.length === 0) {
    return planTerminalModalLayout({
      kind: "policy",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "screen",
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(
          [
            { text: modal.options.prompt, tone: "section" },
            { text: "No options are available for this scope.", tone: "warning" },
          ],
          terminalWidth,
        ),
      }),
    });
  }

  const selectedOption = modal.options.entries[modal.selectedIndex];
  const selectedState = selectedOption ? getPolicyStateForValue(selectedOption.value, modal.valueStates) : undefined;
  return planTerminalModalLayout({
    kind: "policy",
    terminalWidth,
    terminalHeight,
    forcedPresentation: modal.options.presentation ?? "screen",
    headerRows: 3,
    footerRows: 3,
    descriptor: createChoicePromptDescriptor(terminalWidth, modal.options.entries.length, [
      ...buildPromptDetailLines(selectedOption),
      { text: "" },
      { text: `Focused policy: ${selectedState ?? "off"}`, tone: "accent" },
      ...buildPolicySummaryLines(modal.options, modal.valueStates),
    ]),
  });
}
