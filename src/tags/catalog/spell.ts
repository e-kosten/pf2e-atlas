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
    family: "control",
    description: "Spells that pressure morale, obscure sight, or reshape the battlefield.",
    tags: [
      { value: "fear_pressure", description: "Forces fear, panic, dread, or morale collapse onto a target." },
      { value: "concealment", description: "Makes a creature hard to see, hidden, concealed, or undetected." },
      { value: "line_of_sight_control", description: "Blocks vision, obscures sight lines, or denies clear observation across an area." },
      { value: "battlefield_disruption", description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles." },
    ],
  },
  {
    category: "spell",
    family: "support",
    description: "Spells that restore, protect, ward, or reinforce allies and targets.",
    tags: [
      { value: "healing_support", description: "Directly restores hit points or accelerates recovery." },
      { value: "condition_support", description: "Delays, suppresses, or removes afflictions and conditions." },
      { value: "affliction_cleanup", description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions." },
      { value: "escape_support", description: "Helps a creature slip away, break free, flee, or evade pursuit." },
      { value: "protective_ward", description: "Places a ward, sanctuary, shield, or protective boundary." },
      { value: "death_prevention", description: "Prevents death, stabilizes the dying, or brings a creature back from the brink." },
      { value: "resistance_support", description: "Grants resistance or immunity against energy, damage, or hazards." },
    ],
  },
  {
    category: "spell",
    family: "expedition",
    description: "Spells that support travel, survival, and aquatic operations.",
    tags: [
      { value: "aquatic_support", description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement." },
      { value: "sustenance", description: "Provides food, water, rations, or practical nourishment for travel and survival." },
      { value: "field_shelter", description: "Creates shelter, refuge, or a protected resting place in the field." },
    ],
  },
  {
    category: "spell",
    family: "tempo",
    description: "Spells that improve action economy or accelerate allies.",
    tags: [
      { value: "quickened_support", description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration." },
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
  {
    category: "spell",
    family: "impact",
    description: "Spells that impair minds or senses, forcibly reposition targets, or trap them in place.",
    tags: [
      { value: "mental_impairment", description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects." },
      { value: "sensory_impairment", description: "Blinds, deafens, or otherwise directly suppresses a creature's senses." },
      { value: "forced_movement", description: "Pushes, pulls, drags, or otherwise repositions a target against its will." },
      { value: "restraint_capture", description: "Restrains, immobilizes, entangles, or traps a target in place." },
    ],
  },
];
