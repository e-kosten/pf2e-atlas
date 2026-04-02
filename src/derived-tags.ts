import { DerivedTagCatalogEntry, SearchCategory, SearchSubcategory } from "./types.js";
import { normalizeText, uniqueSorted } from "./utils.js";

type DerivedTagContext = {
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  descriptionText: string | null;
  traits: string[];
};

type DerivedTagMatchClause = {
  traitsAny?: string[];
  traitsAll?: string[];
  textAny?: string[];
  textAll?: string[];
};

type DerivedTagRule = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  requiresTags?: string[];
  anyOf?: DerivedTagMatchClause[];
  allOf?: DerivedTagMatchClause[];
  noneOf?: DerivedTagMatchClause[];
};

const OFFENSIVE_TEXT_ANCHORS = [
  "toxin",
  "venom",
  "bomb",
  "injury poison",
  "contact poison",
  "ingested poison",
  "inhaled poison",
  "weapon poison",
  "afflicts the target",
];

const GEARISH_SUBCATEGORIES: SearchSubcategory[] = ["gear", "backpack", "kit", "vehicle"];

const DERIVED_TAG_RULES: DerivedTagRule[] = [
  {
    tag: "offensive",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "thrown_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { traitsAny: ["bomb"] },
      { textAny: ["throw", "thrown", "hurl", "lob", "splash weapon"] },
    ],
  },
  {
    tag: "weapon_applied",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: ["apply to a weapon", "coat a weapon", "weapon poison", "smeared on a weapon", "applied to a weapon"] },
    ],
  },
  {
    tag: "ingested_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: ["ingested poison", "must be eaten", "must be drunk", "consumed by the target", "when swallowed"] },
    ],
  },
  {
    tag: "contact_offense",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["offensive"],
    anyOf: [
      { textAny: ["contact poison", "through skin contact", "through contact", "absorbed through the skin"] },
    ],
  },
  {
    tag: "beneficial",
    category: "equipment",
    subcategories: ["consumable"],
    anyOf: [
      { traitsAny: ["elixir", "healing"] },
      {
        textAny: [
          "restorative",
          "remedy",
          "curative",
          "antidote",
          "antiplague",
          "catharsis",
          "healing",
          "darkvision",
          "resistance to",
          "gain a bonus",
          "bolsters the drinker",
          "steady the emotions",
          "see in the dark",
        ],
      },
    ],
    noneOf: [
      { traitsAny: ["poison", "bomb"] },
      { textAny: OFFENSIVE_TEXT_ANCHORS },
    ],
  },
  {
    tag: "healing_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { traitsAny: ["healing"] },
      { textAny: ["elixir of life", "healing", "restore hit points", "restore hp", "regain hit points"] },
    ],
  },
  {
    tag: "anti_poison",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["antidote", "against poison", "protect against poison", "resist poison", "ward off poison"] },
    ],
  },
  {
    tag: "anti_disease",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["antiplague", "against disease", "protect against disease", "resist disease", "ward off disease"] },
    ],
  },
  {
    tag: "condition_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["condition", "catharsis", "soothe the mind", "steady the emotions", "calm overwhelming emotions", "recover from mental conditions"] },
    ],
  },
  {
    tag: "mental_recovery",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["soothe the mind", "steady the emotions", "calm overwhelming emotions", "mental condition", "mental conditions", "emotion", "emotions", "frightened", "stupefied", "confused", "mental effect"] },
    ],
  },
  {
    tag: "escape_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["escape", "slip away", "break free", "flee", "evade", "concealing smoke", "vanish from sight", "misty"] },
    ],
  },
  {
    tag: "senses_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["darkvision", "see in the dark", "low-light vision", "keen senses", "sharpen your vision", "see invisible", "scent"] },
    ],
  },
  {
    tag: "energy_resistance",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["resistance to fire", "resistance to cold", "resistance to electricity", "resistance to acid", "resistance to sonic", "resistance to energy", "energy resistance"] },
    ],
  },
  {
    tag: "buff_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["gain a bonus", "bonus to", "bolster", "enhance", "empower", "heighten your senses", "increase your speed", "resistance to"] },
    ],
  },
  {
    tag: "self_buff",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["you gain", "the drinker gains", "gain a bonus", "you become", "you gain resistance", "you gain darkvision"] },
    ],
  },
  {
    tag: "ally_support",
    category: "equipment",
    subcategories: ["consumable"],
    requiresTags: ["beneficial"],
    anyOf: [
      { textAny: ["target gains", "an ally gains", "creature that drinks gains", "the drinker gains"] },
    ],
  },
  {
    tag: "climbing",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["climb", "climbing", "rappel", "rappelling", "piton", "grappling"] },
    ],
  },
  {
    tag: "mobility",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["mobility", "move quickly", "increase your speed", "rappel", "climbing"] },
    ],
    requiresTags: ["climbing"],
  },
  {
    tag: "lock_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["lockpick", "lockpicks", "pick locks", "picking locks", "bypass locks", "thieves tools", "thieves' tools", "toolkit"] },
    ],
  },
  {
    tag: "concealable",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["concealable", "hidden on your person", "hidden tools", "slim lockpicks"] },
    ],
  },
  {
    tag: "scouting",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["scout", "scouting", "survey", "recon", "observe from afar", "spyglass"] },
    ],
  },
  {
    tag: "stealth_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["stealth", "quiet", "silent", "without drawing attention", "avoid notice", "infiltration"] },
      { textAny: ["concealable", "hidden on your person"] },
    ],
  },
  {
    tag: "disguise",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["disguise", "impersonate", "false identity", "costume", "masquerade"] },
    ],
  },
  {
    tag: "social_infiltration",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["false identity", "pass as", "blend into society", "social infiltration", "impersonate", "masquerade"] },
    ],
  },
  {
    tag: "illumination",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["light", "illumination", "lantern", "torch", "glow", "illuminate"] },
    ],
  },
  {
    tag: "survival",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["camp", "forage", "wilderness", "survival", "shelter", "weatherproof"] },
    ],
  },
  {
    tag: "navigation",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["navigate", "navigation", "map", "compass", "chart", "track your heading"] },
    ],
  },
  {
    tag: "transport",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["transport", "wagon", "sled", "boat", "vehicle", "carry riders", "haul passengers"] },
    ],
  },
  {
    tag: "trap_bypass",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["disarm a trap", "disable device", "trap mechanism", "tripwire", "bypass a trap"] },
    ],
  },
  {
    tag: "carry_support",
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    anyOf: [
      { textAny: ["storage", "stow", "carry", "haul", "pouch", "backpack", "container", "pack"] },
    ],
  },
  {
    tag: "undead_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["undead", "ghost", "spirit", "skeleton", "ghoul"] },
    ],
  },
  {
    tag: "fey_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["fey"] },
    ],
  },
  {
    tag: "plant_threat",
    category: "creature",
    anyOf: [
      { traitsAny: ["plant", "fungus", "leshy"] },
    ],
  },
  {
    tag: "aquatic_context",
    category: "creature",
    anyOf: [
      { traitsAny: ["water"] },
      { textAny: ["aquatic", "ocean", "sea", "river", "coast", "harbor", "water"] },
    ],
  },
  {
    tag: "nautical",
    category: "creature",
    anyOf: [
      { textAny: ["sailor", "ship", "captain", "mariner", "harbor", "dock", "bilge", "wreck", "crew"] },
    ],
  },
  {
    tag: "forest",
    category: "creature",
    anyOf: [
      { textAny: ["forest", "woodland", "grove", "briar"] },
    ],
  },
  {
    tag: "swamp",
    category: "creature",
    anyOf: [
      { textAny: ["swamp", "bog", "marsh", "fen", "mire"] },
    ],
  },
  {
    tag: "underground",
    category: "creature",
    anyOf: [
      { textAny: ["cave", "cavern", "underground", "tunnel", "subterranean", "underworld", "depths", "crypt", "crypts"] },
    ],
  },
  {
    tag: "urban",
    category: "creature",
    anyOf: [
      { textAny: ["city", "urban", "street", "alley", "market", "sewer", "town"] },
    ],
  },
  {
    tag: "arctic",
    category: "creature",
    anyOf: [
      { textAny: ["arctic", "snow", "ice", "tundra", "frozen", "glacier"] },
    ],
  },
  {
    tag: "desert",
    category: "creature",
    anyOf: [
      { textAny: ["desert", "dune", "sand", "arid", "wastes"] },
    ],
  },
  {
    tag: "mountain",
    category: "creature",
    anyOf: [
      { textAny: ["mountain", "cliff", "peak", "crag", "alp"] },
    ],
  },
  {
    tag: "graveyard",
    category: "creature",
    anyOf: [
      { textAny: ["graveyard", "cemetery", "crypt", "tomb"] },
    ],
  },
  {
    tag: "ruins",
    category: "creature",
    anyOf: [
      { textAny: ["ruins", "ruin", "crumbling hall", "fallen temple", "ancient hall", "derelict"] },
    ],
  },
  {
    tag: "profession_npc",
    category: "creature",
    anyOf: [
      { textAny: ["captain", "commoner", "guard", "scout", "sailor", "merchant", "priest", "noble"] },
    ],
  },
  {
    tag: "scene_adjacent",
    category: "creature",
    requiresTags: ["profession_npc"],
    noneOf: [
      { traitsAny: ["undead", "ghost", "spirit", "skeleton", "ghoul", "fey", "plant", "fungus", "leshy"] },
    ],
  },
];

export const DERIVED_TAG_CATALOG: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "function",
    description: "Beneficial consumable outcome and recovery tags.",
    tags: [
      { value: "beneficial", description: "Broad support-oriented consumable with non-hostile intent." },
      { value: "healing_support", description: "Restores hit points or provides direct healing." },
      { value: "anti_poison", description: "Helps resist, prevent, or recover from poison." },
      { value: "anti_disease", description: "Helps resist, prevent, or recover from disease." },
      { value: "condition_support", description: "Helps clear or mitigate harmful conditions." },
      { value: "mental_recovery", description: "Helps stabilize emotions or recover from mental conditions." },
      { value: "escape_support", description: "Helps flee, slip away, or break free." },
      { value: "senses_support", description: "Improves vision or other senses." },
      { value: "energy_resistance", description: "Grants resistance against one or more energy types." },
      { value: "buff_support", description: "Provides a general beneficial enhancement or bonus." },
    ],
  },
  {
    category: "equipment",
    subcategories: ["consumable"],
    family: "polarity",
    description: "Offense/support polarity and delivery-style consumable tags.",
    tags: [
      { value: "offensive", description: "Hostile consumable primarily meant to harm or debilitate a target." },
      { value: "self_buff", description: "Support consumable primarily applied to the user." },
      { value: "ally_support", description: "Support consumable that can directly benefit another creature." },
      { value: "weapon_applied", description: "Offensive consumable applied to a weapon before use." },
      { value: "thrown_offense", description: "Offensive consumable delivered by throwing it." },
      { value: "ingested_offense", description: "Offensive consumable delivered when swallowed or consumed." },
      { value: "contact_offense", description: "Offensive consumable delivered through touch or skin contact." },
    ],
  },
  {
    category: "equipment",
    subcategories: GEARISH_SUBCATEGORIES,
    family: "purpose",
    description: "Utility, infiltration, and logistics gear-purpose tags.",
    tags: [
      { value: "climbing", description: "Helps climb, rappel, or navigate vertical obstacles." },
      { value: "lock_bypass", description: "Helps open locks or bypass secured entry points." },
      { value: "concealable", description: "Easy to hide on the person or carry discreetly." },
      { value: "scouting", description: "Helps observe, survey, or reconnoiter an area." },
      { value: "mobility", description: "Improves movement or traversal flexibility." },
      { value: "stealth_support", description: "Helps move quietly or avoid notice." },
      { value: "disguise", description: "Helps alter appearance or impersonate another identity." },
      { value: "social_infiltration", description: "Helps blend into a group or pass under social scrutiny." },
      { value: "illumination", description: "Produces or improves light in dark environments." },
      { value: "survival", description: "Supports wilderness travel, shelter, or long-term field use." },
      { value: "navigation", description: "Helps track direction, route, or position." },
      { value: "transport", description: "Helps move creatures or cargo from place to place." },
      { value: "trap_bypass", description: "Helps disarm, disable, or get past traps." },
      { value: "carry_support", description: "Helps stow, carry, or organize equipment." },
    ],
  },
  {
    category: "creature",
    family: "context",
    description: "Creature environment and scene-context tags.",
    tags: [
      { value: "nautical", description: "Strongly associated with ships, sailors, wrecks, or harbors." },
      { value: "aquatic_context", description: "Strongly associated with water or aquatic environments." },
      { value: "forest", description: "Strongly associated with forests, groves, or briar-choked wilds." },
      { value: "swamp", description: "Strongly associated with bogs, marshes, or mires." },
      { value: "underground", description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces." },
      { value: "urban", description: "Strongly associated with cities, streets, alleys, or sewers." },
      { value: "arctic", description: "Strongly associated with snow, ice, tundra, or frozen coasts." },
      { value: "desert", description: "Strongly associated with dunes, sand, or arid wastes." },
      { value: "mountain", description: "Strongly associated with cliffs, peaks, or rocky heights." },
      { value: "graveyard", description: "Strongly associated with cemeteries, tombs, or burial grounds." },
      { value: "ruins", description: "Strongly associated with ancient ruins or derelict structures." },
    ],
  },
  {
    category: "creature",
    family: "scene_fit",
    description: "Creature practical-fit tags for distinguishing scene-adjacent NPCs from primary threats.",
    tags: [
      { value: "profession_npc", description: "Role-defined NPC such as a captain, guard, merchant, or commoner." },
      { value: "scene_adjacent", description: "Fits the scene or social fabric, but is usually not the primary monster answer." },
      { value: "undead_threat", description: "Threat signal derived from undead-like native traits." },
      { value: "fey_threat", description: "Threat signal derived from fey native traits." },
      { value: "plant_threat", description: "Threat signal derived from plant-like native traits." },
    ],
  },
];

function matchesClause(context: { traits: Set<string>; text: string }, clause: DerivedTagMatchClause): boolean {
  if (clause.traitsAny && !clause.traitsAny.some((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.traitsAll && !clause.traitsAll.every((trait) => context.traits.has(normalizeText(trait)))) {
    return false;
  }
  if (clause.textAny && !clause.textAny.some((anchor) => context.text.includes(normalizeText(anchor)))) {
    return false;
  }
  if (clause.textAll && !clause.textAll.every((anchor) => context.text.includes(normalizeText(anchor)))) {
    return false;
  }
  return true;
}

function matchesRule(
  context: { category: SearchCategory; subcategory: SearchSubcategory | null; traits: Set<string>; text: string },
  tags: Set<string>,
  rule: DerivedTagRule,
): boolean {
  if (context.category !== rule.category) {
    return false;
  }
  if (rule.subcategories && (!context.subcategory || !rule.subcategories.includes(context.subcategory))) {
    return false;
  }
  if (rule.requiresTags && !rule.requiresTags.every((tag) => tags.has(tag))) {
    return false;
  }
  if (rule.anyOf && !rule.anyOf.some((clause) => matchesClause(context, clause))) {
    return false;
  }
  if (rule.allOf && !rule.allOf.every((clause) => matchesClause(context, clause))) {
    return false;
  }
  if (rule.noneOf && rule.noneOf.some((clause) => matchesClause(context, clause))) {
    return false;
  }
  return true;
}

export function normalizeDerivedTag(value: string): string {
  return normalizeText(value).replace(/\s+/g, "_");
}

export function deriveRecordTags(input: DerivedTagContext): string[] {
  const context = {
    category: input.category,
    subcategory: input.subcategory,
    traits: new Set(input.traits.map((trait) => normalizeText(trait)).filter(Boolean)),
    text: normalizeText([input.name, input.descriptionText ?? ""].filter(Boolean).join(" ")),
  };
  const tags = new Set<string>();

  for (const rule of DERIVED_TAG_RULES) {
    if (matchesRule(context, tags, rule)) {
      tags.add(rule.tag);
    }
  }

  return uniqueSorted([...tags]);
}
