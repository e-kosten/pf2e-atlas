import type { SearchCategory } from "../types.js";

export type DerivedTagManagedCategory = "affliction" | "creature" | "equipment" | "hazard" | "spell";

export type DerivedTagCategoryManifestEntry = {
  category: DerivedTagManagedCategory;
  label: string;
  exportPrefix: string;
  editorialOrder: number;
  registrationOrder: number;
  supportsLegacySeedMigration: boolean;
};

export const DERIVED_TAG_CATEGORY_MANIFEST = [
  {
    category: "affliction",
    label: "Affliction",
    exportPrefix: "AFFLICTION",
    editorialOrder: 0,
    registrationOrder: 3,
    supportsLegacySeedMigration: false,
  },
  {
    category: "creature",
    label: "Creature",
    exportPrefix: "CREATURE",
    editorialOrder: 1,
    registrationOrder: 4,
    supportsLegacySeedMigration: true,
  },
  {
    category: "equipment",
    label: "Equipment",
    exportPrefix: "EQUIPMENT",
    editorialOrder: 2,
    registrationOrder: 0,
    supportsLegacySeedMigration: false,
  },
  {
    category: "hazard",
    label: "Hazard",
    exportPrefix: "HAZARD",
    editorialOrder: 3,
    registrationOrder: 2,
    supportsLegacySeedMigration: true,
  },
  {
    category: "spell",
    label: "Spell",
    exportPrefix: "SPELL",
    editorialOrder: 4,
    registrationOrder: 1,
    supportsLegacySeedMigration: true,
  },
] as const satisfies readonly DerivedTagCategoryManifestEntry[];

const DERIVED_TAG_CATEGORY_ENTRY_BY_CATEGORY = new Map(
  DERIVED_TAG_CATEGORY_MANIFEST.map((entry) => [entry.category, entry] as const),
);

function sortCategoryManifest(
  compare: (left: DerivedTagCategoryManifestEntry, right: DerivedTagCategoryManifestEntry) => number,
): DerivedTagCategoryManifestEntry[] {
  return [...DERIVED_TAG_CATEGORY_MANIFEST].sort(compare);
}

export const DERIVED_TAG_MANAGED_CATEGORIES = sortCategoryManifest(
  (left, right) => left.editorialOrder - right.editorialOrder,
).map((entry) => entry.category) as readonly DerivedTagManagedCategory[];

export const DERIVED_TAG_REGISTRATION_CATEGORIES = sortCategoryManifest(
  (left, right) => left.registrationOrder - right.registrationOrder,
).map((entry) => entry.category) as readonly DerivedTagManagedCategory[];

export function getDerivedTagCategoryManifestEntry(
  category: DerivedTagManagedCategory,
): DerivedTagCategoryManifestEntry {
  const entry = DERIVED_TAG_CATEGORY_ENTRY_BY_CATEGORY.get(category);
  if (!entry) {
    throw new Error(`Missing derived-tag category manifest entry for "${category}".`);
  }
  return entry;
}

export function isDerivedTagManagedCategory(category: SearchCategory): category is DerivedTagManagedCategory {
  return DERIVED_TAG_CATEGORY_ENTRY_BY_CATEGORY.has(category as DerivedTagManagedCategory);
}

export function expectDerivedTagManagedCategory(category: SearchCategory, consumer: string): DerivedTagManagedCategory {
  if (!isDerivedTagManagedCategory(category)) {
    throw new Error(`${consumer} does not manage category "${category}".`);
  }
  return category;
}
