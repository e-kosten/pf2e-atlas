import React from "react";
import { Box, Text, render as renderInkApp, useApp, useInput, useWindowSize, type Key } from "ink";
import {
  isBackNavigationKey,
  isConfirmKey,
  isMoveDownKey,
  isMoveLeftKey,
  isMoveRightKey,
  isMoveUpKey,
  isPageDownKey,
  isPageUpKey,
} from "./keymap.js";
import {
  TERMINAL_COMMAND_PALETTE_FILTER_FOOTER,
  TERMINAL_DIALOG_CONTINUE_FOOTER,
  TERMINAL_TEXT_INPUT_FOOTER,
  formatTerminalInteractionFooter,
  getTerminalInteractionCycleDirection,
  resolveTerminalInteractionAction,
} from "./interaction-bindings.js";
import {
  createTerminalChoiceSizingDescriptor,
  createTerminalMessageSizingDescriptor,
  createTerminalTextInputSizingDescriptor,
  planTerminalModalLayout,
  type TerminalModalLayoutResult,
  type TerminalModalPresentation,
} from "./terminal-modal-layout.js";

export type DerivedTagTerminalTone =
  | "default"
  | "heading"
  | "section"
  | "dim"
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "selected";

export type DerivedTagTerminalSegment = {
  text: string;
  tone?: DerivedTagTerminalTone;
};

export type DerivedTagTerminalLine = {
  text: string;
  segments?: DerivedTagTerminalSegment[];
  tone?: DerivedTagTerminalTone;
  indent?: number;
  noWrap?: boolean;
};

export type DerivedTagTerminalPane = {
  title: string;
  lines: DerivedTagTerminalLine[];
  active?: boolean;
};

export type DerivedTagTerminalSelectOption<T = string> = {
  value: T;
  label: string;
  description?: string;
  detailLines?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalCommandOption<T extends string = string> = DerivedTagTerminalSelectOption<T> & {
  aliases?: string[];
  disabled?: boolean;
  disabledReason?: string;
  keywords?: string[];
};

export type DerivedTagTerminalTwoPaneFocus = "list" | "detail";
export type DerivedTagTerminalTwoPaneLayoutMode = "split" | "detail-only";
export type DerivedTagTerminalTextInputAction = "submit" | "cancel" | "deleteBackward";
export type DerivedTagTerminalSystemAction = "interrupt";

export type DerivedTagTerminalInputEvent = {
  input: string;
  key: Key;
  printable?: string;
  systemAction?: DerivedTagTerminalSystemAction;
  textInputAction?: DerivedTagTerminalTextInputAction;
  isBackNavigationKey: () => boolean;
  isCommandPaletteKey: () => boolean;
  isConfirmKey: () => boolean;
  isConfirmOrToggleKey: () => boolean;
  isExactPrintableKey: (expected: string) => boolean;
  isExecuteKey: () => boolean;
  isFocusToggleKey: () => boolean;
  isHelpKey: () => boolean;
  isLayoutToggleKey: () => boolean;
  isMoveDownKey: () => boolean;
  isMoveLeftKey: () => boolean;
  isMoveRightKey: () => boolean;
  isMoveUpKey: () => boolean;
  isPageDownKey: () => boolean;
  isPageUpKey: () => boolean;
  isSearchKey: () => boolean;
  isTerminalBoundaryEndKey: () => boolean;
  isTerminalBoundaryStartKey: () => boolean;
  isTerminalJumpBackwardKey: () => boolean;
  isTerminalJumpForwardKey: () => boolean;
  isTerminalQuitKey: () => boolean;
  getCycleDirection: () => 1 | -1 | undefined;
  getReverseCycleDirection: () => 1 | -1 | undefined;
};

export type DerivedTagTerminalTextScreenProps = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalPaneScreenProps = {
  title: string;
  subtitle?: string;
  pane: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalTwoPaneScreenProps = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
};

export type DerivedTagTerminalThreePaneScreenProps = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  center: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
  centerWidth?: number;
};

type DerivedTagTerminalInlinePromptPanelProps = {
  title: string;
  subtitle?: string;
  body: React.ReactNode;
  footer?: DerivedTagTerminalLine[];
  width: number;
  height: number;
  showTopBorder?: boolean;
};

type DialogOptions = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
  presentation?: TerminalModalPresentation;
};

type TextPromptOptions = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalSelectPromptResult<T = string> = { kind: "cancelled" } | { kind: "selected"; value: T };

export type DerivedTagTerminalOptionalSelectPromptResult<T = string> =
  | { kind: "cancelled" }
  | { kind: "all" }
  | { kind: "selected"; value: T };

type SelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T;
  presentation?: TerminalModalPresentation;
};

type OptionalSelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  allOption: Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines">;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T | null;
  presentation?: TerminalModalPresentation;
};

type MultiSelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValues?: T[];
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalPolicyState = "any" | "all" | "exclude";

export type DerivedTagTerminalPolicySelection<T extends string = string> = {
  any: T[];
  all: T[];
  exclude: T[];
};

type PolicyPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  allowedStates: DerivedTagTerminalPolicyState[];
  selectedValues?: Partial<DerivedTagTerminalPolicySelection<T>>;
  presentation?: TerminalModalPresentation;
};

type CommandPaletteOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalCommandOption<T>[];
  presentation?: TerminalModalPresentation;
};

type TerminalSelectOptionDetails = Pick<
  DerivedTagTerminalSelectOption<unknown>,
  "label" | "description" | "detailLines"
>;

type TerminalSelectModalEntry =
  | (TerminalSelectOptionDetails & {
      kind: "selected";
      value: unknown;
    })
  | (TerminalSelectOptionDetails & {
      kind: "all";
    });

type TerminalSelectModalOptions = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: TerminalSelectModalEntry[];
  presentation?: TerminalModalPresentation;
};

type TerminalModalState =
  | null
  | {
      kind: "dialog";
      options: DialogOptions;
      resolve: () => void;
    }
  | {
      kind: "text";
      options: TextPromptOptions;
      value: string;
      resolve: (value: string | undefined) => void;
    }
  | {
      kind: "select";
      options: TerminalSelectModalOptions;
      selectedIndex: number;
      resolve: (
        value: DerivedTagTerminalSelectPromptResult<unknown> | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
      ) => void;
    }
  | {
      kind: "multiselect";
      options: MultiSelectPromptOptions<string>;
      selectedIndex: number;
      selectedValues: string[];
      resolve: (value: string[]) => void;
    }
  | {
      kind: "policy";
      options: PolicyPromptOptions<string>;
      selectedIndex: number;
      valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>;
      resolve: (value: DerivedTagTerminalPolicySelection<string>) => void;
    }
  | {
      kind: "command";
      options: CommandPaletteOptions<string>;
      filterText: string;
      selectedIndex: number;
      resolve: (value: string | undefined) => void;
    };

type DerivedTagTerminalContextValue = {
  exitApp: (result?: unknown) => void;
  getTerminalHeight: () => number;
  getTerminalWidth: () => number;
  modalActive: boolean;
  pauseForAnyKey: (message: string) => Promise<void>;
  promptCommandPalette: <T extends string>(options: CommandPaletteOptions<T>) => Promise<T | undefined>;
  promptOptionalSelectOption: <T>(
    options: OptionalSelectPromptOptions<T>,
  ) => Promise<DerivedTagTerminalOptionalSelectPromptResult<T>>;
  promptPolicySelectOption: <T extends string>(
    options: PolicyPromptOptions<T>,
  ) => Promise<DerivedTagTerminalPolicySelection<T>>;
  promptMultiSelectOption: <T extends string>(options: MultiSelectPromptOptions<T>) => Promise<T[]>;
  promptSelectOption: <T>(options: SelectPromptOptions<T>) => Promise<DerivedTagTerminalSelectPromptResult<T>>;
  promptTextInput: (options: TextPromptOptions) => Promise<string | undefined>;
  showDialog: (options: DialogOptions) => Promise<void>;
};

export type DerivedTagTerminalApp = DerivedTagTerminalContextValue;

const DerivedTagTerminalContext = React.createContext<DerivedTagTerminalContextValue | null>(null);

function ensureTerminalContext(): DerivedTagTerminalContextValue {
  const context = React.useContext(DerivedTagTerminalContext);
  if (!context) {
    throw new Error("DerivedTagTerminalContext is not available.");
  }
  return context;
}

function normalizeLine(line: DerivedTagTerminalLine): Required<DerivedTagTerminalLine> {
  return {
    text: line.text,
    segments: line.segments ?? [],
    tone: line.tone ?? "default",
    indent: line.indent ?? 0,
    noWrap: line.noWrap ?? false,
  };
}

function segmentText(segments: DerivedTagTerminalSegment[]): string {
  return segments.map((segment) => segment.text).join("");
}

function visibleWidth(text: string): number {
  return [...text].length;
}

function truncateText(text: string, width: number): string {
  if (width <= 0) {
    return "";
  }

  const characters = [...text];
  if (characters.length <= width) {
    return text;
  }

  return characters.slice(0, width).join("");
}

function truncateSegments(segments: DerivedTagTerminalSegment[], width: number): DerivedTagTerminalSegment[] {
  if (width <= 0 || segments.length === 0) {
    return [];
  }

  const truncated: DerivedTagTerminalSegment[] = [];
  let remainingWidth = width;

  for (const segment of segments) {
    if (remainingWidth <= 0) {
      break;
    }
    const segmentWidth = visibleWidth(segment.text);
    if (segmentWidth <= remainingWidth) {
      truncated.push(segment);
      remainingWidth -= segmentWidth;
      continue;
    }
    truncated.push({
      text: truncateText(segment.text, remainingWidth),
      tone: segment.tone,
    });
    break;
  }

  return truncated;
}

function fitToWidth(text: string, width: number): string {
  const truncated = truncateText(text, width);
  const paddingWidth = Math.max(0, width - visibleWidth(truncated));
  return `${truncated}${" ".repeat(paddingWidth)}`;
}

function wrapPlainText(text: string, width: number): string[] {
  if (width <= 0) {
    return [];
  }

  const normalized = text.replace(/\s+/g, " ").trim();
  if (!normalized) {
    return [""];
  }

  const words = normalized.split(" ");
  const lines: string[] = [];
  let current = "";

  for (const word of words) {
    if (!current) {
      current = word;
      continue;
    }

    const candidate = `${current} ${word}`;
    if (visibleWidth(candidate) <= width) {
      current = candidate;
      continue;
    }

    lines.push(current);
    if (visibleWidth(word) <= width) {
      current = word;
      continue;
    }

    let remaining = word;
    while (visibleWidth(remaining) > width) {
      const segment = truncateText(remaining, width);
      lines.push(segment);
      remaining = [...remaining].slice([...segment].length).join("");
    }
    current = remaining;
  }

  if (current) {
    lines.push(current);
  }

  return lines.length > 0 ? lines : [""];
}

function buildRenderedTerminalLines(
  lines: DerivedTagTerminalLine[],
  width: number,
): Array<{ text: string; tone: DerivedTagTerminalTone; segments?: DerivedTagTerminalSegment[] }> {
  const renderedLines: Array<{ text: string; tone: DerivedTagTerminalTone; segments?: DerivedTagTerminalSegment[] }> =
    [];

  for (const rawLine of lines) {
    const line = normalizeLine(rawLine);
    const indent = " ".repeat(Math.max(0, line.indent));
    const usableWidth = Math.max(1, width - indent.length);

    if (line.segments.length > 0) {
      const segmentsWithIndent = indent
        ? [{ text: indent, tone: "default" as const }, ...line.segments]
        : line.segments;
      const renderedSegments = truncateSegments(
        line.noWrap
          ? segmentsWithIndent
          : [{ text: truncateText(segmentText(segmentsWithIndent), width), tone: line.tone }],
        width,
      );
      renderedLines.push({
        text: segmentText(renderedSegments),
        tone: line.tone,
        segments: renderedSegments,
      });
      continue;
    }

    const wrapped = line.noWrap ? [truncateText(line.text, usableWidth)] : wrapPlainText(line.text, usableWidth);

    for (const segment of wrapped) {
      renderedLines.push({
        text: `${indent}${segment}`,
        tone: line.tone,
      });
    }
  }

  return renderedLines;
}

function renderRows(
  lines: DerivedTagTerminalLine[],
  width: number,
  height: number,
): Array<{ text: string; tone: DerivedTagTerminalTone; segments?: DerivedTagTerminalSegment[] }> {
  const rows = buildRenderedTerminalLines(lines, width);
  const rendered: Array<{ text: string; tone: DerivedTagTerminalTone; segments?: DerivedTagTerminalSegment[] }> = [];
  for (let index = 0; index < height; index += 1) {
    rendered.push(rows[index] ?? { text: "", tone: "default" });
  }
  return rendered;
}

function terminalToneProps(tone: DerivedTagTerminalTone): React.ComponentProps<typeof Text> {
  switch (tone) {
    case "default":
      return {};
    case "heading":
      return { color: "cyan", bold: true };
    case "section":
      return { bold: true };
    case "dim":
      return { dimColor: true };
    case "accent":
      return { color: "cyan" };
    case "success":
      return { color: "green" };
    case "warning":
      return { color: "yellow" };
    case "danger":
      return { color: "red" };
    case "selected":
      return { inverse: true, bold: true };
  }
}

function TerminalRows({
  lines,
  width,
}: {
  lines: Array<{ text: string; tone: DerivedTagTerminalTone; segments?: DerivedTagTerminalSegment[] }>;
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      {lines.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(line.tone)}>
          {line.segments && line.segments.length > 0
            ? line.segments.map((segment, segmentIndex) => (
                <Text key={segmentIndex} {...terminalToneProps(segment.tone ?? "default")}>
                  {segment.text}
                </Text>
              ))
            : fitToWidth(line.text, width)}
        </Text>
      ))}
    </Box>
  );
}

function TerminalHeader({
  title,
  subtitle,
  width,
}: {
  title: string;
  subtitle?: string;
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      <Text wrap="truncate-end" {...terminalToneProps("heading")}>
        {fitToWidth(title, width)}
      </Text>
      {subtitle ? (
        <Text wrap="truncate-end" {...terminalToneProps("accent")}>
          {fitToWidth(subtitle, width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...terminalToneProps("dim")}>
        {fitToWidth("═".repeat(Math.max(0, width)), width)}
      </Text>
    </Box>
  );
}

function TerminalFooter({
  footer,
  width,
}: {
  footer?: DerivedTagTerminalLine[];
  width: number;
}): React.JSX.Element | null {
  if (!footer || footer.length === 0) {
    return null;
  }

  return (
    <Box flexDirection="column" width={width}>
      {footer.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(line.tone ?? "default")}>
          {line.segments && line.segments.length > 0
            ? line.segments.map((segment, segmentIndex) => (
                <Text key={segmentIndex} {...terminalToneProps(segment.tone ?? "default")}>
                  {segment.text}
                </Text>
              ))
            : fitToWidth(line.text, width)}
        </Text>
      ))}
    </Box>
  );
}

function TerminalPaneView({
  pane,
  width,
  height,
}: {
  pane: DerivedTagTerminalPane;
  width: number;
  height: number;
}): React.JSX.Element {
  const bodyHeight = Math.max(0, height - 2);
  const rows = renderRows(pane.lines, width, bodyHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "selected" : "section")}>
        {fitToWidth(pane.title, width)}
      </Text>
      {height > 1 ? (
        <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "accent" : "dim")}>
          {fitToWidth("─".repeat(Math.max(0, width)), width)}
        </Text>
      ) : null}
      {bodyHeight > 0 ? <TerminalRows lines={rows} width={width} /> : null}
    </Box>
  );
}

function TerminalInlinePromptPanel({
  title,
  subtitle,
  body,
  footer,
  width,
  height,
  showTopBorder = true,
}: DerivedTagTerminalInlinePromptPanelProps): React.JSX.Element {
  const footerHeight = footer?.length ?? 0;
  const headerHeight = showTopBorder ? 3 : 2;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      {showTopBorder ? (
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {fitToWidth("─".repeat(Math.max(0, width)), width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...terminalToneProps("selected")}>
        {fitToWidth(title, width)}
      </Text>
      <Text wrap="truncate-end" {...terminalToneProps(subtitle ? "accent" : "dim")}>
        {fitToWidth(subtitle ?? "", width)}
      </Text>
      <Box width={width} height={bodyHeight}>
        {body}
      </Box>
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function useDerivedTagTerminalApp(): DerivedTagTerminalContextValue {
  return ensureTerminalContext();
}

export function useDerivedTagTerminalInput(
  handler: (event: DerivedTagTerminalInputEvent) => void,
  isActive = true,
): void {
  const terminal = ensureTerminalContext();
  useInput(
    (input, key) => {
      if (terminal.modalActive) {
        return;
      }
      const event = createDerivedTagTerminalInputEvent(input, key);
      if (event.systemAction === "interrupt") {
        terminal.exitApp();
        return;
      }
      handler(event);
    },
    { isActive },
  );
}

export function useDerivedTagTerminalSize(): { width: number; height: number } {
  const terminal = ensureTerminalContext();
  return {
    width: terminal.getTerminalWidth(),
    height: terminal.getTerminalHeight(),
  };
}

export function getTerminalPaneBodyHeight(
  sessionOrHeight: number | { height: number },
  options: { hasSubtitle?: boolean; footerLineCount?: number },
): number {
  const height = typeof sessionOrHeight === "number" ? sessionOrHeight : sessionOrHeight.height;
  const headerHeight = options.hasSubtitle ? 3 : 2;
  const footerHeight = options.footerLineCount ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);
  return Math.max(0, contentHeight - 2);
}

export function getTerminalTwoPaneDimensions(
  sessionOrWidth: number | { width: number },
  preferredLeftWidth?: number,
): { leftWidth: number; rightWidth: number; separatorWidth: number } {
  const totalWidth = typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  const separatorWidth = 1;
  const leftWidth = Math.max(
    24,
    Math.min(preferredLeftWidth ?? Math.floor(totalWidth * 0.38), totalWidth - separatorWidth - 20),
  );
  const rightWidth = Math.max(20, totalWidth - leftWidth - separatorWidth);

  return { leftWidth, rightWidth, separatorWidth };
}

export function getTerminalTwoPaneDetailWidth(
  sessionOrWidth: number | { width: number },
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  preferredLeftWidth?: number,
): number {
  if (layoutMode === "detail-only") {
    return typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  }

  return getTerminalTwoPaneDimensions(sessionOrWidth, preferredLeftWidth).rightWidth;
}

export function getTerminalThreePaneDimensions(
  sessionOrWidth: number | { width: number },
  preferredLeftWidth?: number,
  preferredCenterWidth?: number,
): { leftWidth: number; centerWidth: number; rightWidth: number; separatorWidth: number } {
  const totalWidth = typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  const separatorWidth = 1;
  const separatorCount = 2;
  const availableWidth = Math.max(3, totalWidth - separatorWidth * separatorCount);
  const minimumPaneWidth = Math.max(12, Math.floor(availableWidth / 3));
  const clampWidth = (value: number, min: number, max: number): number => Math.max(min, Math.min(value, max));

  const maxLeftWidth = Math.max(minimumPaneWidth, availableWidth - minimumPaneWidth * 2);
  const leftWidth = clampWidth(preferredLeftWidth ?? Math.floor(totalWidth * 0.28), minimumPaneWidth, maxLeftWidth);

  const maxCenterWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - minimumPaneWidth);
  const centerWidth = clampWidth(
    preferredCenterWidth ?? Math.floor(totalWidth * 0.32),
    minimumPaneWidth,
    maxCenterWidth,
  );
  const rightWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - centerWidth);

  return {
    leftWidth,
    centerWidth,
    rightWidth,
    separatorWidth,
  };
}

export function toggleTerminalTwoPaneFocus(activePane: DerivedTagTerminalTwoPaneFocus): DerivedTagTerminalTwoPaneFocus {
  return activePane === "list" ? "detail" : "list";
}

export function normalizeTerminalTwoPaneLayoutMode(
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  activePane: DerivedTagTerminalTwoPaneFocus,
): DerivedTagTerminalTwoPaneLayoutMode {
  return activePane === "detail" ? layoutMode : "split";
}

export function toggleTerminalTwoPaneLayoutMode(
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  activePane: DerivedTagTerminalTwoPaneFocus,
): DerivedTagTerminalTwoPaneLayoutMode {
  if (activePane !== "detail") {
    return "split";
  }
  return layoutMode === "split" ? "detail-only" : "split";
}

export function getRenderedTerminalLineCount(lines: DerivedTagTerminalLine[], width: number): number {
  return buildRenderedTerminalLines(lines, width).length;
}

export function sliceRenderedTerminalLines(
  lines: DerivedTagTerminalLine[],
  width: number,
  start: number,
  count: number,
): DerivedTagTerminalLine[] {
  return buildRenderedTerminalLines(lines, width)
    .slice(start, start + count)
    .map((line) => ({
      text: line.text,
      tone: line.tone,
      noWrap: true,
    }));
}

export function moveSelection(currentIndex: number, delta: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  const baseIndex = Math.max(0, Math.min(currentIndex, itemCount - 1));
  return Math.max(0, Math.min(baseIndex + delta, itemCount - 1));
}

export function moveSelectionWrapped(currentIndex: number, delta: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  const rawIndex = currentIndex + delta;
  return ((rawIndex % itemCount) + itemCount) % itemCount;
}

export type DerivedTagTerminalListNavigationAction =
  | { kind: "move"; delta: number }
  | { kind: "boundary"; boundary: "start" | "end" }
  | { kind: "confirm" }
  | { kind: "cancel" };

export type DerivedTagTerminalListNavigationState = {
  pendingBoundaryPrefix: "g" | null;
};

export type DerivedTagTerminalListNavigationOptions = {
  pageSize: number;
  jumpSize?: number;
  includeConfirmKeys?: boolean;
  includeCancelKeys?: boolean;
  includeHorizontalConfirmKeys?: boolean;
  includeHorizontalCancelKeys?: boolean;
};

export function createDerivedTagTerminalListNavigationState(): DerivedTagTerminalListNavigationState {
  return {
    pendingBoundaryPrefix: null,
  };
}

export function getDerivedTagTerminalListNavigationAction(
  event: DerivedTagTerminalInputEvent,
  options: DerivedTagTerminalListNavigationOptions,
): DerivedTagTerminalListNavigationAction | undefined {
  const jumpSize = options.jumpSize ?? options.pageSize;

  if (event.isMoveUpKey()) {
    return { kind: "move", delta: -1 };
  }
  if (event.isMoveDownKey()) {
    return { kind: "move", delta: 1 };
  }
  if (event.isTerminalJumpBackwardKey()) {
    return { kind: "move", delta: -jumpSize };
  }
  if (event.isTerminalJumpForwardKey()) {
    return { kind: "move", delta: jumpSize };
  }
  if (event.isPageUpKey()) {
    return { kind: "move", delta: -options.pageSize };
  }
  if (event.isPageDownKey()) {
    return { kind: "move", delta: options.pageSize };
  }
  if (event.isTerminalBoundaryStartKey()) {
    return { kind: "boundary", boundary: "start" };
  }
  if (event.isTerminalBoundaryEndKey()) {
    return { kind: "boundary", boundary: "end" };
  }
  if (
    options.includeConfirmKeys &&
    (event.isConfirmKey() || (options.includeHorizontalConfirmKeys && event.isMoveRightKey()))
  ) {
    return { kind: "confirm" };
  }
  if (
    options.includeCancelKeys &&
    (event.isBackNavigationKey() || (options.includeHorizontalCancelKeys && event.isMoveLeftKey()))
  ) {
    return { kind: "cancel" };
  }

  return undefined;
}

export function resolveDerivedTagTerminalListNavigationAction(
  event: DerivedTagTerminalInputEvent,
  options: DerivedTagTerminalListNavigationOptions,
  state: DerivedTagTerminalListNavigationState = createDerivedTagTerminalListNavigationState(),
): {
  action: DerivedTagTerminalListNavigationAction | undefined;
  state: DerivedTagTerminalListNavigationState;
} {
  const clearedState = createDerivedTagTerminalListNavigationState();

  if (event.isExactPrintableKey("G")) {
    return {
      action: { kind: "boundary", boundary: "end" },
      state: clearedState,
    };
  }

  if (event.isExactPrintableKey("g")) {
    if (state.pendingBoundaryPrefix === "g") {
      return {
        action: { kind: "boundary", boundary: "start" },
        state: clearedState,
      };
    }

    return {
      action: undefined,
      state: {
        pendingBoundaryPrefix: "g",
      },
    };
  }

  return {
    action: getDerivedTagTerminalListNavigationAction(event, options),
    state: clearedState,
  };
}

function getNormalizedKeyName(input: string, key: Key): string {
  if (key.upArrow) {
    return "up";
  }
  if (key.downArrow) {
    return "down";
  }
  if (key.leftArrow) {
    return "left";
  }
  if (key.rightArrow) {
    return "right";
  }
  if (key.pageDown) {
    return "page_down";
  }
  if (key.pageUp) {
    return "page_up";
  }
  if (key.home) {
    return "home";
  }
  if (key.end) {
    return "end";
  }
  if (key.return) {
    return "enter";
  }
  if (input === "\u001b[A") {
    return "up";
  }
  if (input === "\u001b[B") {
    return "down";
  }
  if (input === "\u001b[C") {
    return "right";
  }
  if (input === "\u001b[D") {
    return "left";
  }
  if (input === "\u001b[5~") {
    return "page_up";
  }
  if (input === "\u001b[6~") {
    return "page_down";
  }
  if (input === "\u001b[H" || input === "\u001bOH") {
    return "home";
  }
  if (input === "\u001b[F" || input === "\u001bOF") {
    return "end";
  }
  if (key.escape) {
    return "escape";
  }
  if (key.tab && key.shift) {
    return "shift_tab";
  }
  if (key.tab) {
    return "tab";
  }
  if (key.backspace) {
    return "backspace";
  }
  if (key.delete) {
    return "delete";
  }
  if (input === "\r" || input === "\n") {
    return "enter";
  }
  if (input === "\u001b") {
    return "escape";
  }
  if (input === "\b" || input === "\u007f") {
    return "backspace";
  }
  if (input.length === 1) {
    const code = input.codePointAt(0);
    if (code !== undefined && code >= 1 && code <= 26) {
      return `ctrl_${String.fromCodePoint(code + 96)}`;
    }
  }
  if (key.ctrl && input.length === 1) {
    const lowerInput = input.toLowerCase();
    if (lowerInput >= "a" && lowerInput <= "z") {
      return `ctrl_${lowerInput}`;
    }
  }
  if (input === " ") {
    return "space";
  }
  if (input === "/") {
    return "slash";
  }
  return input.toLowerCase();
}

function getPrintableInput(input: string, key: Key): string | undefined {
  if (key.ctrl || key.meta || key.escape || key.return || key.tab || input.length !== 1) {
    return undefined;
  }
  return input;
}

function isExactPrintableTerminalKey(input: string, key: Key, expected: string): boolean {
  return expected.length === 1 && getPrintableInput(input, key) === expected;
}

export function createDerivedTagTerminalInputEvent(input: string, key: Key): DerivedTagTerminalInputEvent {
  const normalized = getNormalizedKeyName(input, key);
  const printable = getPrintableInput(input, key);

  return {
    input,
    key,
    printable,
    systemAction: normalized === "ctrl_c" ? "interrupt" : undefined,
    textInputAction:
      normalized === "enter"
        ? "submit"
        : normalized === "escape"
          ? "cancel"
          : normalized === "backspace"
            ? "deleteBackward"
            : undefined,
    isBackNavigationKey: () => isBackNavigationKey(normalized),
    isCommandPaletteKey: () => normalized === ":",
    isConfirmKey: () => isConfirmKey(normalized),
    isConfirmOrToggleKey: () => normalized === "enter" || normalized === "space",
    isExactPrintableKey: (expected) => isExactPrintableTerminalKey(input, key, expected),
    isExecuteKey: () => normalized === "tab" || normalized === "shift_tab",
    isFocusToggleKey: () => normalized === "tab" || normalized === "shift_tab" || normalized === "w",
    isHelpKey: () => normalized === "?",
    isLayoutToggleKey: () => normalized === "z",
    isMoveDownKey: () => isMoveDownKey(normalized),
    isMoveLeftKey: () => isMoveLeftKey(normalized),
    isMoveRightKey: () => isMoveRightKey(normalized),
    isMoveUpKey: () => isMoveUpKey(normalized),
    isPageDownKey: () => isPageDownKey(normalized),
    isPageUpKey: () => isPageUpKey(normalized),
    isSearchKey: () => normalized === "slash",
    isTerminalBoundaryEndKey: () => normalized === "end",
    isTerminalBoundaryStartKey: () => normalized === "home",
    isTerminalJumpBackwardKey: () => normalized === "ctrl_u",
    isTerminalJumpForwardKey: () => normalized === "ctrl_d",
    isTerminalQuitKey: () => normalized === "q",
    getCycleDirection: () => (normalized === "enter" || normalized === "space" ? 1 : undefined),
    getReverseCycleDirection: () => undefined,
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

function buildSelectModalOptions<T>(options: SelectPromptOptions<T>): TerminalSelectModalOptions {
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

function buildOptionalSelectModalOptions<T>(options: OptionalSelectPromptOptions<T>): TerminalSelectModalOptions {
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

function getSelectPromptInitialIndex(entries: TerminalSelectModalEntry[], selectedValue: unknown): number {
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

function getFirstEnabledCommandIndex(entries: DerivedTagTerminalCommandOption<string>[]): number {
  const enabledIndex = entries.findIndex((entry) => !entry.disabled);
  return enabledIndex >= 0 ? enabledIndex : 0;
}

function buildCommandPaletteDetailLines(
  option: DerivedTagTerminalCommandOption<string> | undefined,
  filterText: string,
): DerivedTagTerminalLine[] {
  const lines =
    option?.detailLines ?? [
      { text: option?.label ?? "(none)", tone: "section" as const },
      { text: option?.description ?? "No additional details." },
      ...(option?.aliases?.length
        ? [{ text: `Aliases: ${option.aliases.join(", ")}`, tone: "accent" as const }]
        : []),
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
        footer={[{ text: "Type to filter  Backspace edit  Esc cancel", tone: "dim" }]}
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
            tone:
              windowStart + offset === clampedSelectedIndex ? "selected" : entry.disabled ? "dim" : "default",
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
        footer={[{ text: "Esc/backspace/left/q cancel", tone: "dim" }]}
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

function createEmptyPolicySelection<T extends string>(): DerivedTagTerminalPolicySelection<T> {
  return {
    any: [],
    all: [],
    exclude: [],
  };
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

function createValueStateLookup(
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

function createChoicePromptDescriptor(
  terminalWidth: number,
  itemCount: number,
  detailLines: DerivedTagTerminalLine[],
) {
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

function planTerminalModalStateLayout(
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

function DerivedTagTerminalModalHost({
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
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const pageSize = Math.max(1, layout.visibleListCapacity || 10);

  useInput(
    (input, key) => {
      const event = createDerivedTagTerminalInputEvent(input, key);
      if (event.systemAction === "interrupt") {
        exitApp();
        return;
      }
      const selectLikeAction = resolveTerminalInteractionAction(event, [
        { id: "select" },
        { id: "back", label: "cancel" },
      ]);
      const multiSelectLikeAction = resolveTerminalInteractionAction(event, [{ id: "toggle" }, { id: "return" }]);
      const policyLikeAction = resolveTerminalInteractionAction(event, [
        { id: "cycle" },
        { id: "cycleReverse" },
        { id: "return" },
      ]);
      const modalNavigation = resolveDerivedTagTerminalListNavigationAction(
        event,
        {
          pageSize,
          jumpSize: 5,
          includeCancelKeys: true,
          includeHorizontalCancelKeys: true,
        },
        listNavigationStateRef.current,
      );
      listNavigationStateRef.current = modalNavigation.state;

      if (!modal) {
        listNavigationStateRef.current = createDerivedTagTerminalListNavigationState();
        return;
      }

      if (modal.kind === "dialog") {
        const resolver = modal.resolve;
        setModal(null);
        resolver();
        return;
      }

      if (modal.kind === "text") {
        if (event.textInputAction === "submit") {
          const resolver = modal.resolve;
          const trimmed = modal.value.trim();
          setModal(null);
          resolver(trimmed ? trimmed : undefined);
          return;
        }
        if (event.textInputAction === "cancel") {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
          return;
        }
        if (event.textInputAction === "deleteBackward") {
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: [...current.value].slice(0, -1).join("") } : current,
          );
          return;
        }
        if (event.printable) {
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: current.value + event.printable } : current,
          );
        }
        return;
      }

      const modalNavigationAction = modalNavigation.action;

      if (modal.kind === "command") {
        const filteredEntries = filterCommandPaletteEntries(modal.options.entries, modal.filterText);
        const clampedSelectedIndex = clampPromptSelectionIndex(modal.selectedIndex, filteredEntries.length);

        if (event.textInputAction === "deleteBackward") {
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
                    filterCommandPaletteEntries(
                      current.options.entries,
                      [...current.filterText].slice(0, -1).join(""),
                    ),
                  ),
                }
              : current,
          );
          return;
        }
        if (event.printable) {
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  filterText: current.filterText + event.printable,
                  selectedIndex: getFirstEnabledCommandIndex(
                    filterCommandPaletteEntries(current.options.entries, current.filterText + event.printable),
                  ),
                }
              : current,
          );
          return;
        }
        if (modalNavigationAction?.kind === "move") {
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex: moveSelectionWrapped(
                    clampedSelectedIndex,
                    modalNavigationAction.delta,
                    filteredEntries.length,
                  ),
                }
              : current,
          );
          return;
        }
        if (modalNavigationAction?.kind === "boundary") {
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex:
                    modalNavigationAction.boundary === "start" ? 0 : Math.max(0, filteredEntries.length - 1),
                }
              : current,
          );
          return;
        }
        if (selectLikeAction?.id === "select") {
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
        if (selectLikeAction?.id === "back" || event.isTerminalQuitKey()) {
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

      if (modalNavigationAction?.kind === "move") {
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex: moveSelectionWrapped(
                  current.selectedIndex,
                  modalNavigationAction.delta,
                  current.options.entries.length,
                ),
              }
            : current,
        );
        return;
      }
      if (modalNavigationAction?.kind === "boundary") {
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex:
                  modalNavigationAction.boundary === "start" ? 0 : Math.max(0, current.options.entries.length - 1),
              }
            : current,
        );
        return;
      }
      if (modal.kind === "multiselect" && multiSelectLikeAction?.id === "toggle") {
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
      if (modal.kind === "select" && selectLikeAction?.id === "select") {
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
      if (modal.kind === "multiselect" && multiSelectLikeAction?.id === "return") {
        const resolver = modal.resolve;
        const selectedValues = modal.selectedValues;
        setModal(null);
        resolver(selectedValues);
        return;
      }
      const cycleDirection = getTerminalInteractionCycleDirection(event, policyLikeAction);

      if (modal.kind === "policy" && cycleDirection) {
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
                    cycleDirection,
                  ),
                },
              }
            : current,
        );
        return;
      }
      if (
        modal.kind === "policy" &&
        (policyLikeAction?.id === "return" || event.isTerminalQuitKey())
      ) {
        const resolver = modal.resolve;
        const selection = buildPolicySelection(modal.options.entries, modal.valueStates);
        setModal(null);
        resolver(selection);
        return;
      }
      if (
        modal.kind === "select" &&
        (selectLikeAction?.id === "back" || event.isTerminalQuitKey())
      ) {
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

export function DerivedTagTerminalProvider({ children }: { children: React.ReactNode }): React.JSX.Element {
  const { exit } = useApp();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);
  const modalLayout = React.useMemo(() => planTerminalModalStateLayout(modal, columns, rows), [columns, modal, rows]);
  const availableRows =
    modalLayout?.presentation === "inline" ? Math.max(0, rows - modalLayout.totalHeight) : modalLayout ? 0 : rows;

  const contextValue = React.useMemo<DerivedTagTerminalContextValue>(
    () => ({
      exitApp: exit,
      getTerminalHeight: () => availableRows,
      getTerminalWidth: () => columns,
      modalActive: modal !== null,
      pauseForAnyKey: async (message: string) => {
        await new Promise<void>((resolve) => {
          setModal({
            kind: "dialog",
            options: {
              title: "Derived-Tag Workbench",
              body: message.split("\n").map((line) => ({ text: line })),
              footer: [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }],
            },
            resolve,
          });
        });
      },
      promptCommandPalette: async <T extends string>(options: CommandPaletteOptions<T>) =>
        new Promise<T | undefined>((resolve) => {
          const normalizedOptions = options as CommandPaletteOptions<string>;
          setModal({
            kind: "command",
            options: normalizedOptions,
            filterText: "",
            selectedIndex: getFirstEnabledCommandIndex(normalizedOptions.entries),
            resolve: resolve as (value: string | undefined) => void,
          });
        }),
      promptOptionalSelectOption: async <T,>(options: OptionalSelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalOptionalSelectPromptResult<T>>((resolve) => {
          const modalOptions = buildOptionalSelectModalOptions(options);
          setModal({
            kind: "select",
            options: modalOptions,
            selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
            resolve: resolve as (
              value:
                | DerivedTagTerminalSelectPromptResult<unknown>
                | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
            ) => void,
          });
        }),
      promptPolicySelectOption: async <T extends string>(options: PolicyPromptOptions<T>) =>
        new Promise<DerivedTagTerminalPolicySelection<T>>((resolve) => {
          const initialSelection = createEmptyPolicySelection<string>();
          initialSelection.any = options.selectedValues?.any ? [...options.selectedValues.any] : [];
          initialSelection.all = options.selectedValues?.all ? [...options.selectedValues.all] : [];
          initialSelection.exclude = options.selectedValues?.exclude ? [...options.selectedValues.exclude] : [];
          const valueStates = createValueStateLookup(initialSelection);
          const selectedIndex = Math.max(
            0,
            options.entries.findIndex((entry) => valueStates[entry.value] !== undefined),
          );
          setModal({
            kind: "policy",
            options: options as PolicyPromptOptions<string>,
            selectedIndex,
            valueStates,
            resolve: resolve as (value: DerivedTagTerminalPolicySelection<string>) => void,
          });
        }),
      promptMultiSelectOption: async <T extends string>(options: MultiSelectPromptOptions<T>) =>
        new Promise<T[]>((resolve) => {
          const selectedIndex = Math.max(
            0,
            options.entries.findIndex((entry) => options.selectedValues?.includes(entry.value)),
          );
          setModal({
            kind: "multiselect",
            options: options as MultiSelectPromptOptions<string>,
            selectedIndex,
            selectedValues: options.selectedValues ? [...options.selectedValues] : [],
            resolve: resolve as (value: string[]) => void,
          });
        }),
      promptSelectOption: async <T,>(options: SelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalSelectPromptResult<T>>((resolve) => {
          const modalOptions = buildSelectModalOptions(options);
          setModal({
            kind: "select",
            options: modalOptions,
            selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
            resolve: resolve as (
              value:
                | DerivedTagTerminalSelectPromptResult<unknown>
                | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
            ) => void,
          });
        }),
      promptTextInput: async (options: TextPromptOptions) =>
        new Promise<string | undefined>((resolve) => {
          setModal({
            kind: "text",
            options,
            value: options.defaultValue ?? "",
            resolve,
          });
        }),
      showDialog: async (options: DialogOptions) =>
        new Promise<void>((resolve) => {
          setModal({
            kind: "dialog",
            options,
            resolve,
          });
        }),
    }),
    [availableRows, columns, exit, modal],
  );

  return (
    <DerivedTagTerminalContext.Provider value={contextValue}>
      {modal && modalLayout?.presentation === "screen" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          exitApp={exit}
          width={columns}
          layout={modalLayout}
        />
      ) : null}
      <Box flexDirection="column">{children}</Box>
      {modal && modalLayout?.presentation === "inline" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          exitApp={exit}
          width={columns}
          layout={modalLayout}
        />
      ) : null}
    </DerivedTagTerminalContext.Provider>
  );
}

export function TerminalTextScreen({
  title,
  subtitle,
  body,
  footer,
}: DerivedTagTerminalTextScreenProps): React.JSX.Element {
  const { width, height } = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);
  const rows = renderRows(body, width, bodyHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <TerminalHeader title={title} subtitle={subtitle} width={width} />
      <TerminalRows lines={rows} width={width} />
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function TerminalPaneScreen({
  title,
  subtitle,
  pane,
  footer,
}: DerivedTagTerminalPaneScreenProps): React.JSX.Element {
  const { width, height } = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <TerminalHeader title={title} subtitle={subtitle} width={width} />
      <TerminalPaneView pane={pane} width={width} height={contentHeight} />
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function TerminalTwoPaneScreen({
  title,
  subtitle,
  left,
  right,
  footer,
  leftWidth,
}: DerivedTagTerminalTwoPaneScreenProps): React.JSX.Element {
  const size = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, size.height - headerHeight - footerHeight);
  const dimensions = getTerminalTwoPaneDimensions(size.width, leftWidth);
  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");

  return (
    <Box flexDirection="column" width={size.width} height={size.height}>
      <TerminalHeader title={title} subtitle={subtitle} width={size.width} />
      <Box flexDirection="row" width={size.width} height={contentHeight}>
        <TerminalPaneView pane={left} width={dimensions.leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

export function TerminalThreePaneScreen({
  title,
  subtitle,
  left,
  center,
  right,
  footer,
  leftWidth,
  centerWidth,
}: DerivedTagTerminalThreePaneScreenProps): React.JSX.Element {
  const size = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, size.height - headerHeight - footerHeight);
  const dimensions = getTerminalThreePaneDimensions(size.width, leftWidth, centerWidth);
  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");

  return (
    <Box flexDirection="column" width={size.width} height={size.height}>
      <TerminalHeader title={title} subtitle={subtitle} width={size.width} />
      <Box flexDirection="row" width={size.width} height={contentHeight}>
        <TerminalPaneView pane={left} width={dimensions.leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={center} width={dimensions.centerWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

export async function runDerivedTagTerminalApp(node: React.ReactElement): Promise<void> {
  const instance = renderInkApp(<DerivedTagTerminalProvider>{node}</DerivedTagTerminalProvider>, {
    alternateScreen: true,
    exitOnCtrlC: false,
    patchConsole: true,
  });

  try {
    await instance.waitUntilExit();
  } finally {
    instance.cleanup();
  }
}
