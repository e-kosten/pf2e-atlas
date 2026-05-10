import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDiscoverProjectionDeclarations = [
  defineConceptProjections("curse_discovery", {
    spell: {
      tag: "curse_revelation",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Identifies curses, spiritual corruption, or other malign supernatural bindings on a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("hazard_discovery", {
    spell: {
      tag: "hazard_revelation",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Reveals hidden traps, secret wards, concealed passage dangers, or other obscured environmental threats.",
      appliesWhen: [
        "The spell is naturally retrieved to uncover traps, hidden dangers, secret magical wards, or dangerous concealed features in a location.",
        "Hazard discovery matters more than general magical detection or long-range scouting.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisible creatures, or truth without specifically surfacing dangerous hidden features.",
        "The spell merely scouts an area from afar without exposing concealed trap logic or hazard placement.",
      ],
      adjacentTags: ["magic_detection", "scouting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("invisibility_discovery", {
    spell: {
      tag: "invisibility_reveal",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Exposes invisible, hidden, concealed, or magically obscured creatures and objects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("magic_discovery", {
    spell: {
      tag: "magic_detection",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Reveals magical auras, spell presence, active effects, or other supernatural signatures.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("source_discovery", {
    affliction: {
      tag: "source_tracing",
      axis: "response",
      family: "response_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because finding the contaminated source, carrier chain, cursed origin, or initial spread event is central to solving the problem.",
      adjacentTags: ["carrier_vector", "community_outbreak"],
    },
    equipment: {
      tag: "source_revelation",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps identify the cursed anchor, contaminated material, infected origin, hidden carrier, or other source driving the problem before cleanup begins.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from finding or confirming the hidden source of a curse, contamination, outbreak, or spiritually tainted problem.",
        "It is naturally sought for tracing the origin or anchor rather than directly cleansing or disposing of it.",
      ],
      doesNotApplyWhen: [
        "The item only helps perform a cleanup, disposal, or purification step after the source is already known.",
        "The stronger fit is medical_support, tracking, or ritual_support because the item supports a broader process without specifically revealing the source.",
      ],
      adjacentTags: ["source_cleanup", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "source_tracing",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard whose real puzzle is locating the hidden anchor, leak point, cursed source, contamination engine, or origin node before a clean solution becomes possible.",
      appliesWhen: [
        "The hazard is naturally retrieved because identifying the source object, origin point, or contamination engine is a major part of solving it.",
        "Finding what is powering the danger matters more than merely surviving exposure or executing a known disable procedure.",
      ],
      doesNotApplyWhen: [
        "The hazard is fully understandable up front and the real challenge is timing, endurance, or multi-step execution rather than finding an origin.",
        "The stronger fit is source_cleanup_countermeasure because the source is already obvious and the remaining task is neutralizing it.",
      ],
      adjacentTags: ["observation_first", "source_cleanup_countermeasure"],
    },
    spell: {
      tag: "source_revelation",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Reveals the hidden source, curse anchor, carrier, infected origin, or spreading point of a supernatural or outbreak problem.",
      adjacentTags: ["problem_diagnosis", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("truth_discovery", {
    spell: {
      tag: "truth_reveal",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces honesty, exposes lies, or reveals disguised, false, or hidden truths.",
      appliesWhen: [
        "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
        "A user would plausibly look for it when they need an answer spell rather than a sensor spell.",
      ],
      doesNotApplyWhen: [
        "The spell only detects magic, invisibility, or general auras without interrogating truth or deception.",
        "The spell mainly alters memory or emotions rather than revealing facts.",
      ],
      adjacentTags: ["magic_detection", "memory_manipulation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
