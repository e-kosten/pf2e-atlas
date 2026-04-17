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

type DialogOptions = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
};

type TextPromptOptions = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
};

type SelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T;
};

type MultiSelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValues?: T[];
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
  };

type DerivedTagTerminalContextValue = {
  exitApp: (result?: unknown) => void;
  getTerminalHeight: () => number;
  getTerminalWidth: () => number;
  modalActive: boolean;
  pauseForAnyKey: (message: string) => Promise<void>;
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

export function useDerivedTagTerminalApp(): DerivedTagTerminalContextValue {
  return ensureTerminalContext();
}

export function useDerivedTagTerminalInput(
  handler: (input: string, key: Key) => void,
  isActive = true,
): void {
  const terminal = ensureTerminalContext();
  useInput(handler, { isActive: isActive && !terminal.modalActive });
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

export function getNormalizedKeyName(input: string, key: Key): string {
  if (input === "\r" || input === "\n") {
    return "enter";
  }
  if (input === "\u001b") {
    return "escape";
  }
  if (input === "\b" || input === "\u007f") {
    return "backspace";
  }
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

function PromptBody({
  options,
  currentValue,
}: {
  options: TextPromptOptions;
  currentValue: string;
}): React.JSX.Element {
  return (
    <TerminalTextScreen
      title={options.title}
      body={[
        { text: options.prompt, tone: "section" },
        ...(options.hint ? [{ text: options.hint, tone: "dim" as const }] : []),
        { text: "" },
        { text: `> ${currentValue}` },
        { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
      ]}
      footer={[{ text: "Type text  Enter submit  Backspace edit  Esc cancel", tone: "dim" }]}
    />
  );
}

function SelectPromptBody({
  options,
  selectedIndex,
}: {
  options: SelectPromptOptions<string>;
  selectedIndex: number;
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalTextScreen
        title={options.title}
        subtitle={options.subtitle}
        body={[
          { text: options.prompt, tone: "section" },
          { text: "" },
          { text: "No options are available for this scope.", tone: "warning" },
        ]}
        footer={[{ text: "Esc, Backspace, Left, or q cancel", tone: "dim" }]}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const detailLines = selectedOption?.detailLines?.length
    ? selectedOption.detailLines
    : selectedOption?.description
      ? [
        { text: selectedOption.label, tone: "section" as const },
        { text: selectedOption.description },
      ]
      : [
        { text: selectedOption?.label ?? "(none)", tone: "section" as const },
        { text: "No additional details.", tone: "dim" as const },
      ];

  return (
    <TerminalTwoPaneScreen
      title={options.title}
      subtitle={options.subtitle}
      left={{
        title: options.prompt,
        lines: options.entries.map((entry, index) => ({
          text: entry.label,
          tone: index === selectedIndex ? "selected" : "default",
          noWrap: true,
        })),
        active: true,
      }}
      right={{
        title: "Details",
        lines: detailLines,
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter select  Esc/backspace/left cancel", tone: "dim" },
        { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]}
      leftWidth={40}
    />
  );
}

function MultiSelectPromptBody({
  options,
  selectedIndex,
  selectedValues,
}: {
  options: MultiSelectPromptOptions<string>;
  selectedIndex: number;
  selectedValues: string[];
}): React.JSX.Element {
  if (options.entries.length === 0) {
    return (
      <TerminalTextScreen
        title={options.title}
        subtitle={options.subtitle}
        body={[
          { text: options.prompt, tone: "section" },
          { text: "" },
          { text: "No options are available for this scope.", tone: "warning" },
        ]}
        footer={[{ text: "Esc, Backspace, or Left return", tone: "dim" }]}
      />
    );
  }

  const selectedOption = options.entries[selectedIndex];
  const selectedSet = new Set(selectedValues);
  const detailLines = selectedOption?.detailLines?.length
    ? selectedOption.detailLines
    : selectedOption?.description
      ? [
        { text: selectedOption.label, tone: "section" as const },
        { text: selectedOption.description },
      ]
      : [
        { text: selectedOption?.label ?? "(none)", tone: "section" as const },
        { text: "No additional details.", tone: "dim" as const },
      ];
  const selectedLabels = options.entries
    .filter((entry) => selectedSet.has(entry.value))
    .map((entry) => entry.label);

  return (
    <TerminalTwoPaneScreen
      title={options.title}
      subtitle={options.subtitle}
      left={{
        title: options.prompt,
        lines: options.entries.map((entry, index) => ({
          text: `[${selectedSet.has(entry.value) ? "x" : " "}] ${entry.label}`,
          tone: index === selectedIndex ? "selected" : "default",
          noWrap: true,
        })),
        active: true,
      }}
      right={{
        title: "Details",
        lines: [
          ...detailLines,
          { text: "" },
          { text: "Current Selection", tone: "section" },
          { text: selectedLabels.length > 0 ? selectedLabels.join(", ") : "(none)" },
        ],
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter or Space toggle  Esc/backspace/left return", tone: "dim" },
        { text: `${selectedValues.length} selected | Focused: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ]}
      leftWidth={40}
    />
  );
}

function DerivedTagTerminalModalHost({
  modal,
  setModal,
}: {
  modal: TerminalModalState;
  setModal: React.Dispatch<React.SetStateAction<TerminalModalState>>;
}): React.JSX.Element | null {
  useInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const printable = getPrintableInput(input, key);

    if (!modal) {
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

    if (modal.kind === "select" && modal.options.entries.length === 0) {
      if (normalized === "escape" || normalized === "backspace" || normalized === "left" || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver(undefined);
      }
      return;
    }

    if (modal.kind === "multiselect" && modal.options.entries.length === 0) {
      if (normalized === "escape" || normalized === "backspace" || normalized === "left" || normalized === "q" || normalized === "ctrl_c") {
        const resolver = modal.resolve;
        setModal(null);
        resolver([]);
      }
      return;
    }

    if (normalized === "up" || normalized === "k") {
      setModal((current) => current && (current.kind === "select" || current.kind === "multiselect")
        ? { ...current, selectedIndex: moveSelectionWrapped(current.selectedIndex, -1, current.options.entries.length) }
        : current);
      return;
    }
    if (normalized === "down" || normalized === "j") {
      setModal((current) => current && (current.kind === "select" || current.kind === "multiselect")
        ? { ...current, selectedIndex: moveSelectionWrapped(current.selectedIndex, 1, current.options.entries.length) }
        : current);
      return;
    }
    if (modal.kind === "multiselect" && (normalized === "enter" || normalized === "space")) {
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
    if (modal.kind === "select" && (normalized === "enter" || normalized === "right" || normalized === "l")) {
      const resolver = modal.resolve;
      const selected = modal.options.entries[modal.selectedIndex]?.value;
      setModal(null);
      resolver(selected);
      return;
    }
    if (modal.kind === "multiselect" && (normalized === "escape" || normalized === "backspace" || normalized === "left")) {
      const resolver = modal.resolve;
      const selectedValues = modal.selectedValues;
      setModal(null);
      resolver(selectedValues);
      return;
    }
    if (modal.kind === "select" && (normalized === "escape" || normalized === "backspace" || normalized === "q" || normalized === "ctrl_c")) {
      const resolver = modal.resolve;
      setModal(null);
      resolver(undefined);
    }
  }, { isActive: modal !== null });

  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    return <TerminalTextScreen {...modal.options} />;
  }
  if (modal.kind === "text") {
    return <PromptBody options={modal.options} currentValue={modal.value} />;
  }
  if (modal.kind === "multiselect") {
    return (
      <MultiSelectPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        selectedValues={modal.selectedValues}
      />
    );
  }

  return <SelectPromptBody options={modal.options} selectedIndex={modal.selectedIndex} />;
}

export function DerivedTagTerminalProvider({
  children,
}: {
  children: React.ReactNode;
}): React.JSX.Element {
  const { exit } = useApp();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);

  const contextValue = React.useMemo<DerivedTagTerminalContextValue>(() => ({
    exitApp: exit,
    getTerminalHeight: () => rows,
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
  }), [columns, exit, modal, rows]);

  return (
    <DerivedTagTerminalContext.Provider value={contextValue}>
      <Box display={modal ? "none" : "flex"} flexDirection="column">
        {children}
      </Box>
      <Box display={modal ? "flex" : "none"} flexDirection="column">
        {modal ? <DerivedTagTerminalModalHost modal={modal} setModal={setModal} /> : null}
      </Box>
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
