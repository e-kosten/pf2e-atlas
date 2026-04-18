import { firstString, getNested, normalizeText, stripHtml, toStringArray, uniqueSorted } from "../utils.js";

const UUID_REFERENCE_PATTERN = /@UUID\[([^\]]+)\](?:\{([^}]+)\})?/g;

export type AfflictionFamily = "curse" | "disease" | "poison";

export function getRecordTraits(raw: Record<string, unknown>): string[] {
  return uniqueSorted(toStringArray(getNested(raw, ["system", "traits", "value"])));
}

export function getRecordDescriptionMarkup(raw: Record<string, unknown>): string | null {
  return firstString(
    getNested(raw, ["system", "description", "value"]),
    getNested(raw, ["system", "details", "description"]),
    getNested(raw, ["system", "details", "publicNotes"]),
    getNested(raw, ["system", "details", "blurb"]),
  );
}

export function getRecordDescriptionText(raw: Record<string, unknown>): string | null {
  return stripHtml(getRecordDescriptionMarkup(raw));
}

export function getRecordBlurbMarkup(raw: Record<string, unknown>): string | null {
  return firstString(getNested(raw, ["system", "details", "blurb"]));
}

export function getRecordBlurbText(raw: Record<string, unknown>): string | null {
  return stripHtml(getRecordBlurbMarkup(raw));
}

function fallbackLinkedName(locator: string): string | null {
  const trimmed = locator.trim();
  if (!trimmed) {
    return null;
  }

  const segments = trimmed.split(".");
  const tail = segments[segments.length - 1] ?? trimmed;
  const normalizedTail = tail.replace(/[-_]+/g, " ").trim();
  return normalizedTail.length > 0 ? normalizedTail : null;
}

export function extractLinkedNamesFromMarkup(markup: string | null | undefined): string[] {
  if (!markup) {
    return [];
  }

  const names: string[] = [];
  for (const match of markup.matchAll(UUID_REFERENCE_PATTERN)) {
    const displayText = match[2]?.trim() ?? "";
    if (displayText.length > 0) {
      names.push(displayText);
      continue;
    }

    const fallback = fallbackLinkedName(match[1] ?? "");
    if (fallback) {
      names.push(fallback);
    }
  }

  return uniqueSorted(names);
}

export function getRecordSlug(raw: Record<string, unknown>): string | null {
  return firstString(getNested(raw, ["system", "slug"]));
}

export function detectAfflictionFamily(raw: Record<string, unknown>): AfflictionFamily | null {
  const normalizedTraits = new Set(
    getRecordTraits(raw)
      .map((trait) => normalizeText(trait))
      .filter(Boolean),
  );
  const systemCategory = normalizeText(firstString(getNested(raw, ["system", "category"])) ?? "");

  if (normalizedTraits.has("disease") || systemCategory === "disease") {
    return "disease";
  }

  if (normalizedTraits.has("poison") || systemCategory === "poison") {
    return "poison";
  }

  if (normalizedTraits.has("curse") || systemCategory === "curse") {
    return "curse";
  }

  return null;
}

export function hasAfflictionShape(raw: Record<string, unknown>): boolean {
  const descriptionText = getRecordDescriptionText(raw);
  if (!descriptionText) {
    return false;
  }

  return /saving throw/i.test(descriptionText) && /stage 1/i.test(descriptionText);
}

export function buildEmbeddedItemSearchChunks(raw: Record<string, unknown>, maxChunks: number | null = null): string[] {
  const chunks: string[] = [];
  const seen = new Set<string>();
  const items = getNested(raw, ["items"]);
  if (!Array.isArray(items)) {
    return chunks;
  }

  for (const entry of items) {
    if (!entry || typeof entry !== "object") {
      continue;
    }

    const item = entry as Record<string, unknown>;
    const values = [
      firstString(item.name),
      ...getRecordTraits(item),
      ...extractLinkedNamesFromMarkup(getRecordDescriptionMarkup(item)),
    ];

    for (const value of values) {
      if (!value) {
        continue;
      }

      const normalized = normalizeText(value);
      if (!normalized || seen.has(normalized)) {
        continue;
      }

      seen.add(normalized);
      chunks.push(value.trim());

      if (maxChunks !== null && chunks.length >= maxChunks) {
        return chunks;
      }
    }
  }

  return chunks;
}
