export function expandHome(pathValue: string): string {
  if (pathValue === "~") {
    return process.env.HOME ?? pathValue;
  }

  if (pathValue.startsWith("~/")) {
    return `${process.env.HOME ?? ""}/${pathValue.slice(2)}`;
  }

  return pathValue;
}

export function normalizeText(value: string): string {
  return value
    .toLowerCase()
    .replace(/&nbsp;/g, " ")
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

export function uniqueSorted<T extends string>(values: T[]): T[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}
