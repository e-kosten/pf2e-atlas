import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const hazardMechanismProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.MECHANISM_MECHANISM, {
    control_interface: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
    },
    planar_breach: {
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
    pressure_trigger: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
    },
    threshold_lockdown: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
    },
    tripwire_trigger: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
    },
    ward_trigger: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];
