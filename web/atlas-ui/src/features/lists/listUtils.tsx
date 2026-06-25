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
