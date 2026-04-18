import React from "react";
import {
  Box,
  Text,
  render as renderInkApp,
  useApp,
  useInput,
  useWindowSize,
  type Key,
} from "ink";
import {
  isBackNavigationKey,
  isConfirmKey,
  isConfirmOrToggleKey,
  isMoveDownKey,
  isMoveLeftKey,
  isMoveRightKey,
  isMoveUpKey,
  isPageDownKey,
  isPageUpKey,
} from "./keymap.js";
import { formatTerminalInteractionFooter } from "./interaction-bindings.js";

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

export type DerivedTagTerminalLine = {
  text: string;
  tone?: DerivedTagTerminalTone;
  indent?: number;
  noWrap?: boolean;
};

export type DerivedTagTerminalPane = {
  title: string;
  lines: DerivedTagTerminalLine[];
  active?: boolean;
};

export type DerivedTagTerminalSelectOption<T extends string = string> = {
  value: T;
  label: string;
  description?: string;
  detailLines?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalCommandOption<T extends string = string> = DerivedTagTerminalSelectOption<T> & {
  aliases?: string[];
  keywords?: string[];
};

export type DerivedTagTerminalTwoPaneFocus = "list" | "detail";
export type DerivedTagTerminalTwoPaneLayoutMode = "split" | "detail-only";

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

type TerminalModalPresentation = "inline" | "screen";

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

type SelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T;
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
    options: SelectPromptOptions<string>;
    selectedIndex: number;
    resolve: (value: string | undefined) => void;
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
  promptPolicySelectOption: <T extends string>(options: PolicyPromptOptions<T>) => Promise<DerivedTagTerminalPolicySelection<T>>;
  promptMultiSelectOption: <T extends string>(options: MultiSelectPromptOptions<T>) => Promise<T[]>;
  promptSelectOption: <T extends string>(options: SelectPromptOptions<T>) => Promise<T | undefined>;
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
    tone: line.tone ?? "default",
    indent: line.indent ?? 0,
    noWrap: line.noWrap ?? false,
  };
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
): Array<{ text: string; tone: DerivedTagTerminalTone }> {
  const renderedLines: Array<{ text: string; tone: DerivedTagTerminalTone }> = [];

  for (const rawLine of lines) {
    const line = normalizeLine(rawLine);
    const indent = " ".repeat(Math.max(0, line.indent));
    const usableWidth = Math.max(1, width - indent.length);
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
): Array<{ text: string; tone: DerivedTagTerminalTone }> {
  const rows = buildRenderedTerminalLines(lines, width);
  const rendered: Array<{ text: string; tone: DerivedTagTerminalTone }> = [];
  for (let index = 0; index < height; index += 1) {
    rendered.push(rows[index] ?? { text: "", tone: "default" });
  }
  return rendered;
}

function terminalToneProps(tone: DerivedTagTerminalTone): React.ComponentProps<typeof Text> {
  switch (tone) {
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
    default:
      return {};
  }
}

function TerminalRows({
  lines,
  width,
}: {
  lines: Array<{ text: string; tone: DerivedTagTerminalTone }>;
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      {lines.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(line.tone)}>
          {fitToWidth(line.text, width)}
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
          {fitToWidth(line.text, width)}
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

function getModalPresentation(modal: TerminalModalState): TerminalModalPresentation | null {
  if (!modal) {
    return null;
  }

  switch (modal.kind) {
    case "dialog":
      return modal.options.presentation ?? "inline";
    case "text":
      return modal.options.presentation ?? "inline";
    case "command":
      return modal.options.presentation ?? "inline";
    case "select":
      return modal.options.presentation ?? "screen";
    case "multiselect":
      return modal.options.presentation ?? "screen";
    case "policy":
      return modal.options.presentation ?? "screen";
    default:
      return "screen";
  }
}

export function useDerivedTagTerminalApp(): DerivedTagTerminalContextValue {
  return ensureTerminalContext();
}

export function useDerivedTagTerminalInput(
  handler: (input: string, key: Key) => void,
  isActive = true,
): void {
  const terminal = ensureTerminalContext();
  useInput((input, key) => {
    if (terminal.modalActive) {
      return;
    }
    handler(input, key);
  }, { isActive });
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
  const leftWidth = Math.max(24, Math.min(preferredLeftWidth ?? Math.floor(totalWidth * 0.38), totalWidth - separatorWidth - 20));
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
  const availableWidth = Math.max(3, totalWidth - (separatorWidth * separatorCount));
  const minimumPaneWidth = Math.max(12, Math.floor(availableWidth / 3));
  const clampWidth = (value: number, min: number, max: number): number => Math.max(min, Math.min(value, max));

  const maxLeftWidth = Math.max(minimumPaneWidth, availableWidth - (minimumPaneWidth * 2));
  const leftWidth = clampWidth(preferredLeftWidth ?? Math.floor(totalWidth * 0.28), minimumPaneWidth, maxLeftWidth);

  const maxCenterWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - minimumPaneWidth);
  const centerWidth = clampWidth(preferredCenterWidth ?? Math.floor(totalWidth * 0.32), minimumPaneWidth, maxCenterWidth);
  const rightWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - centerWidth);

  return {
    leftWidth,
    centerWidth,
    rightWidth,
    separatorWidth,
  };
}

export function toggleTerminalTwoPaneFocus(
  activePane: DerivedTagTerminalTwoPaneFocus,
): DerivedTagTerminalTwoPaneFocus {
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

export function getRenderedTerminalLineCount(
  lines: DerivedTagTerminalLine[],
  width: number,
): number {
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
  normalizedKey: string,
  options: DerivedTagTerminalListNavigationOptions,
): DerivedTagTerminalListNavigationAction | undefined {
  const jumpSize = options.jumpSize ?? options.pageSize;

  if (isMoveUpKey(normalizedKey)) {
    return { kind: "move", delta: -1 };
  }
  if (isMoveDownKey(normalizedKey)) {
    return { kind: "move", delta: 1 };
  }
  if (normalizedKey === "ctrl_u") {
    return { kind: "move", delta: -jumpSize };
  }
  if (normalizedKey === "ctrl_d") {
    return { kind: "move", delta: jumpSize };
  }
  if (isPageUpKey(normalizedKey)) {
    return { kind: "move", delta: -options.pageSize };
  }
  if (isPageDownKey(normalizedKey)) {
    return { kind: "move", delta: options.pageSize };
  }
  if (normalizedKey === "home") {
    return { kind: "boundary", boundary: "start" };
  }
  if (normalizedKey === "end") {
    return { kind: "boundary", boundary: "end" };
  }
  if (
    options.includeConfirmKeys &&
    (
      isConfirmKey(normalizedKey) ||
      (options.includeHorizontalConfirmKeys && isMoveRightKey(normalizedKey))
    )
  ) {
    return { kind: "confirm" };
  }
  if (
    options.includeCancelKeys &&
    (
      isBackNavigationKey(normalizedKey) ||
      (options.includeHorizontalCancelKeys && isMoveLeftKey(normalizedKey))
    )
  ) {
    return { kind: "cancel" };
  }

  return undefined;
}

export function resolveDerivedTagTerminalListNavigationAction(
  input: string,
  key: Key,
  options: DerivedTagTerminalListNavigationOptions,
  state: DerivedTagTerminalListNavigationState = createDerivedTagTerminalListNavigationState(),
): {
  action: DerivedTagTerminalListNavigationAction | undefined;
  state: DerivedTagTerminalListNavigationState;
} {
  const normalized = getNormalizedKeyName(input, key);
  const clearedState = createDerivedTagTerminalListNavigationState();

  if (normalized === "g") {
    if (isExactPrintableTerminalKey(input, key, "G")) {
      return {
        action: { kind: "boundary", boundary: "end" },
        state: clearedState,
      };
    }

    if (isExactPrintableTerminalKey(input, key, "g")) {
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
  }

  return {
    action: getDerivedTagTerminalListNavigationAction(normalized, options),
    state: clearedState,
  };
}

export function getNormalizedKeyName(input: string, key: Key): string {
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

export function getPrintableInput(input: string, key: Key): string | undefined {
  if (key.ctrl || key.meta || key.escape || key.return || key.tab || input.length !== 1) {
    return undefined;
  }
  return input;
}

export function isExactPrintableTerminalKey(input: string, key: Key, expected: string): boolean {
  return expected.length === 1 && getPrintableInput(input, key) === expected;
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

function buildPromptDetailLines(
  option: DerivedTagTerminalSelectOption<string> | undefined,
): DerivedTagTerminalLine[] {
  if (option?.detailLines?.length) {
    return option.detailLines;
  }
  if (option?.description) {
    return [
      { text: option.label, tone: "section" },
      { text: option.description },
    ];
  }

  return [
    { text: option?.label ?? "(none)", tone: "section" },
    { text: "No additional details.", tone: "dim" },
  ];
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
    const searchableText = [
      entry.label,
      entry.description ?? "",
      ...(entry.aliases ?? []),
      ...(entry.keywords ?? []),
    ]
      .join(" ")
      .toLowerCase();

    return normalizedTerms.every((term) => searchableText.includes(term));
  });
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

function InlinePromptTwoPaneBody({
  prompt,
  entries,
  detailLines,
  focusedLabel,
  width,
  height,
}: {
  prompt: string;
  entries: DerivedTagTerminalLine[];
  detailLines: DerivedTagTerminalLine[];
  focusedLabel: string;
  width: number;
  height: number;
}): React.JSX.Element {
  const dimensions = getTerminalTwoPaneDimensions(width, 38);
  const separator = Array.from({ length: Math.max(1, height) }, () => "│").join("\n");

  return (
    <Box flexDirection="row" width={width} height={height}>
      <TerminalPaneView
        pane={{
          title: prompt,
          lines: entries,
          active: true,
        }}
        width={dimensions.leftWidth}
        height={height}
      />
      <Text wrap="truncate-end" {...terminalToneProps("dim")}>{separator}</Text>
      <TerminalPaneView
        pane={{
          title: focusedLabel,
          lines: detailLines,
        }}
        width={dimensions.rightWidth}
        height={height}
      />
    </Box>
  );
}

function PromptBody({
  options,
  currentValue,
  width,
  height,
}: {
  options: TextPromptOptions;
  currentValue: string;
  width: number;
  height: number;
}): React.JSX.Element {
  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.prompt}
      body={(
        <InlinePromptMessageBody
          width={width}
          height={Math.max(0, height - 4)}
          lines={[
            ...(options.hint ? [{ text: options.hint, tone: "accent" as const }] : []),
            { text: `> ${currentValue || ""}`, tone: "selected" },
            { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
          ]}
        />
      )}
      footer={[{ text: "Type text  Enter submit  Backspace edit  Esc cancel", tone: "dim" }]}
      width={width}
      height={height}
      showTopBorder={options.presentation !== "screen"}
    />
  );
}

function CommandPaletteBody({
  options,
  filterText,
  selectedIndex,
  width,
  height,
}: {
  options: CommandPaletteOptions<string>;
  filterText: string;
  selectedIndex: number;
  width: number;
  height: number;
}): React.JSX.Element {
  const filteredEntries = filterCommandPaletteEntries(options.entries, filterText);
  const clampedSelectedIndex = clampPromptSelectionIndex(selectedIndex, filteredEntries.length);

  if (filteredEntries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={(
          <InlinePromptMessageBody
            width={width}
            height={Math.max(0, height - 4)}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: `Filter: ${filterText || "(none)"}`, tone: "accent" },
              { text: "No commands match the current filter.", tone: "warning" },
            ]}
          />
        )}
        footer={[
          { text: "Type to filter  Backspace edit  Esc cancel", tone: "dim" },
        ]}
        width={width}
        height={height}
        showTopBorder={options.presentation !== "screen"}
      />
    );
  }

  const selectedOption = filteredEntries[clampedSelectedIndex];
  const contentHeight = Math.max(1, height - 6);
  const visibleCount = Math.max(1, contentHeight - 2);
  const windowStart = clampInlinePromptWindowStart(clampedSelectedIndex, filteredEntries.length, visibleCount);
  const visibleEntries = filteredEntries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle ?? options.prompt}
      body={(
        <InlinePromptTwoPaneBody
          prompt={filterText ? `Filter: ${filterText}` : options.prompt}
          entries={visibleEntries.map((entry, offset) => ({
            text: entry.label,
            tone: windowStart + offset === clampedSelectedIndex ? "selected" : "default",
            noWrap: true,
          }))}
          detailLines={[
            ...(selectedOption?.detailLines ?? [
              { text: selectedOption?.label ?? "(none)", tone: "section" },
              { text: selectedOption?.description ?? "No additional details." },
              ...(selectedOption?.aliases?.length
                ? [{ text: `Aliases: ${selectedOption.aliases.join(", ")}`, tone: "accent" as const }]
                : []),
            ]),
            { text: "" },
            { text: `Filter: ${filterText || "(none)"}`, tone: "accent" },
          ]}
          focusedLabel={`Command ${clampedSelectedIndex + 1}/${filteredEntries.length}`}
          width={width}
          height={contentHeight}
        />
      )}
      footer={[
        { text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]), tone: "dim" },
        { text: "Type to filter  Enter/Right select  Backspace edit  Esc cancel", tone: "dim" },
        { text: `${filteredEntries.length} command${filteredEntries.length === 1 ? "" : "s"} visible`, tone: "accent" },
      ]}
      width={width}
      height={height}
      showTopBorder={options.presentation !== "screen"}
    />
  );
}

function SelectPromptBody({
  options,
  selectedIndex,
  width,
  height,
}: {
  options: SelectPromptOptions<string>;
  selectedIndex: number;
  width: number;
  height: number;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={(
          <InlinePromptMessageBody
            width={width}
            height={Math.max(0, height - 4)}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        )}
        footer={[{ text: "Esc/backspace/left/q cancel", tone: "dim" }]}
        width={width}
        height={height}
        showTopBorder={options.presentation !== "screen"}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const contentHeight = Math.max(1, height - 6);
  const visibleCount = Math.max(1, contentHeight - 2);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={(
        <InlinePromptTwoPaneBody
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
          height={contentHeight}
        />
      )}
      footer={[
        { text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]), tone: "dim" },
        { text: formatTerminalInteractionFooter([{ id: "select" }, { id: "back", label: "cancel" }]), tone: "dim" },
        { text: `${selectedIndex + 1}/${options.entries.length} focused`, tone: "accent" },
      ]}
      width={width}
      height={height}
      showTopBorder={options.presentation !== "screen"}
    />
  );
}

function MultiSelectPromptBody({
  options,
  selectedIndex,
  selectedValues,
  width,
  height,
}: {
  options: MultiSelectPromptOptions<string>;
  selectedIndex: number;
  selectedValues: string[];
  width: number;
  height: number;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={(
          <InlinePromptMessageBody
            width={width}
            height={Math.max(0, height - 4)}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        )}
        footer={[{ text: formatTerminalInteractionFooter([{ id: "back", label: "return" }]), tone: "dim" }]}
        width={width}
        height={height}
        showTopBorder={options.presentation !== "screen"}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const selectedSet = new Set(selectedValues);
  const selectedLabels = options.entries
    .filter((entry) => selectedSet.has(entry.value))
    .map((entry) => entry.label);
  const contentHeight = Math.max(1, height - 6);
  const visibleCount = Math.max(1, contentHeight - 2);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={(
        <InlinePromptTwoPaneBody
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
          height={contentHeight}
        />
      )}
      footer={[
        { text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]), tone: "dim" },
        { text: formatTerminalInteractionFooter([{ id: "toggle" }, { id: "return" }]), tone: "dim" },
        { text: `${selectedValues.length} selected | Focused: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]}
      width={width}
      height={height}
      showTopBorder={options.presentation !== "screen"}
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
    case "any":
      return "ANY";
    case "all":
      return "ALL";
    case "exclude":
      return "NOT";
    default:
      return " ";
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
): DerivedTagTerminalPolicyState | undefined {
  const stateOrder: Array<DerivedTagTerminalPolicyState | undefined> = [undefined, ...allowedStates];
  const currentIndex = stateOrder.findIndex((state) => state === currentState);
  const nextIndex = (currentIndex + 1) % stateOrder.length;
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
  height,
}: {
  options: PolicyPromptOptions<string>;
  selectedIndex: number;
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>;
  width: number;
  height: number;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalInlinePromptPanel
        title={options.title}
        subtitle={options.subtitle ?? options.prompt}
        body={(
          <InlinePromptMessageBody
            width={width}
            height={Math.max(0, height - 4)}
            lines={[
              { text: options.prompt, tone: "section" },
              { text: "No options are available for this scope.", tone: "warning" },
            ]}
          />
        )}
        footer={[{ text: formatTerminalInteractionFooter([{ id: "return" }]), tone: "dim" }]}
        width={width}
        height={height}
        showTopBorder={options.presentation !== "screen"}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const selectedState = selectedOption ? getPolicyStateForValue(selectedOption.value, valueStates) : undefined;
  const contentHeight = Math.max(1, height - 6);
  const visibleCount = Math.max(1, contentHeight - 2);
  const windowStart = clampInlinePromptWindowStart(selectedIndex, options.entries.length, visibleCount);
  const visibleEntries = options.entries.slice(windowStart, windowStart + visibleCount);

  return (
    <TerminalInlinePromptPanel
      title={options.title}
      subtitle={options.subtitle}
      body={(
        <InlinePromptTwoPaneBody
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
          height={contentHeight}
        />
      )}
      footer={[
        { text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }]), tone: "dim" },
        { text: formatTerminalInteractionFooter([{ id: "cycle" }, { id: "return" }]), tone: "dim" },
        { text: `Cycle order: off -> ${options.allowedStates.join(" -> ")} -> off`, tone: "accent" },
      ]}
      width={width}
      height={height}
      showTopBorder={options.presentation !== "screen"}
    />
  );
}

function getInlineModalHeight(modal: TerminalModalState, totalRows: number): number {
  if (!modal || getModalPresentation(modal) !== "inline") {
    return 0;
  }

  const minimumMainHeight = 10;
  const maximumInlineHeight = Math.max(4, totalRows - minimumMainHeight);
  const desiredHeight = modal.kind === "text" || modal.kind === "dialog" ? 6 : 10;
  return Math.max(4, Math.min(desiredHeight, maximumInlineHeight));
}

function DerivedTagTerminalModalHost({
  modal,
  setModal,
  width,
  height,
}: {
  modal: TerminalModalState;
  setModal: React.Dispatch<React.SetStateAction<TerminalModalState>>;
  width: number;
  height: number;
}): React.JSX.Element | null {
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const presentation = getModalPresentation(modal);

  useInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const printable = getPrintableInput(input, key);
    const modalNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: 10,
      jumpSize: 5,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    }, listNavigationStateRef.current);
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
      if (normalized === "enter") {
        const resolver = modal.resolve;
        const trimmed = modal.value.trim();
        setModal(null);
        resolver(trimmed ? trimmed : undefined);
        return;
      }
      if (normalized === "escape") {
        const resolver = modal.resolve;
        setModal(null);
        resolver(undefined);
        return;
      }
      if (normalized === "backspace") {
        setModal((current) => current?.kind === "text"
          ? { ...current, value: [...current.value].slice(0, -1).join("") }
          : current);
        return;
      }
      if (printable) {
        setModal((current) => current?.kind === "text"
          ? { ...current, value: current.value + printable }
          : current);
      }
      return;
    }

    const modalNavigationAction = modalNavigation.action;

    if (modal.kind === "command") {
      const filteredEntries = filterCommandPaletteEntries(modal.options.entries, modal.filterText);
      const clampedSelectedIndex = clampPromptSelectionIndex(modal.selectedIndex, filteredEntries.length);

      if (normalized === "backspace") {
        if (modal.filterText.length === 0) {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
          return;
        }
        setModal((current) => current?.kind === "command"
          ? {
            ...current,
            filterText: [...current.filterText].slice(0, -1).join(""),
            selectedIndex: 0,
          }
          : current);
        return;
      }
      if (printable) {
        setModal((current) => current?.kind === "command"
          ? {
            ...current,
            filterText: current.filterText + printable,
            selectedIndex: 0,
          }
          : current);
        return;
      }
      if (modalNavigationAction?.kind === "move") {
        setModal((current) => current?.kind === "command"
          ? {
            ...current,
            selectedIndex: moveSelectionWrapped(
              clampedSelectedIndex,
              modalNavigationAction.delta,
              filteredEntries.length,
            ),
          }
          : current);
        return;
      }
      if (modalNavigationAction?.kind === "boundary") {
        setModal((current) => current?.kind === "command"
          ? {
            ...current,
            selectedIndex: modalNavigationAction.boundary === "start"
              ? 0
              : Math.max(0, filteredEntries.length - 1),
          }
          : current);
        return;
      }
      if (isConfirmKey(normalized) || isMoveRightKey(normalized)) {
        const resolver = modal.resolve;
        const selected = filteredEntries[clampedSelectedIndex]?.value;
        setModal(null);
        resolver(selected);
        return;
      }
      if (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver(undefined);
      }
      return;
    }

    if (modal.kind === "select" && modal.options.entries.length === 0) {
      if (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver(undefined);
      }
      return;
    }

    if (modal.kind === "multiselect" && modal.options.entries.length === 0) {
      if (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver([]);
      }
      return;
    }

    if (modal.kind === "policy" && modal.options.entries.length === 0) {
      if (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver(createEmptyPolicySelection());
      }
      return;
    }

    if (modalNavigationAction?.kind === "move") {
      setModal((current) => current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
        ? {
            ...current,
            selectedIndex: moveSelectionWrapped(
              current.selectedIndex,
              modalNavigationAction.delta,
              current.options.entries.length,
            ),
          }
        : current);
      return;
    }
    if (modalNavigationAction?.kind === "boundary") {
      setModal((current) => current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
        ? {
          ...current,
          selectedIndex: modalNavigationAction.boundary === "start"
            ? 0
            : Math.max(0, current.options.entries.length - 1),
        }
        : current);
      return;
    }
    if (modal.kind === "multiselect" && isConfirmOrToggleKey(normalized)) {
      const selected = modal.options.entries[modal.selectedIndex]?.value;
      if (!selected) {
        return;
      }
      setModal((current) => current?.kind === "multiselect"
        ? {
          ...current,
          selectedValues: current.selectedValues.includes(selected)
            ? current.selectedValues.filter((value) => value !== selected)
            : [...current.selectedValues, selected],
        }
        : current);
      return;
    }
    if (modal.kind === "select" && (isConfirmKey(normalized) || isMoveRightKey(normalized))) {
      const resolver = modal.resolve;
      const selected = modal.options.entries[modal.selectedIndex]?.value;
      setModal(null);
      resolver(selected);
      return;
    }
    if (modal.kind === "multiselect" && isBackNavigationKey(normalized)) {
      const resolver = modal.resolve;
      const selectedValues = modal.selectedValues;
      setModal(null);
      resolver(selectedValues);
      return;
    }
    if (modal.kind === "policy" && isConfirmOrToggleKey(normalized)) {
      const selected = modal.options.entries[modal.selectedIndex]?.value;
      if (!selected) {
        return;
      }
      setModal((current) => current?.kind === "policy"
        ? {
          ...current,
          valueStates: {
            ...current.valueStates,
            [selected]: cyclePolicyState(current.valueStates[selected], current.options.allowedStates),
          },
        }
        : current);
      return;
    }
    if (modal.kind === "policy" && (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c")) {
      const resolver = modal.resolve;
      const selection = buildPolicySelection(modal.options.entries, modal.valueStates);
      setModal(null);
      resolver(selection);
      return;
    }
    if (modal.kind === "select" && (isBackNavigationKey(normalized) || normalized === "q" || normalized === "ctrl_c")) {
      const resolver = modal.resolve;
      setModal(null);
      resolver(undefined);
    }
  }, { isActive: modal !== null });

  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    return (
      <TerminalInlinePromptPanel
        title={modal.options.title}
        subtitle={modal.options.subtitle}
        body={(
          <InlinePromptMessageBody
            width={width}
            height={Math.max(0, height - 4)}
            lines={modal.options.body}
          />
        )}
        footer={modal.options.footer ?? [{ text: "Press any key to continue.", tone: "dim" }]}
        width={width}
        height={height}
        showTopBorder={presentation === "inline"}
      />
    );
  }
  if (modal.kind === "text") {
    return (
      <PromptBody
        options={modal.options}
        currentValue={modal.value}
        width={width}
        height={height}
      />
    );
  }
  if (modal.kind === "command") {
    return (
      <CommandPaletteBody
        options={modal.options}
        filterText={modal.filterText}
        selectedIndex={modal.selectedIndex}
        width={width}
        height={height}
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
        height={height}
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
        height={height}
      />
    );
  }

  return <SelectPromptBody options={modal.options} selectedIndex={modal.selectedIndex} width={width} height={height} />;
}

export function DerivedTagTerminalProvider({
  children,
}: {
  children: React.ReactNode;
}): React.JSX.Element {
  const { exit } = useApp();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);
  const modalPresentation = React.useMemo(() => getModalPresentation(modal), [modal]);
  const inlineModalHeight = React.useMemo(() => getInlineModalHeight(modal, rows), [modal, rows]);
  const availableRows = modalPresentation === "screen"
    ? 0
    : Math.max(0, rows - inlineModalHeight);

  const contextValue = React.useMemo<DerivedTagTerminalContextValue>(() => ({
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
            footer: [{ text: "Press any key to continue.", tone: "dim" }],
          },
          resolve,
        });
      });
    },
    promptCommandPalette: async <T extends string>(options: CommandPaletteOptions<T>) =>
      new Promise<T | undefined>((resolve) => {
        setModal({
          kind: "command",
          options: options as CommandPaletteOptions<string>,
          filterText: "",
          selectedIndex: 0,
          resolve: resolve as (value: string | undefined) => void,
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
        const selectedIndex = Math.max(0, options.entries.findIndex((entry) => options.selectedValues?.includes(entry.value)));
        setModal({
          kind: "multiselect",
          options: options as MultiSelectPromptOptions<string>,
          selectedIndex,
          selectedValues: options.selectedValues ? [...options.selectedValues] : [],
          resolve: resolve as (value: string[]) => void,
        });
      }),
    promptSelectOption: async <T extends string>(options: SelectPromptOptions<T>) =>
      new Promise<T | undefined>((resolve) => {
        const selectedIndex = Math.max(0, options.entries.findIndex((entry) => entry.value === options.selectedValue));
        setModal({
          kind: "select",
          options: options as SelectPromptOptions<string>,
          selectedIndex,
          resolve: resolve as (value: string | undefined) => void,
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
  }), [availableRows, columns, exit, modal]);

  return (
    <DerivedTagTerminalContext.Provider value={contextValue}>
      {modal && modalPresentation === "screen" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          width={columns}
          height={rows}
        />
      ) : null}
      <Box flexDirection="column">
        {children}
      </Box>
      {modal && modalPresentation === "inline" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          width={columns}
          height={inlineModalHeight}
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
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>{separator}</Text>
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
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>{separator}</Text>
        <TerminalPaneView pane={center} width={dimensions.centerWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>{separator}</Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

export async function runDerivedTagTerminalApp(node: React.ReactElement): Promise<void> {
  const instance = renderInkApp(
    <DerivedTagTerminalProvider>{node}</DerivedTagTerminalProvider>,
    {
      alternateScreen: true,
      exitOnCtrlC: false,
      patchConsole: true,
    },
  );

  try {
    await instance.waitUntilExit();
  } finally {
    instance.cleanup();
  }
}
