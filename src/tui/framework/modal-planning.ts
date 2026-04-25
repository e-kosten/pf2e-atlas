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
  filterPromptEntries,
  getFilteredPromptSelectionIndex,
  getMultiSelectPromptFilteringEnabled,
  getPolicyPromptFilteringEnabled,
} from "./modal-helpers.js";
import { getPolicyStateForValue } from "./modal-policy-state.js";
import {
  buildPolicySummaryLines,
  buildTextPromptBodyLines,
} from "./modal-prompt-bodies.js";
import type { DerivedTagTerminalLine, TerminalModalState, TextPromptOptions } from "./types.js";

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

function getCenteredChoicePromptWidth(
  terminalWidth: number,
  options: {
    title: string;
    subtitle?: string;
    prompt: string;
    entryLabels: readonly string[];
    detailLines: readonly DerivedTagTerminalLine[];
  },
): number {
  const detailWidth = Math.max(
    options.title.length,
    options.subtitle?.length ?? 0,
    options.prompt.length,
    options.entryLabels.join("   ").length,
    ...options.detailLines.map((line) => line.text.length),
  );
  return Math.max(40, Math.min(terminalWidth, Math.max(56, detailWidth + 8)));
}

function createCenteredChoicePromptLayout(options: {
  kind: "select";
  terminalWidth: number;
  terminalHeight: number;
  title: string;
  subtitle?: string;
  prompt: string;
  entryLabels: readonly string[];
  detailLines: readonly DerivedTagTerminalLine[];
}) {
  const panelWidth = getCenteredChoicePromptWidth(options.terminalWidth, options);
  const measuredWidth = Math.max(24, Math.min(panelWidth, options.terminalWidth - 2));
  const bodyLineCount = getRenderedTerminalLineCount(
    [
      ...(options.prompt ? [{ text: options.prompt, tone: "section" as const }] : []),
      { text: options.entryLabels.join("   "), noWrap: true },
      { text: "" },
      ...options.detailLines,
    ],
    Math.max(24, measuredWidth - 4),
  );
  const layout = planTerminalModalLayout({
    kind: options.kind,
    terminalWidth: measuredWidth,
    terminalHeight: options.terminalHeight,
    forcedPresentation: "inline",
    headerRows: 3,
    footerRows: 2,
    descriptor: createTerminalMessageSizingDescriptor({
      bodyLineCount,
      minimumInlineWidth: measuredWidth,
    }),
  });

  return {
    ...layout,
    presentation: "centered" as const,
    panelWidth: measuredWidth,
  };
}

function getCenteredTextPromptWidth(
  terminalWidth: number,
  options: TextPromptOptions,
  currentValue: string,
): number {
  const contentWidth = Math.max(
    options.title.length,
    options.prompt.length,
    options.hint?.length ?? 0,
    (`> ${currentValue || ""}`).length,
    (options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.").length,
  );
  return Math.max(44, Math.min(terminalWidth, Math.max(60, contentWidth + 8)));
}

function createCenteredTextPromptLayout(options: {
  terminalWidth: number;
  terminalHeight: number;
  prompt: TextPromptOptions;
  currentValue: string;
}) {
  const panelWidth = getCenteredTextPromptWidth(options.terminalWidth, options.prompt, options.currentValue);
  const measuredWidth = Math.max(24, Math.min(panelWidth, options.terminalWidth - 2));
  const bodyLineCount = getRenderedTerminalLineCount(
    buildTextPromptBodyLines(options.prompt, options.currentValue),
    Math.max(24, measuredWidth - 4),
  );
  const layout = planTerminalModalLayout({
    kind: "text",
    terminalWidth: measuredWidth,
    terminalHeight: options.terminalHeight,
    forcedPresentation: "inline",
    headerRows: 3,
    footerRows: 1,
    descriptor: createTerminalTextInputSizingDescriptor({
      bodyLineCount,
      minimumInlineWidth: measuredWidth,
    }),
  });

  return {
    ...layout,
    presentation: "centered" as const,
    panelWidth: measuredWidth,
  };
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
    if (modal.options.presentation === "centered") {
      return createCenteredTextPromptLayout({
        terminalWidth,
        terminalHeight,
        prompt: modal.options,
        currentValue: modal.value,
      });
    }
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
    const filteredEntries =
      modal.options.filtering && modal.options.choiceLayout !== "horizontal"
        ? filterPromptEntries(modal.options.entries, modal.filterText)
        : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

    if (filteredEntries.length === 0) {
      return planTerminalModalLayout({
        kind: "select",
        terminalWidth,
        terminalHeight,
        forcedPresentation: modal.options.presentation === "centered" ? "screen" : (modal.options.presentation ?? "screen"),
        headerRows: 3,
        footerRows: 2,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(
            [
              { text: modal.options.prompt, tone: "section" },
              ...(modal.filterText ? [{ text: `Search /${modal.filterText}`, tone: "accent" as const }] : []),
              {
                text: modal.filterText
                  ? "No options match the current filter."
                  : "No options are available for this scope.",
                tone: "warning",
              },
            ],
            terminalWidth,
          ),
        }),
      });
    }

    const selectedIndex = getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, modal.filterText);
    const selectedOption = modal.options.entries[selectedIndex];
    if (modal.options.choiceLayout === "horizontal" || modal.options.presentation === "centered") {
      return createCenteredChoicePromptLayout({
        kind: "select",
        terminalWidth,
        terminalHeight,
        title: modal.options.title,
        subtitle: modal.options.subtitle,
        prompt: modal.options.prompt,
        entryLabels: filteredEntries.map((entry) => entry.entry.label),
        detailLines: buildPromptDetailLines(selectedOption),
      });
    }

    return planTerminalModalLayout({
      kind: "select",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "screen",
      headerRows: 3,
      footerRows: 3,
      descriptor: createChoicePromptDescriptor(terminalWidth, filteredEntries.length, [
        ...buildPromptDetailLines(selectedOption),
        { text: "" },
        { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]),
    });
  }

  if (modal.kind === "multiselect") {
    const filteringEnabled = getMultiSelectPromptFilteringEnabled(modal.options);
    const filteredEntries = filteringEnabled
      ? filterPromptEntries(modal.options.entries, modal.filterText)
      : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

    if (filteredEntries.length === 0) {
      return planTerminalModalLayout({
        kind: "multiselect",
        terminalWidth,
        terminalHeight,
        forcedPresentation: modal.options.presentation ?? "screen",
        headerRows: 3,
        footerRows: 2,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(
            [
              { text: modal.options.prompt, tone: "section" },
              ...(modal.filterText ? [{ text: `Search /${modal.filterText}`, tone: "accent" as const }] : []),
              {
                text: modal.filterText
                  ? "No options match the current filter."
                  : "No options are available for this scope.",
                tone: "warning",
              },
            ],
            terminalWidth,
          ),
        }),
      });
    }

    const selectedIndex = getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, modal.filterText);
    const selectedOption = modal.options.entries[selectedIndex];
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
      descriptor: createChoicePromptDescriptor(terminalWidth, filteredEntries.length, [
        ...buildPromptDetailLines(selectedOption),
        { text: "" },
        { text: "Current Selection", tone: "section" },
        { text: selectedLabels.length > 0 ? selectedLabels.join(", ") : "(none)" },
      ]),
    });
  }

  const filteringEnabled = getPolicyPromptFilteringEnabled(modal.options);
  const filteredEntries = filteringEnabled
    ? filterPromptEntries(modal.options.entries, modal.filterText)
    : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

  if (filteredEntries.length === 0) {
    return planTerminalModalLayout({
      kind: "policy",
      terminalWidth,
      terminalHeight,
      forcedPresentation: modal.options.presentation ?? "screen",
      headerRows: 3,
      footerRows: 2,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(
          [
            { text: modal.options.prompt, tone: "section" },
            ...(modal.filterText ? [{ text: `Search /${modal.filterText}`, tone: "accent" as const }] : []),
            {
              text: modal.filterText ? "No options match the current filter." : "No options are available for this scope.",
              tone: "warning",
            },
          ],
          terminalWidth,
        ),
      }),
    });
  }

  const selectedIndex = getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, modal.filterText);
  const selectedOption = modal.options.entries[selectedIndex];
  const selectedState = selectedOption ? getPolicyStateForValue(selectedOption.value, modal.valueStates) : undefined;
  return planTerminalModalLayout({
    kind: "policy",
    terminalWidth,
    terminalHeight,
    forcedPresentation: modal.options.presentation ?? "screen",
    headerRows: 3,
    footerRows: 4,
    descriptor: createChoicePromptDescriptor(terminalWidth, filteredEntries.length, [
      ...buildPromptDetailLines(selectedOption),
      { text: "" },
      { text: `Focused policy: ${selectedState ?? "off"}`, tone: "accent" },
      ...buildPolicySummaryLines(modal.options, modal.valueStates),
    ]),
  });
}
