import type { DerivedTagLegacySeedMigrationCategory } from "../../domain/index.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "./creature.js";
import { HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "./hazard.js";
import { SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "./spell.js";

export {
  CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
  HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
  SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
};

export const DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY: Record<
  DerivedTagManagedCategory,
  DerivedTagLegacySeedMigrationCategory | undefined
> = {
  affliction: undefined,
  creature: CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
  equipment: undefined,
  hazard: HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
  spell: SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
};
