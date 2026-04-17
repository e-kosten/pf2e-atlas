import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const HAZARD_DERIVED_TAG_ONTOLOGY = {
  category: "hazard",
  families: {
    mechanism: {
      description: "Hazards whose threat comes from trigger mechanisms, locking thresholds, control surfaces, or planar breaches.",
      tags: [
        {
          tag: "ward_trigger",
          description: "Hazard triggered by a rune, glyph, sigil, ward, or similar inscribed mechanism.",
          assignmentMode: "hybrid"
        },
        {
          tag: "pressure_trigger",
          description: "Hazard triggered by stepping on, weighing down, or depressing a pressure surface.",
          assignmentMode: "hybrid"
        },
        {
          tag: "tripwire_trigger",
          description: "Hazard triggered by tugging, crossing, or disturbing a tripwire.",
          assignmentMode: "hybrid"
        },
        {
          tag: "threshold_lockdown",
          description: "Hazard that seals, locks, or bars a threshold, doorway, or gate.",
          assignmentMode: "hybrid"
        },
        {
          tag: "control_interface",
          description: "Hazard operated through a button, lever, console, panel, switch, or similar control surface.",
          assignmentMode: "hybrid"
        },
        {
          tag: "planar_breach",
          description: "Hazard centered on a portal, rift, tear, breach, or other unstable opening in reality.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "An unstable portal, planar tear, extradimensional rupture, or reality breach is the central mechanism of the hazard.",
            "The hazard is naturally retrieved for cosmological leakage, portal instability, or something dangerous coming through a breach."
          ],
          doesNotApplyWhen: [
            "The hazard is merely magical or teleportive without a real opening in reality as the core hazard engine.",
            "The stronger fit is dispel_countermeasure or procedural_bypass because the planar flavor is incidental."
          ],
          adjacentTags: [
            "dispel_countermeasure",
            "procedural_bypass"
          ]
        }
      ]
    },
    function: {
      description: "Hazard practical-function tags for alerts, lockdowns, separation, and encounter pressure.",
      tags: [
        {
          tag: "alarm",
          description: "Alerts guardians, onlookers, or nearby creatures to an intrusion.",
          assignmentMode: "hybrid"
        },
        {
          tag: "restraint_capture",
          description: "Hazard that binds, restrains, or holds intruders in place.",
          assignmentMode: "hybrid"
        },
        {
          tag: "barrier_lockdown",
          description: "Hazard that seals, closes, or blocks passage to trap or delay intruders.",
          assignmentMode: "hybrid"
        },
        {
          tag: "spawned_attackers",
          description: "Hazard that summons, creates, or releases separate attackers into the scene.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved because it adds new hostile creatures, constructs, swarms, or manifestations to the scene.",
            "The extra attackers matter as separate encounter pressure rather than only as a damage effect."
          ],
          doesNotApplyWhen: [
            "The hazard only deals direct damage, restrains victims, or creates a temporary illusion without generating distinct assailants.",
            "The stronger fit is phantom_assailants when the threat is specifically a haunt manifestation rather than a broader hazard function."
          ],
          adjacentTags: [
            "alarm",
            "phantom_assailants"
          ]
        },
        {
          tag: "zone_denial",
          description: "Hazard that makes an area costly to enter, cross, or remain inside.",
          assignmentMode: "hybrid"
        },
        {
          tag: "resource_drain",
          description: "Hazard that taxes healing, spellcasting, equipment durability, or other party resources over time.",
          assignmentMode: "hybrid"
        },
        {
          tag: "forced_separation",
          description: "Hazard that splits allies apart through walls, drops, slides, teleports, or other positional disruption.",
          assignmentMode: "hybrid"
        }
      ]
    },
    setting: {
      description: "Hazard scene-placement tags for the kinds of sites and encounter spaces where the danger is most naturally retrieved.",
      tags: [
        {
          tag: "dungeon_hazard",
          description: "Hazard strongly associated with dungeon corridors, chambers, trapped passages, or underground complexes.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved as a classic corridor, chamber, or trap-complex defense in a dungeon environment.",
            "Its encounter identity is more about underground built-space adventuring than a narrower tomb or temple context."
          ],
          doesNotApplyWhen: [
            "The hazard is more specifically tomb-, temple-, bridge-, or aquatic-coded than generic dungeon-coded.",
            "The hazard merely happens to appear indoors once without dungeon-like scene identity."
          ],
          adjacentTags: [
            "tomb_hazard",
            "temple_hazard"
          ]
        },
        {
          tag: "tomb_hazard",
          description: "Hazard strongly associated with tombs, barrows, mausoleums, crypts, or burial defenses.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Burial sites, funerary wards, grave robbing countermeasures, or undead-rest protections are central to the hazard.",
            "A user would retrieve it for crypts, tombs, barrows, or similar funerary scenes."
          ],
          doesNotApplyWhen: [
            "The hazard is only underground or haunted without real funerary or burial-site identity.",
            "The stronger placement is temple_hazard or generic dungeon_hazard."
          ],
          adjacentTags: [
            "dungeon_hazard",
            "temple_hazard"
          ]
        },
        {
          tag: "temple_hazard",
          description: "Hazard strongly associated with shrines, sanctums, cathedrals, monasteries, or consecrated sites.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Sacred architecture, ritual sanctums, or faith-site defense is central to the hazard's scene identity.",
            "The hazard is naturally retrieved for shrine, sanctum, or temple protection rather than generic indoor defense."
          ],
          doesNotApplyWhen: [
            "The hazard is merely magical or haunted without specific sacred-site framing.",
            "The stronger setting is dungeon_hazard or urban_hazard rather than temple-focused."
          ],
          adjacentTags: [
            "dungeon_hazard",
            "appeasement_countermeasure"
          ]
        },
        {
          tag: "urban_hazard",
          description: "Hazard strongly associated with civic districts, alleys, rooftops, sewers, shops, or other built urban spaces.",
          assignmentMode: "hybrid"
        },
        {
          tag: "wilderness_hazard",
          description: "Hazard strongly associated with forests, trails, camps, ruins in the wild, or other outdoor expedition scenes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "bridge_passage_hazard",
          description: "Hazard strongly associated with bridges, chokepoints, gates, stairwells, or forced passage bottlenecks.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved for narrow crossings, gates, stairwells, or other spaces where intruders must pass through a constrained route.",
            "Forced-passage geometry matters more than a broader dungeon, urban, or wilderness setting identity."
          ],
          doesNotApplyWhen: [
            "The hazard only happens to sit near a doorway or bridge once without the chokepoint being central to its design.",
            "The stronger fit is threshold_lockdown or forced_movement because the route bottleneck is not the main retrieval hook."
          ],
          adjacentTags: [
            "threshold_lockdown",
            "forced_movement"
          ]
        },
        {
          tag: "aquatic_hazard",
          description: "Hazard strongly associated with flooded chambers, rivers, docks, ships, reefs, or underwater spaces.",
          assignmentMode: "hybrid"
        },
        {
          tag: "battlefield_hazard",
          description: "Hazard strongly associated with siegeworks, trenches, killing grounds, war engines, or other battlefield scenes.",
          assignmentMode: "hybrid"
        }
      ]
    },
    encounter_role: {
      description: "Hazard combat-role tags for encounter assembly and understanding how the hazard pressures a scene.",
      tags: [
        {
          tag: "ambush_burst",
          description: "Hazard designed to open with a sudden high-damage strike or surprise punish on first contact.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved for a sudden opener, trap-spring punish, or first-contact burst that catches intruders before a longer fight develops.",
            "The surprise spike matters more than sustained zone control or prolonged attrition."
          ],
          doesNotApplyWhen: [
            "The hazard mainly holds territory over time, guards a place persistently, or taxes resources across repeated rounds.",
            "The stronger fit is area_denial or attrition_pressure rather than a front-loaded strike."
          ],
          adjacentTags: [
            "area_denial",
            "attrition_pressure"
          ]
        },
        {
          tag: "area_denial",
          description: "Hazard whose primary role is to control valuable space rather than only harm a single victim.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is retrieved because it locks down territory, corridors, platforms, or objectives over time.",
            "Holding or excluding movement from space matters more than one immediate damage spike."
          ],
          doesNotApplyWhen: [
            "The hazard mainly punishes first contact, guards a threshold, or bursts once without sustained zone pressure.",
            "The stronger role is ambush_burst or sentinel_guardian."
          ],
          adjacentTags: [
            "zone_denial",
            "sentinel_guardian"
          ]
        },
        {
          tag: "attrition_pressure",
          description: "Hazard whose primary role is to wear the party down over time rather than deliver one decisive spike.",
          assignmentMode: "hybrid"
        },
        {
          tag: "pursuit_punisher",
          description: "Hazard that punishes retreat, pursuit, escape routes, or movement through chase-style spaces.",
          assignmentMode: "hybrid"
        },
        {
          tag: "sentinel_guardian",
          description: "Hazard whose role is to guard an area, treasure, threshold, or sanctum as a standing defense layer.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved as a guardian layer protecting a place, object, or route from intrusion.",
            "Its value is in persistent watchfulness or defensive coverage, not just burst damage."
          ],
          doesNotApplyWhen: [
            "The hazard mainly creates open-area denial with no strong guard-post identity.",
            "The hazard is mostly an ambush opener or chase-punishment device."
          ],
          adjacentTags: [
            "alarm",
            "area_denial"
          ]
        },
        {
          tag: "forced_separation_hazard",
          description: "Hazard whose encounter role is to isolate party members and break up normal formation or support lines.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Splitting allies apart is a central reason to retrieve the hazard, not merely a side effect of movement.",
            "The hazard is built to disrupt mutual support, lines of sight, or party clustering."
          ],
          doesNotApplyWhen: [
            "The hazard only repositions victims without meaningfully separating the group.",
            "The stronger role is area_denial or pursuit_punisher rather than party splitting."
          ],
          adjacentTags: [
            "forced_separation",
            "pursuit_punisher"
          ]
        }
      ]
    },
    haunt_manifestation: {
      description: "Haunt manifestations that materially change the encounter through attackers, lures, possession, replayed trauma, or judgment.",
      tags: [
        {
          tag: "life_drain_hazard",
          description: "Haunt that drains life force, vitality, or souls from victims.",
          assignmentMode: "hybrid"
        },
        {
          tag: "phantom_assailants",
          description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
          assignmentMode: "hybrid"
        },
        {
          tag: "lure_compulsion",
          description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
          assignmentMode: "hybrid"
        },
        {
          tag: "battlefield_disruption",
          description: "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
          assignmentMode: "hybrid"
        },
        {
          tag: "replayed_tragedy",
          description: "Haunt that re-enacts a murder, betrayal, execution, disaster, or other fixed traumatic event.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The haunt is naturally retrieved because it replays a specific past calamity, crime, execution, or emotional flashpoint as its core manifestation.",
            "The narrative repetition of an old event matters more than generic life drain, possession, or battlefield disruption."
          ],
          doesNotApplyWhen: [
            "The haunt is only sad, angry, or spiritually active without reenacting a fixed historical scene.",
            "The stronger fit is judgment_haunt or lure_compulsion because the recurring tragedy itself is not the central hook."
          ],
          adjacentTags: [
            "judgment_haunt",
            "appeasement_countermeasure"
          ]
        },
        {
          tag: "possession_haunt",
          description: "Haunt that enters, rides, or briefly controls a victim rather than only attacking them externally.",
          assignmentMode: "hybrid"
        },
        {
          tag: "judgment_haunt",
          description: "Haunt that condemns trespassers through accusation, punishment, curse-like verdicts, or moral reckoning.",
          assignmentMode: "hybrid"
        }
      ]
    },
    countermeasure_profile: {
      description: "Hazard tags for distinctive resolution patterns that matter for prep beyond a raw disable-skill list, especially when the real retrieval question is how the party solves or bypasses the hazard.",
      tags: [
        {
          tag: "appeasement_countermeasure",
          description: "Hazard best resolved through offerings, ritual respect, social appeasement, or meeting a spiritual demand.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard meaningfully invites negotiation-by-ritual, restitution, reverence, or satisfying an unmet dead or sacred demand.",
            "The nonviolent spiritual answer is more central than simply disabling mechanics or dispelling magic."
          ],
          doesNotApplyWhen: [
            "The hazard only needs a standard disable check, counteract, or forceful destruction.",
            "The hazard is haunted but has no appeasement-style resolution path."
          ],
          adjacentTags: [
            "exorcism_countermeasure",
            "judgment_haunt"
          ]
        },
        {
          tag: "exorcism_countermeasure",
          description: "Hazard best resolved through banishment, exorcism, consecration, or another spirit-cleansing answer.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved because cleansing, banishing, or sanctifying the hostile presence is a core answer path.",
            "A spiritual purge matters more than appeasement, anti-magic suppression, or physical mechanism work."
          ],
          doesNotApplyWhen: [
            "The hazard mainly wants offerings, restitution, or ritual respect rather than expulsion.",
            "The hazard is magical or mechanical but not really spirit-cleansed out of existence."
          ],
          adjacentTags: [
            "appeasement_countermeasure",
            "dispel_countermeasure"
          ]
        },
        {
          tag: "dispel_countermeasure",
          description: "Hazard meaningfully invites counteract, dispel, or magical suppression as a core answer path.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "A user would plausibly retrieve the hazard because anti-magic answers are central to resolving it.",
            "Magical suppression matters more than physical disarm or spiritual appeasement."
          ],
          doesNotApplyWhen: [
            "The hazard is magical but best solved through rituals, offerings, or physical tampering instead.",
            "Counteracting is only a minor optional answer path."
          ],
          adjacentTags: [
            "physical_disarm",
            "ward_trigger"
          ]
        },
        {
          tag: "procedural_bypass",
          description: "Hazard best bypassed through the right route, timing, command phrase, ritual sequence, or other correct procedure rather than direct disarm.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
            "A GM would plausibly retrieve the hazard for puzzle-like bypass, safe-route discovery, or passphrase-style navigation."
          ],
          doesNotApplyWhen: [
            "The main answer is simple mechanism tampering, counteracting magic, or appeasing a spirit.",
            "The hazard only has a minor caution or tactical workaround without a real procedural solution."
          ],
          adjacentTags: [
            "physical_disarm",
            "false_safe_route"
          ]
        },
        {
          tag: "physical_disarm",
          description: "Hazard meaningfully invites physical mechanism tampering, disassembly, or trigger-blocking as the core answer path.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard is naturally retrieved because tools, hands-on disable work, or mechanism access are central to solving it.",
            "Mechanical tampering matters more than safe sequencing, anti-magic, or spiritual negotiation."
          ],
          doesNotApplyWhen: [
            "The hazard is mainly solved by learning the right pattern, dispelling an effect, or meeting a spiritual demand.",
            "Physical interaction exists only as one optional fallback rather than the main resolution mode."
          ],
          adjacentTags: [
            "procedural_bypass",
            "dispel_countermeasure"
          ]
        }
      ]
    },
    impact: {
      description: "Hazard impact tags for mental destabilization and movement-limiting effects.",
      tags: [
        {
          tag: "mental_impairment",
          description: "Impairs judgment, emotions, or perception through fear, confusion, or similar effects.",
          assignmentMode: "deterministic"
        },
        {
          tag: "mobility_impairment",
          description: "Paralyzes, immobilizes, or otherwise heavily hampers movement.",
          assignmentMode: "deterministic"
        },
        {
          tag: "sensory_impairment",
          description: "Blinds, deafens, dazzles, or otherwise suppresses a victim's ability to perceive the environment clearly.",
          assignmentMode: "hybrid"
        }
      ]
    },
    environmental_danger: {
      description: "Hazards defined by recurring elemental or toxic environmental threats.",
      tags: [
        {
          tag: "acid_hazard",
          description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "cold_hazard",
          description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "fire_hazard",
          description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
          assignmentMode: "hybrid"
        },
        {
          tag: "electric_hazard",
          description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "poison_hazard",
          description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
          assignmentMode: "hybrid"
        },
        {
          tag: "respiratory_hazard",
          description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
          assignmentMode: "hybrid"
        },
        {
          tag: "sound_hazard",
          description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
          assignmentMode: "hybrid"
        },
        {
          tag: "water_hazard",
          description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
          assignmentMode: "hybrid"
        }
      ]
    },
    forced_position: {
      description: "Hazards that drop, collapse, or forcibly reposition creatures.",
      tags: [
        {
          tag: "pitfall",
          description: "Hazard built around a concealed pit, drop, or similar vertical fall trap.",
          assignmentMode: "deterministic"
        },
        {
          tag: "collapse_hazard",
          description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
          assignmentMode: "deterministic"
        },
        {
          tag: "forced_movement",
          description: "Hazard that pushes, pulls, drags, submerges, or otherwise forcibly repositions creatures.",
          assignmentMode: "deterministic"
        }
      ]
    },
    perception_control: {
      description: "Hazards that distort routes, orientation, or the perceived layout of a space.",
      tags: [
        {
          tag: "navigation_disruption",
          description: "Hazard that confounds routes, loops intruders, or scrambles navigation through distorted perception.",
          assignmentMode: "deterministic"
        },
        {
          tag: "illusion_assault",
          description: "Hazard that attacks through deceptive reflections, phantasms, or other hostile illusion-driven distortions.",
          assignmentMode: "deterministic"
        },
        {
          tag: "false_safe_route",
          description: "Hazard that tempts intruders toward a seemingly safer path or escape line that is itself the trap.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The hazard actively misdirects intruders toward a route that looks protective, faster, or safer but is actually the danger.",
            "The retrieval hook is deceptive path choice rather than only illusion damage or generic navigation confusion."
          ],
          doesNotApplyWhen: [
            "The hazard merely scrambles orientation without presenting a tempting fake safe path.",
            "The hazard is solved through procedure, but misleading route presentation is not central to how it works."
          ],
          adjacentTags: [
            "navigation_disruption",
            "procedural_bypass"
          ]
        }
      ]
    },
    attack_vector: {
      description: "Hazards categorized by how their attack is delivered into the scene.",
      tags: [
        {
          tag: "overhead_strike",
          description: "Hazard that drops, crashes, or attacks from the ceiling or another overhead position.",
          assignmentMode: "deterministic"
        },
        {
          tag: "projectile_emitter",
          description: "Hazard that fires bolts, beams, jets, sprays, or similar directed emissions from a fixed emitter.",
          assignmentMode: "deterministic"
        },
        {
          tag: "proximity_burst",
          description: "Hazard that erupts in an immediate burst, blast, or detonation when a victim comes near or crosses a point.",
          assignmentMode: "hybrid"
        },
        {
          tag: "floor_eruption",
          description: "Hazard that attacks upward from the ground, floor, or a concealed underfoot chamber.",
          assignmentMode: "hybrid"
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology;
