import type { SavedListSummaryView } from "../../generated/atlas";

export function InlineError({ message }: { message: string }) {
  return <div className="error-banner">{message}</div>;
}

export function listCountLabel(count: number): string {
  return count === 1 ? "1 saved list" : `${count.toLocaleString()} saved lists`;
}

export function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function normalizeOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

export function normalizeTags(values: string[] | undefined): string[] {
  const seen = new Set<string>();
  return (values ?? [])
    .map((value) => value.trim())
    .filter((value) => value.length > 0)
    .filter((value) => {
      const key = value.toLowerCase();
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    })
    .sort((left, right) =>
      left.localeCompare(right, undefined, { sensitivity: "base" }),
    );
}

export function savedListTagOptions(lists: SavedListSummaryView[]): string[] {
  const tags = new Set<string>();
  for (const list of lists) {
    for (const tag of list.tags) {
      tags.add(tag);
    }
  }
  return [...tags].sort((left, right) =>
    left.localeCompare(right, undefined, { sensitivity: "base" }),
  );
}
