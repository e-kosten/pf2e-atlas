import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const SPELL_DERIVED_TAG_ONTOLOGY = {
  category: "spell",
  families: {
    infiltration: {
      description: "Appearance-changing and social-passing spells.",
      tags: [
        {
          tag: "disguise",
          description: "Helps alter appearance or impersonate another identity.",
          assignmentMode: "deterministic"
        },
        {
          tag: "social_infiltration",
          description: "Helps blend into a group or pass under social scrutiny.",
          assignmentMode: "deterministic"
        }
      ]
    },
    communication: {
      description: "Spells for signaling, telepathy, and message exchange.",
      tags: [
        {
          tag: "signaling",
          description: "Helps draw attention, mark a location, or coordinate allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "telepathic_link",
          description: "Creates direct mind-to-mind communication, silent tactical coordination, or psychic speech between creatures.",
          assignmentMode: "hybrid"
        },
        {
          tag: "message_delivery",
          description: "Sends, stores, or relays actual content across time or distance.",
          assignmentMode: "deterministic"
        }
      ]
    },
    reconnaissance: {
      description: "Remote-observation and scouting spells.",
      tags: [
        {
          tag: "scouting",
          description: "Helps observe at a distance, extend senses, or locate a target.",
          assignmentMode: "hybrid"
        }
      ]
    },
    revelation: {
      description: "Spells that reveal hidden magic, concealed creatures, curses, or otherwise suppressed truths.",
      tags: [
        {
          tag: "magic_detection",
          description: "Reveals magical auras, spell presence, active effects, or other supernatural signatures.",
          assignmentMode: "hybrid"
        },
        {
          tag: "invisibility_reveal",
          description: "Exposes invisible, hidden, concealed, or magically obscured creatures and objects.",
          assignmentMode: "hybrid"
        },
        {
          tag: "truth_reveal",
          description: "Forces honesty, exposes lies, or reveals disguised, false, or hidden truths.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell's retrieval value comes from exposing deception, forcing truthful answers, or stripping away false presentation.",
            "A user would plausibly look for it when they need an answer spell rather than a sensor spell."
          ],
          doesNotApplyWhen: [
            "The spell only detects magic, invisibility, or general auras without interrogating truth or deception.",
            "The spell mainly alters memory or emotions rather than revealing facts."
          ],
          adjacentTags: [
            "magic_detection",
            "memory_manipulation"
          ]
        },
        {
          tag: "curse_revelation",
          description: "Identifies curses, spiritual corruption, or other malign supernatural bindings on a target.",
          assignmentMode: "hybrid"
        }
      ]
    },
    wayfinding: {
      description: "Spells that guide direction, route-finding, or destination travel.",
      tags: [
        {
          tag: "navigation",
          description: "Helps orient, guide a route, or identify a destination's direction.",
          assignmentMode: "hybrid"
        }
      ]
    },
    traversal: {
      description: "Spells that improve movement modes, speed, or practical traversal.",
      tags: [
        {
          tag: "mobility",
          description: "Helps move faster, gain movement modes, or traverse terrain more effectively.",
          assignmentMode: "hybrid"
        },
        {
          tag: "flight",
          description: "Grants flying movement, sustained aerial travel, or practical airborne maneuvering.",
          assignmentMode: "hybrid"
        }
      ]
    },
    teleportation: {
      description: "Spells that blink, reposition, extract, or transport creatures across long distances or planes.",
      tags: [
        {
          tag: "short_range_teleport",
          description: "Teleports a creature across a short tactical distance, usually within the same scene or encounter area.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell repositions a creature within the current encounter or scene rather than serving as expedition travel.",
            "The tactical blink or reposition is itself a major reason to retrieve the spell."
          ],
          doesNotApplyWhen: [
            "The spell is mainly about escaping custody, extracting allies, or long-distance transport.",
            "The spell primarily crosses planes or major overland distances."
          ],
          adjacentTags: [
            "extraction_teleport",
            "long_range_teleport"
          ]
        },
        {
          tag: "extraction_teleport",
          description: "Teleports a creature out of danger, through restraints, or away from immediate threat pressure.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved as an escape, rescue, or anti-capture tool rather than only a movement spell.",
            "The reposition breaks danger, confinement, or immediate battlefield pressure."
          ],
          doesNotApplyWhen: [
            "The spell is mostly a neutral short-range blink or a long-distance travel effect.",
            "The spell primarily opens planar movement rather than emergency extraction."
          ],
          adjacentTags: [
            "short_range_teleport",
            "escape_support"
          ]
        },
        {
          tag: "long_range_teleport",
          description: "Teleports creatures across major overland distances, settlements, or remote destinations.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved for strategic travel, relocation, or bypassing long routes.",
            "The destination scale is substantially larger than one encounter map or immediate tactical space."
          ],
          doesNotApplyWhen: [
            "The spell is mainly a tactical blink, extraction, or planar crossing effect.",
            "The spell only repositions creatures within the same immediate scene."
          ],
          adjacentTags: [
            "short_range_teleport",
            "planar_travel"
          ]
        },
        {
          tag: "planar_travel",
          description: "Moves creatures between planes, through planar routes, or into extraplanar destinations.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Crossing into another plane, demiplane, or extraplanar route is central to the spell's retrieval value.",
            "The spell is naturally retrieved for cosmological travel rather than mundane relocation."
          ],
          doesNotApplyWhen: [
            "The spell only teleports within the same plane or functions as a normal long-distance travel tool.",
            "Extradimensional storage or shelter is present without real plane-crossing travel."
          ],
          adjacentTags: [
            "long_range_teleport",
            "field_shelter"
          ]
        }
      ]
    },
    transformation: {
      description: "Spells that alter a creature's body, form, or battle shape.",
      tags: [
        {
          tag: "transformation",
          description: "Spells that alter a creature's body, form, or battle shape.",
          assignmentMode: "composite",
          adjacentTags: [
            "battle_form",
            "animal_form",
            "elemental_form"
          ],
          compositeOfAnyTags: [
            "battle_form",
            "animal_form",
            "elemental_form"
          ]
        },
        {
          tag: "battle_form",
          description: "Transforms a creature into a combat-ready form with new statistics or battle-form language.",
          assignmentMode: "deterministic"
        },
        {
          tag: "animal_form",
          description: "Transforms a creature into an animal, beast, pest, or similar natural form.",
          assignmentMode: "deterministic"
        },
        {
          tag: "elemental_form",
          description: "Transforms a creature into an elemental form.",
          assignmentMode: "deterministic"
        }
      ]
    },
    control: {
      description: "Spells that pressure morale, obscure sight, or reshape the battlefield.",
      tags: [
        {
          tag: "fear_pressure",
          description: "Forces fear, panic, dread, or morale collapse onto a target.",
          assignmentMode: "deterministic"
        },
        {
          tag: "concealment",
          description: "Makes a creature hard to see, hidden, concealed, or undetected.",
          assignmentMode: "deterministic"
        },
        {
          tag: "line_of_sight_control",
          description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
          assignmentMode: "deterministic"
        },
        {
          tag: "battlefield_disruption",
          description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
          assignmentMode: "deterministic"
        },
        {
          tag: "barrier_creation",
          description: "Creates a wall, dome, cage, force barrier, or other discrete blocking structure that reshapes access lines.",
          assignmentMode: "hybrid"
        },
        {
          tag: "action_denial",
          description: "Denies actions through paralysis, stupefying shutdown, slowed tempo, or similarly severe turn disruption.",
          assignmentMode: "hybrid"
        }
      ]
    },
    influence: {
      description: "Spells that charm, compel, dominate, or emotionally steer a creature's behavior.",
      tags: [
        {
          tag: "charm_influence",
          description: "Wins cooperation through friendliness, fascination, admiration, or magically altered social regard.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell's main value is improving a target's attitude, trust, or willingness to cooperate.",
            "The spell changes social reception more than it scripts exact behavior."
          ],
          doesNotApplyWhen: [
            "The spell compels exact actions, overrides agency, or takes total control.",
            "The spell only manipulates mood without establishing a social bond or regard shift."
          ],
          adjacentTags: [
            "emotion_control",
            "compulsion_control"
          ]
        },
        {
          tag: "compulsion_control",
          description: "Forces scripted behavior, movement, or obedience against a target's normal will.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell explicitly pressures the target into doing something, moving somewhere, or obeying a commanded pattern.",
            "Loss of agency is more important than affection, calm, or broad mood change."
          ],
          doesNotApplyWhen: [
            "The spell merely charms or emotionally softens the target.",
            "The spell fully dominates the target over sustained actions rather than issuing narrower commands."
          ],
          adjacentTags: [
            "charm_influence",
            "domination"
          ]
        },
        {
          tag: "emotion_control",
          description: "Directly manipulates fear, calm, rage, love, despair, or other emotional states.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved for changing a creature's feelings, morale, or emotional volatility.",
            "The emotional state change matters more than explicit obedience or truth extraction."
          ],
          doesNotApplyWhen: [
            "The spell chiefly compels discrete actions or sustained domination.",
            "The spell only inflicts fear as damage pressure without broader emotional steering."
          ],
          adjacentTags: [
            "fear_pressure",
            "charm_influence"
          ]
        },
        {
          tag: "sleep_magic",
          description: "Puts creatures to sleep, into magical slumber, or into a similarly enforced dormant state.",
          assignmentMode: "deterministic"
        },
        {
          tag: "memory_manipulation",
          description: "Edits, suppresses, restores, or rewrites memories, recollection, and remembered events.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved for altering what a target remembers, forgets, or believes it experienced.",
            "Memory editing is more central than charm, emotion, or truth exposure."
          ],
          doesNotApplyWhen: [
            "The spell only reveals truth or emotions without changing stored recollection.",
            "The spell primarily imposes obedience or domination in the present moment."
          ],
          adjacentTags: [
            "truth_reveal",
            "charm_influence"
          ]
        },
        {
          tag: "domination",
          description: "Seizes sustained control over a target's actions, body, or tactical decision-making.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell grants ongoing, high-authority control over what the target does rather than just one compelled action.",
            "A user would retrieve it as a takeover spell, not merely a charm or suggestion spell."
          ],
          doesNotApplyWhen: [
            "The spell only improves attitude, stirs emotion, or issues narrower one-off compulsions.",
            "The spell mainly suppresses actions without redirecting them under the caster's control."
          ],
          adjacentTags: [
            "compulsion_control",
            "action_denial"
          ]
        }
      ]
    },
    support: {
      description: "Spells that restore, protect, ward, or reinforce allies and targets for combat recovery, scene resilience, and expedition safety.",
      tags: [
        {
          tag: "healing_support",
          description: "Directly restores hit points or accelerates recovery.",
          assignmentMode: "hybrid"
        },
        {
          tag: "condition_support",
          description: "Delays, suppresses, or removes afflictions and conditions.",
          assignmentMode: "hybrid"
        },
        {
          tag: "affliction_cleanup",
          description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
          assignmentMode: "hybrid"
        },
        {
          tag: "escape_support",
          description: "Helps a creature slip away, break free, flee, or evade pursuit.",
          assignmentMode: "hybrid"
        },
        {
          tag: "protective_ward",
          description: "Places a ward, sanctuary, shield, or protective boundary.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved as a defensive ward, sanctuary, or protective boundary rather than only a resistance buff.",
            "Its value comes from shielding a creature, object, or space against incoming harm or intrusion."
          ],
          doesNotApplyWhen: [
            "The spell only grants resistance, temporary Hit Points, or healing without a real warding or boundary element.",
            "The spell is mainly an alarm, anti-scrying, or mobility tool rather than direct protection."
          ],
          adjacentTags: [
            "alarm",
            "resistance_support"
          ]
        },
        {
          tag: "death_prevention",
          description: "Prevents death, stabilizes the dying, or brings a creature back from the brink.",
          assignmentMode: "hybrid"
        },
        {
          tag: "resistance_support",
          description: "Grants resistance or immunity against energy, damage, or hazards.",
          assignmentMode: "hybrid"
        },
        {
          tag: "temporary_hp_support",
          description: "Grants temporary Hit Points or similar buffer protection instead of restoring lost Hit Points.",
          assignmentMode: "hybrid"
        }
      ]
    },
    attrition: {
      description: "Spells that inflict ongoing harm through lingering persistent damage.",
      tags: [
        {
          tag: "persistent_damage",
          description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
          assignmentMode: "hybrid"
        }
      ]
    },
    summoning: {
      description: "Spells that call, create, or bind temporary creatures and servitors into the scene for combat, scouting, labor, or utility retrieval.",
      tags: [
        {
          tag: "creature_summoning",
          description: "Summons, conjures, or calls creatures to act as temporary allies, tools, or battlefield assets.",
          assignmentMode: "hybrid"
        },
        {
          tag: "undead_summoning",
          description: "Summons, calls, or manifests undead entities, spirits of the dead, or corpse-driven servitors.",
          assignmentMode: "hybrid"
        },
        {
          tag: "summoned_servitor",
          description: "Creates a helper, laborer, scout, mount, or similarly task-focused magical servitor rather than a pure combat summon.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved for utility help, labor, scouting, transport, or task performance rather than frontline combat stats.",
            "The conjured ally behaves more like a helper or specialist tool than a main battle summon."
          ],
          doesNotApplyWhen: [
            "The spell's main value is summoning a combat creature to attack, flank, or absorb hits.",
            "The spell only creates an object, barrier, or terrain effect without a real servant-like entity."
          ],
          adjacentTags: [
            "creature_summoning",
            "scouting"
          ]
        }
      ]
    },
    expedition: {
      description: "Spells that support travel, survival, and field logistics such as shelter, food, and aquatic operations.",
      tags: [
        {
          tag: "aquatic_support",
          description: "Helps with swimming, underwater breathing, water-surface travel, or other aquatic movement.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sustenance",
          description: "Provides food, water, rations, or practical nourishment for travel and survival.",
          assignmentMode: "deterministic"
        },
        {
          tag: "field_shelter",
          description: "Creates shelter, refuge, or a protected resting place in the field.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved to create a campsite refuge, safe resting place, or expedition shelter in hostile territory.",
            "Its value is prolonged field habitation or protected rest rather than momentary combat defense."
          ],
          doesNotApplyWhen: [
            "The spell only creates a brief combat ward, cover effect, or instant defensive barrier.",
            "The spell merely transports creatures away instead of establishing a place to rest."
          ],
          adjacentTags: [
            "protective_ward",
            "planar_travel"
          ]
        }
      ]
    },
    tempo: {
      description: "Spells that improve action economy or accelerate allies.",
      tags: [
        {
          tag: "quickened_support",
          description: "Grants extra actions, quickened condition benefits, or similar action-economy acceleration.",
          assignmentMode: "hybrid"
        },
        {
          tag: "initiative_support",
          description: "Improves initiative, pre-combat readiness, or the party's opening tempo before the first turn.",
          assignmentMode: "hybrid"
        }
      ]
    },
    summoner_support: {
      description: "Spells that specifically protect, enhance, or reposition an eidolon.",
      tags: [
        {
          tag: "eidolon_support",
          description: "Directly benefits an eidolon or the summoner-eidolon bond.",
          assignmentMode: "hybrid"
        }
      ]
    },
    magic_interference: {
      description: "Spells that disrupt, dispel, suppress, or meaningfully defend against hostile magic.",
      tags: [
        {
          tag: "countermagic",
          description: "Counteracts, dispels, suppresses, or shuts down magic.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved because stopping, unravelling, or suppressing existing magic is its main job.",
            "Anti-magic response matters more than simple protection, detection, or concealment."
          ],
          doesNotApplyWhen: [
            "The spell mainly protects targets from harm without actually disrupting hostile magic.",
            "The spell only reveals or warns about magic rather than counteracting it."
          ],
          adjacentTags: [
            "magic_detection",
            "protective_ward"
          ]
        }
      ]
    },
    security: {
      description: "Spells for sanctum security, intrusion warning, and anti-surveillance protection around people, objects, or spaces.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts you or others when a watched area, threshold, or ward is crossed.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved to warn about intrusion, threshold crossing, tampering, or unwanted entry.",
            "Detection and notice matter more than directly stopping the intruder."
          ],
          doesNotApplyWhen: [
            "The spell mainly protects, blocks, or hides the target without providing a warning function.",
            "The spell only reveals truth or magic generally rather than guarding a watched perimeter."
          ],
          adjacentTags: [
            "protective_ward",
            "scrying_protection"
          ]
        },
        {
          tag: "scrying_protection",
          description: "Blocks magical observation, remote viewing, divinatory surveillance, or other information leakage from a protected target or space.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The spell is naturally retrieved to keep plans, sanctums, identities, or conversations hidden from magical spying.",
            "Its core value is denying observation or divination rather than only raising an intrusion alarm."
          ],
          doesNotApplyWhen: [
            "The spell only improves mundane concealment or silence without real anti-divination protection.",
            "The spell counters magic broadly but is not specifically about surveillance or remote observation."
          ],
          adjacentTags: [
            "alarm",
            "countermagic"
          ]
        }
      ]
    },
    impact: {
      description: "Spells that impair minds or senses, forcibly reposition targets, or trap them in place.",
      tags: [
        {
          tag: "mental_impairment",
          description: "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "forced_movement",
          description: "Pushes, pulls, drags, or otherwise repositions a target against its will.",
          assignmentMode: "deterministic"
        },
        {
          tag: "restraint_capture",
          description: "Restrains, immobilizes, entangles, or traps a target in place.",
          assignmentMode: "deterministic"
        },
        {
          tag: "silencing",
          description: "Suppresses speech, sound production, verbal casting, or other voice-dependent action.",
          assignmentMode: "hybrid"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
