import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellInfluenceProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.INFLUENCE_INFLUENCE, {
    charm_influence: {
      description:
        "Wins cooperation through friendliness, fascination, admiration, or magically altered social regard.",
      appliesWhen: [
        "The spell's main value is improving a target's attitude, trust, or willingness to cooperate.",
        "The spell changes social reception more than it scripts exact behavior.",
      ],
      doesNotApplyWhen: [
        "The spell compels exact actions, overrides agency, or takes total control.",
        "The spell only manipulates mood without establishing a social bond or regard shift.",
      ],
      adjacentTags: ["emotion_control", "compulsion_control"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    compulsion_control: {
      description: "Forces scripted behavior, movement, or obedience against a target's normal will.",
      appliesWhen: [
        "The spell explicitly pressures the target into doing something, moving somewhere, or obeying a commanded pattern.",
        "Loss of agency is more important than affection, calm, or broad mood change.",
      ],
      doesNotApplyWhen: [
        "The spell merely charms or emotionally softens the target.",
        "The spell fully dominates the target over sustained actions rather than issuing narrower commands.",
      ],
      adjacentTags: ["charm_influence", "domination"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    domination: {
      description: "Seizes sustained control over a target's actions, body, or tactical decision-making.",
      appliesWhen: [
        "The spell grants ongoing, high-authority control over what the target does rather than just one compelled action.",
        "A user would retrieve it as a takeover spell, not merely a charm or suggestion spell.",
      ],
      doesNotApplyWhen: [
        "The spell only improves attitude, stirs emotion, or issues narrower one-off compulsions.",
        "The spell mainly suppresses actions without redirecting them under the caster's control.",
      ],
      adjacentTags: ["compulsion_control", "action_denial"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    emotion_control: {
      description: "Directly manipulates fear, calm, rage, love, despair, or other emotional states.",
      appliesWhen: [
        "The spell is naturally retrieved for changing a creature's feelings, morale, or emotional volatility.",
        "The emotional state change matters more than explicit obedience or truth extraction.",
      ],
      doesNotApplyWhen: [
        "The spell chiefly compels discrete actions or sustained domination.",
        "The spell only inflicts fear as damage pressure without broader emotional steering.",
      ],
      adjacentTags: ["fear_pressure", "charm_influence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    memory_manipulation: {
      description: "Edits, suppresses, restores, or rewrites memories, recollection, and remembered events.",
      appliesWhen: [
        "The spell is naturally retrieved for altering what a target remembers, forgets, or believes it experienced.",
        "Memory editing is more central than charm, emotion, or truth exposure.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals truth or emotions without changing stored recollection.",
        "The spell primarily imposes obedience or domination in the present moment.",
      ],
      adjacentTags: ["truth_reveal", "charm_influence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sleep_magic: {
      description: "Puts creatures to sleep, into magical slumber, or into a similarly enforced dormant state.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
