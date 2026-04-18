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
  return normalizedKey === "page_down";
}

export function isConfirmKey(normalizedKey: string): boolean {
  return normalizedKey === "enter";
}

export function isHelpKey(normalizedKey: string): boolean {
  return normalizedKey === "?";
}
