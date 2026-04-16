import { createInterface } from "node:readline/promises";
import { emitKeypressEvents } from "node:readline";
import { stdin as input, stdout as output } from "node:process";

export type DerivedTagTerminalKey = {
  name?: string;
  sequence: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
};

let keypressInitialized = false;
const ANSI = {
  reset: "\x1B[0m",
  bold: "\x1B[1m",
  dim: "\x1B[2m",
  black: "\x1B[30m",
  red: "\x1B[31m",
  green: "\x1B[32m",
  yellow: "\x1B[33m",
  blue: "\x1B[34m",
  magenta: "\x1B[35m",
  cyan: "\x1B[36m",
  white: "\x1B[97m",
  bgBlue: "\x1B[44m",
  bgCyan: "\x1B[46m",
  bgGreen: "\x1B[42m",
  bgRed: "\x1B[41m",
  bgYellow: "\x1B[43m",
} as const;

export function clearTerminalScreen(): void {
  output.write("\x1B[2J\x1B[0f");
}

function styleText(text: string, ...codes: string[]): string {
  if (!output.isTTY) {
    return text;
  }
  return `${codes.join("")}${text}${ANSI.reset}`;
}

export const terminalTheme = {
  heading: (text: string) => styleText(text, ANSI.bold, ANSI.cyan),
  section: (text: string) => styleText(text, ANSI.bold),
  dim: (text: string) => styleText(text, ANSI.dim),
  muted: (text: string) => styleText(text, ANSI.dim),
  success: (text: string) => styleText(text, ANSI.green),
  warning: (text: string) => styleText(text, ANSI.yellow),
  danger: (text: string) => styleText(text, ANSI.red),
  accent: (text: string) => styleText(text, ANSI.cyan),
  selectedLine: (text: string) => styleText(text, ANSI.bold, ANSI.white, ANSI.bgBlue),
  selectedAction: (text: string) => styleText(` ${text} `, ANSI.bold, ANSI.black, ANSI.bgYellow),
  selectedMarker: (text: string) => styleText(text, ANSI.bold, ANSI.yellow),
  positiveAction: (text: string) => styleText(text, ANSI.green),
  negativeAction: (text: string) => styleText(text, ANSI.red),
  cautionAction: (text: string) => styleText(text, ANSI.yellow),
  neutralAction: (text: string) => styleText(text, ANSI.cyan),
  successBadge: (text: string) => styleText(` ${text} `, ANSI.bold, ANSI.black, ANSI.bgGreen),
  dangerBadge: (text: string) => styleText(` ${text} `, ANSI.bold, ANSI.white, ANSI.bgRed),
  warningBadge: (text: string) => styleText(` ${text} `, ANSI.bold, ANSI.black, ANSI.bgYellow),
};

function ensureKeypressEvents(): void {
  if (keypressInitialized) {
    return;
  }
  emitKeypressEvents(input);
  keypressInitialized = true;
}

export async function readTerminalKey(promptText?: string): Promise<DerivedTagTerminalKey> {
  if (!input.isTTY) {
    const rl = createInterface({ input, output });
    try {
      const answer = (await rl.question(promptText ?? "")).trim().toLowerCase();
      return {
        name: answer,
        sequence: answer,
      };
    } finally {
      rl.close();
    }
  }

  if (promptText) {
    output.write(promptText);
  }

  ensureKeypressEvents();
  input.setRawMode(true);
  input.resume();

  return new Promise((resolve) => {
    const onKeypress = (sequence: string, key: Omit<DerivedTagTerminalKey, "sequence">) => {
      input.off("keypress", onKeypress);
      input.setRawMode(false);
      input.pause();
      resolve({
        sequence,
        ...key,
      });
    };

    input.on("keypress", onKeypress);
  });
}

export async function pauseForAnyKey(message: string): Promise<void> {
  console.log(`\n${message}`);
  await readTerminalKey("Press any key to continue...");
  console.log();
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
