import type { DerivedTagCatalogEntry, DerivedTagCatalogTag, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import type { DerivedTagContext } from "./matcher.js";
import { normalizeDerivedTag } from "./shared.js";

function entryAppliesToSubcategory(
  entry: DerivedTagCatalogEntry,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!entry.subcategories || entry.subcategories.length === 0) {
    return true;
  }

  return subcategory !== null && entry.subcategories.includes(subcategory);
}

function appendCatalogTag(
  tags: DerivedTagCatalogTag[],
  tag: DerivedTagCatalogTag,
): DerivedTagCatalogTag[] {
  const normalized = normalizeDerivedTag(tag.value);
  if (tags.some((candidate) => normalizeDerivedTag(candidate.value) === normalized)) {
    return tags;
  }

  return [...tags, tag];
}

export function publishDerivedTagCatalog(
  catalog: DerivedTagCatalogEntry[],
): DerivedTagCatalogEntry[] {
  return catalog.map((entry) => {
    if (!entry.promoteFamilyToTag) {
      return entry;
    }

    return {
      ...entry,
      tags: appendCatalogTag(entry.tags, {
        value: normalizeDerivedTag(entry.family),
        description: entry.description,
      }),
    };
  });
}

export function applyPromotedFamilyTags(
  catalog: DerivedTagCatalogEntry[],
  input: Pick<DerivedTagContext, "category" | "subcategory">,
  tags: string[],
): string[] {
  const expanded = new Set(tags.map((tag) => normalizeDerivedTag(tag)));

  for (const entry of catalog) {
    if (!entry.promoteFamilyToTag || entry.category !== input.category) {
      continue;
    }
    if (!entryAppliesToSubcategory(entry, input.subcategory)) {
      continue;
    }

    const hasMatchingChildTag = entry.tags.some((tag) => expanded.has(normalizeDerivedTag(tag.value)));
    if (hasMatchingChildTag) {
      expanded.add(normalizeDerivedTag(entry.family));
    }
  }

  return uniqueSorted([...expanded]);
}
