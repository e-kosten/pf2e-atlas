import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const HAZARD_DERIVED_TAG_ONTOLOGY = {
  category: "hazard",
  families: {
    mechanism: {
      description: "Hazards whose threat comes from trigger mechanisms, locking thresholds, control surfaces, or planar breaches.",
      tags: [
        {
          tag: "ward_trigger",
          description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
          assignmentMode: "hybrid"
        },
        {
          tag: "pressure_trigger",
          description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
          assignmentMode: "hybrid"
        },
        {
          tag: "tripwire_trigger",
          description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
          assignmentMode: "hybrid"
        },
        {
          tag: "threshold_lockdown",
          description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
          assignmentMode: "hybrid"
        },
        {
          tag: "control_interface",
          description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
          assignmentMode: "hybrid"
        },
        {
          tag: "planar_breach",
          description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
          assignmentMode: "hybrid"
        }
      ]
    },
    function: {
      description: "Hazard practical-function tags for alerts and restraint effects.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
          assignmentMode: "hybrid"
        },
        {
          tag: "restraint_capture",
          description: "Hazard that binds, restrains, or holds intruders in place.",
          assignmentMode: "hybrid"
        },
        {
          tag: "barrier_lockdown",
          description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
          assignmentMode: "hybrid"
        },
        {
          tag: "spawned_attackers",
          description: "Hazard that summons, creates, or releases separate attackers into the scene.",
          assignmentMode: "hybrid"
        }
      ]
    },
    haunt_manifestation: {
      description: "Haunt manifestations that materially change the encounter through attackers, lures, or life-draining effects.",
      tags: [
        {
          tag: "life_drain_hazard",
          description: "Haunt that drains life force, vitality, or souls from victims.",
          assignmentMode: "hybrid"
        },
        {
          tag: "phantom_assailants",
          description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
          assignmentMode: "hybrid"
        },
        {
          tag: "lure_compulsion",
          description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
          assignmentMode: "hybrid"
        },
        {
          tag: "battlefield_disruption",
          description: "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
          assignmentMode: "hybrid"
        }
      ]
    },
    impact: {
      description: "Hazard impact tags for mental destabilization and movement-limiting effects.",
      tags: [
        {
          tag: "mental_impairment",
          description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mobility_impairment",
          description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
          assignmentMode: "deterministic"
        }
      ]
    },
    environmental_danger: {
      description: "Hazards defined by recurring elemental or toxic environmental threats.",
      tags: [
        {
          tag: "acid_hazard",
          description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "cold_hazard",
          description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "fire_hazard",
          description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
          assignmentMode: "hybrid"
        },
        {
          tag: "electric_hazard",
          description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "poison_hazard",
          description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "respiratory_hazard",
          description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
          assignmentMode: "hybrid"
        },
        {
          tag: "sound_hazard",
          description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
          assignmentMode: "hybrid"
        },
        {
          tag: "water_hazard",
          description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
          assignmentMode: "hybrid"
        }
      ]
    },
    forced_position: {
      description: "Hazards that drop, collapse, or forcibly reposition creatures.",
      tags: [
        {
          tag: "pitfall",
          description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
          assignmentMode: "deterministic"
        },
        {
          tag: "collapse_hazard",
          description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
          assignmentMode: "deterministic"
        },
        {
          tag: "forced_movement",
          description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
          assignmentMode: "deterministic"
        }
      ]
    },
    perception_control: {
      description: "Hazards that distort routes, orientation, or the perceived layout of a space.",
      tags: [
        {
          tag: "navigation_disruption",
          description: "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
          assignmentMode: "deterministic"
        },
        {
          tag: "illusion_assault",
          description: "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
          assignmentMode: "deterministic"
        }
      ]
    },
    attack_vector: {
      description: "Hazards categorized by how their attack is delivered into the scene.",
      tags: [
        {
          tag: "overhead_strike",
          description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
          assignmentMode: "deterministic"
        },
        {
          tag: "projectile_emitter",
          description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
          assignmentMode: "deterministic"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
