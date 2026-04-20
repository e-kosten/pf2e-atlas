export function isApplicationExitKey(normalizedKey: string): boolean {
  return normalizedKey === "ctrl_c" || normalizedKey === "q";
}

export function isMoveLeftKey(normalizedKey: string): boolean {
  return normalizedKey === "left" || normalizedKey === "h";
}

export function isMoveRightKey(normalizedKey: string): boolean {
  return normalizedKey === "right" || normalizedKey === "l";
}

export function isBackNavigationKey(normalizedKey: string): boolean {
  return normalizedKey === "backspace" || normalizedKey === "escape" || isMoveLeftKey(normalizedKey);
}

export function isBackOrExitKey(normalizedKey: string): boolean {
  return isApplicationExitKey(normalizedKey) || isBackNavigationKey(normalizedKey);
}

export function isMoveUpKey(normalizedKey: string): boolean {
  return normalizedKey === "up" || normalizedKey === "k";
}

export function isMoveDownKey(normalizedKey: string): boolean {
  return normalizedKey === "down" || normalizedKey === "j";
}

export function isPageUpKey(normalizedKey: string): boolean {
  return normalizedKey === "page_up" || normalizedKey === "b";
}

export function isPageDownKey(normalizedKey: string): boolean {
  return normalizedKey === "page_down" || normalizedKey === "f";
}

export function isConfirmKey(normalizedKey: string): boolean {
  return normalizedKey === "enter";
}

export function isConfirmOrToggleKey(normalizedKey: string): boolean {
  return normalizedKey === "enter" || normalizedKey === "space";
}

export type TerminalCycleDirection = 1 | -1;

export function getCycleDirection(normalizedKey: string): TerminalCycleDirection | undefined {
  if (isConfirmOrToggleKey(normalizedKey)) {
    return 1;
  }
  return undefined;
}

export function isHelpKey(normalizedKey: string): boolean {
  return normalizedKey === "?";
}

export function isCommandPaletteKey(normalizedKey: string): boolean {
  return normalizedKey === ":";
}

export function isFocusToggleKey(normalizedKey: string): boolean {
  return normalizedKey === "tab" || normalizedKey === "shift_tab" || normalizedKey === "w";
}

export function isLayoutToggleKey(normalizedKey: string): boolean {
  return normalizedKey === "z";
}

export function isSearchKey(normalizedKey: string): boolean {
  return normalizedKey === "slash";
}
