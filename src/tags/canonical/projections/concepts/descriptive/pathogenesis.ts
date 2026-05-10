import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptivePathogenesisProjectionDeclarations = [
  defineConceptProjections("bestial_transformation", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Defined by animalistic mutation, feral reshaping, or a cursed slide into beastlike behavior and form.",
      appliesWhen: [
        "The affliction's retrieval value comes from becoming feral, animalistic, lycanthropic, or beast-shaped over time.",
        "Behavioral and bodily slide toward a beast identity are both salient.",
      ],
      doesNotApplyWhen: [
        "The transformation is general corruption without a real animalistic or feral endpoint.",
        "The affliction only compels violent behavior without reshaping the victim's form or identity.",
      ],
      adjacentTags: ["transformative_corruption", "violence_compulsion"],
    },
  }),
  defineConceptProjections("blood_rot", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Defined by corrupted blood, blackened veins, hemorrhagic poisoning, or other blood-borne bodily collapse.",
      appliesWhen: [
        "Corrupted blood, blackened veins, or bloodstream collapse is the core thematic identity of the affliction.",
        "A user would retrieve it for blood-plague, hemorrhagic corruption, or vein-darkening disease imagery.",
      ],
      doesNotApplyWhen: [
        "The affliction only causes bleeding as one symptom without a real blood-corruption identity.",
        "The stronger fit is hemorrhagic_failure because the focus is immediate bleeding outcome rather than disease theme.",
      ],
      adjacentTags: ["hemorrhagic_failure", "physical_debilitation"],
    },
  }),
  defineConceptProjections("fungal_growth", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Defined by fungal bloom, spores, mycelial takeover, or mushroom-like growths spreading through the body.",
    },
  }),
  defineConceptProjections("infestation_implant", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
    },
  }),
  defineConceptProjections("petrifying_corruption", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Defined by calcification, ossification, stone-turning, or gradual bodily hardening toward an inert form.",
      appliesWhen: [
        "The affliction is retrieved for gradual hardening, calcification, or stoneward corruption across stages.",
        "The process of becoming stone-like matters as much as the final condition.",
      ],
      doesNotApplyWhen: [
        "The affliction chiefly imposes a final petrified state without broader progressive corruption framing.",
        "The stronger fit is transformative_corruption without specifically stoneward identity.",
      ],
      adjacentTags: ["petrification", "cumulative_transformation"],
    },
  }),
  defineConceptProjections("rot_decay", {
    affliction: {
      axis: "disease_model",
      family: "pathogenesis",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
