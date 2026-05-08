import { buildCanonicalConceptMap, type CanonicalConceptSeed } from "../builders.js";
import { CANONICAL_VOCABULARY } from "../vocabulary.js";

const DERIVED_TAG_OPERATIONAL_CANONICAL_CONCEPTS_BY_ID_SEEDS: Record<string, CanonicalConceptSeed> = {
  acid_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ACID,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  active_magic_counteraction: {
    label: "countermagic",
    domainId: CANONICAL_VOCABULARY.DOMAIN.ACTIVE_MAGIC,
    operation: CANONICAL_VOCABULARY.OPERATION.COUNTERACT,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  affliction_cleanup: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.AFFLICTION,
    operation: CANONICAL_VOCABULARY.OPERATION.CLEAN_UP,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  ambush_grab: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.GRAPPLE,
    operation: CANONICAL_VOCABULARY.OPERATION.AMBUSH,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  animal_form: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ANIMAL,
    operation: CANONICAL_VOCABULARY.OPERATION.TRANSFORM,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  barrier_breaking: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BARRIER,
    operation: CANONICAL_VOCABULARY.OPERATION.BREAK,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  barrier_bypass: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BARRIER,
    operation: CANONICAL_VOCABULARY.OPERATION.BYPASS,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  barrier_creation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BARRIER,
    operation: CANONICAL_VOCABULARY.OPERATION.CREATE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  battle_form: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BATTLE,
    operation: CANONICAL_VOCABULARY.OPERATION.TRANSFORM,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  bleed_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BLEED,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  blight_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BLIGHT,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  buff_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BUFF,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  burst_damage: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.BURST,
    operation: CANONICAL_VOCABULARY.OPERATION.DEAL,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  caster_disruption_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CASTER_DISRUPTION,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  charm_influence: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CHARM,
    operation: CANONICAL_VOCABULARY.OPERATION.INFLUENCE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  cold_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.COLD,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  collapse_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.COLLAPSE,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  compulsion_control: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.COMPULSION,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTROL,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  condition_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CONDITION,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  confusion_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CONDITION,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  contamination_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CONTAMINATION,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  contamination_cleanup: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CONTAMINATION,
    operation: CANONICAL_VOCABULARY.OPERATION.CLEAN_UP,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  creature_summoning: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CREATURE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  crowd_clearing: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CROWD,
    operation: CANONICAL_VOCABULARY.OPERATION.CLEAR,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  curse_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CURSE,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  curse_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CURSE,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  curse_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CURSE,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  cursefield_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.CURSEFIELD,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  death_burst: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DEATH_BURST,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  death_prevention: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DEATH,
    operation: CANONICAL_VOCABULARY.OPERATION.PREVENT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  demolition: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.STRUCTURE,
    operation: CANONICAL_VOCABULARY.OPERATION.DEMOLISH,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  disease_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DISEASE,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  disease_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DISEASE,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  domination: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.AGENCY,
    operation: CANONICAL_VOCABULARY.OPERATION.DOMINATE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  door_breaching: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.DOOR,
    operation: CANONICAL_VOCABULARY.OPERATION.BREACH,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  eidolon_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.EIDOLON,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  electric_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ELECTRIC,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  elemental_form: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ELEMENTAL,
    operation: CANONICAL_VOCABULARY.OPERATION.TRANSFORM,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  emotion_control: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.EMOTION,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTROL,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  escape_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ESCAPE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  excavation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.TERRAIN,
    operation: CANONICAL_VOCABULARY.OPERATION.EXCAVATE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  false_safe_route: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.ROUTE,
    operation: CANONICAL_VOCABULARY.OPERATION.MISLEAD,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  fear_pressure: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FEAR,
    operation: CANONICAL_VOCABULARY.OPERATION.PRESSURE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  fear_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FEAR,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  fire_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FIRE,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  floor_eruption: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.GROUND,
    operation: CANONICAL_VOCABULARY.OPERATION.ERUPT,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  forced_movement: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.POSITION,
    operation: CANONICAL_VOCABULARY.OPERATION.REPOSITION,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  fortune_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FORTUNE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  hazard_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.HAZARD,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  healing_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.HEALING,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  hostile_presence_expulsion: {
    label: "exorcism",
    domainId: CANONICAL_VOCABULARY.DOMAIN.HOSTILE_PRESENCE,
    operation: CANONICAL_VOCABULARY.OPERATION.EXPEL,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  illusion_assault: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PERCEPTION,
    operation: CANONICAL_VOCABULARY.OPERATION.ASSAULT,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  infiltration: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.INFILTRATION,
    operation: CANONICAL_VOCABULARY.OPERATION.INFILTRATE,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  initiative_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.INITIATIVE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  invisibility_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.INVISIBILITY,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  life_drain_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.LIFE_DRAIN,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  line_of_sight_control: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.LINE_OF_SIGHT,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTROL,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  lock_bypass: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.LOCK,
    operation: CANONICAL_VOCABULARY.OPERATION.BYPASS,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  magic_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MAGIC,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  magic_protection: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MAGIC,
    operation: CANONICAL_VOCABULARY.OPERATION.PROTECT,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  mechanism_manipulation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MECHANISM,
    operation: CANONICAL_VOCABULARY.OPERATION.MANIPULATE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  memory_manipulation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MEMORY,
    operation: CANONICAL_VOCABULARY.OPERATION.MANIPULATE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  mental_recovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MENTAL_STATE,
    operation: CANONICAL_VOCABULARY.OPERATION.RECOVER,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  mobility_denial: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MOBILITY,
    operation: CANONICAL_VOCABULARY.OPERATION.DENY,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  navigation_disruption: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.NAVIGATION,
    operation: CANONICAL_VOCABULARY.OPERATION.DISRUPT,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  offensive_summons: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.OFFENSIVE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  outbreak_containment: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.OUTBREAK,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTAIN,
    text: {
      definition: "Response-side pressure family with approved operational exceptions.",
    },
  },
  overgrowth_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.OVERGROWTH,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  overhead_strike: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.OVERHEAD,
    operation: CANONICAL_VOCABULARY.OPERATION.STRIKE,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  paralysis_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PARALYSIS,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  parasite_removal: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PARASITE_OR_IMPLANT,
    operation: CANONICAL_VOCABULARY.OPERATION.REMOVE,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  persistent_damage: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PERSISTENT,
    operation: CANONICAL_VOCABULARY.OPERATION.DEAL,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  petrification_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PETRIFICATION,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  petrification_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PETRIFICATION,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
    },
  },
  physical_disarm: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.MECHANISM,
    operation: CANONICAL_VOCABULARY.OPERATION.DISARM,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  pitfall: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.FALL,
    operation: CANONICAL_VOCABULARY.OPERATION.CAUSE,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  poison_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.POISON,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  poison_remediation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.POISON,
    operation: CANONICAL_VOCABULARY.OPERATION.REMEDIATE,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  possession_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.POSSESSION,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  prey_control: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PREY,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTROL,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  procedural_bypass: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PROCEDURAL,
    operation: CANONICAL_VOCABULARY.OPERATION.BYPASS,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  projectile_emitter: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PROJECTILE,
    operation: CANONICAL_VOCABULARY.OPERATION.EMIT,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  protective_ward: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.WARD,
    operation: CANONICAL_VOCABULARY.OPERATION.CREATE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  proximity_burst: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.PROXIMITY,
    operation: CANONICAL_VOCABULARY.OPERATION.BURST,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  quickened_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.QUICKENED,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  regeneration: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.REGENERATION,
    operation: CANONICAL_VOCABULARY.OPERATION.SUSTAIN,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  reinforcement: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.REINFORCEMENT,
    operation: CANONICAL_VOCABULARY.OPERATION.CALL,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  resistance_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.RESISTANCE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  respiratory_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.RESPIRATORY,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  restraint_capture: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.RESTRAINT,
    operation: CANONICAL_VOCABULARY.OPERATION.CAPTURE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  restraint_escape: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.RESTRAINT,
    operation: CANONICAL_VOCABULARY.OPERATION.ESCAPE,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  ritual_appeasement: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.RITUAL_GRIEVANCE,
    operation: CANONICAL_VOCABULARY.OPERATION.APPEASE,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  sacred_taint_sanctification: {
    label: "sanctification",
    domainId: CANONICAL_VOCABULARY.DOMAIN.SACRED_TAINT,
    operation: CANONICAL_VOCABULARY.OPERATION.SANCTIFY,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  scouting_summons: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SCOUTING,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  screening_summons: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SCREENING,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  siege_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SIEGE,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  single_target_removal: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SINGLE_TARGET,
    operation: CANONICAL_VOCABULARY.OPERATION.REMOVE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  sleep_magic: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SLEEP,
    operation: CANONICAL_VOCABULARY.OPERATION.INDUCE,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  sound_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SOUND,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
  source_cleanup: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.HIDDEN_SOURCE,
    operation: CANONICAL_VOCABULARY.OPERATION.CLEAN_UP,
    text: {
      definition: "Answer-path concepts.",
    },
  },
  source_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.HIDDEN_SOURCE,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Response-side pressure family with approved operational exceptions.",
    },
  },
  spawn_creation: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SPAWN,
    operation: CANONICAL_VOCABULARY.OPERATION.CREATE,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  summoned_servitor: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.SERVITOR,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  temporary_hp_support: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.TEMPORARY_HP,
    operation: CANONICAL_VOCABULARY.OPERATION.SUPPORT,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  terrain_control: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.TERRAIN,
    operation: CANONICAL_VOCABULARY.OPERATION.CONTROL,
    text: {
      definition: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
    },
  },
  trap_bypass: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.TRAP,
    operation: CANONICAL_VOCABULARY.OPERATION.BYPASS,
    text: {
      definition: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
    },
  },
  truth_discovery: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.TRUTH,
    operation: CANONICAL_VOCABULARY.OPERATION.DISCOVER,
    text: {
      definition: "Discovery-side spell tags normalize as operational discover concepts.",
    },
  },
  undead_summoning: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.UNDEAD,
    operation: CANONICAL_VOCABULARY.OPERATION.SUMMON,
    text: {
      definition: "Operational spell effects or answer paths.",
    },
  },
  water_application: {
    domainId: CANONICAL_VOCABULARY.DOMAIN.WATER,
    operation: CANONICAL_VOCABULARY.OPERATION.APPLY,
    text: {
      definition: "Hazard effect and countermeasure tags.",
    },
  },
};

export const DERIVED_TAG_OPERATIONAL_CANONICAL_CONCEPTS_BY_ID = buildCanonicalConceptMap(
  CANONICAL_VOCABULARY.SCHEMA.KIND.OPERATIONAL,
  DERIVED_TAG_OPERATIONAL_CANONICAL_CONCEPTS_BY_ID_SEEDS,
);
