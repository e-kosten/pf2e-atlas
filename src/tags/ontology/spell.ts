import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const SPELL_DERIVED_TAG_ONTOLOGY = {
  category: "spell",
  families: {
    infiltration: {
      description: "Appearance-changing and social-passing spells.",
      tags: [
        {
          tag: "disguise",
          description: "Helps alter appearance or impersonate another identity.",
          assignmentMode: "deterministic"
        },
        {
          tag: "social_infiltration",
          description: "Helps blend into a group or pass under social scrutiny.",
          assignmentMode: "deterministic"
        }
      ]
    },
    communication: {
      description: "Spells for signaling, telepathy, and message exchange.",
      tags: [
        {
          tag: "signaling",
          description: "Helps draw attention, mark a location, or coordinate allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "message_delivery",
          description: "Sends, stores, or relays actual content across time or distance.",
          assignmentMode: "deterministic"
        }
      ]
    },
    reconnaissance: {
      description: "Remote-observation and scouting spells.",
      tags: [
        {
          tag: "scouting",
          description: "Helps observe at a distance, extend senses, or locate a target.",
          assignmentMode: "hybrid"
        }
      ]
    },
    wayfinding: {
      description: "Spells that guide direction, route-finding, or destination travel.",
      tags: [
        {
          tag: "navigation",
          description: "Helps orient, guide a route, or identify a destination's direction.",
          assignmentMode: "hybrid"
        }
      ]
    },
    traversal: {
      description: "Spells that improve movement modes, speed, or practical traversal.",
      tags: [
        {
          tag: "mobility",
          description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
          assignmentMode: "hybrid"
        }
      ]
    },
    transformation: {
      description: "Spells that alter a creature's body, form, or battle shape.",
      tags: [
        {
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
          tag: "battle_form",
          description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
          assignmentMode: "deterministic"
        },
        {
          tag: "animal_form",
          description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
          assignmentMode: "deterministic"
        },
        {
          tag: "elemental_form",
          description: "Transforms a creature into an elemental form.",
          assignmentMode: "deterministic"
        }
      ]
    },
    control: {
      description: "Spells that pressure morale, obscure sight, or reshape the battlefield.",
      tags: [
        {
          tag: "fear_pressure",
          description: "Forces fear, panic, dread, or morale collapse onto a target.",
          assignmentMode: "deterministic"
        },
        {
          tag: "concealment",
          description: "Makes a creature hard to see, hidden, concealed, or undetected.",
          assignmentMode: "deterministic"
        },
        {
          tag: "line_of_sight_control",
          description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
          assignmentMode: "deterministic"
        },
        {
          tag: "battlefield_disruption",
          description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
          assignmentMode: "deterministic"
        }
      ]
    },
    support: {
      description: "Spells that restore, protect, ward, or reinforce allies and targets.",
      tags: [
        {
          tag: "healing_support",
          description: "Directly restores hit points or accelerates recovery.",
          assignmentMode: "hybrid"
        },
        {
          tag: "condition_support",
          description: "Delays, suppresses, or removes afflictions and conditions.",
          assignmentMode: "hybrid"
        },
        {
          tag: "affliction_cleanup",
          description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
          assignmentMode: "hybrid"
        },
        {
          tag: "escape_support",
          description: "Helps a creature slip away, break free, flee, or evade pursuit.",
          assignmentMode: "hybrid"
        },
        {
          tag: "protective_ward",
          description: "Places a ward, sanctuary, shield, or protective boundary.",
          assignmentMode: "hybrid"
        },
        {
          tag: "death_prevention",
          description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
          assignmentMode: "hybrid"
        },
        {
          tag: "resistance_support",
          description: "Grants resistance or immunity against energy, damage, or hazards.",
          assignmentMode: "hybrid"
        },
        {
          tag: "temporary_hp_support",
          description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
          assignmentMode: "hybrid"
        }
      ]
    },
    attrition: {
      description: "Spells that inflict ongoing harm through lingering persistent damage.",
      tags: [
        {
          tag: "persistent_damage",
          description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
          assignmentMode: "hybrid"
        }
      ]
    },
    expedition: {
      description: "Spells that support travel, survival, and aquatic operations.",
      tags: [
        {
          tag: "aquatic_support",
          description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sustenance",
          description: "Provides food, water, rations, or practical nourishment for travel and survival.",
          assignmentMode: "deterministic"
        },
        {
          tag: "field_shelter",
          description: "Creates shelter, refuge, or a protected resting place in the field.",
          assignmentMode: "deterministic"
        }
      ]
    },
    tempo: {
      description: "Spells that improve action economy or accelerate allies.",
      tags: [
        {
          tag: "quickened_support",
          description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
          assignmentMode: "hybrid"
        },
        {
          tag: "initiative_support",
          description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
          assignmentMode: "hybrid"
        }
      ]
    },
    summoner_support: {
      description: "Spells that specifically protect, enhance, or reposition an eidolon.",
      tags: [
        {
          tag: "eidolon_support",
          description: "Directly benefits an eidolon or the summoner-eidolon bond.",
          assignmentMode: "hybrid"
        }
      ]
    },
    magic_interference: {
      description: "Spells that disrupt, dispel, or suppress magic.",
      tags: [
        {
          tag: "countermagic",
          description: "Counteracts, dispels, suppresses, or shuts down magic.",
          assignmentMode: "hybrid"
        }
      ]
    },
    security: {
      description: "Area-warning and intrusion-alert spells.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
          assignmentMode: "deterministic"
        }
      ]
    },
    impact: {
      description: "Spells that impair minds or senses, forcibly reposition targets, or trap them in place.",
      tags: [
        {
          tag: "mental_impairment",
          description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "forced_movement",
          description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
          assignmentMode: "deterministic"
        },
        {
          tag: "restraint_capture",
          description: "Restrains, immobilizes, entangles, or traps a target in place.",
          assignmentMode: "deterministic"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
