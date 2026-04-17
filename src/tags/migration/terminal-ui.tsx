import React from "react";
import { Box, Text, render, useInput, useWindowSize, type Instance, type Key } from "ink";

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

export type DerivedTagTerminalKey = {
  name: string;
  normalizedName: string;
  matches: string[];
  data: {
    codepoint?: number;
    code?: number | Buffer;
    isCharacter?: boolean;
    meta?: string;
  };
};

export type DerivedTagTerminalLine = {
  text: string;
  tone?: DerivedTagTerminalTone;
  indent?: number;
  noWrap?: boolean;
};

export type DerivedTagTerminalTextScreen = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalPane = {
  title: string;
  lines: DerivedTagTerminalLine[];
  active?: boolean;
};

export type DerivedTagTerminalTwoPaneFocus = "list" | "detail";
export type DerivedTagTerminalTwoPaneLayoutMode = "split" | "detail-only";

export type DerivedTagTerminalTwoPaneScreen = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
};

export type DerivedTagTerminalPaneScreen = {
  title: string;
  subtitle?: string;
  pane: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalSelectOption<T extends string = string> = {
  value: T;
  label: string;
  description?: string;
  detailLines?: DerivedTagTerminalLine[];
};

type SessionScreen =
  | { kind: "text"; screen: DerivedTagTerminalTextScreen }
  | { kind: "pane"; screen: DerivedTagTerminalPaneScreen }
  | { kind: "two-pane"; screen: DerivedTagTerminalTwoPaneScreen };

type PendingKeyResolver = {
  allowResize: boolean;
  resolve: (key: DerivedTagTerminalKey) => void;
};

export type DerivedTagTerminalSession = {
  height: number;
  instance: Instance | null;
  pendingKeyResolver: PendingKeyResolver | null;
  queuedKeys: DerivedTagTerminalKey[];
  screen: SessionScreen;
  width: number;
};

const DEFAULT_SCREEN: SessionScreen = {
  kind: "text",
  screen: {
    title: "Derived-Tag Workbench",
    body: [{ text: "Loading...", tone: "dim" }],
  },
};

function normalizeKeyName(name: string): string {
  return name.toLowerCase();
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

function normalizeLine(line: DerivedTagTerminalLine): Required<DerivedTagTerminalLine> {
  return {
    text: line.text,
    tone: line.tone ?? "default",
    indent: line.indent ?? 0,
    noWrap: line.noWrap ?? false,
  };
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

function emitResize(session: DerivedTagTerminalSession): void {
  if (!session.pendingKeyResolver?.allowResize) {
    return;
  }

  const resolver = session.pendingKeyResolver;
  session.pendingKeyResolver = null;
  resolver.resolve({
    name: "resize",
    normalizedName: "resize",
    matches: [],
    data: {},
  });
}

function emitKey(session: DerivedTagTerminalSession, key: DerivedTagTerminalKey): void {
  if (session.pendingKeyResolver) {
    const resolver = session.pendingKeyResolver;
    session.pendingKeyResolver = null;
    resolver.resolve(key);
    return;
  }

  session.queuedKeys.push(key);
}

function normalizeInkKey(input: string, key: Key): DerivedTagTerminalKey {
  const character = input.length === 1 ? input : undefined;

  let name = input;
  if (key.upArrow) {
    name = "up";
  } else if (key.downArrow) {
    name = "down";
  } else if (key.leftArrow) {
    name = "left";
  } else if (key.rightArrow) {
    name = "right";
  } else if (key.pageDown) {
    name = "page_down";
  } else if (key.pageUp) {
    name = "page_up";
  } else if (key.home) {
    name = "home";
  } else if (key.end) {
    name = "end";
  } else if (key.return) {
    name = "enter";
  } else if (key.escape) {
    name = "escape";
  } else if (key.tab && key.shift) {
    name = "shift_tab";
  } else if (key.tab) {
    name = "tab";
  } else if (key.backspace) {
    name = "backspace";
  } else if (key.delete) {
    name = "delete";
  } else if (key.ctrl && character && /^[a-zA-Z]$/.test(character)) {
    name = `ctrl_${character.toLowerCase()}`;
  } else if (character === " ") {
    name = "space";
  } else if (character === "/") {
    name = "slash";
  }

  return {
    name,
    normalizedName: normalizeKeyName(name),
    matches: [],
    data: {
      codepoint: character ? character.codePointAt(0) : undefined,
      isCharacter: Boolean(character && !key.ctrl && !key.meta),
      meta: key.meta ? "meta" : undefined,
    },
  };
}

function requestRender(session: DerivedTagTerminalSession): void {
  if (!session.instance) {
    return;
  }
  session.instance.rerender(<TerminalRoot session={session} />);
}

function ScreenRows({
  lines,
  tone,
  width,
}: {
  lines: Array<{ text: string; tone: DerivedTagTerminalTone }>;
  tone?: DerivedTagTerminalTone;
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      {lines.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(tone ?? line.tone)}>
          {fitToWidth(line.text, width)}
        </Text>
      ))}
    </Box>
  );
}

function Header({
  title,
  subtitle,
  width,
}: {
  title: string;
  subtitle?: string;
  width: number;
}): React.JSX.Element {
  const divider = fitToWidth("═".repeat(Math.max(0, width)), width);

  return (
    <Box flexDirection="column" width={width}>
      <Text wrap="truncate-end" {...terminalToneProps("heading")}>{fitToWidth(title, width)}</Text>
      {subtitle ? (
        <Text wrap="truncate-end" {...terminalToneProps("accent")}>{fitToWidth(subtitle, width)}</Text>
      ) : null}
      <Text wrap="truncate-end" {...terminalToneProps("dim")}>{divider}</Text>
    </Box>
  );
}

function Footer({
  footer,
  width,
}: {
  footer?: DerivedTagTerminalLine[];
  width: number;
}): React.JSX.Element | null {
  if (!footer || footer.length === 0) {
    return null;
  }

  const rows = footer.map((line) => ({
    text: fitToWidth(line.text, width),
    tone: line.tone ?? "default",
  }));

  return <ScreenRows lines={rows} width={width} />;
}

function PaneView({
  pane,
  width,
  height,
}: {
  pane: DerivedTagTerminalPane;
  width: number;
  height: number;
}): React.JSX.Element {
  const divider = fitToWidth("─".repeat(Math.max(0, width)), width);
  const bodyHeight = Math.max(0, height - 2);
  const rows = renderRows(pane.lines, width, bodyHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "selected" : "section")}>
        {fitToWidth(pane.title, width)}
      </Text>
      {height > 1 ? (
        <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "accent" : "dim")}>
          {divider}
        </Text>
      ) : null}
      {bodyHeight > 0 ? <ScreenRows lines={rows} width={width} /> : null}
    </Box>
  );
}

function TextScreenView({
  screen,
  width,
  height,
}: {
  screen: DerivedTagTerminalTextScreen;
  width: number;
  height: number;
}): React.JSX.Element {
  const headerHeight = screen.subtitle ? 3 : 2;
  const footerHeight = screen.footer?.length ?? 0;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);
  const rows = renderRows(screen.body, width, bodyHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Header title={screen.title} subtitle={screen.subtitle} width={width} />
      <ScreenRows lines={rows} width={width} />
      <Footer footer={screen.footer} width={width} />
    </Box>
  );
}

function PaneScreenView({
  screen,
  width,
  height,
}: {
  screen: DerivedTagTerminalPaneScreen;
  width: number;
  height: number;
}): React.JSX.Element {
  const headerHeight = screen.subtitle ? 3 : 2;
  const footerHeight = screen.footer?.length ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Header title={screen.title} subtitle={screen.subtitle} width={width} />
      <PaneView pane={screen.pane} width={width} height={contentHeight} />
      <Footer footer={screen.footer} width={width} />
    </Box>
  );
}

function TwoPaneScreenView({
  screen,
  width,
  height,
}: {
  screen: DerivedTagTerminalTwoPaneScreen;
  width: number;
  height: number;
}): React.JSX.Element {
  const headerHeight = screen.subtitle ? 3 : 2;
  const footerHeight = screen.footer?.length ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);
  const { leftWidth, rightWidth } = getTerminalTwoPaneDimensions({
    ...createDetachedSession(),
    width,
    height,
  }, screen.leftWidth);

  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Header title={screen.title} subtitle={screen.subtitle} width={width} />
      <Box flexDirection="row" width={width} height={contentHeight}>
        <PaneView pane={screen.left} width={leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>{separator}</Text>
        <PaneView pane={screen.right} width={rightWidth} height={contentHeight} />
      </Box>
      <Footer footer={screen.footer} width={width} />
    </Box>
  );
}

function TerminalRoot({ session }: { session: DerivedTagTerminalSession }): React.JSX.Element {
  const { columns, rows } = useWindowSize();

  React.useEffect(() => {
    session.width = columns;
    session.height = rows;
    emitResize(session);
  }, [columns, rows, session]);

  useInput((input, key) => {
    emitKey(session, normalizeInkKey(input, key));
  });

  const width = Math.max(20, session.width || columns);
  const height = Math.max(8, session.height || rows);

  return (
    <Box width={width} height={height}>
      {session.screen.kind === "text" ? <TextScreenView screen={session.screen.screen} width={width} height={height} /> : null}
      {session.screen.kind === "pane" ? <PaneScreenView screen={session.screen.screen} width={width} height={height} /> : null}
      {session.screen.kind === "two-pane" ? <TwoPaneScreenView screen={session.screen.screen} width={width} height={height} /> : null}
    </Box>
  );
}

function createDetachedSession(): DerivedTagTerminalSession {
  return {
    height: 24,
    instance: null,
    pendingKeyResolver: null,
    queuedKeys: [],
    screen: DEFAULT_SCREEN,
    width: 80,
  };
}

export async function runWithDerivedTagTerminalSession<T>(
  callback: (session: DerivedTagTerminalSession) => Promise<T>,
): Promise<T> {
  const session: DerivedTagTerminalSession = createDetachedSession();
  const instance = render(<TerminalRoot session={session} />, {
    alternateScreen: true,
    exitOnCtrlC: false,
    patchConsole: true,
  });
  session.instance = instance;

  try {
    return await callback(session);
  } finally {
    instance.unmount();
    await instance.waitUntilExit();
    instance.cleanup();
  }
}

export function clearTerminalScreen(session: DerivedTagTerminalSession): void {
  session.screen = DEFAULT_SCREEN;
  requestRender(session);
}

export function renderTerminalTextScreen(
  session: DerivedTagTerminalSession,
  screen: DerivedTagTerminalTextScreen,
): void {
  session.screen = { kind: "text", screen };
  requestRender(session);
}

export function renderTerminalTwoPaneScreen(
  session: DerivedTagTerminalSession,
  screen: DerivedTagTerminalTwoPaneScreen,
): void {
  session.screen = { kind: "two-pane", screen };
  requestRender(session);
}

export function renderTerminalPaneScreen(
  session: DerivedTagTerminalSession,
  screen: DerivedTagTerminalPaneScreen,
): void {
  session.screen = { kind: "pane", screen };
  requestRender(session);
}

export function getTerminalTwoPaneDimensions(
  session: DerivedTagTerminalSession,
  preferredLeftWidth?: number,
): { leftWidth: number; rightWidth: number; separatorWidth: number } {
  const totalWidth = session.width;
  const separatorWidth = 1;
  const leftWidth = Math.max(24, Math.min(preferredLeftWidth ?? Math.floor(totalWidth * 0.38), totalWidth - separatorWidth - 20));
  const rightWidth = Math.max(20, totalWidth - leftWidth - separatorWidth);

  return { leftWidth, rightWidth, separatorWidth };
}

export function getTerminalTwoPaneDetailWidth(
  session: DerivedTagTerminalSession,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  preferredLeftWidth?: number,
): number {
  if (layoutMode === "detail-only") {
    return session.width;
  }
  return getTerminalTwoPaneDimensions(session, preferredLeftWidth).rightWidth;
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

export function getTerminalPaneBodyHeight(
  session: DerivedTagTerminalSession,
  options: { hasSubtitle?: boolean; footerLineCount?: number },
): number {
  const headerBottom = options.hasSubtitle ? 3 : 2;
  const footerHeight = options.footerLineCount ?? 0;
  const contentHeight = Math.max(0, session.height - headerBottom - footerHeight);
  return Math.max(0, contentHeight - 2);
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

function nextQueuedKey(session: DerivedTagTerminalSession): DerivedTagTerminalKey | undefined {
  return session.queuedKeys.shift();
}

export async function readTerminalKey(
  session: DerivedTagTerminalSession,
): Promise<DerivedTagTerminalKey> {
  const queued = nextQueuedKey(session);
  if (queued) {
    return queued;
  }

  return new Promise((resolve) => {
    session.pendingKeyResolver = { allowResize: false, resolve };
  });
}

export async function readTerminalKeyOrResize(
  session: DerivedTagTerminalSession,
): Promise<DerivedTagTerminalKey> {
  const queued = nextQueuedKey(session);
  if (queued) {
    return queued;
  }

  return new Promise((resolve) => {
    session.pendingKeyResolver = { allowResize: true, resolve };
  });
}

function printableCharacterForKey(key: DerivedTagTerminalKey): string | undefined {
  if (!key.data.isCharacter) {
    return undefined;
  }
  if (typeof key.data.codepoint === "number") {
    return String.fromCodePoint(key.data.codepoint);
  }
  return key.name.length === 1 ? key.name : undefined;
}

export async function promptTerminalTextInput(
  session: DerivedTagTerminalSession,
  options: {
    title: string;
    prompt: string;
    defaultValue?: string;
    hint?: string;
  },
): Promise<string | undefined> {
  let value = options.defaultValue ?? "";

  while (true) {
    renderTerminalTextScreen(session, {
      title: options.title,
      body: [
        { text: options.prompt, tone: "section" },
        ...(options.hint ? [{ text: options.hint, tone: "dim" as const }] : []),
        { text: "" },
        { text: `> ${value || ""}` },
        { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
      ],
      footer: [{ text: "Type text  Enter submit  Backspace edit  Esc cancel", tone: "dim" }],
    });

    const key = await readTerminalKey(session);
    const normalized = key.normalizedName;
    const printable = printableCharacterForKey(key);

    if (normalized === "enter") {
      const trimmed = value.trim();
      return trimmed ? trimmed : undefined;
    }
    if (normalized === "escape") {
      return undefined;
    }
    if (normalized === "backspace") {
      value = [...value].slice(0, -1).join("");
      continue;
    }
    if (printable) {
      value += printable;
    }
  }
}

function buildTerminalSelectListLines<T extends string>(
  session: DerivedTagTerminalSession,
  options: DerivedTagTerminalSelectOption<T>[],
  selectedIndex: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, getTerminalPaneBodyHeight(session, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const windowStart = Math.max(0, Math.min(
    selectedIndex - Math.floor(visibleCount / 2),
    Math.max(0, options.length - visibleCount),
  ));

  return options.slice(windowStart, windowStart + visibleCount).map((option, offset) => ({
    text: option.label,
    tone: windowStart + offset === selectedIndex ? "selected" : "default",
    noWrap: true,
  }));
}

function buildTerminalSelectDetailLines<T extends string>(
  option: DerivedTagTerminalSelectOption<T> | undefined,
): DerivedTagTerminalLine[] {
  if (!option) {
    return [{ text: "No option selected.", tone: "dim" }];
  }

  if (option.detailLines && option.detailLines.length > 0) {
    return option.detailLines;
  }

  return option.description
    ? [
      { text: option.label, tone: "section" },
      { text: option.description },
    ]
    : [
      { text: option.label, tone: "section" },
      { text: "No additional details.", tone: "dim" },
    ];
}

export async function promptTerminalSelectOption<T extends string>(
  session: DerivedTagTerminalSession,
  options: {
    title: string;
    subtitle?: string;
    prompt: string;
    entries: DerivedTagTerminalSelectOption<T>[];
    selectedValue?: T;
  },
): Promise<T | undefined> {
  if (options.entries.length === 0) {
    renderTerminalTextScreen(session, {
      title: options.title,
      subtitle: options.subtitle,
      body: [
        { text: options.prompt, tone: "section" },
        { text: "" },
        { text: "No options are available for this scope.", tone: "warning" },
      ],
      footer: [{ text: "Esc, Backspace, Left, or q cancel", tone: "dim" }],
    });

    while (true) {
      const key = await readTerminalKey(session);
      const normalized = key.normalizedName;
      if (normalized === "escape" || normalized === "backspace" || normalized === "left" || normalized === "q" || normalized === "ctrl_c") {
        return undefined;
      }
    }
  }

  let selectedIndex = Math.max(0, options.entries.findIndex((entry) => entry.value === options.selectedValue));

  while (true) {
    const selectedOption = options.entries[selectedIndex];
    renderTerminalTwoPaneScreen(session, {
      title: options.title,
      subtitle: options.subtitle,
      left: {
        title: options.prompt,
        lines: buildTerminalSelectListLines(session, options.entries, selectedIndex),
      },
      right: {
        title: "Details",
        lines: buildTerminalSelectDetailLines(selectedOption),
      },
      footer: [
        { text: "Up/Down or j/k move  Enter select  Esc/backspace/left cancel", tone: "dim" },
        { text: `Selected: ${selectedOption?.label ?? "(none)"}`, tone: "accent" },
      ],
      leftWidth: 40,
    });

    const key = await readTerminalKey(session);
    const normalized = key.normalizedName;
    if (normalized === "up" || normalized === "k") {
      selectedIndex = moveSelectionWrapped(selectedIndex, -1, options.entries.length);
      continue;
    }
    if (normalized === "down" || normalized === "j") {
      selectedIndex = moveSelectionWrapped(selectedIndex, 1, options.entries.length);
      continue;
    }
    if (normalized === "enter" || normalized === "right" || normalized === "l") {
      return selectedOption?.value;
    }
    if (normalized === "escape" || normalized === "backspace" || normalized === "left" || normalized === "q" || normalized === "ctrl_c") {
      return undefined;
    }
  }
}

export async function pauseForAnyKey(
  session: DerivedTagTerminalSession,
  message: string,
): Promise<void> {
  renderTerminalTextScreen(session, {
    title: "Derived-Tag Workbench",
    body: message.split("\n").map((line) => ({ text: line })),
    footer: [{ text: "Press any key to continue.", tone: "dim" }],
  });
  await readTerminalKey(session);
}

export function moveSelection(currentIndex: number, delta: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(currentIndex + delta, itemCount - 1));
}

export function moveSelectionWrapped(currentIndex: number, delta: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  const rawIndex = currentIndex + delta;
  return ((rawIndex % itemCount) + itemCount) % itemCount;
}
