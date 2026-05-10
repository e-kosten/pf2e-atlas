import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellBattlefieldProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.BATTLEFIELD_CONTROL, {
    action_denial: {
      description:
        "Denies actions through paralysis, stupefying shutdown, slowed tempo, or similarly severe turn disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    anti_caster_disruption: {
      concept: "caster_disruption_remediation",
      description:
        "Disrupts casting, punishes spell use, or specifically suppresses hostile spellcasters in the moment.",
      adjacentTags: ["countermagic", "silencing"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    barrier_creation: {
      description:
        "Creates a wall, dome, cage, force barrier, or other discrete blocking structure that reshapes access lines.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    battlefield_disruption: {
      description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    concealment: {
      description: "Makes a creature hard to see, hidden, concealed, or undetected.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    countermagic: {
      concept: "active_magic_counteraction",
      label: "countermagic",
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      appliesWhen: [
        "The spell is naturally retrieved because stopping, unravelling, or suppressing existing magic is its main job.",
        "Anti-magic response matters more than simple protection, detection, or concealment.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects targets from harm without actually disrupting hostile magic.",
        "The spell only reveals or warns about magic rather than counteracting it.",
      ],
      adjacentTags: ["magic_detection", "protective_ward"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    fear_pressure: {
      description: "Forces fear, panic, dread, or morale collapse onto a target.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    forced_movement: {
      description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    line_of_sight_control: {
      description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mobility_denial: {
      description:
        "Pins, slows, grounds, or otherwise prevents normal repositioning without necessarily functioning as a full restraint effect.",
      adjacentTags: ["forced_movement", "restraint_capture"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    restraint_capture: {
      description: "Restrains, immobilizes, entangles, or traps a target in place.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    silencing: {
      description: "Suppresses speech, sound production, verbal casting, or other voice-dependent action.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];
