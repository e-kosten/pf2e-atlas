import type { DerivedTagLegacySeedMigrationCategory } from "../../types.js";

// Temporary migration bucket for legacy carried-over "seed" records that are still
// applied live but need manual review to become either true exemplars or assignments.
export const CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS: DerivedTagLegacySeedMigrationCategory = {
  category: "creature",
  tags: [
    {
      tag: "faceless_horror",
      includeRecords: [
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Arcane)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Divine)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Martial)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Occult)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Primal)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Noppera-Bo Impersonator (Skilled)",
        },
      ],
    },
  ],
};
