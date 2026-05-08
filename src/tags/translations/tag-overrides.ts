import type { DerivedTagTranslationMapping } from "../../domain/derived-tag-types.js";

export type DerivedTagTranslationOverride = Partial<
  Pick<DerivedTagTranslationMapping, "targetProjectionId" | "translationStatus" | "renameNote" | "notes">
>;

export const DERIVED_TAG_TRANSLATION_OVERRIDES = new Map<string, DerivedTagTranslationOverride>([
  ["affliction:antidote_resolution", { renameNote: "Aligns affliction-side antidote demand with spell/equipment anti_poison." }],
  ["affliction:cursebreaking_resolution", { renameNote: "Aligns with spell/equipment curse_removal." }],
  ["affliction:quarantine_containment_resolution", { renameNote: "Canonical rename from quarantine_containment to outbreak_containment." }],
  ["affliction:source_tracing", { renameNote: "Unifies source_tracing with source_revelation." }],
  ["affliction:quarantine_risk", { renameNote: "Operational move to outbreak_containment." }],
  ["affliction:outbreak_management", { renameNote: "Collapse from outbreak_management into outbreak_containment." }],
  ["affliction:cure_clock_urgency", { renameNote: "Approved rename from cure_clock_urgency to time_critical_resolution." }],
  ["affliction:epidemic_pestilence", { notes: "Thematic rather than pure response-demand descriptor." }],
  ["equipment:energy_resistance", { notes: "Capability-style support concept rather than direct answer path." }],
  ["equipment:senses_support", { notes: "Capability-style enhancement concept rather than direct answer path." }],
  ["equipment:creature_bane", { notes: "Targeting specialization rather than delivery." }],
  ["spell:revelation", { renameNote: "Discovery umbrella renamed from revelation to problem_discovery." }],
  ["spell:quarantine_containment", { renameNote: "Canonical rename from quarantine_containment to outbreak_containment." }],
  ["equipment:quarantine_containment", { renameNote: "Canonical rename from quarantine_containment to outbreak_containment." }],
  ["hazard:observation_first", { renameNote: "Rename from observation_first to observation_driven." }],
  ["hazard:layered_resolution", { renameNote: "Rename from layered_resolution to multi_stage_resolution." }],
  ["hazard:sentinel_guardian", { renameNote: "Rename from sentinel_guardian to guard_post." }],
  ["creature:undead_adjacent", { renameNote: "Creature-family normalization from undead_adjacent to undead_family." }],
]);

export function getDerivedTagTranslationOverride(key: string): DerivedTagTranslationOverride | undefined {
  return DERIVED_TAG_TRANSLATION_OVERRIDES.get(key);
}
