import { useInput, type Key } from "ink";

import {
  isBackNavigationKey,
  isConfirmKey,
  isMoveDownKey,
  isMoveLeftKey,
  isMoveRightKey,
  isMoveUpKey,
  isPageDownKey,
  isPageUpKey,
} from "../keymap.js";
import { ensureTerminalContext } from "./context.js";
import type { DerivedTagTerminalInputEvent } from "./types.js";

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
    isCommandPaletteKey: () => normalized === ":" || printable === ":",
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
  };
}
