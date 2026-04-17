import terminalKit from "terminal-kit";

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
};

export type DerivedTagTerminalTwoPaneScreen = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
};

type Terminal = ReturnType<typeof terminalKit.createTerminal>;

export type DerivedTagTerminalSession = {
  term: Terminal;
};

const { terminal } = terminalKit;
const { stringWidth, truncateString } = terminalKit;

function normalizeKeyName(name: string): string {
  return name.toLowerCase();
}

function repeatCharacter(character: string, width: number): string {
  return width > 0 ? character.repeat(width) : "";
}

function fitToWidth(text: string, width: number): string {
  if (width <= 0) {
    return "";
  }
  const truncated = truncateString(text, width);
  const paddingWidth = Math.max(0, width - stringWidth(truncated));
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
    if (stringWidth(candidate) <= width) {
      current = candidate;
      continue;
    }

    lines.push(current);
    if (stringWidth(word) <= width) {
      current = word;
      continue;
    }

    let remaining = word;
    while (stringWidth(remaining) > width) {
      lines.push(truncateString(remaining, width));
      remaining = remaining.slice(truncateString(remaining, width).length);
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

function applyTone(term: Terminal, tone: DerivedTagTerminalTone, text: string): void {
  switch (tone) {
    case "heading":
      term.bold.cyan(text);
      return;
    case "section":
      term.bold(text);
      return;
    case "dim":
      term.dim(text);
      return;
    case "accent":
      term.cyan(text);
      return;
    case "success":
      term.green(text);
      return;
    case "warning":
      term.yellow(text);
      return;
    case "danger":
      term.red(text);
      return;
    case "selected":
      term.inverse.bold(text);
      return;
    default:
      term(text);
  }
}

function renderLines(
  session: DerivedTagTerminalSession,
  x: number,
  startY: number,
  width: number,
  height: number,
  lines: DerivedTagTerminalLine[],
): void {
  const term = session.term;
  const renderedLines: Array<{ text: string; tone: DerivedTagTerminalTone }> = [];

  for (const rawLine of lines) {
    const line = normalizeLine(rawLine);
    const indent = " ".repeat(Math.max(0, line.indent));
    const usableWidth = Math.max(1, width - indent.length);
    const wrapped = line.noWrap ? [truncateString(line.text, usableWidth)] : wrapPlainText(line.text, usableWidth);

    for (const segment of wrapped) {
      renderedLines.push({
        text: `${indent}${segment}`,
        tone: line.tone,
      });
    }
  }

  for (let row = 0; row < height; row += 1) {
    term.moveTo(x, startY + row);
    const line = renderedLines[row];
    if (!line) {
      term(" ".repeat(width));
      continue;
    }
    applyTone(term, line.tone, fitToWidth(line.text, width));
  }
}

function renderPane(
  session: DerivedTagTerminalSession,
  x: number,
  startY: number,
  width: number,
  height: number,
  pane: DerivedTagTerminalPane,
): void {
  const term = session.term;
  if (width <= 0 || height <= 0) {
    return;
  }

  term.moveTo(x, startY);
  applyTone(term, "section", fitToWidth(pane.title, width));
  if (height === 1) {
    return;
  }

  term.moveTo(x, startY + 1);
  applyTone(term, "dim", fitToWidth(repeatCharacter("─", width), width));
  renderLines(session, x, startY + 2, width, Math.max(0, height - 2), pane.lines);
}

function renderHeader(
  session: DerivedTagTerminalSession,
  title: string,
  subtitle: string | undefined,
): number {
  const term = session.term;
  const width = term.width;
  term.moveTo(1, 1);
  applyTone(term, "heading", fitToWidth(title, width));

  if (!subtitle) {
    term.moveTo(1, 2);
    applyTone(term, "dim", fitToWidth(repeatCharacter("═", width), width));
    return 3;
  }

  term.moveTo(1, 2);
  applyTone(term, "accent", fitToWidth(subtitle, width));
  term.moveTo(1, 3);
  applyTone(term, "dim", fitToWidth(repeatCharacter("═", width), width));
  return 4;
}

function renderFooter(
  session: DerivedTagTerminalSession,
  footer: DerivedTagTerminalLine[] | undefined,
): number {
  if (!footer || footer.length === 0) {
    return 0;
  }

  const term = session.term;
  const width = term.width;
  const renderedFooter = footer.slice(-Math.max(1, term.height - 1));
  const startY = term.height - renderedFooter.length + 1;

  for (let index = 0; index < renderedFooter.length; index += 1) {
    term.moveTo(1, startY + index);
    const line = normalizeLine(renderedFooter[index]!);
    applyTone(term, line.tone, fitToWidth(line.text, width));
  }

  return renderedFooter.length;
}

export async function runWithDerivedTagTerminalSession<T>(
  callback: (session: DerivedTagTerminalSession) => Promise<T>,
): Promise<T> {
  const session: DerivedTagTerminalSession = { term: terminal };
  terminal.fullscreen({ noAlternate: false });
  terminal.grabInput({ mouse: "button", safe: true });
  terminal.windowTitle("Derived-Tag Workbench");

  try {
    return await callback(session);
  } finally {
    terminal.styleReset();
    terminal.grabInput(false);
    terminal.fullscreen(false);
    await terminal.asyncCleanup();
  }
}

export function clearTerminalScreen(session: DerivedTagTerminalSession): void {
  session.term.clear();
}

export function renderTerminalTextScreen(
  session: DerivedTagTerminalSession,
  screen: DerivedTagTerminalTextScreen,
): void {
  clearTerminalScreen(session);
  const headerBottom = renderHeader(session, screen.title, screen.subtitle);
  const footerHeight = renderFooter(session, screen.footer);
  const bodyHeight = Math.max(0, session.term.height - headerBottom - footerHeight + 1);
  renderLines(session, 1, headerBottom, session.term.width, bodyHeight, screen.body);
}

export function renderTerminalTwoPaneScreen(
  session: DerivedTagTerminalSession,
  screen: DerivedTagTerminalTwoPaneScreen,
): void {
  clearTerminalScreen(session);
  const headerBottom = renderHeader(session, screen.title, screen.subtitle);
  const footerHeight = renderFooter(session, screen.footer);
  const contentHeight = Math.max(0, session.term.height - headerBottom - footerHeight + 1);
  const totalWidth = session.term.width;
  const separatorWidth = 3;
  const leftWidth = Math.max(24, Math.min(screen.leftWidth ?? Math.floor(totalWidth * 0.38), totalWidth - separatorWidth - 20));
  const rightWidth = Math.max(20, totalWidth - leftWidth - separatorWidth);

  renderPane(session, 1, headerBottom, leftWidth, contentHeight, screen.left);
  session.term.moveTo(leftWidth + 2, headerBottom);
  applyTone(session.term, "dim", fitToWidth("│", 1));
  for (let row = 1; row < contentHeight; row += 1) {
    session.term.moveTo(leftWidth + 2, headerBottom + row);
    applyTone(session.term, "dim", fitToWidth("│", 1));
  }
  renderPane(session, leftWidth + separatorWidth, headerBottom, rightWidth, contentHeight, screen.right);
}

export function getTerminalPaneBodyHeight(
  session: DerivedTagTerminalSession,
  options: { hasSubtitle?: boolean; footerLineCount?: number },
): number {
  const headerBottom = options.hasSubtitle ? 4 : 3;
  const footerHeight = options.footerLineCount ?? 0;
  const contentHeight = Math.max(0, session.term.height - headerBottom - footerHeight + 1);
  return Math.max(0, contentHeight - 2);
}

export async function readTerminalKey(
  session: DerivedTagTerminalSession,
): Promise<DerivedTagTerminalKey> {
  return new Promise((resolve) => {
    const onKey = (name: string, matches: string[], data: DerivedTagTerminalKey["data"]) => {
      session.term.off("key", onKey);
      resolve({
        name,
        normalizedName: normalizeKeyName(name),
        matches,
        data,
      });
    };

    session.term.on("key", onKey);
  });
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
  renderTerminalTextScreen(session, {
    title: options.title,
    body: [
      { text: options.prompt, tone: "section" },
      ...(options.hint ? [{ text: options.hint, tone: "dim" as const }] : []),
      { text: "" },
      { text: options.defaultValue ? `Default: ${options.defaultValue}` : "Leave blank to skip.", tone: "dim" },
    ],
    footer: [{ text: "Enter submit  Esc cancel", tone: "dim" }],
  });

  session.term.moveTo(1, Math.min(session.term.height, 6));
  session.term.eraseLine();
  session.term("> ");
  const response = await session.term.inputField({
    cancelable: true,
    default: options.defaultValue,
  }).promise;
  return response?.trim() ? response.trim() : undefined;
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
