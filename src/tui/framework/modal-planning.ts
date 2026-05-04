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
  buildPromptDetailLines,
  filterPromptEntries,
  getFilteredPromptSelectionIndex,
  getMultiSelectPromptFilteringEnabled,
} from "./modal-helpers.js";
import {
  buildTextPromptBodyLines,
} from "./modal-prompt-bodies.js";
import {
  isCenteredPromptPresentation,
  normalizeTerminalPromptPresentation,
  type NormalizedTerminalPromptPresentation,
  type TerminalCenteredPromptBackgroundTreatment,
} from "./prompt-presentation.js";
import type { DerivedTagTerminalLine, TerminalModalState, TextPromptOptions } from "./types.js";

export type { TerminalModalLayoutResult } from "../terminal-modal-layout.js";
export type FrameworkTerminalModalLayoutResult = TerminalModalLayoutResult & {
  centeredPromptBackground?: TerminalCenteredPromptBackgroundTreatment;
};

function coerceBaseModalPresentation(
  presentation: NormalizedTerminalPromptPresentation | undefined,
  fallback: "inline" | "screen",
): "inline" | "screen" {
  if (presentation === "inline" || presentation === "screen") {
    return presentation;
  }
  return fallback;
}

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

function createChoicePromptDescriptor(
  terminalWidth: number,
  itemCount: number,
  detailLines: readonly DerivedTagTerminalLine[],
) {
  const detailLineCount = getRenderedTerminalLineCount([...detailLines], getChoiceDetailMeasureWidth(terminalWidth));
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

function createCenteredPanelLayout(options: {
  kind: "select" | "text";
  terminalHeight: number;
  panelWidth: number;
  headerRows: number;
  footerRows: number;
  bodyLineCount: number;
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment;
}) {
  const layout = planTerminalModalLayout({
    kind: options.kind,
    terminalWidth: options.panelWidth,
    terminalHeight: options.terminalHeight,
    forcedPresentation: "inline",
    headerRows: options.headerRows,
    footerRows: options.footerRows,
    descriptor:
      options.kind === "text"
        ? createTerminalTextInputSizingDescriptor({
            bodyLineCount: options.bodyLineCount,
            minimumInlineWidth: options.panelWidth,
          })
        : createTerminalMessageSizingDescriptor({
            bodyLineCount: options.bodyLineCount,
            minimumInlineWidth: options.panelWidth,
          }),
  });

  return {
    ...layout,
    panelWidth: options.panelWidth,
    centeredPromptBackground: options.backgroundTreatment,
  } satisfies FrameworkTerminalModalLayoutResult;
}

function getRenderedPromptDetailLineCount(
  detailLines: readonly DerivedTagTerminalLine[],
  terminalWidth: number,
): number {
  return getRenderedTerminalLineCount([...detailLines], getChoiceDetailMeasureWidth(terminalWidth));
}

function getDeepestPromptDetailLines(
  detailLineSets: ReadonlyArray<readonly DerivedTagTerminalLine[]>,
  terminalWidth: number,
): DerivedTagTerminalLine[] {
  return detailLineSets.reduce<DerivedTagTerminalLine[]>((deepest, current) => {
    return getRenderedPromptDetailLineCount(current, terminalWidth) > getRenderedPromptDetailLineCount(deepest, terminalWidth)
      ? [...current]
      : deepest;
  }, []);
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
  terminalHeight: number;
  prompt: string;
  entryLabels: readonly string[];
  detailLineSets: ReadonlyArray<readonly DerivedTagTerminalLine[]>;
  panelWidth: number;
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment;
}) {
  const bodyLineCount = Math.max(
    1,
    ...options.detailLineSets.map((detailLines) =>
      getRenderedTerminalLineCount(
        [
          ...(options.prompt ? [{ text: options.prompt, tone: "section" as const }] : []),
          { text: options.entryLabels.join("   "), noWrap: true },
          { text: "" },
          ...detailLines,
        ],
        Math.max(24, options.panelWidth - 4),
      ),
    ),
  );
  return createCenteredPanelLayout({
    kind: "select",
    terminalHeight: options.terminalHeight,
    panelWidth: options.panelWidth,
    headerRows: 3,
    footerRows: 2,
    bodyLineCount,
    backgroundTreatment: options.backgroundTreatment,
  });
}

function createCenteredListPromptLayout(options: {
  terminalWidth: number;
  terminalHeight: number;
  itemCount: number;
  detailLineSets: ReadonlyArray<readonly DerivedTagTerminalLine[]>;
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment;
}) {
  const measuredWidth = Math.max(24, Math.min(84, options.terminalWidth - 2));
  const layout = planTerminalModalLayout({
    kind: "select",
    terminalWidth: measuredWidth,
    terminalHeight: options.terminalHeight,
    forcedPresentation: "inline",
    headerRows: 3,
    footerRows: 3,
    descriptor: createChoicePromptDescriptor(
      measuredWidth,
      options.itemCount,
      getDeepestPromptDetailLines(options.detailLineSets, measuredWidth),
    ),
  });

  return {
    ...layout,
    panelWidth: measuredWidth,
    centeredPromptBackground: options.backgroundTreatment,
  } satisfies FrameworkTerminalModalLayoutResult;
}

function getCenteredTextPromptWidth(
  terminalWidth: number,
  options: TextPromptOptions,
): number {
  const contentWidth = Math.max(
    options.title.length,
    options.prompt.length,
    options.hint?.length ?? 0,
    (options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.").length,
  );
  return Math.max(44, Math.min(terminalWidth, Math.max(60, contentWidth + 8)));
}

function createCenteredTextPromptLayout(options: {
  terminalWidth: number;
  terminalHeight: number;
  prompt: TextPromptOptions;
  currentValue: string;
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment;
}) {
  const panelWidth = getCenteredTextPromptWidth(options.terminalWidth, options.prompt);
  const measuredWidth = Math.max(24, Math.min(panelWidth, options.terminalWidth - 2));
  return createCenteredPanelLayout({
    kind: "text",
    terminalHeight: options.terminalHeight,
    panelWidth: measuredWidth,
    headerRows: 3,
    footerRows: 1,
    bodyLineCount: getRenderedTerminalLineCount(
      buildTextPromptBodyLines(options.prompt, options.currentValue),
      Math.max(24, measuredWidth - 4),
    ),
    backgroundTreatment: options.backgroundTreatment,
  });
}

function createCenteredMessageLayout(options: {
  kind: "select";
  terminalHeight: number;
  terminalWidth: number;
  bodyLines: readonly DerivedTagTerminalLine[];
  footerRows: number;
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment;
}) {
  const panelWidth = Math.max(24, Math.min(84, options.terminalWidth - 2));
  return createCenteredPanelLayout({
    kind: options.kind,
    terminalHeight: options.terminalHeight,
    panelWidth,
    headerRows: 3,
    footerRows: options.footerRows,
    bodyLineCount: getRenderedTerminalLineCount(options.bodyLines as DerivedTagTerminalLine[], Math.max(24, panelWidth - 4)),
    backgroundTreatment: options.backgroundTreatment,
  });
}

export function planTerminalModalStateLayout(
  modal: TerminalModalState,
  terminalWidth: number,
  terminalHeight: number,
): FrameworkTerminalModalLayoutResult | null {
  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    const footer = modal.options.footer ?? [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }];
    const normalizedPresentation = normalizeTerminalPromptPresentation(modal.options.presentation);
    return planTerminalModalLayout({
      kind: "dialog",
      terminalWidth,
      terminalHeight,
      forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "inline"),
      headerRows: 3,
      footerRows: footer.length,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(modal.options.body, terminalWidth),
      }),
    });
  }

  if (modal.kind === "text") {
    const normalizedPresentation = normalizeTerminalPromptPresentation(modal.options.presentation);
    if (isCenteredPromptPresentation(normalizedPresentation)) {
      return createCenteredTextPromptLayout({
        terminalWidth,
        terminalHeight,
        prompt: modal.options,
        currentValue: modal.value,
        backgroundTreatment: normalizedPresentation,
      });
    }
    return planTerminalModalLayout({
      kind: "text",
      terminalWidth,
      terminalHeight,
      forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "inline"),
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalTextInputSizingDescriptor({
        bodyLineCount: getRenderedTerminalLineCount(buildTextPromptBodyLines(modal.options, modal.value), terminalWidth),
      }),
    });
  }

  if (modal.kind === "select") {
    const normalizedPresentation = normalizeTerminalPromptPresentation(modal.options.presentation);
    const filteredEntries =
      modal.options.filtering && modal.options.choiceLayout !== "horizontal"
        ? filterPromptEntries(modal.options.entries, modal.filterText)
        : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

    if (filteredEntries.length === 0) {
      const bodyLines: DerivedTagTerminalLine[] = [
        { text: modal.options.prompt, tone: "section" },
        ...(modal.filterText ? [{ text: `Search /${modal.filterText}`, tone: "accent" as const }] : []),
        {
          text: modal.filterText
            ? "No options match the current filter."
            : "No options are available for this scope.",
          tone: "warning",
        },
      ];
      if (isCenteredPromptPresentation(normalizedPresentation)) {
        return createCenteredMessageLayout({
          kind: "select",
          terminalHeight,
          terminalWidth,
          bodyLines,
          footerRows: 2,
          backgroundTreatment: normalizedPresentation,
        });
      }
      return planTerminalModalLayout({
        kind: "select",
        terminalWidth,
        terminalHeight,
        forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "screen"),
        headerRows: 3,
        footerRows: 2,
        descriptor: createTerminalMessageSizingDescriptor({
          bodyLineCount: getRenderedTerminalLineCount(bodyLines, terminalWidth),
        }),
      });
    }

    const selectedIndex = getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, modal.filterText);
    const selectedOption = modal.options.entries[selectedIndex];
    if (modal.options.choiceLayout === "horizontal" && isCenteredPromptPresentation(normalizedPresentation)) {
      const detailLineSets = filteredEntries.map((entry) => buildPromptDetailLines(entry.entry));
      const panelWidth = getCenteredChoicePromptWidth(terminalWidth, {
        title: modal.options.title,
        subtitle: modal.options.subtitle,
        prompt: modal.options.prompt,
        entryLabels: filteredEntries.map((entry) => entry.entry.label),
        detailLines: detailLineSets.flat(),
      });
      return createCenteredChoicePromptLayout({
        terminalHeight,
        prompt: modal.options.prompt,
        entryLabels: filteredEntries.map((entry) => entry.entry.label),
        detailLineSets,
        panelWidth: Math.max(24, Math.min(panelWidth, terminalWidth - 2)),
        backgroundTreatment: normalizedPresentation,
      });
    }
    if (isCenteredPromptPresentation(normalizedPresentation)) {
      return createCenteredListPromptLayout({
        terminalWidth,
        terminalHeight,
        itemCount: filteredEntries.length,
        detailLineSets: filteredEntries.map((entry) => [
          ...buildPromptDetailLines(entry.entry),
          { text: "" },
          { text: `Selected: ${entry.entry.label}`, tone: "accent" },
        ]),
        backgroundTreatment: normalizedPresentation,
      });
    }

    return planTerminalModalLayout({
      kind: "select",
      terminalWidth,
      terminalHeight,
      forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "screen"),
      headerRows: 3,
      footerRows: 3,
      descriptor: createChoicePromptDescriptor(terminalWidth, filteredEntries.length, [
        ...buildPromptDetailLines(selectedOption),
        { text: "" },
        { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]),
    });
  }

  const normalizedPresentation = normalizeTerminalPromptPresentation(modal.options.presentation);
  const filteringEnabled = getMultiSelectPromptFilteringEnabled(modal.options);
  const filteredEntries = filteringEnabled
    ? filterPromptEntries(modal.options.entries, modal.filterText)
    : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

  if (filteredEntries.length === 0) {
    return planTerminalModalLayout({
      kind: "multiselect",
      terminalWidth,
      terminalHeight,
      forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "screen"),
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
    forcedPresentation: coerceBaseModalPresentation(normalizedPresentation, "screen"),
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
