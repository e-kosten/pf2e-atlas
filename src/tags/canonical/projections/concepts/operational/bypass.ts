import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalBypassProjectionDeclarations = [
  defineConceptProjections("barrier_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps get through barred windows, grates, force screens, or other blocked passage without relying on brute-force breaching.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Gets a creature through a blocked threshold, wall, seal, force barrier, or magical ward that otherwise prevents passage.",
      appliesWhen: [
        "The spell is naturally retrieved to pass through, nullify, or ignore a blocking wall, sealed threshold, force barrier, or magical ward.",
        "Crossing the obstruction matters more than simply traveling farther or counteracting magic in the abstract.",
      ],
      doesNotApplyWhen: [
        "The spell only unlocks a door or manipulates a mechanism without really solving a barrier or ward.",
        "The spell's value is ordinary travel or relocation rather than penetrating a blocked passage.",
      ],
      adjacentTags: ["lock_bypass", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lock_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps open locks or bypass secured entry points.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Opens locks, sealed containers, secured doors, or similar closed access points through magic rather than physical lockpicking.",
      appliesWhen: [
        "The spell is naturally retrieved to unlock, unseal, or open a secured entry point, door, chest, manacle, or similar closure.",
        "Accessing something closed matters more than broad movement, damage, or generic anti-magic.",
      ],
      doesNotApplyWhen: [
        "The spell mainly destroys the obstacle, bypasses the whole wall, or teleports past the problem without interacting with the locked access point.",
        "The spell only manipulates unattended objects generally and opening secured access is not a real retrieval hook.",
      ],
      adjacentTags: ["trap_bypass", "barrier_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("procedural_bypass", {
    hazard: {
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best bypassed through the right route, timing, command phrase, ritual sequence, or other correct procedure rather than direct disarm.",
      appliesWhen: [
        "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
        "A GM would plausibly retrieve the hazard for puzzle-like bypass, safe-route discovery, or passphrase-style navigation.",
      ],
      doesNotApplyWhen: [
        "The main answer is simple mechanism tampering, counteracting magic, or appeasing a spirit.",
        "The hazard only has a minor caution or tactical workaround without a real procedural solution.",
      ],
      adjacentTags: ["physical_disarm", "false_safe_route"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("trap_bypass", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps disarm, disable, or get past traps.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Disarms, suppresses, safely triggers, or helps bypass a trap, warded threshold, or similar trapped access problem.",
      appliesWhen: [
        "The spell is naturally retrieved to disable, neutralize, or get past a trap or trapped access point without simply enduring the hazard.",
        "Trap-solving matters more than generic revelation, scouting, or damage prevention.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals that a trap exists without helping bypass or disable it.",
        "The spell mainly counters open combat hazards or battlefield effects rather than access-facing traps.",
      ],
      adjacentTags: ["lock_bypass", "mechanism_manipulation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
