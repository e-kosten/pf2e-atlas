import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellSupportProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.SUPPORT_SUPPORT, {
    affliction_cleanup: {
      description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_bleed: {
      concept: "bleed_remediation",
      description: "Staunches bleeding, ends persistent bleed damage, or closes wounds that keep draining a target.",
      adjacentTags: ["healing_support", "condition_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_confusion: {
      concept: "confusion_remediation",
      description:
        "Ends confusion, steadies a disordered mind, or protects a target from confusion-like mental unraveling.",
      adjacentTags: ["condition_support", "anti_fear"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_disease: {
      concept: "disease_remediation",
      description:
        "Cures disease, counteracts infections, or protects a target against plague, fever, and similar disease effects.",
      adjacentTags: ["affliction_cleanup", "anti_poison"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_fear: {
      concept: "fear_remediation",
      description: "Counters frightened or fear effects, bolsters courage, or protects a target against fear.",
      adjacentTags: ["condition_support", "anti_confusion"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_paralysis: {
      concept: "paralysis_remediation",
      description:
        "Ends paralysis, restores bodily function, or frees a target from magic or afflictions that leave it unable to move.",
      adjacentTags: ["condition_support", "escape_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_petrification: {
      concept: "petrification_remediation",
      description: "Prevents, reverses, or counteracts petrification and other turn-to-stone effects.",
      adjacentTags: ["affliction_cleanup", "condition_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_poison: {
      concept: "poison_remediation",
      description:
        "Cures poison, counters toxic afflictions, or protects a target against venom and similar poisoning effects.",
      adjacentTags: ["affliction_cleanup", "anti_disease"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    condition_support: {
      description: "Delays, suppresses, or removes afflictions and conditions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    death_prevention: {
      description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    eidolon_support: {
      description: "Directly benefits an eidolon or the summoner-eidolon bond.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    escape_support: {
      description: "Helps a creature slip away, break free, flee, or evade pursuit.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    healing_support: {
      description: "Directly restores hit points or accelerates recovery.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    initiative_support: {
      description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    protective_ward: {
      description: "Places a ward, sanctuary, shield, or protective boundary.",
      appliesWhen: [
        "The spell is naturally retrieved as a defensive ward, sanctuary, or protective boundary rather than only a resistance buff.",
        "Its value comes from shielding a creature, object, or space against incoming harm or intrusion.",
      ],
      doesNotApplyWhen: [
        "The spell only grants resistance, temporary Hit Points, or healing without a real warding or boundary element.",
        "The spell is mainly an alarm, anti-scrying, or mobility tool rather than direct protection.",
      ],
      adjacentTags: ["alarm", "resistance_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    quickened_support: {
      description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    resistance_support: {
      description: "Grants resistance or immunity against energy, damage, or hazards.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    temporary_hp_support: {
      description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
