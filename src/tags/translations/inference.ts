import type { DerivedTagTranslationRecord } from "../../domain/derived-tag-types.js";

type InferredTranslationDefaults = Pick<
  DerivedTagTranslationRecord,
  | "canonicalConceptId"
  | "canonicalConceptLabel"
  | "domainId"
  | "operation"
>;

export function inferOperationalTranslationDefaults(tag: string): InferredTranslationDefaults {
  if (tag.startsWith("anti_")) {
    const domain = tag.replace(/^anti_/, "");
    return { canonicalConceptId: `${domain}_remediation`, canonicalConceptLabel: `${domain}_remediation`, domainId: domain, operation: "remediate" };
  }
  if (tag.endsWith("_support")) {
    const domain = tag.replace(/_support$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "support" };
  }
  if (tag.endsWith("_control")) {
    const domain = tag.replace(/_control$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "control" };
  }
  if (tag.endsWith("_creation")) {
    const domain = tag.replace(/_creation$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "create" };
  }
  if (tag.endsWith("_damage")) {
    const domain = tag.replace(/_damage$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain || "damage", operation: "deal" };
  }
  if (tag.endsWith("_impairment")) {
    const domain = tag.replace(/_impairment$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "impair" };
  }
  if (tag.endsWith("_denial")) {
    const domain = tag.replace(/_denial$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "deny" };
  }
  if (tag.endsWith("_disruption")) {
    const domain = tag.replace(/_disruption$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "disrupt" };
  }
  if (tag.endsWith("_pressure")) {
    const domain = tag.replace(/_pressure$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "pressure" };
  }
  if (tag.endsWith("_breaking")) {
    const domain = tag.replace(/_breaking$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "break" };
  }
  if (tag.endsWith("_emitter")) {
    const domain = tag.replace(/_emitter$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "emit" };
  }
  if (tag.endsWith("_burst")) {
    const domain = tag.replace(/_burst$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "burst" };
  }
  if (tag.endsWith("_strike")) {
    const domain = tag.replace(/_strike$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "strike" };
  }
  if (tag.endsWith("_consultation")) {
    const domain = tag.replace(/_consultation$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "consult" };
  }
  if (tag.endsWith("_revelation")) {
    const domain = tag.replace(/_revelation$/, "");
    return { canonicalConceptId: `${domain}_discovery`, canonicalConceptLabel: `${domain}_discovery`, domainId: domain, operation: "discover" };
  }
  if (tag.endsWith("_reveal")) {
    const domain = tag.replace(/_reveal$/, "");
    return { canonicalConceptId: `${domain}_discovery`, canonicalConceptLabel: `${domain}_discovery`, domainId: domain, operation: "discover" };
  }
  if (tag.endsWith("_detection")) {
    const domain = tag.replace(/_detection$/, "");
    return { canonicalConceptId: `${domain}_discovery`, canonicalConceptLabel: `${domain}_discovery`, domainId: domain, operation: "discover" };
  }
  if (tag.endsWith("_guidance")) {
    const domain = tag.replace(/_guidance$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "guide" };
  }
  if (tag.endsWith("_diagnosis")) {
    const domain = tag.replace(/_diagnosis$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "diagnose" };
  }
  if (tag.endsWith("_bypass")) {
    const domain = tag.replace(/_bypass$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "bypass" };
  }
  if (tag.endsWith("_capture")) {
    const domain = tag.replace(/_capture$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "capture" };
  }
  if (tag.endsWith("_escape")) {
    const domain = tag.replace(/_escape$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "escape" };
  }
  if (tag.endsWith("_cleanup")) {
    const domain = tag.replace(/_cleanup$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "clean_up" };
  }
  if (tag.endsWith("_containment")) {
    const domain = tag.replace(/_containment$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "contain" };
  }
  if (tag.endsWith("_summoning")) {
    const domain = tag.replace(/_summoning$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain || "summon", operation: "summon" };
  }
  if (tag.endsWith("_summons")) {
    const domain = tag.replace(/_summons$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain || "summon", operation: "summon" };
  }
  if (tag.endsWith("_manipulation")) {
    const domain = tag.replace(/_manipulation$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "manipulate" };
  }
  if (tag.endsWith("_protection")) {
    const domain = tag.replace(/_protection$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "protect" };
  }
  if (tag.endsWith("_teleport")) {
    const domain = tag.replace(/_teleport$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain || "teleportation", operation: "teleport" };
  }
  if (tag.endsWith("_travel")) {
    const domain = tag.replace(/_travel$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain || "travel", operation: "travel" };
  }
  if (tag.endsWith("_hazard")) {
    const domain = tag.replace(/_hazard$/, "");
    return { canonicalConceptId: `${domain}_application`, canonicalConceptLabel: `${domain}_application`, domainId: domain, operation: "apply" };
  }
  if (tag.endsWith("_threat")) {
    const domain = tag.replace(/_threat$/, "");
    return { canonicalConceptId: `${domain}_application`, canonicalConceptLabel: `${domain}_application`, domainId: domain, operation: "apply" };
  }

  const exactMatches: Record<string, InferredTranslationDefaults> = {
    countermagic: { canonicalConceptId: "active_magic_counteraction", canonicalConceptLabel: "countermagic", domainId: "active_magic", operation: "counteract" },
    healing_support: { canonicalConceptId: "healing_support", canonicalConceptLabel: "healing_support", domainId: "healing", operation: "restore" },
    condition_support: { canonicalConceptId: "condition_support", canonicalConceptLabel: "condition_support", domainId: "condition", operation: "mitigate" },
    beneficial: { canonicalConceptId: "beneficial", canonicalConceptLabel: "beneficial", domainId: "benefit", operation: "provide" },
    energy_resistance: { canonicalConceptId: "energy_resistance", canonicalConceptLabel: "energy_resistance", domainId: "energy", operation: "resist" },
    mental_recovery: { canonicalConceptId: "mental_recovery", canonicalConceptLabel: "mental_recovery", domainId: "mental_state", operation: "recover" },
    burst_damage: { canonicalConceptId: "burst_damage", canonicalConceptLabel: "burst_damage", domainId: "area_damage", operation: "deal" },
    persistent_damage: { canonicalConceptId: "persistent_damage", canonicalConceptLabel: "persistent_damage", domainId: "persistent_damage", operation: "deal" },
    crowd_clearing: { canonicalConceptId: "crowd_clearing", canonicalConceptLabel: "crowd_clearing", domainId: "crowd", operation: "clear" },
    concealment: { canonicalConceptId: "concealment", canonicalConceptLabel: "concealment", domainId: "visibility", operation: "obscure" },
    silencing: { canonicalConceptId: "silencing", canonicalConceptLabel: "silencing", domainId: "sound", operation: "suppress" },
    single_target_removal: { canonicalConceptId: "single_target_removal", canonicalConceptLabel: "single_target_removal", domainId: "single_target", operation: "remove" },
    charm_influence: { canonicalConceptId: "charm_influence", canonicalConceptLabel: "charm_influence", domainId: "charm", operation: "influence" },
    domination: { canonicalConceptId: "domination", canonicalConceptLabel: "domination", domainId: "agency", operation: "dominate" },
    sleep_magic: { canonicalConceptId: "sleep_magic", canonicalConceptLabel: "sleep_magic", domainId: "sleep", operation: "induce" },
    exorcism: { canonicalConceptId: "hostile_presence_expulsion", canonicalConceptLabel: "exorcism", domainId: "hostile_presence", operation: "expel" },
    door_breaching: { canonicalConceptId: "door_breaching", canonicalConceptLabel: "door_breaching", domainId: "door", operation: "breach" },
    breaching: { canonicalConceptId: "breaching", canonicalConceptLabel: "breaching", domainId: "barrier", operation: "breach" },
    demolition: { canonicalConceptId: "demolition", canonicalConceptLabel: "demolition", domainId: "structure", operation: "demolish" },
    excavation: { canonicalConceptId: "excavation", canonicalConceptLabel: "excavation", domainId: "terrain", operation: "excavate" },
    resolution: { canonicalConceptId: "problem_resolution", canonicalConceptLabel: "problem_resolution", domainId: "problem", operation: "resolve" },
    revelation: { canonicalConceptId: "problem_discovery", canonicalConceptLabel: "problem_discovery", domainId: "problem", operation: "discover" },
    ritual_appeasement: { canonicalConceptId: "ritual_appeasement", canonicalConceptLabel: "ritual_appeasement", domainId: "ritual_grievance", operation: "appease" },
    sanctification: { canonicalConceptId: "sacred_taint_sanctification", canonicalConceptLabel: "sanctification", domainId: "sacred_taint", operation: "sanctify" },
    affliction_cleanup: { canonicalConceptId: "affliction_cleanup", canonicalConceptLabel: "affliction_cleanup", domainId: "affliction", operation: "clean_up" },
    protective_ward: { canonicalConceptId: "protective_ward", canonicalConceptLabel: "protective_ward", domainId: "ward", operation: "create" },
    death_prevention: { canonicalConceptId: "death_prevention", canonicalConceptLabel: "death_prevention", domainId: "death", operation: "prevent" },
    resistance_support: { canonicalConceptId: "resistance_support", canonicalConceptLabel: "resistance_support", domainId: "resistance", operation: "grant" },
    temporary_hp_support: { canonicalConceptId: "temporary_hp_support", canonicalConceptLabel: "temporary_hp_support", domainId: "temporary_hp", operation: "grant" },
    quickened_support: { canonicalConceptId: "quickened_support", canonicalConceptLabel: "quickened_support", domainId: "quickened", operation: "grant" },
    initiative_support: { canonicalConceptId: "initiative_support", canonicalConceptLabel: "initiative_support", domainId: "initiative", operation: "boost" },
    eidolon_support: { canonicalConceptId: "eidolon_support", canonicalConceptLabel: "eidolon_support", domainId: "eidolon", operation: "support" },
    transformation: { canonicalConceptId: "transformation", canonicalConceptLabel: "transformation", domainId: "form", operation: "transform" },
    forced_movement: { canonicalConceptId: "forced_movement", canonicalConceptLabel: "forced_movement", domainId: "position", operation: "reposition" },
    pitfall: { canonicalConceptId: "pitfall", canonicalConceptLabel: "pitfall", domainId: "fall", operation: "cause" },
    floor_eruption: { canonicalConceptId: "floor_eruption", canonicalConceptLabel: "floor_eruption", domainId: "ground", operation: "erupt" },
    false_safe_route: { canonicalConceptId: "false_safe_route", canonicalConceptLabel: "false_safe_route", domainId: "route", operation: "mislead" },
    illusion_assault: { canonicalConceptId: "illusion_assault", canonicalConceptLabel: "illusion_assault", domainId: "perception", operation: "assault" },
    navigation_disruption: { canonicalConceptId: "navigation_disruption", canonicalConceptLabel: "navigation_disruption", domainId: "navigation", operation: "disrupt" },
    physical_disarm: { canonicalConceptId: "physical_disarm", canonicalConceptLabel: "physical_disarm", domainId: "mechanism", operation: "disarm" },
    spawn_creator: { canonicalConceptId: "spawn_creation", canonicalConceptLabel: "spawn_creation", domainId: "spawn", operation: "create" },
    ambush_grabber: { canonicalConceptId: "ambush_grab", canonicalConceptLabel: "ambush_grab", domainId: "grapple", operation: "ambush" },
    prey_control_threat: { canonicalConceptId: "prey_control", canonicalConceptLabel: "prey_control", domainId: "prey", operation: "control" },
    regeneration_threat: { canonicalConceptId: "regeneration", canonicalConceptLabel: "regeneration", domainId: "regeneration", operation: "sustain" },
    reinforcement_threat: { canonicalConceptId: "reinforcement", canonicalConceptLabel: "reinforcement", domainId: "reinforcement", operation: "call" },
    infiltration_threat: { canonicalConceptId: "infiltration", canonicalConceptLabel: "infiltration", domainId: "infiltration", operation: "infiltrate" },
    death_burst_threat: { canonicalConceptId: "death_burst", canonicalConceptLabel: "death_burst", domainId: "death_burst", operation: "apply" },
    battle_form: { canonicalConceptId: "battle_form", canonicalConceptLabel: "battle_form", domainId: "battle_form", operation: "transform" },
    creature_summoning: { canonicalConceptId: "creature_summoning", canonicalConceptLabel: "creature_summoning", domainId: "creature", operation: "summon" },
    undead_summoning: { canonicalConceptId: "undead_summoning", canonicalConceptLabel: "undead_summoning", domainId: "undead", operation: "summon" },
    summoned_servitor: { canonicalConceptId: "summoned_servitor", canonicalConceptLabel: "summoned_servitor", domainId: "servitor", operation: "summon" },
    telepathic_communication: { canonicalConceptId: "telepathic_communication", canonicalConceptLabel: "telepathic_communication", domainId: "communication", operation: "enable" },
    message_delivery: { canonicalConceptId: "message_delivery", canonicalConceptLabel: "message_delivery", domainId: "message", operation: "deliver" },
    translation_support: { canonicalConceptId: "translation_support", canonicalConceptLabel: "translation_support", domainId: "language", operation: "bridge" },
    signaling: { canonicalConceptId: "signaling", canonicalConceptLabel: "signaling", domainId: "signal", operation: "send" },
  };

  if (tag.endsWith("_form")) {
    const domain = tag.replace(/_form$/, "");
    return { canonicalConceptId: tag, canonicalConceptLabel: tag, domainId: domain, operation: "transform" };
  }

  return exactMatches[tag] ?? { canonicalConceptId: tag, canonicalConceptLabel: tag };
}
