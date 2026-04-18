import type { DerivedTagLegacySeedMigrationCategory } from "../../types.js";

// Temporary migration bucket for legacy carried-over "seed" records that are still
// applied live but need manual review to become either true exemplars or assignments.
export const HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS = {
  category: "hazard",
  tags: [
    {
      tag: "ward_trigger",
      includeRecords: [
        {
          pack: "sky-kings-tomb-bestiary",
          name: "Buried Shock Glyph",
        },
        {
          pack: "shades-of-blood-bestiary",
          name: "Conundrum Ward",
        },
        {
          pack: "quest-for-the-frozen-flame-bestiary",
          name: "Death's Slumber Ward",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Glyph of Warding (B8)",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Glyph of Warding (D5)",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Glyph of Warding (Kingmaker)",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Kharnas's Lesser Glyph",
        },
        {
          pack: "extinction-curse-bestiary",
          name: "Krooth Summoning Rune",
        },
        {
          pack: "age-of-ashes-bestiary",
          name: "Luminous Ward",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Mask Summoning Rune",
        },
        {
          pack: "extinction-curse-bestiary",
          name: "Mukradi Summoning Runes",
        },
        {
          pack: "hazards",
          name: "Pharaoh's Ward",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Serpent Ward",
        },
        {
          pack: "the-slithering-bestiary",
          name: "Stalker Summoning Rune",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Stinger Ward Trap",
        },
        {
          pack: "hazards",
          name: "Summoning Rune",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Summoning Rune (Barbazu Devil)",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Summoning Rune (Cinder Rat)",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Summoning Rune (Cockatrice)",
        },
        {
          pack: "season-of-ghosts-bestiary",
          name: "Witherweird Runes",
        },
      ],
    },
    {
      tag: "threshold_lockdown",
      includeRecords: [
        {
          pack: "curtain-call-bestiary",
          name: "Archway Barrier",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Harmonic Barrier",
        },
      ],
    },
    {
      tag: "planar_breach",
      includeRecords: [
        {
          pack: "pathfinder-dark-archive",
          name: "Exhaling Portal",
        },
        {
          pack: "hazards",
          name: "Planar Rift",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Time Rift",
        },
      ],
    },
    {
      tag: "restraint_capture",
      includeRecords: [
        {
          pack: "book-of-the-dead-bestiary",
          name: "Phantom Jailer",
        },
      ],
    },
    {
      tag: "barrier_lockdown",
      includeRecords: [
        {
          pack: "curtain-call-bestiary",
          name: "Archway Barrier",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Harmonic Barrier",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Subduing Gas Chamber",
        },
      ],
    },
    {
      tag: "spawned_attackers",
      includeRecords: [
        {
          pack: "extinction-curse-bestiary",
          name: "Mukradi Summoning Runes",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Haunted Aiudara",
        },
        {
          pack: "hazards",
          name: "Malevolent Mannequins",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Mocking Puppets",
        },
        {
          pack: "hellbreakers-bestiary",
          name: "Play Time",
        },
      ],
    },
    {
      tag: "phantom_assailants",
      includeRecords: [
        {
          pack: "gatewalkers-bestiary",
          name: "Haunted Aiudara",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Ghost Stampede",
        },
        {
          pack: "book-of-the-dead-bestiary",
          name: "Phantom Jailer",
        },
        {
          pack: "curtain-call-bestiary",
          name: "Spectral Opera",
        },
        {
          pack: "triumph-of-the-tusk-bestiary",
          name: "Trampling Livestock",
        },
      ],
    },
    {
      tag: "fire_hazard",
      includeRecords: [
        {
          pack: "spore-war-bestiary",
          name: "Enhanced Fireball Rune",
        },
        {
          pack: "hazards",
          name: "Fireball Rune",
        },
        {
          pack: "hazards",
          name: "Eternal Flame",
        },
        {
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Floating Flamethrower",
        },
        {
          pack: "quest-for-the-frozen-flame-bestiary",
          name: "Forest Fire",
        },
        {
          pack: "shades-of-blood-bestiary",
          name: "Furious Flame",
        },
        {
          pack: "claws-of-the-tyrant-bestiary",
          name: "Iomedae's Flame",
        },
        {
          pack: "stolen-fate-bestiary",
          name: "Stage Fire",
        },
        {
          pack: "kingmaker-bestiary",
          name: "Stygian Fires",
        },
        {
          pack: "troubles-in-otari-bestiary",
          name: "Tongues of Flame",
        },
        {
          pack: "blood-lords-bestiary",
          name: "Unstable Fiendflame Cage",
        },
      ],
    },
    {
      tag: "poison_hazard",
      includeRecords: [
        {
          pack: "extinction-curse-bestiary",
          name: "Dream Pollen Pod",
        },
        {
          pack: "extinction-curse-bestiary",
          name: "Envenomed Thorns Trap",
        },
        {
          pack: "prey-for-death-bestiary",
          name: "Flensing Floor",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Formian Sting Trench",
        },
        {
          pack: "agents-of-edgewatch-bestiary",
          name: "Needling Stairs",
        },
        {
          pack: "one-shot-bestiary",
          name: "Poisoned Lock (Lionlodge)",
        },
        {
          pack: "extinction-curse-bestiary",
          name: "Poisoned Secret Door Trap",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Poisonous Atmosphere",
        },
        {
          pack: "hazards",
          name: "Poisonous Mold",
        },
        {
          pack: "wardens-of-wildwood-bestiary",
          name: "Poisonous Pollen Trap",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Soporific Lecture",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Subduing Gas Chamber",
        },
        {
          pack: "strength-of-thousands-bestiary",
          name: "Venom Pool",
        },
        {
          pack: "hazards",
          name: "Yellow Mold",
        },
      ],
    },
    {
      tag: "respiratory_hazard",
      includeRecords: [
        {
          pack: "gatewalkers-bestiary",
          name: "Poisonous Atmosphere",
        },
        {
          pack: "gatewalkers-bestiary",
          name: "Soporific Lecture",
        },
        {
          pack: "outlaws-of-alkenstar-bestiary",
          name: "Subduing Gas Chamber",
        },
      ],
    },
  ],
} satisfies DerivedTagLegacySeedMigrationCategory;
