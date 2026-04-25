import React from "react";
import { Box, Text } from "ink";

import {
  TERMINAL_COMMAND_PALETTE_EMPTY_FILTER_FOOTER,
  TERMINAL_COMMAND_PALETTE_FILTER_FOOTER,
  TERMINAL_LIVE_FILTER_FOOTER,
  TERMINAL_SELECT_EMPTY_FOOTER,
  TERMINAL_TEXT_INPUT_FOOTER,
  formatTerminalInteractionFooter,
} from "../interaction-bindings.js";
import {
  isCenteredModalPresentation,
  type TerminalModalLayoutResult,
} from "../terminal-modal-layout.js";
import {
  TerminalRows,
  fitToWidth,
  renderRows,
  terminalToneProps,
} from "./line-rendering.js";
import {
  TerminalInlinePromptPanel,
  TerminalPaneView,
} from "./screen-components.js";
import {
  buildFilterExplorerPolicyBadgeSegments,
  buildFilterExplorerPolicyLabelSegments,
  buildFilterExplorerPolicySequenceSegments,
} from "./policy-presentation.js";
import {
  buildCommandPaletteDetailLines,
  buildPromptDetailLines,
  clampInlinePromptWindowStart,
  clampPromptSelectionIndex,
  filterCommandPaletteEntries,
  filterPromptEntries,
  getFilteredPromptSelectionIndex,
  getMultiSelectPromptFilteringEnabled,
  getPolicyPromptFilteringEnabled,
} from "./modal-helpers.js";
import { buildPolicySelection, getPolicyStateForValue } from "./modal-policy-state.js";
import type {
  CommandPaletteOptions,
  DerivedTagTerminalLine,
  DerivedTagTerminalPolicyState,
  MultiSelectPromptOptions,
  PolicyPromptOptions,
  TerminalSelectModalOptions,
  TextPromptOptions,
} from "./types.js";

export function InlinePromptMessageBody({
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

function buildPickerFilterStatusLine(filterText: string): DerivedTagTerminalLine {
  return {
    text: filterText ? `Search /${filterText}` : "Search /(type to filter)",
    tone: "accent",
  };
}

function buildPickerFooterLines(options: {
  filterMode: boolean;
  filterText: string;
  filteringEnabled: boolean;
  countText: string;
  primaryFooterText: string;
  secondaryFooterText: string;
  supportsCommands?: boolean;
}): DerivedTagTerminalLine[] {
  if (options.filterMode) {
    return [
      { text: TERMINAL_LIVE_FILTER_FOOTER, tone: "dim" },
      buildPickerFilterStatusLine(options.filterText),
      { text: options.countText, tone: "accent" },
    ];
  }

  return [
    { text: options.primaryFooterText, tone: "dim" },
    {
      text: options.filteringEnabled
        ? `${options.secondaryFooterText}${options.supportsCommands ? "  : commands" : ""}  / filter`
        : `${options.secondaryFooterText}${options.supportsCommands ? "  : commands" : ""}`,
      tone: "dim",
    },
    { text: options.countText, tone: "accent" },
  ];
}

function CenteredChoicePromptBody({
  prompt,
  options,
  selectedIndex,
  width,
  height,
}: {
  prompt: string;
  options: ReadonlyArray<TerminalSelectModalOptions["entries"][number]>;
  selectedIndex: number;
  width: number;
  height: number;
}): React.JSX.Element {
  const promptLines = prompt ? [{ text: prompt, tone: "section" as const }] : [];
  const detailLines = buildPromptDetailLines(options[selectedIndex]);
  const chromeRows = promptLines.length + 2;
  const detailHeight = Math.max(0, height - chromeRows);
  const optionSegments = options.flatMap((entry, index) => {
    const isSelected = index === selectedIndex;
    return [
      ...(index > 0 ? [{ text: "   " }] : []),
      ...(isSelected ? [{ text: "[", tone: "dim" as const }] : []),
      { text: entry.label, tone: isSelected ? ("accent" as const) : ("default" as const) },
      ...(isSelected ? [{ text: "]", tone: "dim" as const }] : []),
    ];
  });

  return (
    <Box flexDirection="column" width={width} height={height}>
      {promptLines.length > 0 ? <InlinePromptMessageBody width={width} height={promptLines.length} lines={promptLines} /> : null}
      <Box width={width} justifyContent="center">
        <TerminalRows width={width} lines={[{ text: "", tone: "default", segments: optionSegments }]} />
      </Box>
      <Text wrap="truncate-end">{""}</Text>
      {detailHeight > 0 ? <InlinePromptMessageBody width={width} height={detailHeight} lines={detailLines} /> : null}
    </Box>
  );
}

export function buildTextPromptBodyLines(options: TextPromptOptions, currentValue: string): DerivedTagTerminalLine[] {
  const previewLines = options.buildPreviewLines?.(currentValue) ?? [];

  return [
    ...(options.hint ? [{ text: options.hint, tone: "accent" as const }] : []),
    {
      text: "",
      segments: [
        { text: "> ", tone: "dim" },
        { text: currentValue || "(blank)", tone: currentValue ? "accent" : "dim" },
      ],
    },
    { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
    ...(previewLines.length > 0
      ? [
          { text: "" },
          { text: options.previewTitle ?? "Preview", tone: "section" as const },
          ...previewLines,
        ]
      : []),
  ];
}

export function TextPromptBody({
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
          lines={buildTextPromptBodyLines(options, currentValue)}
        />
      }
      footer={[{ text: TERMINAL_TEXT_INPUT_FOOTER, tone: "dim" }]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

export function CommandPaletteBody({
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

export function SelectPromptBody({
  options,
  selectedIndex,
  filterText,
  filterMode,
  width,
  layout,
}: {
  options: TerminalSelectModalOptions;
  selectedIndex: number;
  filterText: string;
  filterMode: boolean;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  const filteredEntries =
    options.filtering && options.choiceLayout !== "horizontal"
      ? filterPromptEntries(options.entries, filterText)
      : options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

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
              ...(filterText ? [buildPickerFilterStatusLine(filterText)] : []),
              {
                text: filterText ? "No options match the current filter." : "No options are available for this scope.",
                tone: "warning",
              },
            ]}
          />
        }
        footer={filterText ? [{ text: TERMINAL_LIVE_FILTER_FOOTER, tone: "dim" }, buildPickerFilterStatusLine(filterText)] : [{ text: TERMINAL_SELECT_EMPTY_FOOTER, tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const filteredSelectedIndex = getFilteredPromptSelectionIndex(options.entries, selectedIndex, filterText);
  const selectedOption = options.entries[filteredSelectedIndex];
  if (options.choiceLayout === "horizontal" && isCenteredModalPresentation(layout.presentation)) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle}
        body={
          <CenteredChoicePromptBody
            prompt={options.prompt}
            options={filteredEntries.map((entry) => entry.entry)}
            selectedIndex={Math.max(
              0,
              filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
            )}
            width={width}
            height={layout.bodyHeight}
          />
        }
        footer={[
          { text: formatTerminalInteractionFooter([{ id: "moveHorizontal", label: "change mode" }]), tone: "dim" },
          { text: formatTerminalInteractionFooter([{ id: "select", label: "confirm" }, { id: "cancel" }]), tone: "dim" },
        ]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const selectedVisibleIndex = Math.max(
    0,
    filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
  );
  const windowStart = clampInlinePromptWindowStart(selectedVisibleIndex, filteredEntries.length, visibleCount);
  const visibleEntries = filteredEntries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={filterMode ? `Search /${filterText}` : options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: entry.entry.label,
            tone: windowStart + offset === selectedVisibleIndex ? "selected" : "default",
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
      footer={buildPickerFooterLines({
        filterMode,
        filterText,
        filteringEnabled: options.filtering,
        countText: `${selectedVisibleIndex + 1}/${filteredEntries.length} focused`,
        primaryFooterText: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
        secondaryFooterText: formatTerminalInteractionFooter([{ id: "select" }, { id: "back", label: "cancel" }]),
        supportsCommands: options.supportsCommands,
      })}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

export function MultiSelectPromptBody({
  options,
  selectedIndex,
  filterText,
  filterMode,
  selectedValues,
  width,
  layout,
}: {
  options: MultiSelectPromptOptions<string>;
  selectedIndex: number;
  filterText: string;
  filterMode: boolean;
  selectedValues: string[];
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  const filteringEnabled = getMultiSelectPromptFilteringEnabled(options);
  const filteredEntries = filteringEnabled
    ? filterPromptEntries(options.entries, filterText)
    : options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

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
              ...(filterText ? [buildPickerFilterStatusLine(filterText)] : []),
              {
                text: filterText ? "No options match the current filter." : "No options are available for this scope.",
                tone: "warning",
              },
            ]}
          />
        }
        footer={
          filterText
            ? [{ text: TERMINAL_LIVE_FILTER_FOOTER, tone: "dim" }, buildPickerFilterStatusLine(filterText)]
            : [{ text: formatTerminalInteractionFooter([{ id: "back", label: "return" }]), tone: "dim" }]
        }
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const filteredSelectedIndex = getFilteredPromptSelectionIndex(options.entries, selectedIndex, filterText);
  const selectedOption = options.entries[filteredSelectedIndex];
  const selectedSet = new Set(selectedValues);
  const selectedLabels = options.entries.filter((entry) => selectedSet.has(entry.value)).map((entry) => entry.label);
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const selectedVisibleIndex = Math.max(
    0,
    filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
  );
  const windowStart = clampInlinePromptWindowStart(selectedVisibleIndex, filteredEntries.length, visibleCount);
  const visibleEntries = filteredEntries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={filterMode ? `Search /${filterText}` : options.prompt}
          entries={visibleEntries.map((entry, offset) => {
            const isSelected = selectedSet.has(entry.entry.value);
            const isFocused = windowStart + offset === selectedVisibleIndex;
            return {
              text: "",
              segments: [
                { text: "[", tone: "dim" as const },
                { text: isSelected ? "✓" : " ", tone: isSelected ? ("success" as const) : ("dim" as const) },
                { text: "] ", tone: "dim" as const },
                { text: entry.entry.label, tone: isFocused ? ("accent" as const) : ("default" as const) },
              ],
              tone: "default" as const,
              noWrap: true,
            };
          })}
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
      footer={buildPickerFooterLines({
        filterMode,
        filterText,
        filteringEnabled,
        countText: `${selectedValues.length} selected | Focused: ${selectedOption?.label ?? "(none)"}`,
        primaryFooterText: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
        secondaryFooterText: formatTerminalInteractionFooter([{ id: "toggle" }, { id: "return" }]),
        supportsCommands: options.supportsCommands,
      })}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}

export function buildPolicySummaryLines(
  options: PolicyPromptOptions<string>,
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalLine[] {
  const selection = buildPolicySelection(options.entries, valueStates);
  const labelsByValue = new Map(options.entries.map((entry) => [entry.value, entry.label]));

  return options.allowedStates.map((state) => ({
    text: "",
    segments: [
      ...buildFilterExplorerPolicyLabelSegments(state),
      { text: ": ", tone: "dim" },
      {
        text:
          selection[state].length > 0
            ? selection[state].map((value) => labelsByValue.get(value) ?? value).join(", ")
            : "(none)",
      },
    ],
  }));
}

export function PolicyPromptBody({
  options,
  selectedIndex,
  filterText,
  filterMode,
  valueStates,
  width,
  layout,
}: {
  options: PolicyPromptOptions<string>;
  selectedIndex: number;
  filterText: string;
  filterMode: boolean;
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element {
  const filteringEnabled = getPolicyPromptFilteringEnabled(options);
  const filteredEntries = filteringEnabled
    ? filterPromptEntries(options.entries, filterText)
    : options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));

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
              ...(filterText ? [buildPickerFilterStatusLine(filterText)] : []),
              {
                text: filterText ? "No options match the current filter." : "No options are available for this scope.",
                tone: "warning",
              },
            ]}
          />
        }
        footer={
          filterText
            ? [{ text: TERMINAL_LIVE_FILTER_FOOTER, tone: "dim" }, buildPickerFilterStatusLine(filterText)]
            : [{ text: formatTerminalInteractionFooter([{ id: "return" }]), tone: "dim" }]
        }
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }

  const filteredSelectedIndex = getFilteredPromptSelectionIndex(options.entries, selectedIndex, filterText);
  const selectedOption = options.entries[filteredSelectedIndex];
  const selectedState = selectedOption ? getPolicyStateForValue(selectedOption.value, valueStates) : undefined;
  const visibleCount = Math.max(1, layout.visibleListCapacity);
  const selectedVisibleIndex = Math.max(
    0,
    filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
  );
  const windowStart = clampInlinePromptWindowStart(selectedVisibleIndex, filteredEntries.length, visibleCount);
  const visibleEntries = filteredEntries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={
        <InlinePromptChoiceBody
          prompt={filterMode ? `Search /${filterText}` : options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: "",
            segments: [
              ...buildFilterExplorerPolicyBadgeSegments(getPolicyStateForValue(entry.entry.value, valueStates)),
              {
                text: ` ${entry.entry.label}`,
                tone: windowStart + offset === selectedVisibleIndex ? "selected" : "default",
              },
            ],
            tone: "default",
            noWrap: true,
          }))}
          detailLines={[
            ...buildPromptDetailLines(selectedOption),
            { text: "" },
            {
              text: "",
              segments: [
                { text: "Focused policy: ", tone: "accent" },
                ...buildFilterExplorerPolicyLabelSegments(selectedState),
              ],
            },
            ...buildPolicySummaryLines(options, valueStates),
          ]}
          focusedLabel={`Focused ${selectedIndex + 1}/${options.entries.length}`}
          width={width}
          height={layout.bodyHeight}
          layout={layout}
        />
      }
      footer={[
        ...buildPickerFooterLines({
          filterMode,
          filterText,
          filteringEnabled,
          countText: `${selectedVisibleIndex + 1}/${filteredEntries.length} focused`,
          primaryFooterText: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]),
          secondaryFooterText: formatTerminalInteractionFooter([{ id: "cycle" }, { id: "return" }]),
        }),
        {
          text: "",
          segments: [
            { text: "Cycle order: ", tone: "accent" },
            ...buildFilterExplorerPolicySequenceSegments([undefined, ...options.allowedStates, undefined]),
          ],
        },
      ]}
      width={width}
      height={layout.totalHeight}
      showTopBorder={layout.showTopBorder}
    />
  );
}
