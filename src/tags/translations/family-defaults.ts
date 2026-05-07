import type { DerivedTagConceptSchemaKind, DerivedTagTranslationStatus } from "../../domain/derived-tag-types.js";

export type DerivedTagFamilyTranslationDefaults = {
  schemaKind: DerivedTagConceptSchemaKind;
  translationStatus: DerivedTagTranslationStatus;
  primaryFacetKind?: string;
  primaryFacetValue?: string;
  notes?: string;
};

export const DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS = new Map<string, DerivedTagFamilyTranslationDefaults>();

function setFamilies(keys: string[], config: DerivedTagFamilyTranslationDefaults): void {
  for (const key of keys) {
    DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS.set(key, config);
  }
}

setFamilies(
  ["affliction:impact", "affliction:physiology_override", "affliction:metaphysical_profile"],
  { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "effect", primaryFacetValue: "effect_profile", notes: "Effect/state descriptors." },
);
setFamilies(["affliction:pathogenesis"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "pathogenesis",
  primaryFacetValue: "pathogenesis",
  notes: "Disease-pattern descriptors.",
});
setFamilies(["affliction:epidemiological_profile"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "response_demand",
  primaryFacetValue: "response_demand",
  notes: "Heterogeneous browse family; rows split across response_demand, delivery, and theme.",
});
setFamilies(["affliction:response_profile"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "response_demand",
  primaryFacetValue: "response_demand",
  notes: "Response-side pressure family with approved operational exceptions.",
});
setFamilies(["affliction:behavioral_override"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "behavior_override",
  primaryFacetValue: "behavior_override",
});
setFamilies(["affliction:delivery_profile"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "delivery",
  primaryFacetValue: "delivery_profile",
});
setFamilies(["affliction:progression_profile"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "progression",
  primaryFacetValue: "progression_profile",
});
setFamilies(["affliction:resolution_profile"], {
  schemaKind: "operational",
  translationStatus: "mapped",
  notes: "Answer-path concepts.",
});

setFamilies(["creature:habitat_setting"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "setting", primaryFacetValue: "habitat" });
setFamilies(["creature:site_setting"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "setting", primaryFacetValue: "site" });
setFamilies(["creature:regional_setting"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "setting",
  primaryFacetValue: "regional",
  notes: "Approved primary setting shape.",
});
setFamilies(["creature:planar_setting"], { schemaKind: "descriptive", translationStatus: "provisional", primaryFacetKind: "setting", primaryFacetValue: "planar" });
setFamilies(["creature:genre_motif", "creature:story_motif", "creature:visual_motif"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "theme",
  primaryFacetValue: "theme",
});
setFamilies(["creature:bound_object"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "creature_family",
  primaryFacetValue: "bound_object_family",
});
setFamilies(["creature:combat_role"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "role", primaryFacetValue: "tactical" });
setFamilies(["creature:cohort_role", "creature:scene_role", "creature:social_role"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "role",
  primaryFacetValue: "role",
});
setFamilies(["creature:ontology_cluster"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "creature_family",
  primaryFacetValue: "creature_family",
});
setFamilies(["creature:casting_profile"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "capability",
  primaryFacetValue: "casting_profile",
});
setFamilies(["creature:corruption_profile"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "theme", primaryFacetValue: "corruption" });
setFamilies(["creature:threat_profile"], {
  schemaKind: "operational",
  translationStatus: "mapped",
  notes: "Heterogeneous operational family split row-by-row across application/control/creation/sustain/call/infiltration.",
});

setFamilies(
  ["equipment:consumable_role", "equipment:party_role", "equipment:play_pattern", "equipment:access_system", "equipment:defense_profile"],
  { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "capability", primaryFacetValue: "capability" },
);
setFamilies(["equipment:delivery_profile"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "delivery", primaryFacetValue: "delivery_profile" });
setFamilies(["equipment:impact"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "effect", primaryFacetValue: "effect_profile" });
setFamilies(["equipment:offensive_profile", "equipment:access_bypass", "equipment:anti_magic", "equipment:breaching", "equipment:resolution", "equipment:restraint"], {
  schemaKind: "operational",
  translationStatus: "provisional",
  notes: "Actionable answer/effect concepts; family-level shape still needs refinement in some spaces.",
});
setFamilies(["equipment:function"], {
  schemaKind: "operational",
  translationStatus: "mapped",
  notes: "Heterogeneous browse family split across remediation, support operations, and capability concepts.",
});
setFamilies(["equipment:ammunition_payload"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "delivery",
  primaryFacetValue: "payload",
  notes: "Payload rows reuse delivery semantics, with targeting specialization overrides.",
});
setFamilies(["equipment:carry_logistics", "equipment:communication", "equipment:crafting_support", "equipment:expedition", "equipment:infiltration", "equipment:movement_traversal", "equipment:reconnaissance", "equipment:security"], {
  schemaKind: "descriptive",
  translationStatus: "provisional",
  primaryFacetKind: "capability",
  primaryFacetValue: "capability",
});

setFamilies(["hazard:attack_vector", "hazard:environmental_danger", "hazard:forced_position", "hazard:impact", "hazard:perception_control", "hazard:countermeasure_profile"], {
  schemaKind: "operational",
  translationStatus: "provisional",
  notes: "Hazard effect and countermeasure tags.",
});
setFamilies(["hazard:function"], {
  schemaKind: "descriptive",
  translationStatus: "provisional",
  primaryFacetKind: "function",
  primaryFacetValue: "hazard_function",
});
setFamilies(["hazard:haunt_manifestation"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "theme", primaryFacetValue: "haunt_manifestation" });
setFamilies(["hazard:mechanism"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "mechanism",
  primaryFacetValue: "mechanism",
});
setFamilies(["hazard:problem_shape"], {
  schemaKind: "descriptive",
  translationStatus: "mapped",
  primaryFacetKind: "challenge_structure",
  primaryFacetValue: "challenge_structure",
});
setFamilies(["hazard:setting"], { schemaKind: "descriptive", translationStatus: "mapped", primaryFacetKind: "setting", primaryFacetValue: "site" });

setFamilies(["spell:control", "spell:impact", "spell:influence", "spell:summoning", "spell:support", "spell:transformation", "spell:access_bypass", "spell:resolution"], {
  schemaKind: "operational",
  translationStatus: "provisional",
  notes: "Operational spell effects or answer paths.",
});
setFamilies(["spell:communication", "spell:consultation", "spell:expedition", "spell:infiltration", "spell:reconnaissance", "spell:security", "spell:sensory_support", "spell:teleportation", "spell:wayfinding"], {
  schemaKind: "descriptive",
  translationStatus: "provisional",
  primaryFacetKind: "capability",
  primaryFacetValue: "capability",
});
setFamilies(["spell:revelation"], {
  schemaKind: "operational",
  translationStatus: "provisional",
  notes: "Discovery-side spell tags normalize as operational discover concepts.",
});

export function getDerivedTagFamilyTranslationDefaults(key: string): DerivedTagFamilyTranslationDefaults | undefined {
  return DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS.get(key);
}
