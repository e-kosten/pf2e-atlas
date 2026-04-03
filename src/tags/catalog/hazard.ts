import { DerivedTagCatalogEntry } from "../../types.js";

export const HAZARD_DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "hazard",
    family: "mechanism",
    description: "Hazards whose threat comes from trigger mechanisms, locking thresholds, control surfaces, or planar breaches.",
    tags: [
      { value: "ward_trigger", description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism." },
      { value: "threshold_lockdown", description: "Hazard that seals, locks, or bars a threshold, doorway, or gate." },
      { value: "control_interface", description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface." },
      { value: "planar_breach", description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality." },
    ],
  },
  {
    category: "hazard",
    family: "function",
    description: "Hazard practical-function tags for alerts and restraint effects.",
    tags: [
      { value: "alarm", description: "Alerts guardians, onlookers, or nearby creatures to an intrusion." },
      { value: "restraint_capture", description: "Hazard that binds, restrains, or holds intruders in place." },
      { value: "barrier_lockdown", description: "Hazard that seals, closes, or blocks passage to trap or delay intruders." },
    ],
  },
  {
    category: "hazard",
    family: "impact",
    description: "Hazard impact tags for mental destabilization and movement-limiting effects.",
    tags: [
      { value: "mental_impairment", description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects." },
      { value: "mobility_impairment", description: "Paralyzes, immobilizes, or otherwise heavily hampers movement." },
    ],
  },
  {
    category: "hazard",
    family: "environmental_danger",
    description: "Hazards defined by recurring elemental or toxic environmental threats.",
    tags: [
      { value: "acid_hazard", description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure." },
      { value: "cold_hazard", description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure." },
      { value: "fire_hazard", description: "Hazard centered on open fire, flames, burning spread, or explosive ignition." },
      { value: "electric_hazard", description: "Hazard centered on lightning, shock, static discharge, or electrical exposure." },
      { value: "poison_hazard", description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure." },
      { value: "respiratory_hazard", description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing." },
      { value: "sound_hazard", description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption." },
      { value: "water_hazard", description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure." },
    ],
  },
  {
    category: "hazard",
    family: "forced_position",
    description: "Hazards that drop, collapse, or forcibly reposition creatures.",
    tags: [
      { value: "pitfall", description: "Hazard built around a concealed pit, drop, or similar vertical fall trap." },
      { value: "collapse_hazard", description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground." },
      { value: "forced_movement", description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures." },
    ],
  },
];
