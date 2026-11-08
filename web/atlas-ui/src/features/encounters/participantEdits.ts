export function optionalIntegerInput(value: string): number | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return null;
  }
  const parsed = Number(trimmed);
  return Number.isSafeInteger(parsed) ? parsed : null;
}

export function asNumber(value: number | undefined): number {
  return value === undefined ? 0 : value;
}

export function displayNumber(value: number | undefined): string {
  return value === undefined ? "--" : value.toString();
}
