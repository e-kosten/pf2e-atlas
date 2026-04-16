import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../types.js";

export const SPELL_DERIVED_TAG_ONTOLOGY_FAMILIES: DerivedTagOntologyFamily[] = [
  {
    category: "spell",
    family: "infiltration",
    description: "Appearance-changing and social-passing spells."
  },
  {
    category: "spell",
    family: "communication",
    description: "Spells for signaling, telepathy, and message exchange."
  },
  {
    category: "spell",
    family: "reconnaissance",
    description: "Remote-observation and scouting spells."
  },
  {
    category: "spell",
    family: "wayfinding",
    description: "Spells that guide direction, route-finding, or destination travel."
  },
  {
    category: "spell",
    family: "traversal",
    description: "Spells that improve movement modes, speed, or practical traversal."
  },
  {
    category: "spell",
    family: "transformation",
    description: "Spells that alter a creature's body, form, or battle shape."
  },
  {
    category: "spell",
    family: "control",
    description: "Spells that pressure morale, obscure sight, or reshape the battlefield."
  },
  {
    category: "spell",
    family: "support",
    description: "Spells that restore, protect, ward, or reinforce allies and targets."
  },
  {
    category: "spell",
    family: "attrition",
    description: "Spells that inflict ongoing harm through lingering persistent damage."
  },
  {
    category: "spell",
    family: "expedition",
    description: "Spells that support travel, survival, and aquatic operations."
  },
  {
    category: "spell",
    family: "tempo",
    description: "Spells that improve action economy or accelerate allies."
  },
  {
    category: "spell",
    family: "summoner_support",
    description: "Spells that specifically protect, enhance, or reposition an eidolon."
  },
  {
    category: "spell",
    family: "magic_interference",
    description: "Spells that disrupt, dispel, or suppress magic."
  },
  {
    category: "spell",
    family: "security",
    description: "Area-warning and intrusion-alert spells."
  },
  {
    category: "spell",
    family: "impact",
    description: "Spells that impair minds or senses, forcibly reposition targets, or trap them in place."
  }
];

export const SPELL_DERIVED_TAG_ONTOLOGY_TAGS: DerivedTagOntologyTag[] = [
  {
    category: "spell",
    family: "infiltration",
    tag: "disguise",
    description: "Helps alter appearance or impersonate another identity.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "infiltration",
    tag: "social_infiltration",
    description: "Helps blend into a group or pass under social scrutiny.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "communication",
    tag: "signaling",
    description: "Helps draw attention, mark a location, or coordinate allies.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "communication",
    tag: "message_delivery",
    description: "Sends, stores, or relays actual content across time or distance.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "reconnaissance",
    tag: "scouting",
    description: "Helps observe at a distance, extend senses, or locate a target.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Enhance Senses"
      },
      {
        pack: "spells-srd",
        name: "Grave Impressions"
      },
      {
        pack: "spells-srd",
        name: "Hunter's Vision"
      },
      {
        pack: "spells-srd",
        name: "Locate"
      }
    ]
  },
  {
    category: "spell",
    family: "wayfinding",
    tag: "navigation",
    description: "Helps orient, guide a route, or identify a destination's direction.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Show the Way"
      },
      {
        pack: "spells-srd",
        name: "Return Beacon"
      },
      {
        pack: "spells-srd",
        name: "Nature's Pathway"
      },
      {
        pack: "spells-srd",
        name: "Fire's Pathway"
      },
      {
        pack: "spells-srd",
        name: "Interplanar Teleport"
      }
    ]
  },
  {
    category: "spell",
    family: "traversal",
    tag: "mobility",
    description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Air Walk"
      },
      {
        pack: "spells-srd",
        name: "Airlift"
      },
      {
        pack: "spells-srd",
        name: "Dive and Breach"
      },
      {
        pack: "spells-srd",
        name: "Friendly Push"
      },
      {
        pack: "spells-srd",
        name: "Levitate"
      },
      {
        pack: "spells-srd",
        name: "Migration"
      },
      {
        pack: "spells-srd",
        name: "Water Walk"
      },
      {
        pack: "spells-srd",
        name: "Nature's Pathway"
      },
      {
        pack: "spells-srd",
        name: "Fire's Pathway"
      }
    ]
  },
  {
    category: "spell",
    family: "transformation",
    tag: "transformation",
    description: "Spells that alter a creature's body, form, or battle shape.",
    assignmentMode: "composite",
    adjacentTags: [
      "battle_form",
      "animal_form",
      "elemental_form"
    ],
    compositeOfAnyTags: [
      "battle_form",
      "animal_form",
      "elemental_form"
    ]
  },
  {
    category: "spell",
    family: "transformation",
    tag: "battle_form",
    description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "transformation",
    tag: "animal_form",
    description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "transformation",
    tag: "elemental_form",
    description: "Transforms a creature into an elemental form.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "control",
    tag: "fear_pressure",
    description: "Forces fear, panic, dread, or morale collapse onto a target.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "control",
    tag: "concealment",
    description: "Makes a creature hard to see, hidden, concealed, or undetected.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "control",
    tag: "line_of_sight_control",
    description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "control",
    tag: "battlefield_disruption",
    description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "support",
    tag: "healing_support",
    description: "Directly restores hit points or accelerates recovery.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Field of Life"
      },
      {
        pack: "spells-srd",
        name: "Life-Giving Form"
      },
      {
        pack: "spells-srd",
        name: "Life Link"
      },
      {
        pack: "spells-srd",
        name: "Regenerate"
      },
      {
        pack: "spells-srd",
        name: "Soothing Mist"
      },
      {
        pack: "spells-srd",
        name: "Stabilize"
      },
      {
        pack: "spells-srd",
        name: "Soothe"
      },
      {
        pack: "spells-srd",
        name: "Spiritual Renewal"
      },
      {
        pack: "spells-srd",
        name: "Positive Attunement"
      },
      {
        pack: "spells-srd",
        name: "Vital Beacon"
      },
      {
        pack: "spells-srd",
        name: "Life Boost"
      },
      {
        pack: "spells-srd",
        name: "Halcyon Mists"
      }
    ]
  },
  {
    category: "spell",
    family: "support",
    tag: "condition_support",
    description: "Delays, suppresses, or removes afflictions and conditions.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Clear Mind"
      },
      {
        pack: "spells-srd",
        name: "Restoration"
      },
      {
        pack: "spells-srd",
        name: "Cleansing Flames"
      },
      {
        pack: "spells-srd",
        name: "Delay Affliction"
      },
      {
        pack: "spells-srd",
        name: "Soothe"
      },
      {
        pack: "spells-srd",
        name: "Moment of Renewal"
      }
    ]
  },
  {
    category: "spell",
    family: "support",
    tag: "affliction_cleanup",
    description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Restoration"
      },
      {
        pack: "spells-srd",
        name: "Cleansing Flames"
      },
      {
        pack: "spells-srd",
        name: "Delay Affliction"
      },
      {
        pack: "spells-srd",
        name: "Moment of Renewal"
      }
    ]
  },
  {
    category: "spell",
    family: "support",
    tag: "escape_support",
    description: "Helps a creature slip away, break free, flee, or evade pursuit.",
    assignmentMode: "hybrid"
  },
  {
    category: "spell",
    family: "support",
    tag: "protective_ward",
    description: "Places a ward, sanctuary, shield, or protective boundary.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Protective Wards"
      },
      {
        pack: "spells-srd",
        name: "Spellmaster's Ward"
      },
      {
        pack: "spells-srd",
        name: "Phoenix Ward"
      },
      {
        pack: "spells-srd",
        name: "Divine Aura"
      },
      {
        pack: "spells-srd",
        name: "Mystic Armor"
      },
      {
        pack: "spells-srd",
        name: "Protector's Sphere"
      },
      {
        pack: "spells-srd",
        name: "Shielded Arm"
      },
      {
        pack: "spells-srd",
        name: "Warding Aggression"
      },
      {
        pack: "spells-srd",
        name: "Arcane Countermeasure"
      }
    ]
  },
  {
    category: "spell",
    family: "support",
    tag: "death_prevention",
    description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
    assignmentMode: "hybrid"
  },
  {
    category: "spell",
    family: "support",
    tag: "resistance_support",
    description: "Grants resistance or immunity against energy, damage, or hazards.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Phoenix Ward"
      },
      {
        pack: "spells-srd",
        name: "Protector's Sphere"
      },
      {
        pack: "spells-srd",
        name: "Fire Shield"
      },
      {
        pack: "spells-srd",
        name: "Elemental Gift"
      },
      {
        pack: "spells-srd",
        name: "Life-Giving Form"
      },
      {
        pack: "spells-srd",
        name: "Aerial Form"
      },
      {
        pack: "spells-srd",
        name: "Cosmic Form"
      },
      {
        pack: "spells-srd",
        name: "Element Embodied"
      }
    ]
  },
  {
    category: "spell",
    family: "support",
    tag: "temporary_hp_support",
    description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
    assignmentMode: "hybrid"
  },
  {
    category: "spell",
    family: "attrition",
    tag: "persistent_damage",
    description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Achaekek's Clutch"
      },
      {
        pack: "spells-srd",
        name: "Acid Arrow"
      },
      {
        pack: "spells-srd",
        name: "Acid Grip"
      },
      {
        pack: "spells-srd",
        name: "Acid Splash"
      },
      {
        pack: "spells-srd",
        name: "Advanced Scurvy"
      },
      {
        pack: "spells-srd",
        name: "Ancient Dust"
      },
      {
        pack: "spells-srd",
        name: "Armor of Thorn and Claw"
      },
      {
        pack: "spells-srd",
        name: "Beheading Buzz Saw"
      },
      {
        pack: "spells-srd",
        name: "Blazing Blade"
      },
      {
        pack: "spells-srd",
        name: "Blinding Foam"
      },
      {
        pack: "spells-srd",
        name: "Blister Bomb"
      },
      {
        pack: "spells-srd",
        name: "Blistering Invective"
      },
      {
        pack: "spells-srd",
        name: "Blood Feast"
      },
      {
        pack: "spells-srd",
        name: "Blood Vendetta"
      },
      {
        pack: "spells-srd",
        name: "Blood in the Water"
      },
      {
        pack: "spells-srd",
        name: "Bloodspray Curse"
      },
      {
        pack: "spells-srd",
        name: "Bone Flense"
      },
      {
        pack: "spells-srd",
        name: "Bone Spray"
      },
      {
        pack: "spells-srd",
        name: "Brine Dragon Bile"
      },
      {
        pack: "spells-srd",
        name: "Bursting Bloom"
      },
      {
        pack: "spells-srd",
        name: "Charged Javelin"
      },
      {
        pack: "spells-srd",
        name: "Cinder Swarm"
      },
      {
        pack: "spells-srd",
        name: "Combustion"
      },
      {
        pack: "spells-srd",
        name: "Cutting Insult"
      },
      {
        pack: "spells-srd",
        name: "Dehydrate"
      },
      {
        pack: "spells-srd",
        name: "Diadem of Divine Radiance"
      },
      {
        pack: "spells-srd",
        name: "Dimensional Excision"
      },
      {
        pack: "spells-srd",
        name: "Divine Immolation"
      },
      {
        pack: "spells-srd",
        name: "Earth's Bile"
      },
      {
        pack: "spells-srd",
        name: "Enervation"
      },
      {
        pack: "spells-srd",
        name: "Feral Shades"
      },
      {
        pack: "spells-srd",
        name: "Field of Razors"
      },
      {
        pack: "spells-srd",
        name: "Final Fate of the Locust Host"
      },
      {
        pack: "spells-srd",
        name: "Flame Dancer"
      },
      {
        pack: "spells-srd",
        name: "Flense"
      },
      {
        pack: "spells-srd",
        name: "Flourishing Flora"
      },
      {
        pack: "spells-srd",
        name: "Funeral Flames"
      },
      {
        pack: "spells-srd",
        name: "Fungal Infestation"
      },
      {
        pack: "spells-srd",
        name: "Glass Sand"
      },
      {
        pack: "spells-srd",
        name: "Gouging Claw"
      },
      {
        pack: "spells-srd",
        name: "Grim Tendrils"
      },
      {
        pack: "spells-srd",
        name: "Heat Metal"
      },
      {
        pack: "spells-srd",
        name: "Ignite Fireworks"
      },
      {
        pack: "spells-srd",
        name: "Ignition"
      },
      {
        pack: "spells-srd",
        name: "Implement of Destruction"
      },
      {
        pack: "spells-srd",
        name: "Live Wire"
      },
      {
        pack: "spells-srd",
        name: "Musical Shift"
      },
      {
        pack: "spells-srd",
        name: "Mutilate"
      },
      {
        pack: "spells-srd",
        name: "Noxious Metals"
      },
      {
        pack: "spells-srd",
        name: "Phantom Pain"
      },
      {
        pack: "spells-srd",
        name: "Puff of Poison"
      },
      {
        pack: "spells-srd",
        name: "Rainbow Fumarole"
      },
      {
        pack: "spells-srd",
        name: "Rusting Grasp"
      },
      {
        pack: "spells-srd",
        name: "Savor the Sting"
      },
      {
        pack: "spells-srd",
        name: "Sawtooth Terrain"
      },
      {
        pack: "spells-srd",
        name: "Scorching Blast"
      },
      {
        pack: "spells-srd",
        name: "Shocking Grasp"
      },
      {
        pack: "spells-srd",
        name: "Slashing Gust"
      }
    ]
  },
  {
    category: "spell",
    family: "expedition",
    tag: "aquatic_support",
    description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "expedition",
    tag: "sustenance",
    description: "Provides food, water, rations, or practical nourishment for travel and survival.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "expedition",
    tag: "field_shelter",
    description: "Creates shelter, refuge, or a protected resting place in the field.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "tempo",
    tag: "quickened_support",
    description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
    assignmentMode: "hybrid"
  },
  {
    category: "spell",
    family: "tempo",
    tag: "initiative_support",
    description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Anticipate Peril"
      },
      {
        pack: "spells-srd",
        name: "Call to Arms"
      },
      {
        pack: "spells-srd",
        name: "Rewrite Possibility"
      },
      {
        pack: "spells-srd",
        name: "Song of Marching"
      },
      {
        pack: "spells-srd",
        name: "Zeal for Battle"
      },
      {
        pack: "spells-srd",
        name: "Know the Enemy"
      },
      {
        pack: "spells-srd",
        name: "Dull Ambition"
      },
      {
        pack: "spells-srd",
        name: "Perseis's Precautions"
      },
      {
        pack: "spells-srd",
        name: "Shock to the System"
      }
    ]
  },
  {
    category: "spell",
    family: "summoner_support",
    tag: "eidolon_support",
    description: "Directly benefits an eidolon or the summoner-eidolon bond.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Boost Eidolon"
      },
      {
        pack: "spells-srd",
        name: "Extend Boost"
      },
      {
        pack: "spells-srd",
        name: "Unfetter Eidolon"
      },
      {
        pack: "spells-srd",
        name: "Summoner's Precaution"
      },
      {
        pack: "spells-srd",
        name: "Summoner's Visage"
      },
      {
        pack: "spells-srd",
        name: "Protect Companion"
      },
      {
        pack: "spells-srd",
        name: "Reinforce Eidolon"
      },
      {
        pack: "spells-srd",
        name: "Evolution Surge"
      },
      {
        pack: "spells-srd",
        name: "Lifelink Surge"
      },
      {
        pack: "spells-srd",
        name: "Domora's Defense"
      }
    ]
  },
  {
    category: "spell",
    family: "magic_interference",
    tag: "countermagic",
    description: "Counteracts, dispels, suppresses, or shuts down magic.",
    assignmentMode: "hybrid",
    seedRecords: [
      {
        pack: "spells-srd",
        name: "Antimagic Field"
      },
      {
        pack: "spells-srd",
        name: "Veil of Privacy"
      },
      {
        pack: "spells-srd",
        name: "Hidden Mind"
      },
      {
        pack: "spells-srd",
        name: "Blind Eye"
      },
      {
        pack: "spells-srd",
        name: "Detect Scrying"
      },
      {
        pack: "spells-srd",
        name: "False Vision"
      },
      {
        pack: "spells-srd",
        name: "Arcane Countermeasure"
      }
    ]
  },
  {
    category: "spell",
    family: "security",
    tag: "alarm",
    description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "impact",
    tag: "mental_impairment",
    description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "impact",
    tag: "sensory_impairment",
    description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "impact",
    tag: "forced_movement",
    description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
    assignmentMode: "deterministic"
  },
  {
    category: "spell",
    family: "impact",
    tag: "restraint_capture",
    description: "Restrains, immobilizes, entangles, or traps a target in place.",
    assignmentMode: "deterministic"
  }
];
