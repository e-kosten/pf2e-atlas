import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalAppeaseProjectionDeclarations = [
  defineConceptProjections("ritual_appeasement", {
    affliction: {
      tag: "ritual_appeasement_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because restitution, offerings, ritual respect, or meeting a spiritual demand is central to ending the affliction.",
      adjacentTags: ["cursebreaking_resolution", "exorcism_resolution"],
    },
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Supports offerings, restitution, funerary observance, or appeasement ceremonies used to satisfy a spirit, haunt, curse, or sacred demand without directly expelling it.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from helping perform offerings, appeasement rites, restitution rituals, or ceremonial observance meant to settle a supernatural grievance.",
        "It is naturally sought for placation or ritual satisfaction rather than direct cleansing, banishment, or ordinary worship.",
      ],
      doesNotApplyWhen: [
        "The item only supports broad ritual process with no real appeasement, offering, or restitution-facing role.",
        "The stronger fit is sanctification or ritual_support because the item purifies generally or supports any rite rather than a placation answer path.",
      ],
      adjacentTags: ["ritual_support", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "appeasement_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved through offerings, ritual respect, social appeasement, or meeting a spiritual demand.",
      appliesWhen: [
        "The hazard meaningfully invites negotiation-by-ritual, restitution, reverence, or satisfying an unmet dead or sacred demand.",
        "The nonviolent spiritual answer is more central than simply disabling mechanics or dispelling magic.",
      ],
      doesNotApplyWhen: [
        "The hazard only needs a standard disable check, counteract, or forceful destruction.",
        "The hazard is haunted but has no appeasement-style resolution path.",
      ],
      adjacentTags: ["exorcism_countermeasure", "judgment_haunt"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Ends a supernatural problem through offerings, restitution, funerary respect, ritual observance, or otherwise satisfying a spiritual demand rather than expelling the presence outright.",
      adjacentTags: ["sanctification", "exorcism"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
