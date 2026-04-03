import { DerivedTagCatalogEntry } from "../../types.js";

export const SPELL_DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "spell",
    family: "infiltration",
    description: "Appearance-changing and social-passing spells.",
    tags: [
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
    ],
  },
  {
    category: "spell",
    family: "communication",
    description: "Spells for signaling, telepathy, and message exchange.",
    tags: [
      { value: "signaling", description: "Helps draw attention, mark a location, or coordinate allies." },
      { value: "message_delivery", description: "Sends, stores, or relays actual content across time or distance." },
    ],
  },
  {
    category: "spell",
    family: "reconnaissance",
    description: "Remote-observation and scouting spells.",
    tags: [
      { value: "scouting", description: "Helps observe at a distance, extend senses, or locate a target." },
    ],
  },
  {
    category: "spell",
    family: "wayfinding",
    description: "Spells that guide direction, route-finding, or destination travel.",
    tags: [
      { value: "navigation", description: "Helps orient, guide a route, or identify a destination's direction." },
    ],
  },
  {
    category: "spell",
    family: "traversal",
    description: "Spells that improve movement modes, speed, or practical traversal.",
    tags: [
      { value: "mobility", description: "Helps move faster, gain movement modes, or traverse terrain more effectively." },
    ],
  },
  {
    category: "spell",
    family: "transformation",
    description: "Spells that alter a creature's body, form, or battle shape.",
    promoteFamilyToTag: true,
    tags: [
      { value: "battle_form", description: "Transforms a creature into a combat-ready form with new statistics or battle-form language." },
      { value: "animal_form", description: "Transforms a creature into an animal, beast, pest, or similar natural form." },
      { value: "elemental_form", description: "Transforms a creature into an elemental form." },
    ],
  },
  {
    category: "spell",
    family: "support",
    description: "Spells that restore, protect, ward, or reinforce allies and targets.",
    tags: [
      { value: "healing_support", description: "Directly restores hit points or accelerates recovery." },
      { value: "condition_support", description: "Delays, suppresses, or removes afflictions and conditions." },
      { value: "protective_ward", description: "Places a ward, sanctuary, shield, or protective boundary." },
      { value: "death_prevention", description: "Prevents death, stabilizes the dying, or brings a creature back from the brink." },
      { value: "resistance_support", description: "Grants resistance or immunity against energy, damage, or hazards." },
    ],
  },
  {
    category: "spell",
    family: "magic_interference",
    description: "Spells that disrupt, dispel, or suppress magic.",
    tags: [
      { value: "countermagic", description: "Counteracts, dispels, suppresses, or shuts down magic." },
    ],
  },
  {
    category: "spell",
    family: "security",
    description: "Area-warning and intrusion-alert spells.",
    tags: [
      { value: "alarm", description: "Alerts you or others when a watched area, threshold, or ward is crossed." },
    ],
  },
];
