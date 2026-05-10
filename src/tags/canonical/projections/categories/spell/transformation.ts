import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellTransformationProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.TRANSFORMATION_TRANSFORMATION, {
    animal_form: {
      description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    battle_form: {
      description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    elemental_form: {
      description: "Transforms a creature into an elemental form.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    transformation: {
isComposite: true,
      description: "Spells that alter a creature's body, form, or battle shape.",
      adjacentTags: ["battle_form", "animal_form", "elemental_form"],
      compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
