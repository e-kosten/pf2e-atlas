import type { SearchCategory } from "../../domain/search-types.js";

export const CANONICAL_VOCABULARY = {
  SCHEMA: {
    KIND: {
      DESCRIPTIVE: "descriptive",
      OPERATIONAL: "operational",
      AGGREGATE: "aggregate",
    },
  },
  FACET: {
    KIND: {
      SETTING: "setting",
      THEME: "theme",
      CAPABILITY: "capability",
      CREATURE_FAMILY: "creature_family",
      EFFECT: "effect",
      ROLE: "role",
      FUNCTION: "function",
      MECHANISM: "mechanism",
      DELIVERY: "delivery",
      PROGRESSION: "progression",
      PATHOGENESIS: "pathogenesis",
      RESPONSE_DEMAND: "response_demand",
      BEHAVIOR_OVERRIDE: "behavior_override",
      RESPONSE_PROFILE: "response_profile",
      CHALLENGE_STRUCTURE: "challenge_structure",
    },
    VALUE: {
      PLANAR: "planar",
      REGIONAL: "regional",
      HABITAT: "habitat",
      SITE: "site",
      THEME: "theme",
      EFFECT_PROFILE: "effect_profile",
      TACTICAL: "tactical",
      HAZARD_FUNCTION: "hazard_function",
      MECHANISM: "mechanism",
      VECTOR: "vector",
      DELIVERY_PROFILE: "delivery_profile",
      PROGRESSION_PROFILE: "progression_profile",
      RESPONSE_PROFILE: "response_profile",
      EPIDEMIOLOGICAL_PROFILE: "epidemiological_profile",
      BOUND_OBJECT_FAMILY: "bound_object_family",
      CAPABILITY: "capability",
      BEHAVIOR_OVERRIDE: "behavior_override",
      CASTING_PROFILE: "casting_profile",
      CHALLENGE_STRUCTURE: "challenge_structure",
      CORRUPTION: "corruption",
      CREATURE_FAMILY: "creature_family",
      DEFENSIVE_CAPABILITY: "defensive_capability",
      HAUNT_MANIFESTATION: "haunt_manifestation",
      PATHOGENESIS: "pathogenesis",
      PAYLOAD: "payload",
      PESTILENCE: "pestilence",
      RESPONSE_DEMAND: "response_demand",
      ROLE: "role",
      SENSORY_CAPABILITY: "sensory_capability",
      TARGETING_CAPABILITY: "targeting_capability",
    },
  },
  OPERATION: {
    REMEDIATE: "remediate",
    DISCOVER: "discover",
    CONTAIN: "contain",
    CLEAN_UP: "clean_up",
    REMOVE: "remove",
    COUNTERACT: "counteract",
    EXPEL: "expel",
    APPEASE: "appease",
    APPLY: "apply",
    CONTROL: "control",
    CREATE: "create",
    SUSTAIN: "sustain",
    CALL: "call",
    INFILTRATE: "infiltrate",
    SUPPORT: "support",
    BYPASS: "bypass",
    SUMMON: "summon",
    TRANSFORM: "transform",
    DEAL: "deal",
    BREAK: "break",
    CLEAR: "clear",
    RESOLVE: "resolve",
    PROTECT: "protect",
    BREACH: "breach",
    AMBUSH: "ambush",
    ASSAULT: "assault",
    BURST: "burst",
    CAPTURE: "capture",
    CAUSE: "cause",
    DEMOLISH: "demolish",
    DENY: "deny",
    DISARM: "disarm",
    DISRUPT: "disrupt",
    DOMINATE: "dominate",
    EMIT: "emit",
    ERUPT: "erupt",
    ESCAPE: "escape",
    EXCAVATE: "excavate",
    INDUCE: "induce",
    INFLUENCE: "influence",
    MANIPULATE: "manipulate",
    MISLEAD: "mislead",
    PRESSURE: "pressure",
    PREVENT: "prevent",
    RECOVER: "recover",
    REPOSITION: "reposition",
    RESIST: "resist",
    SANCTIFY: "sanctify",
    STRIKE: "strike",
  },
  DOMAIN: {
    POISON: "poison",
    CURSE: "curse",
    DISEASE: "disease",
    OUTBREAK: "outbreak",
    HIDDEN_SOURCE: "hidden_source",
    ACTIVE_MAGIC: "active_magic",
    FEAR: "fear",
    FORM: "form",
    ACID: "acid",
    AFFLICTION: "affliction",
    AGENCY: "agency",
    ANIMAL: "animal",
    BARRIER: "barrier",
    BATTLE: "battle",
    BLEED: "bleed",
    BLIGHT: "blight",
    BUFF: "buff",
    BURST: "burst",
    CASTER_DISRUPTION: "caster_disruption",
    CHARM: "charm",
    COLD: "cold",
    COLLAPSE: "collapse",
    COMPULSION: "compulsion",
    CONDITION: "condition",
    CONTAMINATION: "contamination",
    CREATURE: "creature",
    CROWD: "crowd",
    CURSEFIELD: "cursefield",
    DEATH: "death",
    DEATH_BURST: "death_burst",
    DISPLACEMENT: "displacement",
    DOOR: "door",
    EIDOLON: "eidolon",
    ELECTRIC: "electric",
    ELEMENTAL: "elemental",
    EMOTION: "emotion",
    ENERGY: "energy",
    ENVIRONMENTAL: "environmental",
    ESCAPE: "escape",
    FALL: "fall",
    FIRE: "fire",
    FORTUNE: "fortune",
    GRAPPLE: "grapple",
    GROUND: "ground",
    HAZARD: "hazard",
    HEALING: "healing",
    HOSTILE_PRESENCE: "hostile_presence",
    INFILTRATION: "infiltration",
    INITIATIVE: "initiative",
    INVISIBILITY: "invisibility",
    LIFE_DRAIN: "life_drain",
    LINE_OF_SIGHT: "line_of_sight",
    LOCK: "lock",
    MAGIC: "magic",
    MECHANISM: "mechanism",
    MEMORY: "memory",
    MENTAL_STATE: "mental_state",
    MOBILITY: "mobility",
    NAVIGATION: "navigation",
    OFFENSIVE: "offensive",
    OVERGROWTH: "overgrowth",
    OVERHEAD: "overhead",
    PARALYSIS: "paralysis",
    PARASITE_OR_IMPLANT: "parasite_or_implant",
    PERCEPTION: "perception",
    PERSISTENT: "persistent",
    PETRIFICATION: "petrification",
    POSITION: "position",
    POSSESSION: "possession",
    PREY: "prey",
    PROBLEM: "problem",
    PROCEDURAL: "procedural",
    PROJECTILE: "projectile",
    PROXIMITY: "proximity",
    QUICKENED: "quickened",
    REGENERATION: "regeneration",
    REINFORCEMENT: "reinforcement",
    RESISTANCE: "resistance",
    RESPIRATORY: "respiratory",
    RESTRAINT: "restraint",
    RITUAL_GRIEVANCE: "ritual_grievance",
    ROUTE: "route",
    SACRED_TAINT: "sacred_taint",
    SCOUTING: "scouting",
    SCREENING: "screening",
    SENSES: "senses",
    SERVITOR: "servitor",
    SIEGE: "siege",
    SINGLE_TARGET: "single_target",
    SLEEP: "sleep",
    SOUND: "sound",
    SPAWN: "spawn",
    STRUCTURE: "structure",
    TEMPORARY_HP: "temporary_hp",
    TERRAIN: "terrain",
    TRAP: "trap",
    TRUTH: "truth",
    UNDEAD: "undead",
    WARD: "ward",
    WATER: "water",
  },
  RELATION: {
    KINDS: {
      COUNTERACTS: "counteracts",
      APPLIES: "applies",
      REQUIRES: "requires",
      ENABLES: "enables",
      EVOKES: "evokes",
      OVERLAPS_WITH: "overlaps_with",
      SPECIALIZES: "specializes",
    },
  },
  ASSIGNMENT: {
    MODE: {
      DETERMINISTIC: "deterministic",
      EDITORIAL: "editorial",
      HYBRID: "hybrid",
      COMPOSITE: "composite",
    },
  },
  TRANSLATION: {
    STATUS: {
      MAPPED: "mapped",
      PROVISIONAL: "provisional",
      UNMAPPED: "unmapped",
      DROPPED: "dropped",
    },
  },
  CATEGORY: {
    AFFLICTION: "affliction",
    CREATURE: "creature",
    EQUIPMENT: "equipment",
    HAZARD: "hazard",
    SPELL: "spell",
  },
} as const;

export type SchemaKind = (typeof CANONICAL_VOCABULARY.SCHEMA.KIND)[keyof typeof CANONICAL_VOCABULARY.SCHEMA.KIND];

export type FacetKind = (typeof CANONICAL_VOCABULARY.FACET.KIND)[keyof typeof CANONICAL_VOCABULARY.FACET.KIND] | (string & {});

export type FacetValue =
  (typeof CANONICAL_VOCABULARY.FACET.VALUE)[keyof typeof CANONICAL_VOCABULARY.FACET.VALUE] | (string & {});

export type OperationId =
  (typeof CANONICAL_VOCABULARY.OPERATION)[keyof typeof CANONICAL_VOCABULARY.OPERATION] | (string & {});

export type DomainId = (typeof CANONICAL_VOCABULARY.DOMAIN)[keyof typeof CANONICAL_VOCABULARY.DOMAIN] | (string & {});

export type RelationId = (typeof CANONICAL_VOCABULARY.RELATION.KINDS)[keyof typeof CANONICAL_VOCABULARY.RELATION.KINDS] | (string & {});

export type AssignmentModeId = (typeof CANONICAL_VOCABULARY.ASSIGNMENT.MODE)[keyof typeof CANONICAL_VOCABULARY.ASSIGNMENT.MODE];

export type TranslationStatus =
  (typeof CANONICAL_VOCABULARY.TRANSLATION.STATUS)[keyof typeof CANONICAL_VOCABULARY.TRANSLATION.STATUS];

export type CategoryId = (typeof CANONICAL_VOCABULARY.CATEGORY)[keyof typeof CANONICAL_VOCABULARY.CATEGORY];

export type AxisToken<C extends SearchCategory = SearchCategory> = {
  readonly category: C;
  readonly value: string;
};

export type FamilyToken<C extends SearchCategory = SearchCategory> = {
  readonly category: C;
  readonly value: string;
  readonly axis: AxisToken<C>;
  readonly description: string;
  readonly label?: string;
  readonly subcategories?: readonly string[];
  readonly variantInheritance?: boolean;
};

export function defineAxes<C extends SearchCategory, T extends Record<string, string>>(
  category: C,
  axes: T,
): { readonly [K in keyof T]: AxisToken<C> } {
  return Object.fromEntries(
    Object.entries(axes).map(([axisKey, axisValue]) => [
      axisKey,
      { category, value: axisValue },
    ]),
  ) as { readonly [K in keyof T]: AxisToken<C> };
}

export function defineFamilies<
  C extends SearchCategory,
  T extends Record<
    string,
    {
      axis: AxisToken<C>;
      description: string;
      label?: string;
      subcategories?: readonly string[];
      variantInheritance?: boolean;
    }
  >,
>(
  category: C,
  families: T,
): { readonly [K in keyof T]: FamilyToken<C> } {
  return Object.fromEntries(
    Object.entries(families).map(([familyKey, familyValue]) => [
      familyKey,
      {
        category,
        value: familyKey,
        ...familyValue,
      },
    ]),
  ) as { readonly [K in keyof T]: FamilyToken<C> };
}

