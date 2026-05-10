import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const AggregateSettingProjectionDeclarations = [
  defineConceptProjections("cosmic_framework_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Strongly associated with the cosmic framework planes of Axis, the Boneyard, and the Maelstrom, which govern order, judgment, and transformative change.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["axis_setting", "boneyard_setting", "maelstrom_setting"],
      compositeOfAnyTags: ["axis_setting", "boneyard_setting", "maelstrom_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("elemental_plane_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Strongly associated with one of the elemental planes of Fire, Air, Water, or Earth.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: [
        "plane_of_fire_setting",
        "plane_of_air_setting",
        "plane_of_water_setting",
        "plane_of_earth_setting",
      ],
      compositeOfAnyTags: [
        "plane_of_fire_setting",
        "plane_of_air_setting",
        "plane_of_water_setting",
        "plane_of_earth_setting",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("lower_plane_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Strongly associated with the lower planes of Hell, the Abyss, or Abaddon.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["hell_setting", "abyss_setting", "abaddon_setting"],
      compositeOfAnyTags: ["hell_setting", "abyss_setting", "abaddon_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("upper_plane_setting", {
    creature: {
      axis: "setting",
      family: "planar_setting",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Strongly associated with the upper planes of Heaven, Nirvana, or Elysium.",
      nativeOntologyPolicy: "aggregates_native_signals",
      adjacentTags: ["heaven_setting", "nirvana_setting", "elysium_setting"],
      compositeOfAnyTags: ["heaven_setting", "nirvana_setting", "elysium_setting"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];
