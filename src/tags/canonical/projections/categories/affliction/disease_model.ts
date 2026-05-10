import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const afflictionDiseaseModelProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.DISEASE_MODEL_DELIVERY_PROFILE, {
    contact_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by touch, skin contact, slime, ooze, or another surface-level transfer mechanism.",
    },
    dreamborne_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through sleep, nightmares, dreaming contact, or other oneiric pathways.",
      appliesWhen: [
        "The affliction is contracted or transmitted through dreams, sleep, shared nightmares, or oneiric contact.",
        "Dream-state exposure is central to how the condition reaches the victim.",
      ],
      doesNotApplyWhen: [
        "The affliction only causes nightmares after infection but is actually spread by wounds, air, or cursed objects.",
        "The stronger fit is nightmare_torment because the symptom profile matters more than the exposure path.",
      ],
      adjacentTags: ["nightmare_torment", "inhaled_exposure"],
    },
    ingested_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread by swallowing contaminated food, drink, medicine, or another consumed substance.",
    },
    inhaled_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through smoke, breath, spores, vapor, dust, or another inhaled medium.",
    },
    injury_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Spread through bites, stings, punctures, or other blood-entering injury vectors.",
    },
    waterborne_exposure: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through tainted water, drowning contact, flood exposure, or cursed immersion.",
      appliesWhen: [
        "Contaminated water, immersion, flood contact, or cursed liquid exposure is central to how victims contract the affliction.",
        "A user would retrieve it for tainted wells, river contagion, drowning curses, or similar water-linked spread.",
      ],
      doesNotApplyWhen: [
        "The affliction merely affects aquatic creatures without actually spreading through water contact.",
        "The stronger fit is inhaled_exposure, ingested_exposure, or carrier_vector instead of water-linked transmission.",
      ],
      adjacentTags: ["ingested_exposure", "carrier_vector"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.DISEASE_MODEL_EPIDEMIOLOGICAL_PROFILE, {
    carrier_vector: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Spread through vermin, insects, bites, stings, or other living disease vectors.",
      appliesWhen: [
        "The affliction's spread is materially tied to rats, mosquitoes, parasites, infected animals, or other living carriers.",
        "Tracing or controlling the living vector is central to understanding the disease.",
      ],
      doesNotApplyWhen: [
        "The disease spreads through air, water, dreams, or contaminated objects without a real living carrier.",
        "Injury transmission happens, but the record does not frame a recurring vector population or host species.",
      ],
      adjacentTags: ["injury_exposure", "waterborne_exposure"],
    },
    community_outbreak: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Framed around camp-, village-, ship-, monastery-, or settlement-scale spread rather than one isolated victim.",
      appliesWhen: [
        "The affliction is naturally retrieved as a spreading local crisis affecting a community, camp, ship, or institution.",
        "Outbreak management, quarantine pressure, or multi-victim spread matters more than a single infected host.",
      ],
      doesNotApplyWhen: [
        "The affliction remains a one-target curse or isolated infection without wider spread framing.",
        "The stronger fit is epidemic_pestilence only because of plague flavor, but community-scale spread is not actually present.",
      ],
      adjacentTags: ["epidemic_pestilence", "carrier_vector"],
    },
    epidemic_pestilence: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.DISEASE_MODEL_PATHOGENESIS, {
    bestial_transformation: {
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
    blood_rot: {
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
    fungal_growth: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Defined by fungal bloom, spores, mycelial takeover, or mushroom-like growths spreading through the body.",
    },
    infestation_implant: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
    },
    petrifying_corruption: {
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
    rot_decay: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.DISEASE_MODEL_PROGRESSION_PROFILE, {
    cumulative_transformation: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction whose stages progressively rewrite the victim into a visibly altered or corrupted form.",
      appliesWhen: [
        "The stage-by-stage march toward a changed body is a major reason to retrieve the affliction.",
        "Transformation is progressive and cumulative rather than an instantaneous one-step effect.",
      ],
      doesNotApplyWhen: [
        "The affliction only imposes one stable transformed state without escalation.",
        "The stronger fit is delayed_onset, terminal_collapse, or a more specific pathogenesis tag.",
      ],
      adjacentTags: ["transformative_corruption", "petrifying_corruption"],
    },
    delayed_onset: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Affliction whose symptoms, danger, or full transformation emerge after a notable delay rather than immediately.",
      appliesWhen: [
        "The affliction is naturally retrieved because the real danger appears after an incubation period, hidden delay, or deceptively quiet initial stage.",
        "The timing gap between exposure and serious consequence matters to prep, diagnosis, or quarantine decisions.",
      ],
      doesNotApplyWhen: [
        "The affliction starts harming victims right away even if it worsens later.",
        "The stronger fit is recurrent_flare or cumulative_transformation because the main hook is cycling or progressive change rather than delayed emergence.",
      ],
      adjacentTags: ["recurrent_flare", "cumulative_transformation"],
    },
    recurrent_flare: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Affliction that subsides and returns in episodes, repeating attacks, or cyclical symptom spikes.",
      appliesWhen: [
        "The affliction is naturally retrieved because symptoms repeatedly subside and then surge back in cycles, attacks, or flare-ups.",
        "Its pacing matters as a recurring problem rather than a single steady decline.",
      ],
      doesNotApplyWhen: [
        "The affliction mainly incubates once and then worsens in a straight line.",
        "The stronger fit is delayed_onset or terminal_collapse because recurrence is not the main progression hook.",
      ],
      adjacentTags: ["delayed_onset", "terminal_collapse"],
    },
    terminal_collapse: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Affliction explicitly framed around catastrophic bodily failure, death, or a final ruinous end state if unchecked.",
      appliesWhen: [
        "The affliction is naturally retrieved because the later stages end in death, total ruin, or another catastrophic final break point.",
        "The inevitability of a terminal end state matters more than day-to-day impairment or transformation imagery.",
      ],
      doesNotApplyWhen: [
        "The affliction is dangerous but does not build toward a distinct catastrophic endpoint.",
        "The stronger fit is cumulative_transformation or physical_debilitation because ongoing deterioration matters more than terminal collapse.",
      ],
      adjacentTags: ["cumulative_transformation", "physical_debilitation"],
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"affliction">[];
