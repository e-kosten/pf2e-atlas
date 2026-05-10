import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveFunctionProjectionDeclarations = [
  defineConceptProjections("ambush_burst", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard designed to open with a sudden high-damage strike or surprise punish on first contact.",
      appliesWhen: [
        "The hazard is naturally retrieved for a sudden opener, trap-spring punish, or first-contact burst that catches intruders before a longer fight develops.",
        "The surprise spike matters more than sustained zone control or prolonged attrition.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly holds territory over time, guards a place persistently, or taxes resources across repeated rounds.",
        "The stronger fit is zone_denial or attrition_pressure rather than a front-loaded strike.",
      ],
      adjacentTags: ["zone_denial", "attrition_pressure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("area_denial", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Legacy synonym for zone_denial preserved for compatibility while downstream hazard planning surfaces migrate to the simpler area-control vocabulary.",
      adjacentTags: ["zone_denial", "sentinel_guardian"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("attrition_pressure", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard whose primary role is to wear the party down over time rather than deliver one decisive spike.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("barrier_lockdown", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forced_separation", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that splits allies apart through walls, drops, slides, teleports, or other positional disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("forced_separation_hazard", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Legacy synonym for forced_separation preserved for compatibility while downstream surfaces migrate to the simpler split-party vocabulary.",
      adjacentTags: ["forced_separation", "pursuit_punisher"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("guard_post", {
    hazard: {
      tag: "sentinel_guardian",
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard whose role is to guard an area, treasure, threshold, or sanctum as a standing defense layer.",
      appliesWhen: [
        "The hazard is naturally retrieved as a guardian layer protecting a place, object, or route from intrusion.",
        "Its value is in persistent watchfulness or defensive coverage, not just burst damage.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly creates open-area denial with no strong guard-post identity.",
        "The hazard is mostly an ambush opener or chase-punishment device.",
      ],
      adjacentTags: ["alarm", "zone_denial"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("pursuit_punisher", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that punishes retreat, pursuit, escape routes, or movement through chase-style spaces.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("resource_drain", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that taxes healing, spellcasting, equipment durability, or other party resources over time.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("spawned_attackers", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that summons, creates, or releases separate attackers into the scene.",
      appliesWhen: [
        "The hazard is naturally retrieved because it adds new hostile creatures, constructs, swarms, or manifestations to the scene.",
        "The extra attackers matter as separate encounter pressure rather than only as a damage effect.",
      ],
      doesNotApplyWhen: [
        "The hazard only deals direct damage, restrains victims, or creates a temporary illusion without generating distinct assailants.",
        "The stronger fit is phantom_assailants when the threat is specifically a haunt manifestation rather than a broader hazard function.",
      ],
      adjacentTags: ["alarm", "phantom_assailants"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("zone_denial", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that makes an area costly to enter, cross, or remain inside.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
