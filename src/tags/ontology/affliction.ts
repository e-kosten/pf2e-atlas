import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const AFFLICTION_DERIVED_TAG_ONTOLOGY = {
  category: "affliction",
  families: {
    impact: {
      description: "Affliction impact tags for practical downstream consequences.",
      tags: [
        {
          tag: "mental_impairment",
          description: "Impairs judgment, emotions, or perception through confusion, fear, or delirium.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mobility_impairment",
          description: "Reduces speed, stiffens movement, or leaves the victim immobilized.",
          assignmentMode: "deterministic"
        },
        {
          tag: "physical_debilitation",
          description: "Weakens the body through exhaustion, sickness, drained vitality, blood loss, or similar bodily degradation.",
          assignmentMode: "deterministic"
        },
        {
          tag: "healing_suppression",
          description: "Prevents normal healing or sharply reduces healing received.",
          assignmentMode: "deterministic"
        },
        {
          tag: "cognitive_impairment",
          description: "Dulls thought, memory, decision-making, or mental clarity without being just fear or confusion.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Blinds, deafens, or otherwise suppresses perception and the senses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sedation",
          description: "Induces sleep, deep drowsiness, trance-like unconsciousness, or difficulty waking.",
          assignmentMode: "deterministic"
        },
        {
          tag: "action_denial",
          description: "Prevents normal action-taking through paralysis, stupefying shutdown, or similarly severe operational lockout.",
          assignmentMode: "hybrid"
        }
      ]
    },
    pathogenesis: {
      description: "Affliction tags for recurring infection, corruption, and body-changing disease patterns.",
      tags: [
        {
          tag: "rot_decay",
          description: "Defined by bodily rot, necrosis, blight, mummification, or similar physical decay.",
          assignmentMode: "deterministic"
        },
        {
          tag: "infestation_implant",
          description: "Defined by eggs, larvae, spores, parasites, or other host-colonizing implantation.",
          assignmentMode: "deterministic"
        },
        {
          tag: "blood_rot",
          description: "Defined by corrupted blood, blackened veins, hemorrhagic poisoning, or other blood-borne bodily collapse.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Corrupted blood, blackened veins, or bloodstream collapse is the core thematic identity of the affliction.",
            "A user would retrieve it for blood-plague, hemorrhagic corruption, or vein-darkening disease imagery."
          ],
          doesNotApplyWhen: [
            "The affliction only causes bleeding as one symptom without a real blood-corruption identity.",
            "The stronger fit is hemorrhagic_failure because the focus is immediate bleeding outcome rather than disease theme."
          ],
          adjacentTags: [
            "hemorrhagic_failure",
            "physical_debilitation"
          ]
        },
        {
          tag: "fungal_growth",
          description: "Defined by fungal bloom, spores, mycelial takeover, or mushroom-like growths spreading through the body.",
          assignmentMode: "hybrid"
        },
        {
          tag: "petrifying_corruption",
          description: "Defined by calcification, ossification, stone-turning, or gradual bodily hardening toward an inert form.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction is retrieved for gradual hardening, calcification, or stoneward corruption across stages.",
            "The process of becoming stone-like matters as much as the final condition."
          ],
          doesNotApplyWhen: [
            "The affliction chiefly imposes a final petrified state without broader progressive corruption framing.",
            "The stronger fit is transformative_corruption without specifically stoneward identity."
          ],
          adjacentTags: [
            "petrification",
            "cumulative_transformation"
          ]
        },
        {
          tag: "bestial_transformation",
          description: "Defined by animalistic mutation, feral reshaping, or a cursed slide into beastlike behavior and form.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction's retrieval value comes from becoming feral, animalistic, lycanthropic, or beast-shaped over time.",
            "Behavioral and bodily slide toward a beast identity are both salient."
          ],
          doesNotApplyWhen: [
            "The transformation is general corruption without a real animalistic or feral endpoint.",
            "The affliction only compels violent behavior without reshaping the victim's form or identity."
          ],
          adjacentTags: [
            "transformative_corruption",
            "violence_compulsion"
          ]
        }
      ]
    },
    epidemiological_profile: {
      description: "Affliction tags for outbreak scale, vector style, and contagion-facing retrieval such as quarantine, source tracing, and settlement risk.",
      tags: [
        {
          tag: "epidemic_pestilence",
          description: "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
          assignmentMode: "deterministic"
        },
        {
          tag: "community_outbreak",
          description: "Framed around camp-, village-, ship-, monastery-, or settlement-scale spread rather than one isolated victim.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction is naturally retrieved as a spreading local crisis affecting a community, camp, ship, or institution.",
            "Outbreak management, quarantine pressure, or multi-victim spread matters more than a single infected host."
          ],
          doesNotApplyWhen: [
            "The affliction remains a one-target curse or isolated infection without wider spread framing.",
            "The stronger fit is epidemic_pestilence only because of plague flavor, but community-scale spread is not actually present."
          ],
          adjacentTags: [
            "epidemic_pestilence",
            "carrier_vector"
          ]
        },
        {
          tag: "carrier_vector",
          description: "Spread through vermin, insects, bites, stings, or other living disease vectors.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction's spread is materially tied to rats, mosquitoes, parasites, infected animals, or other living carriers.",
            "Tracing or controlling the living vector is central to understanding the disease."
          ],
          doesNotApplyWhen: [
            "The disease spreads through air, water, dreams, or contaminated objects without a real living carrier.",
            "Injury transmission happens, but the record does not frame a recurring vector population or host species."
          ],
          adjacentTags: [
            "injury_exposure",
            "waterborne_exposure"
          ]
        }
      ]
    },
    behavioral_override: {
      description: "Affliction tags for forced behavior and explicit agency override.",
      tags: [
        {
          tag: "compulsion",
          description: "Overrides agency through commanded behavior, forced truth-telling, or similarly scripted actions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "violence_compulsion",
          description: "Forces hostile aggression, murderous rage, or other attack-driven loss of self-control.",
          assignmentMode: "hybrid"
        },
        {
          tag: "truth_compulsion",
          description: "Forces confession, honesty, or involuntary revelation against the victim's will.",
          assignmentMode: "hybrid"
        },
        {
          tag: "self_destructive_impulse",
          description: "Drives reckless self-harm, suicidal behavior, or dangerous compulsions against the victim's own interests.",
          assignmentMode: "hybrid"
        }
      ]
    },
    delivery_profile: {
      description: "Affliction tags for how exposure or transmission enters the body, breath, bloodstream, or dreaming mind.",
      tags: [
        {
          tag: "inhaled_exposure",
          description: "Spread through smoke, breath, spores, vapor, dust, or another inhaled medium.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ingested_exposure",
          description: "Spread by swallowing contaminated food, drink, medicine, or another consumed substance.",
          assignmentMode: "deterministic"
        },
        {
          tag: "contact_exposure",
          description: "Spread by touch, skin contact, slime, ooze, or another surface-level transfer mechanism.",
          assignmentMode: "deterministic"
        },
        {
          tag: "injury_exposure",
          description: "Spread through bites, stings, punctures, or other blood-entering injury vectors.",
          assignmentMode: "deterministic"
        },
        {
          tag: "dreamborne_exposure",
          description: "Spread through sleep, nightmares, dreaming contact, or other oneiric pathways.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction is contracted or transmitted through dreams, sleep, shared nightmares, or oneiric contact.",
            "Dream-state exposure is central to how the condition reaches the victim."
          ],
          doesNotApplyWhen: [
            "The affliction only causes nightmares after infection but is actually spread by wounds, air, or cursed objects.",
            "The stronger fit is nightmare_torment because the symptom profile matters more than the exposure path."
          ],
          adjacentTags: [
            "nightmare_torment",
            "inhaled_exposure"
          ]
        },
        {
          tag: "waterborne_exposure",
          description: "Spread through tainted water, drowning contact, flood exposure, or cursed immersion.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Contaminated water, immersion, flood contact, or cursed liquid exposure is central to how victims contract the affliction.",
            "A user would retrieve it for tainted wells, river contagion, drowning curses, or similar water-linked spread."
          ],
          doesNotApplyWhen: [
            "The affliction merely affects aquatic creatures without actually spreading through water contact.",
            "The stronger fit is inhaled_exposure, ingested_exposure, or carrier_vector instead of water-linked transmission."
          ],
          adjacentTags: [
            "ingested_exposure",
            "carrier_vector"
          ]
        }
      ]
    },
    progression_profile: {
      description: "Affliction tags for pacing, relapse patterns, and how the condition escalates toward its end state.",
      tags: [
        {
          tag: "delayed_onset",
          description: "Affliction whose symptoms, danger, or full transformation emerge after a notable delay rather than immediately.",
          assignmentMode: "hybrid"
        },
        {
          tag: "recurrent_flare",
          description: "Affliction that subsides and returns in episodes, repeating attacks, or cyclical symptom spikes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "cumulative_transformation",
          description: "Affliction whose stages progressively rewrite the victim into a visibly altered or corrupted form.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The stage-by-stage march toward a changed body is a major reason to retrieve the affliction.",
            "Transformation is progressive and cumulative rather than an instantaneous one-step effect."
          ],
          doesNotApplyWhen: [
            "The affliction only imposes one stable transformed state without escalation.",
            "The stronger fit is delayed_onset, terminal_collapse, or a more specific pathogenesis tag."
          ],
          adjacentTags: [
            "transformative_corruption",
            "petrifying_corruption"
          ]
        },
        {
          tag: "terminal_collapse",
          description: "Affliction explicitly framed around catastrophic bodily failure, death, or a final ruinous end state if unchecked.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction is naturally retrieved because the later stages end in death, total ruin, or another catastrophic final break point.",
            "The inevitability of a terminal end state matters more than day-to-day impairment or transformation imagery."
          ],
          doesNotApplyWhen: [
            "The affliction is dangerous but does not build toward a distinct catastrophic endpoint.",
            "The stronger fit is cumulative_transformation or physical_debilitation because ongoing deterioration matters more than terminal collapse."
          ],
          adjacentTags: [
            "cumulative_transformation",
            "physical_debilitation"
          ]
        }
      ]
    },
    physiology_override: {
      description: "Affliction tags for forced breathing failure, catastrophic body change, and organ-level disruption.",
      tags: [
        {
          tag: "respiratory_impairment",
          description: "Prevents normal breathing or fills the victim's lungs with water, fluid, or similar suffocating effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "transformative_corruption",
          description: "Progressively transforms the body into crystal, plant matter, fungus, or another corrupted form.",
          assignmentMode: "deterministic"
        },
        {
          tag: "hemorrhagic_failure",
          description: "Causes uncontrolled bleeding, blood loss, vessel rupture, or similar collapse of the body's circulatory integrity.",
          assignmentMode: "hybrid"
        },
        {
          tag: "petrification",
          description: "Turns flesh to stone or otherwise locks the body into a rigid mineralized state.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction is naturally retrieved because victims end up petrified, stone-locked, or mineralized.",
            "The end-state of stony immobilization matters more than the thematic cause."
          ],
          doesNotApplyWhen: [
            "The affliction only trends toward calcification without actually functioning as a petrifying condition.",
            "The stronger fit is petrifying_corruption because the gradual corruption process is the real hook."
          ],
          adjacentTags: [
            "petrifying_corruption",
            "mobility_impairment"
          ]
        },
        {
          tag: "wasting_hunger",
          description: "Imposes unnatural starvation, ravenous craving, or a consuming metabolic drive that degrades the body over time.",
          assignmentMode: "hybrid"
        }
      ]
    },
    metaphysical_profile: {
      description: "Affliction tags for soul-straining corruption, curse marks, and dream- or spirit-facing torment.",
      tags: [
        {
          tag: "void_soul_corruption",
          description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
          assignmentMode: "deterministic"
        },
        {
          tag: "nightmare_torment",
          description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
          assignmentMode: "deterministic"
        },
        {
          tag: "curse_marking",
          description: "Brands the victim with a curse mark, doom sign, inherited hex, or similarly explicit supernatural stigma.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction leaves an explicit mark, sign, sigil, stain, or named curse-brand that matters to its identity.",
            "A user would retrieve it for visible or narratively explicit cursed marking, not just soul damage."
          ],
          doesNotApplyWhen: [
            "The affliction is metaphysical but has no distinct branded or marked stigma.",
            "The stronger fit is void_soul_corruption or soul_binding without a visible curse sign."
          ],
          adjacentTags: [
            "void_soul_corruption",
            "soul_binding"
          ]
        },
        {
          tag: "soul_binding",
          description: "Pins, traps, anchors, or otherwise entangles the victim's soul with an object, oath, place, or hostile power.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction's metaphysical identity depends on the victim's soul being anchored, trapped, pledged, or externally entangled.",
            "A user would retrieve it for ghost anchors, cursed bindings, oath-linked doom, or similar soul-tether effects."
          ],
          doesNotApplyWhen: [
            "The affliction only marks, weakens, or spiritually corrupts the victim without actually binding the soul to something.",
            "The stronger fit is curse_marking or possession_seed rather than tethering or anchoring."
          ],
          adjacentTags: [
            "curse_marking",
            "possession_seed"
          ]
        },
        {
          tag: "possession_seed",
          description: "Plants an invading spirit, hostile presence, or takeover-ready metaphysical foothold inside the victim.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The affliction prepares a victim to be ridden, entered, replaced, or overtaken by another presence.",
            "A latent invading spirit or takeover-ready foothold is central to the condition's danger."
          ],
          doesNotApplyWhen: [
            "The affliction only compels behavior without an actual possessing entity or metaphysical foothold.",
            "The stronger fit is soul_binding or compulsion because takeover is not really part of the disease model."
          ],
          adjacentTags: [
            "compulsion",
            "soul_binding"
          ]
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
