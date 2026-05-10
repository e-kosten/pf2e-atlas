import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalContainProjectionDeclarations = [
  defineConceptProjections("outbreak_containment", {
    affliction: [
      {
        tag: "outbreak_management",
        axis: "response",
        family: "response_profile",
        assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
        description:
          "Naturally retrieved as a disease or curse that creates a wider containment, treatment, and community-management problem rather than only an isolated victim.",
        adjacentTags: ["community_outbreak", "quarantine_risk"],
      },
      {
        tag: "quarantine_containment_resolution",
        axis: "response",
        family: "resolution_profile",
        assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
        description:
          "Naturally retrieved because isolation, contact control, and containment are core to preventing further spread while treatment proceeds.",
        adjacentTags: ["outbreak_management", "quarantine_risk"],
      },
      {
        tag: "quarantine_risk",
        axis: "response",
        family: "response_profile",
        assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
        description:
          "Creates a strong need to isolate victims, restrict contact, or manage who can safely enter or leave an affected area.",
        adjacentTags: ["community_outbreak", "inhaled_exposure"],
      },
    ],
    equipment: {
      tag: "quarantine_containment",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps isolate victims, secure contaminated areas, or impose practical containment procedures that stop spread while treatment proceeds.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["contamination_cleanup", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "quarantine_containment_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best managed by isolating victims, sealing off the site, or imposing containment boundaries that stop spread while the danger is being handled.",
      appliesWhen: [
        "The hazard is naturally retrieved because the first meaningful answer is locking down spread, controlling access, or containing dangerous exposure.",
        "Containment procedures matter more than immediately dispelling, disarming, or appeasing the hazard.",
      ],
      doesNotApplyWhen: [
        "The hazard is dangerous but does not meaningfully spread, linger, or demand isolation and access control.",
        "The stronger fit is barrier_lockdown or sentinel_guardian because preventing passage is the hazard's function, not the party's resolution plan.",
      ],
      adjacentTags: ["contamination_cleanup_countermeasure", "source_cleanup_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "quarantine_containment",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Helps isolate victims, secure a dangerous area, or impose protective boundaries that stop spread while the problem is being solved.",
      adjacentTags: ["protective_ward", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
