import { firstString, getNested, toStringArray } from "../shared/nested-values.js";

export { firstString, getNested, toStringArray };

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
    .replace(/&quot;/g, '"')
    .replace(/\s+\n/g, "\n")
    .replace(/\n\s+/g, "\n")
    .replace(/[ \t]+/g, " ")
    .trim();

  return withoutTags.length > 0 ? withoutTags : null;
}
