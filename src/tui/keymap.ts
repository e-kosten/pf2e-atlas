export function isApplicationExitKey(normalizedKey: string): boolean {
  return normalizedKey === "ctrl_c" || normalizedKey === "q";
}

export function isBackNavigationKey(normalizedKey: string): boolean {
  return normalizedKey === "backspace" || normalizedKey === "left" || normalizedKey === "escape";
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

export function isConfirmKey(normalizedKey: string): boolean {
  return normalizedKey === "enter";
}

export function isHelpKey(normalizedKey: string): boolean {
  return normalizedKey === "?";
}
