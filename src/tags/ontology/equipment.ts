import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const EQUIPMENT_DERIVED_TAG_ONTOLOGY = {
  category: "equipment",
  families: {
    function: {
      axis: "effect",
      subcategories: [
        "consumable"
      ],
      description: "Beneficial consumable outcome and recovery tags.",
      tags: [
        {
          tag: "beneficial",
          description: "Broad support-oriented consumable with non-hostile intent.",
          assignmentMode: "deterministic"
        },
        {
          tag: "healing_support",
          description: "Restores hit points or provides direct healing.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_poison",
          description: "Helps resist, prevent, or recover from poison.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_disease",
          description: "Helps resist, prevent, or recover from disease.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_fear",
          description: "Helps resist fear, recover from frightened effects, or steady courage.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_confusion",
          description: "Helps clear confusion, restore mental steadiness, or recover from disordered thinking.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_paralysis",
          description: "Helps break paralysis, restore movement, or free a creature from immobilizing body shutdown.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_petrification",
          description: "Helps prevent or reverse petrification and other stone-turning effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_bleed",
          description: "Helps staunch bleeding, end persistent bleed damage, or close ongoing wounds.",
          assignmentMode: "deterministic"
        },
        {
          tag: "curse_removal",
          description: "Helps remove, break, or counteract curses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "condition_support",
          description: "Helps clear or mitigate harmful conditions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mental_recovery",
          description: "Helps stabilize emotions or recover from mental conditions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "escape_support",
          description: "Helps flee, slip away, or break free.",
          assignmentMode: "deterministic"
        },
        {
          tag: "senses_support",
          description: "Improves vision or other senses.",
          assignmentMode: "deterministic"
        },
        {
          tag: "energy_resistance",
          description: "Grants resistance against one or more energy types.",
          assignmentMode: "deterministic"
        },
        {
          tag: "buff_support",
          description: "Provides a general beneficial enhancement or bonus.",
          assignmentMode: "deterministic"
        },
        {
          tag: "fortune_support",
          description: "Improves a creature's odds with rerolls, better-result effects, or failure rescue.",
          assignmentMode: "deterministic"
        }
      ]
    },
    consumable_role: {
      axis: "effect",
      subcategories: [
        "consumable"
      ],
      description: "Consumable role tags for whether an item is primarily hostile, self-directed, or meant to aid another creature.",
      tags: [
        {
          tag: "offensive",
          description: "Hostile consumable primarily meant to harm or debilitate a target.",
          assignmentMode: "deterministic"
        },
        {
          tag: "self_buff",
          description: "Support consumable primarily applied to the user.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ally_support",
          description: "Support consumable that can directly benefit another creature.",
          assignmentMode: "deterministic"
        }
      ]
    },
    delivery_profile: {
      axis: "effect",
      subcategories: [
        "consumable"
      ],
      description: "Consumable delivery tags for how a hostile item is applied, delivered, or introduced to the target.",
      tags: [
        {
          tag: "weapon_applied",
          description: "Offensive consumable applied to a weapon before use.",
          assignmentMode: "deterministic"
        },
        {
          tag: "thrown_offense",
          description: "Offensive consumable delivered by throwing it.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ingested_offense",
          description: "Offensive consumable delivered when swallowed or consumed.",
          assignmentMode: "deterministic"
        },
        {
          tag: "contact_offense",
          description: "Offensive consumable delivered through touch or skin contact.",
          assignmentMode: "deterministic"
        }
      ]
    },
    // TODO: Remove this legacy family after downstream loot-planning surfaces migrate to the narrower movement_traversal, reconnaissance, access_bypass, carry_logistics, and restraint families.
    purpose: {
      axis: "legacy",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Legacy family preserved for compatibility. Use the narrower retrieval families instead.",
      tags: []
    },
    movement_traversal: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Equipment that helps move, climb, navigate, or transport creatures and cargo from place to place.",
      tags: [
        {
          tag: "climbing",
          description: "Helps climb, rappel, or navigate vertical obstacles.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mobility",
          description: "Improves movement or traversal flexibility.",
          assignmentMode: "deterministic"
        },
        {
          tag: "navigation",
          description: "Helps track direction, route, or position.",
          assignmentMode: "deterministic"
        },
        {
          tag: "transport",
          description: "Helps move creatures or cargo from place to place.",
          assignmentMode: "deterministic"
        }
      ]
    },
    reconnaissance: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Equipment for recon, field observation, visibility support, evidence capture, and tracking or anti-tracking work.",
      tags: [
        {
          tag: "scouting",
          description: "Helps observe, survey, or reconnoiter an area.",
          assignmentMode: "deterministic"
        },
        {
          tag: "illumination",
          description: "Produces or improves light in dark environments.",
          assignmentMode: "deterministic"
        },
        {
          tag: "surveillance_recording",
          description: "Captures, stores, or replays images, sound, or other evidence for later review.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value comes from preserving sights, sounds, or observations for later replay, proof, or analysis.",
            "It is naturally sought as evidence capture, remote monitoring, or watch-post support rather than live conversation gear."
          ],
          doesNotApplyWhen: [
            "The item only sends a live message or helps coordinate allies without retaining evidence.",
            "The item protects against observation instead of performing it."
          ],
          adjacentTags: [
            "scouting",
            "tamper_evidence"
          ]
        },
        {
          tag: "tracking",
          description: "Helps follow trails, mark a target, or relocate something later.",
          assignmentMode: "deterministic"
        },
        {
          tag: "anti_tracking",
          description: "Helps hide your trail, mask scent, or make pursuit harder.",
          assignmentMode: "deterministic"
        }
      ]
    },
    access_bypass: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Equipment for technical or precision bypass of locks, traps, and secured access points without folding force-entry tools into the same bucket.",
      tags: [
        {
          tag: "lock_bypass",
          description: "Helps open locks or bypass secured entry points.",
          assignmentMode: "deterministic"
        },
        {
          tag: "trap_bypass",
          description: "Helps disarm, disable, or get past traps.",
          assignmentMode: "deterministic"
        }
      ]
    },
    carry_logistics: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Equipment for stowing, carrying, organizing, and transporting gear through play.",
      tags: [
        {
          tag: "carry_support",
          description: "Helps stow, carry, or organize equipment.",
          assignmentMode: "deterministic"
        }
      ]
    },
    restraint: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "weapon"
      ],
      description: "Equipment for capturing, binding, securing, or escaping restraints and immobilizing holds.",
      tags: [
        {
          tag: "restraint_escape",
          description: "Helps break free from grabs, restraints, or similar immobilizing holds.",
          assignmentMode: "deterministic"
        },
        {
          tag: "restraint_capture",
          description: "Helps capture, bind, or keep a target restrained.",
          assignmentMode: "deterministic"
        }
      ]
    },
    party_role: {
      axis: "party_role",
      description: "Party- and build-facing retrieval tags for loot that is valuable because of who benefits from it and how it changes play patterns.",
      tags: [
        {
          tag: "defender_support",
          description: "Especially valuable to shield users, bodyguards, line-holders, or other defender-style characters.",
          assignmentMode: "hybrid"
        },
        {
          tag: "skirmisher_support",
          description: "Especially valuable to mobile flankers, hit-and-run melee characters, or other skirmisher-style builds.",
          assignmentMode: "hybrid"
        },
        {
          tag: "caster_support",
          description: "Especially valuable to spellcasters for magical prep, casting reliability, spell defense, or spell-adjacent utility.",
          assignmentMode: "hybrid"
        },
        {
          tag: "action_economy_support",
          description: "Especially valuable because it compresses setup, speeds access, or meaningfully improves in-combat action efficiency.",
          assignmentMode: "hybrid"
        },
        {
          tag: "emergency_recovery",
          description: "Especially valuable as a panic-button item for sudden healing, escape, stabilization, or critical condition rescue.",
          assignmentMode: "hybrid"
        },
        {
          tag: "scouting_package",
          description: "Especially valuable to scouts, infiltrators, or advance-party play through quiet entry, recon, and information-gathering support.",
          assignmentMode: "hybrid"
        }
      ]
    },
    crafting_support: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "consumable"
      ],
      description: "Tools and supplies that enable treatment, repair, paperwork, crafting, or ritual preparation.",
      tags: [
        {
          tag: "medical_support",
          description: "Supports first aid, diagnosis, treatment, or ongoing medical care outside direct magical healing.",
          assignmentMode: "deterministic"
        },
        {
          tag: "repair_support",
          description: "Supports item repair, patchwork, upkeep, or restoring damaged gear and structures.",
          assignmentMode: "deterministic"
        },
        {
          tag: "alchemical_crafting",
          description: "Supports alchemical preparation, formula work, reagent handling, or crafting-related field setup.",
          assignmentMode: "deterministic"
        },
        {
          tag: "forgery_support",
          description: "Supports document falsification, seal imitation, signature copying, or bureaucratic deception.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value comes from imitating documents, seals, credentials, or official paperwork.",
            "It supports passing administrative scrutiny rather than just changing clothing or physical appearance."
          ],
          doesNotApplyWhen: [
            "The item only changes appearance or supports social disguise without document work.",
            "The item is a normal writing or archival tool with no deception-facing use."
          ],
          adjacentTags: [
            "disguise",
            "writing_recordkeeping"
          ]
        },
        {
          tag: "ritual_support",
          description: "Supports ritual casting, ceremonial setup, circles, offerings, or other extended magical preparation.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item meaningfully supports ceremonial, circle-based, offering-based, or extended-casting magic work.",
            "A user would retrieve it for magic preparation rather than ordinary crafting or adventuring gear."
          ],
          doesNotApplyWhen: [
            "The item is only generally magical without helping ritual process or setup.",
            "The item is mainly a focus of worship or symbolism rather than ritual procedure."
          ],
          adjacentTags: [
            "alchemical_crafting",
            "magic_protection"
          ]
        },
        {
          tag: "writing_recordkeeping",
          description: "Supports note-taking, mapmaking, copying text, archival work, or durable information storage.",
          assignmentMode: "deterministic"
        }
      ]
    },
    access_system: {
      axis: "item_mechanical",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "ammo",
        "armor",
        "weapon"
      ],
      description: "Storage, draw, and ammunition-handling equipment that changes how gear is accessed in play.",
      tags: [
        {
          tag: "extradimensional_storage",
          description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
          assignmentMode: "deterministic"
        },
        {
          tag: "weapon_staging",
          description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
          assignmentMode: "deterministic"
        },
        {
          tag: "ammo_management",
          description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
          assignmentMode: "deterministic"
        }
      ]
    },
    ammunition_payload: {
      axis: "item_mechanical",
      subcategories: [
        "ammo"
      ],
      description: "Ammunition payload tags for recurring on-hit effects and tailored projectile roles.",
      tags: [
        {
          tag: "creature_bane",
          description: "Tailored ammunition for a selected creature type or trait.",
          assignmentMode: "deterministic"
        },
        {
          tag: "elemental_payload",
          description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
          assignmentMode: "deterministic"
        },
        {
          tag: "explosive_payload",
          description: "Ammunition that detonates or scatters area damage on impact.",
          assignmentMode: "deterministic"
        },
        {
          tag: "spell_payload",
          description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
          assignmentMode: "deterministic"
        }
      ]
    },
    breaching: {
      axis: "utility",
      subcategories: [
        "gear",
        "kit",
        "consumable",
        "weapon"
      ],
      description: "Equipment built to force entry, open routes through barriers, or dismantle hardened environmental features.",
      tags: [
        {
          tag: "door_breaching",
          description: "Helps force doors, shutters, gates, or similar entry points open by strength, impact, or destructive entry.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value is getting through doors, shutters, gates, or secured entry points.",
            "It solves access by force rather than by keys, stealth, or lock tools."
          ],
          doesNotApplyWhen: [
            "The item is for larger demolition or siegework rather than point-of-entry breach.",
            "The item bypasses access quietly through locks or trickery instead of force."
          ],
          adjacentTags: [
            "lock_bypass",
            "barrier_breaking"
          ]
        },
        {
          tag: "barrier_breaking",
          description: "Designed to tear through walls, barricades, ice, webs, or other physical obstructions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "excavation",
          description: "Helps dig, cut through earth or stone, or otherwise open a route by excavation or practical earth-moving work.",
          assignmentMode: "deterministic"
        },
        {
          tag: "siege_support",
          description: "Supports attacking gates, fortifications, vehicles, or other larger hardened targets.",
          assignmentMode: "deterministic"
        },
        {
          tag: "demolition",
          description: "Designed for blasting, collapsing, or otherwise violently dismantling structures and obstacles.",
          assignmentMode: "deterministic"
        }
      ]
    },
    impact: {
      axis: "effect",
      subcategories: [
        "ammo",
        "consumable"
      ],
      description: "Hostile equipment tags for recurring target impairments and disabling outcomes.",
      tags: [
        {
          tag: "mobility_impairment",
          description: "Impairs movement through slowing, restraining, sticking, or immobilizing effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Impairs sight, hearing, or other senses through blinding, dazzling, or deafening effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mental_impairment",
          description: "Impairs thought, judgment, composure, or behavior through fear, confusion, or similar effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "physical_debilitation",
          description: "Weakens the body through drained vitality, sickness, fatigue, clumsiness, or similar bodily degradation.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sedation",
          description: "Induces sleep, lethargy, unconsciousness, or similar incapacitating drowsiness.",
          assignmentMode: "deterministic"
        },
        {
          tag: "silencing",
          description: "Suppresses speech, voice, or other sound-dependent action through gagging, muting, or numbing effects.",
          assignmentMode: "deterministic"
        }
      ]
    },
    expedition: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable",
        "armor",
        "shield",
        "weapon"
      ],
      description: "Field-sustainment equipment for expedition travel, provisioning, camp life, aquatic operations, and hostile-environment endurance.",
      tags: [
        {
          tag: "survival",
          description: "Supports wilderness travel, shelter, or long-term field use.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mounted_support",
          description: "Supports mounted combat, rider control, saddle use, or mount-specific loadouts.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sustenance",
          description: "Provides food, feed, water, or other practical nourishment for travel and survival.",
          assignmentMode: "deterministic"
        },
        {
          tag: "aquatic_support",
          description: "Helps with swimming, underwater breathing, flotation, water-surface travel, or watercraft use.",
          assignmentMode: "deterministic"
        },
        {
          tag: "environmental_adaptation",
          description: "Helps travelers endure extreme weather, thin air, smoke, pressure, or other dangerous environmental exposure.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value comes from surviving punishing climate, altitude, breathing hazards, immersion pressure, or similar expedition environments.",
            "It is naturally sought as environmental survival gear rather than a general defense item or campsite tool."
          ],
          doesNotApplyWhen: [
            "The item only protects against one incoming attack or hazard burst without broader travel-survival use.",
            "The item mainly creates camp infrastructure, carries provisions, or improves aquatic movement instead of adapting the user to the environment."
          ],
          adjacentTags: [
            "aquatic_support",
            "camp_setup",
            "hazard_shielding"
          ]
        },
        {
          tag: "camp_setup",
          description: "Supports campsite creation, resting infrastructure, shelter setup, or extended overland staging.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item is naturally retrieved for making camp, resting outdoors, or supporting expedition downtime.",
            "Its value is in field habitation rather than only carrying gear or feeding travelers."
          ],
          doesNotApplyWhen: [
            "The item only provides sustenance, mobility, or transport without campsite infrastructure.",
            "The item is merely general survival gear with no setup or shelter-facing role."
          ],
          adjacentTags: [
            "sustenance",
            "carry_support"
          ]
        }
      ]
    },
    communication: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Coordination, signaling, language-bridging, and message-relay equipment.",
      tags: [
        {
          tag: "signaling",
          description: "Helps draw attention, mark a location, or coordinate allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "telepathic_communication",
          description: "Enables silent mind-to-mind coordination, psychic speech, or communication that bypasses normal hearing.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value comes from silent psychic coordination, mind-to-mind speech, or communication that bypasses ordinary sound.",
            "It is naturally sought when stealth, silence, distance, or noise would make spoken coordination unreliable."
          ],
          doesNotApplyWhen: [
            "The item only boosts ordinary signaling, writing, or message relay without true mind-to-mind communication.",
            "The item is mainly about surveillance or recording rather than live coordination."
          ],
          adjacentTags: [
            "signaling",
            "message_delivery"
          ]
        },
        {
          tag: "message_delivery",
          description: "Sends, stores, or relays actual content across time or distance.",
          assignmentMode: "deterministic"
        },
        {
          tag: "translation_support",
          description: "Bridges language barriers through translation, deciphering, script interpretation, or speech-understanding aids.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's retrieval value comes from understanding foreign languages, translating speech, or decoding otherwise unreadable text or symbols.",
            "It is naturally sought when communication fails because of language barriers rather than distance or secrecy."
          ],
          doesNotApplyWhen: [
            "The item only stores, relays, or broadcasts messages without solving comprehension.",
            "The item only provides psychic communication between already understood participants."
          ],
          adjacentTags: [
            "telepathic_communication",
            "message_delivery"
          ]
        }
      ]
    },
    infiltration: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Appearance-changing, discreet-carry, and social-passing equipment across gear and consumables.",
      tags: [
        {
          tag: "concealable",
          description: "Easy to hide on the person or carry discreetly.",
          assignmentMode: "deterministic"
        },
        {
          tag: "disguise",
          description: "Helps alter appearance or impersonate another identity.",
          assignmentMode: "deterministic"
        },
        {
          tag: "social_infiltration",
          description: "Helps blend into a group or pass under social scrutiny.",
          assignmentMode: "deterministic"
        },
        {
          tag: "concealment",
          description: "Helps obscure a creature, item, or area from sight or make it harder to perceive.",
          assignmentMode: "deterministic"
        }
      ]
    },
    anti_magic: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Equipment for countering hostile magic or protecting against harmful magical effects.",
      tags: [
        {
          tag: "countermagic",
          description: "Counteracts, dispels, suppresses, or shuts down magic.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's main value is actively cancelling, suppressing, or interfering with hostile or ongoing magic.",
            "A user would retrieve it as an anti-magic tool rather than a general protective charm."
          ],
          doesNotApplyWhen: [
            "The item only protects the wearer from magical harm without disrupting the spell itself.",
            "The item focuses on blocking surveillance or hiding information rather than broader anti-magic interference."
          ],
          adjacentTags: [
            "magic_protection",
            "scrying_protection"
          ]
        },
        {
          tag: "magic_protection",
          description: "Protects the user or target against hostile magical effects.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's value comes from warding the bearer against curses, spells, hostile magical conditions, or magical damage.",
            "Protection matters more than actually counteracting or suppressing the incoming magic."
          ],
          doesNotApplyWhen: [
            "The item mainly shuts down active magic rather than defending a wearer or target.",
            "The stronger fit is scrying_protection because surveillance denial is the specific retrieval hook."
          ],
          adjacentTags: [
            "countermagic",
            "hazard_shielding"
          ]
        }
      ]
    },
    defense_profile: {
      axis: "item_mechanical",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "armor",
        "shield"
      ],
      description: "Shield and armor tags for interception, cover, and protection against ranged fire, hazardous scenes, or vertical danger.",
      tags: [
        {
          tag: "ally_cover",
          description: "Provides cover or upgraded cover to nearby allies.",
          assignmentMode: "deterministic"
        },
        {
          tag: "projectile_defense",
          description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
          assignmentMode: "deterministic"
        },
        {
          tag: "hazard_shielding",
          description: "Protects against environmental hazards, area effects, or other damaging exposures.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item is naturally retrieved for surviving traps, breath weapons, alchemical blasts, or other hazardous exposure.",
            "Protection against scenes and effects matters more than only deflecting direct weapon attacks."
          ],
          doesNotApplyWhen: [
            "The item's main value is cover against arrows or weapon strikes rather than wider hazard exposure.",
            "The item mainly protects against spells specifically, making magic_protection the stronger hook."
          ],
          adjacentTags: [
            "projectile_defense",
            "magic_protection"
          ]
        },
        {
          tag: "fall_protection",
          description: "Reduces falling harm, cushions impact, or protects against vertical movement accidents and collapse.",
          assignmentMode: "deterministic"
        }
      ]
    },
    security: {
      axis: "utility",
      subcategories: [
        "gear",
        "backpack",
        "kit",
        "vehicle",
        "consumable"
      ],
      description: "Intrusion-warning, anti-surveillance, seal-checking, and after-the-fact security gear for camps, vaults, cargo, and protected rooms.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts you or others when a watched area, threshold, or device is triggered.",
          assignmentMode: "deterministic"
        },
        {
          tag: "scrying_protection",
          description: "Blocks magical observation, remote viewing, or information leakage through divination-like effects.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item's main value is preventing magical spying, remote observation, or divination-led tracking.",
            "A user would retrieve it to keep plans, rooms, or identities hidden from magical surveillance."
          ],
          doesNotApplyWhen: [
            "The item only improves ordinary stealth or concealment without anti-divination protection.",
            "The item counters magic generally but is not particularly about observation or information leakage."
          ],
          adjacentTags: [
            "alarm",
            "concealment"
          ]
        },
        {
          tag: "tamper_evidence",
          description: "Makes intrusion, opening, theft, or interference easier to notice after the fact.",
          assignmentMode: "deterministic",
          appliesWhen: [
            "The item is naturally retrieved to reveal whether a lock, crate, seal, letter, cache, or room has been disturbed.",
            "Evidence of interference matters more than immediate warning, direct defense, or anti-scrying."
          ],
          doesNotApplyWhen: [
            "The item mainly alerts in real time when someone crosses a boundary.",
            "The item hides or protects a target without preserving signs of intrusion."
          ],
          adjacentTags: [
            "alarm",
            "surveillance_recording"
          ]
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology<"equipment">;
