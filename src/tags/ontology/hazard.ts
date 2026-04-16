import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../types.js";

export const HAZARD_DERIVED_TAG_ONTOLOGY_FAMILIES: DerivedTagOntologyFamily[] = [
  {
    category: "hazard",
    family: "mechanism",
    description: "Hazards whose threat comes from trigger mechanisms, locking thresholds, control surfaces, or planar breaches."
  },
  {
    category: "hazard",
    family: "function",
    description: "Hazard practical-function tags for alerts and restraint effects."
  },
  {
    category: "hazard",
    family: "haunt_manifestation",
    description: "Haunt manifestations that materially change the encounter through attackers, lures, or life-draining effects."
  },
  {
    category: "hazard",
    family: "impact",
    description: "Hazard impact tags for mental destabilization and movement-limiting effects."
  },
  {
    category: "hazard",
    family: "environmental_danger",
    description: "Hazards defined by recurring elemental or toxic environmental threats."
  },
  {
    category: "hazard",
    family: "forced_position",
    description: "Hazards that drop, collapse, or forcibly reposition creatures."
  },
  {
    category: "hazard",
    family: "perception_control",
    description: "Hazards that distort routes, orientation, or the perceived layout of a space."
  },
  {
    category: "hazard",
    family: "attack_vector",
    description: "Hazards categorized by how their attack is delivered into the scene."
  }
];

export const HAZARD_DERIVED_TAG_ONTOLOGY_TAGS: DerivedTagOntologyTag[] = [
  {
    category: "hazard",
    family: "mechanism",
    tag: "ward_trigger",
    description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "sky-kings-tomb-bestiary",
        name: "Buried Shock Glyph"
      },
      {
        pack: "shades-of-blood-bestiary",
        name: "Conundrum Ward"
      },
      {
        pack: "quest-for-the-frozen-flame-bestiary",
        name: "Death's Slumber Ward"
      },
      {
        pack: "blood-lords-bestiary",
        name: "Glyph of Warding (B8)"
      },
      {
        pack: "blood-lords-bestiary",
        name: "Glyph of Warding (D5)"
      },
      {
        pack: "kingmaker-bestiary",
        name: "Glyph of Warding (Kingmaker)"
      },
      {
        pack: "agents-of-edgewatch-bestiary",
        name: "Kharnas's Lesser Glyph"
      },
      {
        pack: "extinction-curse-bestiary",
        name: "Krooth Summoning Rune"
      },
      {
        pack: "age-of-ashes-bestiary",
        name: "Luminous Ward"
      },
      {
        pack: "curtain-call-bestiary",
        name: "Mask Summoning Rune"
      },
      {
        pack: "extinction-curse-bestiary",
        name: "Mukradi Summoning Runes"
      },
      {
        pack: "hazards",
        name: "Pharaoh's Ward"
      },
      {
        pack: "strength-of-thousands-bestiary",
        name: "Serpent Ward"
      },
      {
        pack: "the-slithering-bestiary",
        name: "Stalker Summoning Rune"
      },
      {
        pack: "strength-of-thousands-bestiary",
        name: "Stinger Ward Trap"
      },
      {
        pack: "hazards",
        name: "Summoning Rune"
      },
      {
        pack: "agents-of-edgewatch-bestiary",
        name: "Summoning Rune (Barbazu Devil)"
      },
      {
        pack: "agents-of-edgewatch-bestiary",
        name: "Summoning Rune (Cinder Rat)"
      },
      {
        pack: "blood-lords-bestiary",
        name: "Summoning Rune (Cockatrice)"
      },
      {
        pack: "season-of-ghosts-bestiary",
        name: "Witherweird Runes"
      }
    ]
  },
  {
    category: "hazard",
    family: "mechanism",
    tag: "pressure_trigger",
    description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "mechanism",
    tag: "tripwire_trigger",
    description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "mechanism",
    tag: "threshold_lockdown",
    description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "curtain-call-bestiary",
        name: "Archway Barrier"
      },
      {
        pack: "curtain-call-bestiary",
        name: "Harmonic Barrier"
      }
    ]
  },
  {
    category: "hazard",
    family: "mechanism",
    tag: "control_interface",
    description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "mechanism",
    tag: "planar_breach",
    description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "pathfinder-dark-archive",
        name: "Exhaling Portal"
      },
      {
        pack: "hazards",
        name: "Planar Rift"
      },
      {
        pack: "blood-lords-bestiary",
        name: "Time Rift"
      }
    ]
  },
  {
    category: "hazard",
    family: "function",
    tag: "alarm",
    description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "function",
    tag: "restraint_capture",
    description: "Hazard that binds, restrains, or holds intruders in place.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "book-of-the-dead-bestiary",
        name: "Phantom Jailer"
      }
    ]
  },
  {
    category: "hazard",
    family: "function",
    tag: "barrier_lockdown",
    description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "curtain-call-bestiary",
        name: "Archway Barrier"
      },
      {
        pack: "curtain-call-bestiary",
        name: "Harmonic Barrier"
      },
      {
        pack: "outlaws-of-alkenstar-bestiary",
        name: "Subduing Gas Chamber"
      }
    ]
  },
  {
    category: "hazard",
    family: "function",
    tag: "spawned_attackers",
    description: "Hazard that summons, creates, or releases separate attackers into the scene.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "extinction-curse-bestiary",
        name: "Mukradi Summoning Runes"
      },
      {
        pack: "gatewalkers-bestiary",
        name: "Haunted Aiudara"
      },
      {
        pack: "hazards",
        name: "Malevolent Mannequins"
      },
      {
        pack: "curtain-call-bestiary",
        name: "Mocking Puppets"
      },
      {
        pack: "hellbreakers-bestiary",
        name: "Play Time"
      }
    ]
  },
  {
    category: "hazard",
    family: "haunt_manifestation",
    tag: "life_drain_hazard",
    description: "Haunt that drains life force, vitality, or souls from victims.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "haunt_manifestation",
    tag: "phantom_assailants",
    description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "gatewalkers-bestiary",
        name: "Haunted Aiudara"
      },
      {
        pack: "book-of-the-dead-bestiary",
        name: "Ghost Stampede"
      },
      {
        pack: "book-of-the-dead-bestiary",
        name: "Phantom Jailer"
      },
      {
        pack: "curtain-call-bestiary",
        name: "Spectral Opera"
      },
      {
        pack: "triumph-of-the-tusk-bestiary",
        name: "Trampling Livestock"
      }
    ]
  },
  {
    category: "hazard",
    family: "haunt_manifestation",
    tag: "lure_compulsion",
    description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "haunt_manifestation",
    tag: "battlefield_disruption",
    description: "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "impact",
    tag: "mental_impairment",
    description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "impact",
    tag: "mobility_impairment",
    description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "acid_hazard",
    description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "cold_hazard",
    description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "fire_hazard",
    description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spore-war-bestiary",
        name: "Enhanced Fireball Rune"
      },
      {
        pack: "hazards",
        name: "Fireball Rune"
      },
      {
        pack: "hazards",
        name: "Eternal Flame"
      },
      {
        pack: "fists-of-the-ruby-phoenix-bestiary",
        name: "Floating Flamethrower"
      },
      {
        pack: "quest-for-the-frozen-flame-bestiary",
        name: "Forest Fire"
      },
      {
        pack: "shades-of-blood-bestiary",
        name: "Furious Flame"
      },
      {
        pack: "claws-of-the-tyrant-bestiary",
        name: "Iomedae's Flame"
      },
      {
        pack: "stolen-fate-bestiary",
        name: "Stage Fire"
      },
      {
        pack: "kingmaker-bestiary",
        name: "Stygian Fires"
      },
      {
        pack: "troubles-in-otari-bestiary",
        name: "Tongues of Flame"
      },
      {
        pack: "blood-lords-bestiary",
        name: "Unstable Fiendflame Cage"
      }
    ]
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "electric_hazard",
    description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "poison_hazard",
    description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "extinction-curse-bestiary",
        name: "Dream Pollen Pod"
      },
      {
        pack: "extinction-curse-bestiary",
        name: "Envenomed Thorns Trap"
      },
      {
        pack: "prey-for-death-bestiary",
        name: "Flensing Floor"
      },
      {
        pack: "gatewalkers-bestiary",
        name: "Formian Sting Trench"
      },
      {
        pack: "agents-of-edgewatch-bestiary",
        name: "Needling Stairs"
      },
      {
        pack: "one-shot-bestiary",
        name: "Poisoned Lock (Lionlodge)"
      },
      {
        pack: "extinction-curse-bestiary",
        name: "Poisoned Secret Door Trap"
      },
      {
        pack: "gatewalkers-bestiary",
        name: "Poisonous Atmosphere"
      },
      {
        pack: "hazards",
        name: "Poisonous Mold"
      },
      {
        pack: "wardens-of-wildwood-bestiary",
        name: "Poisonous Pollen Trap"
      },
      {
        pack: "gatewalkers-bestiary",
        name: "Soporific Lecture"
      },
      {
        pack: "outlaws-of-alkenstar-bestiary",
        name: "Subduing Gas Chamber"
      },
      {
        pack: "strength-of-thousands-bestiary",
        name: "Venom Pool"
      },
      {
        pack: "hazards",
        name: "Yellow Mold"
      }
    ]
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "respiratory_hazard",
    description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "gatewalkers-bestiary",
        name: "Poisonous Atmosphere"
      },
      {
        pack: "gatewalkers-bestiary",
        name: "Soporific Lecture"
      },
      {
        pack: "outlaws-of-alkenstar-bestiary",
        name: "Subduing Gas Chamber"
      }
    ]
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "sound_hazard",
    description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "environmental_danger",
    tag: "water_hazard",
    description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
    assignmentMode: "hybrid"
  },
  {
    category: "hazard",
    family: "forced_position",
    tag: "pitfall",
    description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "forced_position",
    tag: "collapse_hazard",
    description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "forced_position",
    tag: "forced_movement",
    description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "perception_control",
    tag: "navigation_disruption",
    description: "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "perception_control",
    tag: "illusion_assault",
    description: "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "attack_vector",
    tag: "overhead_strike",
    description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
    assignmentMode: "deterministic"
  },
  {
    category: "hazard",
    family: "attack_vector",
    tag: "projectile_emitter",
    description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
    assignmentMode: "deterministic"
  }
];
