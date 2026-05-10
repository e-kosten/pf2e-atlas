import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellSummoningProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.SUMMONING_SUMMONING, {
    creature_summoning: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Summons, conjures, or calls creatures to act as temporary allies, tools, or battlefield assets.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    offensive_summons: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates summons primarily retrieved for direct damage, flanking pressure, or aggressive battlefield threat.",
      adjacentTags: ["creature_summoning", "screening_summons"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    scouting_summons: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates a summon primarily valued for reconnaissance, watch duty, sensing, or forward information gathering.",
      adjacentTags: ["summoned_servitor", "creature_summoning"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    screening_summons: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates bodies mainly used to block space, absorb hits, or protect allies rather than maximize damage.",
      adjacentTags: ["creature_summoning", "temporary_hp_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    summoned_servitor: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates a helper, laborer, scout, mount, or similarly task-focused magical servitor rather than a pure combat summon.",
      appliesWhen: [
        "The spell is naturally retrieved for utility help, labor, scouting, transport, or task performance rather than frontline combat stats.",
        "The conjured ally behaves more like a helper or specialist tool than a main battle summon.",
      ],
      doesNotApplyWhen: [
        "The spell's main value is summoning a combat creature to attack, flank, or absorb hits.",
        "The spell only creates an object, barrier, or terrain effect without a real servant-like entity.",
      ],
      adjacentTags: ["creature_summoning", "scouting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    undead_summoning: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Summons, calls, or manifests undead entities, spirits of the dead, or corpse-driven servitors.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
