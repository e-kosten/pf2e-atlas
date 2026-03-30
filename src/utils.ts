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

export function stripHtml(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }

  const withoutTags = value
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<\/p>/gi, "\n")
    .replace(/<[^>]+>/g, " ")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#39;/g, "'")
    .replace(/&quot;/g, "\"")
    .replace(/\s+\n/g, "\n")
    .replace(/\n\s+/g, "\n")
    .replace(/[ \t]+/g, " ")
    .trim();

  return withoutTags.length > 0 ? withoutTags : null;
}

export function clampLimit(value: number | undefined, fallback = 20, max = 100): number {
  if (value === undefined || Number.isNaN(value)) {
    return fallback;
  }

  return Math.max(1, Math.min(max, Math.trunc(value)));
}

export function clampOffset(value: number | undefined): number {
  if (value === undefined || Number.isNaN(value)) {
    return 0;
  }

  return Math.max(0, Math.trunc(value));
}

export function firstString(...values: unknown[]): string | null {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) {
      return value;
    }
  }

  return null;
}

export function getNested(value: unknown, path: string[]): unknown {
  let current: unknown = value;

  for (const segment of path) {
    if (!current || typeof current !== "object" || !(segment in current)) {
      return undefined;
    }

    current = (current as Record<string, unknown>)[segment];
  }

  return current;
}

export function toStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.filter((entry): entry is string => typeof entry === "string" && entry.length > 0);
}

export function uniqueSorted(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

export function bigramDice(query: string, candidate: string): number {
  if (query === candidate) {
    return 1;
  }

  if (query.length < 2 || candidate.length < 2) {
    return query === candidate ? 1 : 0;
  }

  const queryBigrams = new Map<string, number>();
  for (let index = 0; index < query.length - 1; index += 1) {
    const slice = query.slice(index, index + 2);
    queryBigrams.set(slice, (queryBigrams.get(slice) ?? 0) + 1);
  }

  let overlap = 0;
  for (let index = 0; index < candidate.length - 1; index += 1) {
    const slice = candidate.slice(index, index + 2);
    const count = queryBigrams.get(slice) ?? 0;
    if (count > 0) {
      overlap += 1;
      queryBigrams.set(slice, count - 1);
    }
  }

  return (2 * overlap) / ((query.length - 1) + (candidate.length - 1));
}
