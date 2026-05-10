import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveMechanismProjectionDeclarations = [
  defineConceptProjections("control_interface", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
    },
  }),
  defineConceptProjections("planar_breach", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
      appliesWhen: [
        "An unstable portal, planar tear, extradimensional rupture, or reality breach is the central mechanism of the hazard.",
        "The hazard is naturally retrieved for cosmological leakage, portal instability, or something dangerous coming through a breach.",
      ],
      doesNotApplyWhen: [
        "The hazard is merely magical or teleportive without a real opening in reality as the core hazard engine.",
        "The stronger fit is dispel_countermeasure or procedural_bypass because the planar flavor is incidental.",
      ],
      adjacentTags: ["dispel_countermeasure", "procedural_bypass"],
    },
  }),
  defineConceptProjections("pressure_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
    },
  }),
  defineConceptProjections("threshold_lockdown", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
    },
  }),
  defineConceptProjections("tripwire_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
    },
  }),
  defineConceptProjections("ward_trigger", {
    hazard: {
      axis: "mechanism",
      family: "mechanism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];
