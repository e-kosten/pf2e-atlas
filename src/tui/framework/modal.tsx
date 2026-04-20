import React from "react";
import { Box, Text, useInput } from "ink";

import {
  TERMINAL_COMMAND_PALETTE_EMPTY_FILTER_FOOTER,
  TERMINAL_COMMAND_PALETTE_FILTER_FOOTER,
  TERMINAL_DIALOG_CONTINUE_FOOTER,
  TERMINAL_SELECT_EMPTY_FOOTER,
  TERMINAL_TEXT_INPUT_FOOTER,
  formatTerminalInteractionFooter,
} from "../interaction-bindings.js";
import {
  createTerminalCommandPaletteInteractionContext,
  createTerminalInteractionContextRouterState,
  createTerminalMultiSelectPromptInteractionContext,
  createTerminalPolicyPromptInteractionContext,
  createTerminalSelectPromptInteractionContext,
  createTerminalTextPromptInteractionContext,
  routeTerminalInteractionContext,
} from "../interaction-context-router.js";
import {
  createTerminalChoiceSizingDescriptor,
  createTerminalMessageSizingDescriptor,
  createTerminalTextInputSizingDescriptor,
  planTerminalModalLayout,
  type TerminalModalLayoutResult,
} from "../terminal-modal-layout.js";
import { createDerivedTagTerminalInputEvent, moveSelectionWrapped } from "./input.js";
import {
  TerminalInlinePromptPanel,
  TerminalPaneView,
  TerminalRows,
  fitToWidth,
  getRenderedTerminalLineCount,
  renderRows,
  terminalToneProps,
} from "./rendering.js";
import type {
  CommandPaletteOptions,
  DerivedTagTerminalCommandOption,
  DerivedTagTerminalLine,
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalPolicyState,
  DerivedTagTerminalSelectOption,
  MultiSelectPromptOptions,
  OptionalSelectPromptOptions,
  PolicyPromptOptions,
  SelectPromptOptions,
  TerminalModalState,
  TerminalSelectModalEntry,
  TerminalSelectModalOptions,
  TerminalSelectOptionDetails,
  TextPromptOptions,
} from "./types.js";

export function createEmptyPolicySelection<T extends string>(): DerivedTagTerminalPolicySelection<T> {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

function clampInlinePromptWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (itemCount <= visibleCount) {
    return 0;
  }

  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function clampPromptSelectionIndex(selectedIndex: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(selectedIndex, itemCount - 1));
}

function buildPromptDetailLines(option: TerminalSelectOptionDetails | undefined): DerivedTagTerminalLine[] {
  if (option?.detailLines?.length) {
    return option.detailLines;
  }
  if (option?.description) {
    return [{ text: option.label, tone: "section" }, { text: option.description }];
  }

  return [
    { text: option?.label ?? "(none)", tone: "section" },
    { text: "No additional details.", tone: "dim" },
  ];
}

export function buildSelectModalOptions<T>(options: SelectPromptOptions<T>): TerminalSelectModalOptions {
  return {
    title: options.title,
    subtitle: options.subtitle,
    prompt: options.prompt,
    entries: options.entries.map((entry) => ({
      kind: "selected" as const,
      value: entry.value,
      label: entry.label,
      description: entry.description,
      detailLines: entry.detailLines,
    })),
    presentation: options.presentation,
  };
}

export function buildOptionalSelectModalOptions<T>(
  options: OptionalSelectPromptOptions<T>,
): TerminalSelectModalOptions {
  return {
    title: options.title,
    subtitle: options.subtitle,
    prompt: options.prompt,
    entries: [
      {
        kind: "all" as const,
        label: options.allOption.label,
        description: options.allOption.description,
        detailLines: options.allOption.detailLines,
      },
      ...options.entries.map((entry) => ({
        kind: "selected" as const,
        value: entry.value,
        label: entry.label,
        description: entry.description,
        detailLines: entry.detailLines,
      })),
    ],
    presentation: options.presentation,
  };
}

export function getSelectPromptInitialIndex(entries: TerminalSelectModalEntry[], selectedValue: unknown): number {
  return Math.max(
    0,
    entries.findIndex((entry) =>
      entry.kind === "all" ? selectedValue === null : Object.is(entry.value, selectedValue),
    ),
  );
}

function filterCommandPaletteEntries(
  entries: DerivedTagTerminalCommandOption<string>[],
  filterText: string,
): DerivedTagTerminalCommandOption<string>[] {
  const normalizedTerms = filterText
    .toLowerCase()
    .split(/\s+/)
    .map((term) => term.trim())
    .filter((term) => term.length > 0);

  if (normalizedTerms.length === 0) {
    return entries;
  }

  return entries.filter((entry) => {
    const searchableText = [entry.label, entry.description ?? "", ...(entry.aliases ?? []), ...(entry.keywords ?? [])]
      .join(" ")
      .toLowerCase();

    return normalizedTerms.every((term) => searchableText.includes(term));
  });
}

export function getFirstEnabledCommandIndex(entries: DerivedTagTerminalCommandOption<string>[]): number {
  const enabledIndex = entries.findIndex((entry) => !entry.disabled);
  return enabledIndex >= 0 ? enabledIndex : 0;
}

function buildCommandPaletteDetailLines(
  option: DerivedTagTerminalCommandOption<string> | undefined,
  filterText: string,
): DerivedTagTerminalLine[] {
  const lines = option?.detailLines ?? [
    { text: option?.label ?? "(none)", tone: "section" as const },
    { text: option?.description ?? "No additional details." },
    ...(option?.aliases?.length ? [{ text: `Aliases: ${option.aliases.join(", ")}`, tone: "accent" as const }] : []),
  ];

  return [
    ...lines,
    ...(option?.disabled
      ? [
          {
            text: option.disabledReason
              ? `Unavailable: ${option.disabledReason}`
              : "This command is currently unavailable.",
            tone: "warning" as const,
          },
        ]
      : []),
    { text: "" },
    { text: `Filter: ${filterText || "(none)"}`, tone: "accent" as const },
  ];
}

function InlinePromptMessageBody({
  lines,
  width,
  height,
}: {
  lines: DerivedTagTerminalLine[];
  width: number;
  height: number;
}): React.JSX.Element {
  return <TerminalRows lines={renderRows(lines, width, height)} width={width} />;
}

function InlinePromptChoiceBody({
  prompt,
  entries,
  detailLines,
  focusedLabel,
  width,
  height,
  layout,
}: {
  prompt: string;
  entries: DerivedTagTerminalLine[];
  detailLines: DerivedTagTerminalLine[];
  focusedLabel: string;
  width: number;
  height: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  if (layout.paneMode === "single-column") {
    const listRows = Math.max(0, layout.regions.listRows);
    const detailRows = Math.max(0, layout.regions.detailRows);
    const visibleListCount = Math.max(0, listRows - 2);
    const visibleDetailCount = Math.max(0, detailRows - 2);

    return (
      <Box flexDirection="column" width={width} height={height}>
        {listRows > 0 ? (
          <>
            <Text wrap="truncate-end" {...terminalToneProps("section")}>
              {fitToWidth(prompt, width)}
            </Text>
            {listRows > 1 ? (
              <Text wrap="truncate-end" {...terminalToneProps("dim")}>
                {fitToWidth("─".repeat(Math.max(0, width)), width)}
              </Text>
            ) : null}
            {visibleListCount > 0 ? (
              <TerminalRows lines={renderRows(entries, width, visibleListCount)} width={width} />
            ) : null}
          </>
        ) : null}
        {detailRows > 0 ? (
          <>
            <Text wrap="truncate-end" {...terminalToneProps("dim")}>
              {fitToWidth("─".repeat(Math.max(0, width)), width)}
            </Text>
            {detailRows > 1 ? (
              <Text wrap="truncate-end" {...terminalToneProps("accent")}>
                {fitToWidth(focusedLabel, width)}
              </Text>
            ) : null}
            {visibleDetailCount > 0 ? (
              <TerminalRows lines={renderRows(detailLines, width, visibleDetailCount)} width={width} />
            ) : null}
          </>
        ) : null}
      </Box>
    );
  }

  const separator = Array.from({ length: Math.max(1, height) }, () => "│").join("\n");

  return (
    <Box flexDirection="row" width={width} height={height}>
      <TerminalPaneView
        pane={{
          title: prompt,
          lines: entries,
          active: true,
        }}
        width={layout.paneWidths?.left ?? width}
        height={layout.regions.listRows}
      />
      <Text wrap="truncate-end" {...terminalToneProps("dim")}>
        {separator}
      </Text>
      <TerminalPaneView
        pane={{
          title: focusedLabel,
          lines: detailLines,
        }}
        width={layout.paneWidths?.right ?? width}
        height={layout.regions.detailRows}
      />
    </Box>
  );
}

function TextPromptBody({
  options,
  currentValue,
  width,
  layout,
}: {
  options: TextPromptOptions;
  currentValue: string;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.prompt}
      body={
        <InlinePromptMessageBody
          width={width}
          height={layout.bodyHeight}
          lines={[
            ...(options.hint ? [{ text: options.hint, tone: "accent" as const }] : []),
            { text: `> ${currentValue || ""}`, tone: "selected" },
            { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
          ]}
        />
      }
      footer={[{ text: TERMINAL_TEXT_INPUT_FOOTER, tone: "dim" }]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

function CommandPaletteBody({
  options,
  filterText,
  selectedIndex,
  width,
  layout,
}: {
  options: CommandPaletteOptions<string>;
  filterText: string;
  selectedIndex: number;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  const filteredEntries = filterCommandPaletteEntries(options.entries, filterText);
  const clampedSelectedIndex = clampPromptSelectionIndex(selectedIndex, filteredEntries.length);

  if (filteredEntries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={
          <InlinePromptMessageBody
            width={width}
            height={layout.bodyHeight}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: `Filter: ${filterText || "(none)"}`, tone: "accent" },
              { text: "No commands match the current filter.", tone: "warning" },
            ]}
          />
        }
        footer={[{ text: TERMINAL_COMMAND_PALETTE_EMPTY_FILTER_FOOTER, tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const selectedOption = filteredEntries[clampedSelectedIndex];
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const windowStart = clampInlinePromptWindowStart(clampedSelectedIndex, filteredEntries.length, visibleCount);
  const visibleEntries = filteredEntries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle ?? options.prompt}
      body={
        <InlinePromptChoiceBody
          prompt={filterText ? `Filter: ${filterText}` : options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: `${entry.label}${entry.disabled ? " | unavailable" : ""}`,
            tone: windowStart + offset === clampedSelectedIndex ? "selected" : entry.disabled ? "dim" : "default",
            noWrap: true,
          }))}
          detailLines={buildCommandPaletteDetailLines(selectedOption, filterText)}
          focusedLabel={`Command ${clampedSelectedIndex + 1}/${filteredEntries.length}`}
          width={width}
          height={layout.bodyHeight}
          layout={layout}
        />
      }
      footer={[
        {
          text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
          tone: "dim",
        },
        { text: TERMINAL_COMMAND_PALETTE_FILTER_FOOTER, tone: "dim" },
        { text: `${filteredEntries.length} command${filteredEntries.length === 1 ? "" : "s"} visible`, tone: "accent" },
      ]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

function SelectPromptBody({
  options,
  selectedIndex,
  width,
  layout,
}: {
  options: TerminalSelectModalOptions;
  selectedIndex: number;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={
          <InlinePromptMessageBody
            width={width}
            height={layout.bodyHeight}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        }
        footer={[{ text: TERMINAL_SELECT_EMPTY_FOOTER, tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: entry.label,
            tone: windowStart + offset === selectedIndex ? "selected" : "default",
            noWrap: true,
          }))}
          detailLines={[
            ...buildPromptDetailLines(selectedOption),
            { text: "" },
            { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
          ]}
          focusedLabel={`Selected: ${selectedOption?.label ?? "(none)"}`}
          width={width}
          height={layout.bodyHeight}
          layout={layout}
        />
      }
      footer={[
        {
          text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
          tone: "dim",
        },
        { text: formatTerminalInteractionFooter([{ id: "select" }, { id: "back", label: "cancel" }]), tone: "dim" },
        { text: `${selectedIndex + 1}/${options.entries.length} focused`, tone: "accent" },
      ]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

function MultiSelectPromptBody({
  options,
  selectedIndex,
  selectedValues,
  width,
  layout,
}: {
  options: MultiSelectPromptOptions<string>;
  selectedIndex: number;
  selectedValues: string[];
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={
          <InlinePromptMessageBody
            width={width}
            height={layout.bodyHeight}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        }
        footer={[{ text: formatTerminalInteractionFooter([{ id: "back", label: "return" }]), tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const selectedSet = new Set(selectedValues);
  const selectedLabels = options.entries.filter((entry) => selectedSet.has(entry.value)).map((entry) => entry.label);
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: `[${selectedSet.has(entry.value) ? "x" : " "}] ${entry.label}`,
            tone: windowStart + offset === selectedIndex ? "selected" : "default",
            noWrap: true,
          }))}
          detailLines={[
            ...buildPromptDetailLines(selectedOption),
            { text: "" },
            { text: "Current Selection", tone: "section" },
            { text: selectedLabels.length > 0 ? selectedLabels.join(", ") : "(none)" },
          ]}
          focusedLabel={`Focused ${selectedIndex + 1}/${options.entries.length}`}
          width={width}
          height={layout.bodyHeight}
          layout={layout}
        />
      }
      footer={[
        {
          text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
          tone: "dim",
        },
        { text: formatTerminalInteractionFooter([{ id: "toggle" }, { id: "return" }]), tone: "dim" },
        { text: `${selectedValues.length} selected | Focused: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

function getPolicyStateForValue(
  value: string,
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalPolicyState | undefined {
  return valueStates[value];
}

function policyStateLabel(state: DerivedTagTerminalPolicyState | undefined): string {
  switch (state) {
    case undefined:
      return " ";
    case "any":
      return "ANY";
    case "all":
      return "ALL";
    case "exclude":
      return "NOT";
  }
}

export function createValueStateLookup(
  selection: Partial<DerivedTagTerminalPolicySelection<string>> | undefined,
): Record<string, DerivedTagTerminalPolicyState | undefined> {
  const valueStates: Record<string, DerivedTagTerminalPolicyState | undefined> = {};

  for (const value of selection?.exclude ?? []) {
    valueStates[value] = "exclude";
  }
  for (const value of selection?.all ?? []) {
    valueStates[value] = "all";
  }
  for (const value of selection?.any ?? []) {
    valueStates[value] = "any";
  }

  return valueStates;
}

function buildPolicySelection(
  entries: DerivedTagTerminalSelectOption<string>[],
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalPolicySelection<string> {
  const selection = createEmptyPolicySelection<string>();

  for (const entry of entries) {
    const state = getPolicyStateForValue(entry.value, valueStates);
    if (!state) {
      continue;
    }
    selection[state].push(entry.value);
  }

  selection.any.sort((left, right) => left.localeCompare(right));
  selection.all.sort((left, right) => left.localeCompare(right));
  selection.exclude.sort((left, right) => left.localeCompare(right));
  return selection;
}

function cyclePolicyState(
  currentState: DerivedTagTerminalPolicyState | undefined,
  allowedStates: DerivedTagTerminalPolicyState[],
  direction: 1 | -1 = 1,
): DerivedTagTerminalPolicyState | undefined {
  const stateOrder: Array<DerivedTagTerminalPolicyState | undefined> = [undefined, ...allowedStates];
  const currentIndex = stateOrder.findIndex((state) => state === currentState);
  const nextIndex = (((currentIndex + direction) % stateOrder.length) + stateOrder.length) % stateOrder.length;
  return stateOrder[nextIndex];
}

function buildPolicySummaryLines(
  options: PolicyPromptOptions<string>,
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalLine[] {
  const selection = buildPolicySelection(options.entries, valueStates);
  const labelsByValue = new Map(options.entries.map((entry) => [entry.value, entry.label]));

  return options.allowedStates.map((state) => ({
    text: `${state[0]!.toUpperCase()}${state.slice(1)}: ${
      selection[state].length > 0
        ? selection[state].map((value) => labelsByValue.get(value) ?? value).join(", ")
        : "(none)"
    }`,
  }));
}

function PolicyPromptBody({
  options,
  selectedIndex,
  valueStates,
  width,
  layout,
}: {
  options: PolicyPromptOptions<string>;
  selectedIndex: number;
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={
          <InlinePromptMessageBody
            width={width}
            height={layout.bodyHeight}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        }
        footer={[{ text: formatTerminalInteractionFooter([{ id: "return" }]), tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const selectedState = selectedOption ? getPolicyStateForValue(selectedOption.value, valueStates) : undefined;
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: `[${policyStateLabel(getPolicyStateForValue(entry.value, valueStates))}] ${entry.label}`,
            tone: windowStart + offset === selectedIndex ? "selected" : "default",
            noWrap: true,
          }))}
          detailLines={[
            ...buildPromptDetailLines(selectedOption),
            { text: "" },
            { text: `Focused policy: ${selectedState ?? "off"}`, tone: "accent" },
            ...buildPolicySummaryLines(options, valueStates),
          ]}
          focusedLabel={`Focused ${selectedIndex + 1}/${options.entries.length}`}
          width={width}
          height={layout.bodyHeight}
          layout={layout}
        />
      }
      footer={[
        {
          text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
          tone: "dim",
        },
        { text: formatTerminalInteractionFooter([{ id: "cycle" }, { id: "return" }]), tone: "dim" },
        { text: `Cycle order: off -> ${options.allowedStates.join(" -> ")} -> off`, tone: "accent" },
      ]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

function buildTextPromptBodyLines(options: TextPromptOptions, currentValue: string): DerivedTagTerminalLine[] {
  return [
    ...(options.hint ? [{ text: options.hint, tone: "accent" as const }] : []),
    { text: `> ${currentValue || ""}`, tone: "selected" },
    { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
  ];
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
        bodyLineCount: getRenderedTerminalLineCount(
          buildTextPromptBodyLines(modal.options, modal.value),
          terminalWidth,
        ),
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

export function DerivedTagTerminalModalHost({
  modal,
  setModal,
  exitApp,
  width,
  layout,
}: {
  modal: TerminalModalState;
  setModal: React.Dispatch<React.SetStateAction<TerminalModalState>>;
  exitApp: () => void;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element | null {
  const routerStateRef = React.useRef(
    createTerminalInteractionContextRouterState<
      "commandPalette" | "multiSelectPrompt" | "policyPrompt" | "selectPrompt" | "textPrompt"
    >(),
  );
  const pageSize = Math.max(1, layout.visibleListCapacity || 10);

  useInput(
    (input, key) => {
      const event = createDerivedTagTerminalInputEvent(input, key);
      if (event.systemAction === "interrupt") {
        exitApp();
        return;
      }

      if (!modal) {
        routerStateRef.current = createTerminalInteractionContextRouterState();
        return;
      }

      if (modal.kind === "dialog") {
        const resolver = modal.resolve;
        setModal(null);
        resolver();
        return;
      }

      if (modal.kind === "text") {
        const routed = routeTerminalInteractionContext(
          event,
          createTerminalTextPromptInteractionContext(),
          routerStateRef.current,
        );
        routerStateRef.current = routed.state;

        if (routed.route.textEntryIntent?.kind === "submit") {
          const resolver = modal.resolve;
          const trimmed = modal.value.trim();
          setModal(null);
          resolver(trimmed ? trimmed : undefined);
          return;
        }
        if (routed.route.textEntryIntent?.kind === "cancel") {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
          return;
        }
        if (routed.route.textEntryIntent?.kind === "deleteBackward") {
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: [...current.value].slice(0, -1).join("") } : current,
          );
          return;
        }
        if (routed.route.textEntryIntent?.kind === "append") {
          const appendText = routed.route.textEntryIntent.text;
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: current.value + appendText } : current,
          );
        }
        return;
      }

      if (modal.kind === "command") {
        const routed = routeTerminalInteractionContext(
          event,
          createTerminalCommandPaletteInteractionContext(pageSize),
          routerStateRef.current,
        );
        routerStateRef.current = routed.state;
        const filteredEntries = filterCommandPaletteEntries(modal.options.entries, modal.filterText);
        const clampedSelectedIndex = clampPromptSelectionIndex(modal.selectedIndex, filteredEntries.length);

        if (routed.route.textEntryIntent?.kind === "deleteBackward") {
          if (modal.filterText.length === 0) {
            const resolver = modal.resolve;
            setModal(null);
            resolver(undefined);
            return;
          }
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  filterText: [...current.filterText].slice(0, -1).join(""),
                  selectedIndex: getFirstEnabledCommandIndex(
                    filterCommandPaletteEntries(current.options.entries, [...current.filterText].slice(0, -1).join("")),
                  ),
                }
              : current,
          );
          return;
        }
        if (routed.route.textEntryIntent?.kind === "append") {
          const appendText = routed.route.textEntryIntent.text;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  filterText: current.filterText + appendText,
                  selectedIndex: getFirstEnabledCommandIndex(
                    filterCommandPaletteEntries(current.options.entries, current.filterText + appendText),
                  ),
                }
              : current,
          );
          return;
        }
        if (routed.route.navigationAction?.kind === "move") {
          const delta = routed.route.navigationAction.delta;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex: moveSelectionWrapped(clampedSelectedIndex, delta, filteredEntries.length),
                }
              : current,
          );
          return;
        }
        if (routed.route.navigationAction?.kind === "boundary") {
          const boundary = routed.route.navigationAction.boundary;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex: boundary === "start" ? 0 : Math.max(0, filteredEntries.length - 1),
                }
              : current,
          );
          return;
        }
        if (routed.route.interactionAction?.id === "select") {
          const selectedEntry = filteredEntries[clampedSelectedIndex];
          if (selectedEntry?.disabled) {
            return;
          }
          const resolver = modal.resolve;
          const selected = selectedEntry?.value;
          setModal(null);
          resolver(selected);
          return;
        }
        if (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
        }
        return;
      }

      if (modal.kind === "select" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver({ kind: "cancelled" });
        }
        return;
      }

      if (modal.kind === "multiselect" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver([]);
        }
        return;
      }

      if (modal.kind === "policy" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver(createEmptyPolicySelection());
        }
        return;
      }

      const choiceContext =
        modal.kind === "multiselect"
          ? createTerminalMultiSelectPromptInteractionContext(pageSize)
          : modal.kind === "policy"
            ? createTerminalPolicyPromptInteractionContext(pageSize)
            : createTerminalSelectPromptInteractionContext(pageSize);
      const routed = routeTerminalInteractionContext(event, choiceContext, routerStateRef.current);
      routerStateRef.current = routed.state;

      if (routed.route.navigationAction?.kind === "move") {
        const delta = routed.route.navigationAction.delta;
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex: moveSelectionWrapped(current.selectedIndex, delta, current.options.entries.length),
              }
            : current,
        );
        return;
      }
      if (routed.route.navigationAction?.kind === "boundary") {
        const boundary = routed.route.navigationAction.boundary;
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex: boundary === "start" ? 0 : Math.max(0, current.options.entries.length - 1),
              }
            : current,
        );
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "toggle") {
        const selected = modal.options.entries[modal.selectedIndex]?.value;
        if (!selected) {
          return;
        }
        setModal((current) =>
          current?.kind === "multiselect"
            ? {
                ...current,
                selectedValues: current.selectedValues.includes(selected)
                  ? current.selectedValues.filter((value) => value !== selected)
                  : [...current.selectedValues, selected],
              }
            : current,
        );
        return;
      }
      if (modal.kind === "select" && routed.route.interactionAction?.id === "select") {
        const resolver = modal.resolve;
        const selected = modal.options.entries[modal.selectedIndex];
        setModal(null);
        if (!selected) {
          resolver({ kind: "cancelled" });
          return;
        }
        resolver(selected.kind === "all" ? { kind: "all" } : { kind: "selected", value: selected.value });
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "return") {
        const resolver = modal.resolve;
        const selectedValues = modal.selectedValues;
        setModal(null);
        resolver(selectedValues);
        return;
      }
      if (modal.kind === "policy" && routed.route.cycleDirection) {
        const selected = modal.options.entries[modal.selectedIndex]?.value;
        if (!selected) {
          return;
        }
        setModal((current) =>
          current?.kind === "policy"
            ? {
                ...current,
                valueStates: {
                  ...current.valueStates,
                  [selected]: cyclePolicyState(
                    current.valueStates[selected],
                    current.options.allowedStates,
                    routed.route.cycleDirection,
                  ),
                },
              }
            : current,
        );
        return;
      }
      if (modal.kind === "policy" && (routed.route.interactionAction?.id === "return" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        const selection = buildPolicySelection(modal.options.entries, modal.valueStates);
        setModal(null);
        resolver(selection);
        return;
      }
      if (modal.kind === "select" && (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        setModal(null);
        resolver({ kind: "cancelled" });
      }
    },
    { isActive: modal !== null },
  );

  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    return (
      <TerminalInlinePromptPanel
        title={modal.options.title}
        subtitle={modal.options.subtitle}
        body={<InlinePromptMessageBody width={width} height={layout.bodyHeight} lines={modal.options.body} />}
        footer={modal.options.footer ?? [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }
  if (modal.kind === "text") {
    return <TextPromptBody options={modal.options} currentValue={modal.value} width={width} layout={layout} />;
  }
  if (modal.kind === "command") {
    return (
      <CommandPaletteBody
        options={modal.options}
        filterText={modal.filterText}
        selectedIndex={modal.selectedIndex}
        width={width}
        layout={layout}
      />
    );
  }
  if (modal.kind === "multiselect") {
    return (
      <MultiSelectPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        selectedValues={modal.selectedValues}
        width={width}
        layout={layout}
      />
    );
  }
  if (modal.kind === "policy") {
    return (
      <PolicyPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        valueStates={modal.valueStates}
        width={width}
        layout={layout}
      />
    );
  }

  return <SelectPromptBody options={modal.options} selectedIndex={modal.selectedIndex} width={width} layout={layout} />;
}
