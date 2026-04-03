import { DerivedTagCatalogEntry } from "../../types.js";

export const HAZARD_DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
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
      { value: "fire_hazard", description: "Hazard centered on open fire, flames, burning spread, or explosive ignition." },
      { value: "poison_hazard", description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure." },
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
