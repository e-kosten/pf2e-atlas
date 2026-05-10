import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";

export const creatureSpecializationProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SPECIALIZATION_CASTING_PROFILE, {
    arcane_spellcaster: {
      description:
        "Creature whose spellcasting is substantially framed through arcane traditions, wizardry, runes, or similarly arcane technique.",
    },
    divine_spellcaster: {
      description:
        "Creature whose spellcasting is substantially framed through divine prayer, sacred miracles, or deity-facing magic.",
    },
    dragon_spellcaster: {
      description:
        "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
      appliesWhen: [
        "Use when a dragon or archdragon variant explicitly presents meaningful spellcasting as part of its encounter identity.",
        "The spellcasting matters for prep and counterplay beyond incidental magical flavor.",
      ],
      doesNotApplyWhen: [
        "The dragon only has innate magical flavor, one-off magical actions, or a few utility effects without real spellcaster framing.",
        "The stronger fit is only a tradition-specific spellcaster tag without a dragon-specific spellcaster presentation.",
      ],
      adjacentTags: ["arcane_spellcaster", "ritualist_creature"],
    },
    occult_spellcaster: {
      description:
        "Creature whose spellcasting is substantially framed through occult lore, spirits, emotion, dreams, or esoteric mental power.",
    },
    primal_spellcaster: {
      description:
        "Creature whose spellcasting is substantially framed through nature, elemental power, druidic force, or instinctive primal magic.",
    },
    ritualist_creature: {
      description:
        "Creature strongly associated with ritual casting, ceremonial magic, or extended occult or divine preparations.",
      appliesWhen: [
        "Use when ceremonial, circle-based, sacrificial, or extended-casting magic is a major reason to retrieve the creature.",
        "The creature is naturally used as a ritual leader, ritual threat, or ritual-supporting encounter element.",
      ],
      doesNotApplyWhen: [
        "The creature merely casts normal encounter spells without a meaningful ritual identity.",
        "The stronger fit is a tradition spellcaster tag and ritual work is only incidental flavor.",
      ],
      adjacentTags: ["divine_spellcaster", "occult_spellcaster"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SPECIALIZATION_CORRUPTION_PROFILE, {
    blight_tainted: {
      description:
        "Creature strongly defined by ecological blight, withering nature, corrupted groves, or land-sickened wilderness.",
      adjacentTags: ["wasteland_setting", "body_horror"],
    },
    cursewarped: {
      description:
        "Creature strongly defined by curse-driven distortion, doom-warping, or being transformed into its current state by a curse.",
      adjacentTags: ["curse_threat", "cursed_transformation"],
    },
    fungal_infested: {
      description:
        "Creature strongly defined by mycelium, spores, mushroom overgrowth, or fungus-driven bodily infestation.",
      adjacentTags: ["disease_vector", "body_horror"],
    },
    nightmare_tainted: {
      description:
        "Creature strongly defined by dream corruption, sleep-haunting influence, or oneiric pollution leaking into the waking world.",
      adjacentTags: ["dream_nightmare", "dreamlands_setting"],
    },
    parasite_ridden: {
      description:
        "Creature strongly defined by hosting burrowing larvae, parasitic colonies, implanted broods, or other invasive life within the body.",
      adjacentTags: ["spawn_creator", "plaguebearing"],
    },
    plaguebearing: {
      description:
        "Creature strongly defined by carrying, spreading, or embodying pestilence, fever, or outbreak-causing corruption.",
      adjacentTags: ["disease_vector", "parasite_ridden"],
    },
    void_tainted: {
      description:
        "Creature strongly defined by void corruption, cosmic hollowness, or metaphysical pollution that feels alien, cold, or reality-thinning.",
      adjacentTags: ["cosmic_dread", "forbidden_knowledge"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.CREATURE.SPECIALIZATION_ONTOLOGY_CLUSTER, {
    sinspawn_family: {
      description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
    },
    undead_adjacent: {
      concept: "undead_family",
      description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"creature">[];
