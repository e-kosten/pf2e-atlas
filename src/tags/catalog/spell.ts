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
