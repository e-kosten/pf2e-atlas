import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const hazardHauntProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.HAUNT_HAUNT_MANIFESTATION, {
    battlefield_disruption: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
    },
    judgment_haunt: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Haunt that condemns trespassers through accusation, punishment, curse-like verdicts, or moral reckoning.",
    },
    life_drain_hazard: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that drains life force, vitality, or souls from victims.",
    },
    lure_compulsion: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
    },
    phantom_assailants: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
    },
    possession_haunt: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that enters, rides, or briefly controls a victim rather than only attacking them externally.",
    },
    replayed_tragedy: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that re-enacts a murder, betrayal, execution, disaster, or other fixed traumatic event.",
      appliesWhen: [
        "The haunt is naturally retrieved because it replays a specific past calamity, crime, execution, or emotional flashpoint as its core manifestation.",
        "The narrative repetition of an old event matters more than generic life drain, possession, or battlefield disruption.",
      ],
      doesNotApplyWhen: [
        "The haunt is only sad, angry, or spiritually active without reenacting a fixed historical scene.",
        "The stronger fit is judgment_haunt or lure_compulsion because the recurring tragedy itself is not the central hook.",
      ],
      adjacentTags: ["judgment_haunt", "appeasement_countermeasure"],
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];
