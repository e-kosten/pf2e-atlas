import type { DerivedTagAuthoredCategoryOntology } from "../../types.js";

export const CREATURE_DERIVED_TAG_ONTOLOGY = {
  category: "creature",
  families: {
    // TODO: Remove this legacy family after downstream creature planning surfaces migrate to habitat_setting, site_setting, regional_setting, and planar_setting.
    setting: {
      axis: "legacy",
      description: "Legacy umbrella family preserved for compatibility. Use the narrower creature setting families instead.",
      variantInheritance: true,
      tags: []
    },
    habitat_setting: {
      axis: "setting",
      description: "Creature habitat tags for terrain, climate, and ecological placement such as aquatic, desert, sky, or underground retrieval.",
      variantInheritance: true,
      tags: [
        {
          tag: "aquatic_setting",
          description: "Strongly associated with open water, underwater spaces, or aquatic environments.",
          assignmentMode: "hybrid"
        },
        {
          tag: "freshwater_setting",
          description: "Strongly associated with rivers, lakes, ponds, streams, springs, or other inland waters.",
          assignmentMode: "hybrid"
        },
        {
          tag: "coastal_setting",
          description: "Strongly associated with coasts, shores, reefs, or littoral edges.",
          assignmentMode: "hybrid"
        },
        {
          tag: "island_setting",
          description: "Strongly associated with islands, archipelagos, or isolated isles.",
          assignmentMode: "hybrid"
        },
        {
          tag: "jungle_setting",
          description: "Strongly associated with jungles, rainforests, or dense tropical canopies.",
          assignmentMode: "hybrid"
        },
        {
          tag: "forest_setting",
          description: "Strongly associated with forests, jungles, groves, or briar-choked wilds.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plains_setting",
          description: "Strongly associated with open plains, grasslands, prairies, or savannas.",
          assignmentMode: "hybrid"
        },
        {
          tag: "canyon_setting",
          description: "Strongly associated with canyons, gorges, mesas, or badlands.",
          assignmentMode: "hybrid"
        },
        {
          tag: "swamp_setting",
          description: "Strongly associated with bogs, marshes, fens, or mires.",
          assignmentMode: "hybrid"
        },
        {
          tag: "underground_setting",
          description: "Strongly associated with caves, tunnels, crypts, or subterranean spaces.",
          assignmentMode: "hybrid"
        },
        {
          tag: "arctic_setting",
          description: "Strongly associated with snow, ice, tundra, or frozen reaches.",
          assignmentMode: "hybrid"
        },
        {
          tag: "desert_setting",
          description: "Strongly associated with dunes, sand, or arid wastes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "wasteland_setting",
          description: "Strongly associated with barren wastes, blasted wastelands, or desolate badlands.",
          assignmentMode: "hybrid"
        },
        {
          tag: "mountain_setting",
          description: "Strongly associated with cliffs, peaks, passes, or rocky heights.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Mountain terrain is a recurring habitat or encounter frame.",
            "The creature is tied to peaks, passes, cliffs, or alpine strongholds."
          ],
          doesNotApplyWhen: [
            "Rocky terrain is incidental rather than defining.",
            "The creature is better described as underground_setting or sky_setting."
          ],
          adjacentTags: [
            "underground_setting",
            "sky_setting"
          ]
        },
        {
          tag: "sky_setting",
          description: "Strongly associated with open skies, storm clouds, or high-altitude aerial habitats.",
          assignmentMode: "hybrid"
        },
        {
          tag: "volcanic_setting",
          description: "Strongly associated with volcanoes, calderas, lava, or magma.",
          assignmentMode: "hybrid"
        }
      ]
    },
    site_setting: {
      axis: "setting",
      description: "Creature site and scene-placement tags for ships, settlements, ruins, temples, battlefields, graveyards, and other encounter locations.",
      variantInheritance: true,
      tags: [
        {
          tag: "nautical_setting",
          description: "Strongly associated with ships, sailors, wrecks, or harbors.",
          assignmentMode: "hybrid"
        },
        {
          tag: "urban_setting",
          description: "Strongly associated with urban encounter scenes such as cities, streets, alleys, dense buildings, markets, or sewers.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is primarily framed as belonging in city or sewer encounter spaces.",
            "Urban placement is a recurring part of its identity, role, or habitat."
          ],
          doesNotApplyWhen: [
            "The record merely mentions a city once.",
            "The creature can appear in towns but is not specifically urban-coded.",
            "The creature is better modeled by fortress_setting or small_settlement_setting."
          ],
          positiveSignals: [
            "city guard roles",
            "sewer habitat",
            "street or alley patrol scenes",
            "market or district anchoring"
          ],
          negativeSignals: [
            "generic settlement mention",
            "single adventure location",
            "fortress-only residence"
          ],
          adjacentTags: [
            "small_settlement_setting",
            "fortress_setting"
          ]
        },
        {
          tag: "battlefield_setting",
          description: "Strongly associated with battlefields, war zones, organized military deployments, or mass-combat scenes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "small_settlement_setting",
          description: "Strongly associated with villages, hamlets, small towns, or other low-density community settlements.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is repeatedly framed around villages, hamlets, or small town life.",
            "Its encounter identity is tied to low-density community spaces rather than major urban districts."
          ],
          doesNotApplyWhen: [
            "The record only says the creature can appear near people.",
            "The creature is more strongly urban, rural, or fortress-coded than village-coded."
          ],
          adjacentTags: [
            "urban_setting",
            "rural_setting"
          ]
        },
        {
          tag: "graveyard_setting",
          description: "Strongly associated with cemeteries, tombs, barrows, or burial grounds.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ruins_setting",
          description: "Strongly associated with ancient ruins or derelict structures.",
          assignmentMode: "hybrid"
        },
        {
          tag: "temple_setting",
          description: "Strongly associated with temples, shrines, monasteries, or other sacred encounter sites.",
          assignmentMode: "hybrid"
        },
        {
          tag: "fortress_setting",
          description: "Strongly associated with castles, fortresses, citadels, watchtowers, or other fortified encounter sites.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Fortified structures are part of the creature's recurring encounter identity.",
            "The creature is framed around castles, citadels, watchtowers, keeps, or defensive strongholds."
          ],
          doesNotApplyWhen: [
            "The record only uses a fortress as a one-off location.",
            "The broader identity is urban_setting or temple_setting rather than fortified-site specific."
          ],
          adjacentTags: [
            "urban_setting",
            "temple_setting"
          ]
        },
        {
          tag: "rural_setting",
          description: "Strongly associated with farms, pastures, croplands, countryside routes, mills, or other agricultural rural encounter scenes.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is repeatedly framed around farms, fields, mills, roadsides, or countryside scenes.",
            "Agricultural or open-country placement is a recurring part of its encounter identity."
          ],
          doesNotApplyWhen: [
            "The record only implies generic overland travel.",
            "The creature is better modeled by small_settlement_setting or plains_setting."
          ],
          adjacentTags: [
            "small_settlement_setting",
            "plains_setting"
          ]
        }
      ]
    },
    regional_setting: {
      axis: "setting",
      description: "Creature setting tags for broad Golarion regions, nations, cultural spheres, and other macro-scale canonical geographies that materially affect retrieval.",
      variantInheritance: true,
      tags: [
        {
          tag: "geb_setting",
          description: "Strongly associated with Geb, its necromantic society, or iconic locales such as Graydirge.",
          assignmentMode: "hybrid"
        },
        {
          tag: "gravelands_setting",
          description: "Strongly associated with the Gravelands, Lastwall's shattered front, or iconic sites such as Fort Ozem and Gallowspire.",
          assignmentMode: "hybrid"
        },
        {
          tag: "mwangi_setting",
          description: "Strongly associated with the Mwangi Expanse, its jungle polities, or similarly Mwangi-rooted regional framing.",
          assignmentMode: "hybrid"
        },
        {
          tag: "darklands_setting",
          description: "Strongly associated with the Darklands as a civilization-bearing underworld region rather than generic underground terrain.",
          assignmentMode: "hybrid"
        },
        {
          tag: "tian_xia_setting",
          description: "Strongly associated with Tian Xia regions such as Minata, Bonmu, or Tian cultural subregions like Tian-Shu, Tian-Hwan, Tian-Sing, and Tian-La.",
          assignmentMode: "hybrid"
        }
      ]
    },
    named_locale_setting: {
      axis: "setting",
      description: "Creature setting tags for specific named Pathfinder cities, fortresses, ruins, landmarks, and other marquee canonical locales that are narrower than a whole region.",
      variantInheritance: true,
      tags: [
        {
          tag: "absalom_setting",
          description: "Strongly associated with Absalom, its districts, surrounding Isle of Kortos culture, or similarly Absalom-specific civic scenes.",
          assignmentMode: "hybrid"
        }
      ]
    },
    planar_setting: {
      axis: "setting",
      description: "Creature setting tags for planes, cosmological realms, and extraplanar retrieval patterns.",
      variantInheritance: true,
      tags: [
        {
          tag: "astral_setting",
          description: "Strongly associated with Astral Plane scenes, silver-void travel, or stable portal routes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ethereal_setting",
          description: "Strongly associated with the Ethereal Plane, its native wildlife, or recurring hunting and travel routes through it.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_fire_setting",
          description: "Strongly associated with the Plane of Fire, its native denizens, or its infernal-bright cityscapes and battlefields.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_air_setting",
          description: "Strongly associated with the Plane of Air, its native denizens, or its endless winds and cloud realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_water_setting",
          description: "Strongly associated with the Plane of Water, its native denizens, or its endless seas and oceanic realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "plane_of_earth_setting",
          description: "Strongly associated with the Plane of Earth, its native denizens, or its crystal caverns and stonebound realms.",
          assignmentMode: "hybrid"
        },
        {
          tag: "elemental_plane_setting",
          description: "Strongly associated with one of the elemental planes of Fire, Air, Water, or Earth.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "plane_of_fire_setting",
            "plane_of_air_setting",
            "plane_of_water_setting",
            "plane_of_earth_setting"
          ],
          compositeOfAnyTags: [
            "plane_of_fire_setting",
            "plane_of_air_setting",
            "plane_of_water_setting",
            "plane_of_earth_setting"
          ]
        },
        {
          tag: "first_world_setting",
          description: "Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.",
          assignmentMode: "hybrid"
        },
        {
          tag: "dreamlands_setting",
          description: "Strongly associated with the Dreamlands, Leng-linked dream roads, or iconic denizens that dwell there.",
          assignmentMode: "hybrid"
        },
        {
          tag: "boneyard_setting",
          description: "Strongly associated with the Boneyard, psychopomp duties, or soul-processing afterlife scenes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "heaven_setting",
          description: "Strongly associated with Heaven, archon hosts, or ordered celestial service.",
          assignmentMode: "hybrid"
        },
        {
          tag: "nirvana_setting",
          description: "Strongly associated with Nirvana, enlightened celestials, or benevolent contemplative service.",
          assignmentMode: "hybrid"
        },
        {
          tag: "elysium_setting",
          description: "Strongly associated with Elysium, azatas, or freedom-driven celestial heroism.",
          assignmentMode: "hybrid"
        },
        {
          tag: "upper_plane_setting",
          description: "Strongly associated with the upper planes of Heaven, Nirvana, or Elysium.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "heaven_setting",
            "nirvana_setting",
            "elysium_setting"
          ],
          compositeOfAnyTags: [
            "heaven_setting",
            "nirvana_setting",
            "elysium_setting"
          ]
        },
        {
          tag: "hell_setting",
          description: "Strongly associated with Hell, devil hosts, or infernal hierarchy.",
          assignmentMode: "hybrid"
        },
        {
          tag: "abyss_setting",
          description: "Strongly associated with the Abyss, demon realms, or qlippoth-infested outer rifts.",
          assignmentMode: "hybrid"
        },
        {
          tag: "abaddon_setting",
          description: "Strongly associated with Abaddon, daemons, or soul-devouring lower-planar wastelands.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals"
        },
        {
          tag: "lower_plane_setting",
          description: "Strongly associated with the lower planes of Hell, the Abyss, or Abaddon.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "hell_setting",
            "abyss_setting",
            "abaddon_setting"
          ],
          compositeOfAnyTags: [
            "hell_setting",
            "abyss_setting",
            "abaddon_setting"
          ]
        },
        {
          tag: "axis_setting",
          description: "Strongly associated with Axis, the Eternal City, or its lawful planar order.",
          assignmentMode: "hybrid"
        },
        {
          tag: "shadow_plane_setting",
          description: "Strongly associated with the Shadow Plane or the Plane of Shadow.",
          assignmentMode: "hybrid"
        },
        {
          tag: "maelstrom_setting",
          description: "Strongly associated with the Maelstrom, proteans, or its chaotic planar fringes.",
          assignmentMode: "hybrid"
        },
        {
          tag: "cosmic_framework_setting",
          description: "Strongly associated with the cosmic framework planes of Axis, the Boneyard, and the Maelstrom, which govern order, judgment, and transformative change.",
          assignmentMode: "composite",
          nativeOntologyPolicy: "aggregates_native_signals",
          adjacentTags: [
            "axis_setting",
            "boneyard_setting",
            "maelstrom_setting"
          ],
          compositeOfAnyTags: [
            "axis_setting",
            "boneyard_setting",
            "maelstrom_setting"
          ]
        }
      ]
    },
    combat_role: {
      axis: "encounter",
      description: "Creature combat-role tags for encounter assembly, tactical reading, and balancing mixed enemy groups.",
      tags: [
        {
          tag: "brute_combatant",
          description: "Built to pressure the front line through durability, direct damage, and straightforward melee threat.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved as a heavy hitter that advances, endures punishment, and threatens through direct force.",
            "Its tactical identity is more about sturdy pressure than mobility, command, or precision control."
          ],
          doesNotApplyWhen: [
            "The creature mainly operates from range, through battlefield control, or through ally support.",
            "High damage is present but the stronger combat identity is ambush, artillery, or commander play."
          ],
          adjacentTags: [
            "defender_combatant",
            "artillery_combatant"
          ]
        },
        {
          tag: "skirmisher_combatant",
          description: "Built around mobility, repositioning, hit-and-run pressure, or opportunistic strikes.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature's retrieval value comes from darting movement, flanking, repositioning, or repeated opportunistic attacks.",
            "Mobility and pressure cycling matter more than armor, command, or raw ranged bombardment."
          ],
          doesNotApplyWhen: [
            "The creature mainly opens from stealth once and then behaves like another clearer combat role.",
            "The stronger identity is brute, ambusher, or artillery rather than mobile harassment."
          ],
          adjacentTags: [
            "ambusher_combatant",
            "controller_combatant"
          ]
        },
        {
          tag: "harrier_combatant",
          description: "Built to chip away from safety through repeated ranged harassment, flyby pressure, or evasive standoff attacks that force pursuit.",
          assignmentMode: "hybrid",
          adjacentTags: [
            "skirmisher_combatant",
            "artillery_combatant"
          ]
        },
        {
          tag: "controller_combatant",
          description: "Built to reshape the battlefield through debuffs, forced movement, terrain control, or other tactical denial.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved for immobilizing, repositioning, walling off, slowing, or otherwise dictating battlefield shape.",
            "Tactical denial matters more than direct damage output."
          ],
          doesNotApplyWhen: [
            "The creature only has one incidental debuff while otherwise fighting as a brute, defender, or artillery piece.",
            "The stronger identity is support or commander rather than enemy-space control."
          ],
          adjacentTags: [
            "support_combatant",
            "artillery_combatant"
          ]
        },
        {
          tag: "artillery_combatant",
          description: "Built to pressure targets from range through volleys, spell barrages, breath attacks, or other standoff offense.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved as a ranged damage or spell-barrage threat that prefers distance.",
            "Its tactical identity is standoff pressure more than command, mobility, or pure control."
          ],
          doesNotApplyWhen: [
            "The creature happens to have one ranged option but still fundamentally plays as a brute, controller, or support piece.",
            "Its main retrieval hook is casting support or battlefield control rather than ranged offense."
          ],
          adjacentTags: [
            "harrier_combatant",
            "controller_combatant",
            "support_combatant"
          ]
        },
        {
          tag: "defender_combatant",
          description: "Built to hold space, intercept attacks, bodyguard allies, or punish passage through a defended line.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved for guarding chokepoints, bodyguarding allies, or making passage costly.",
            "Space-holding matters more than raw pursuit, burst damage, or command."
          ],
          doesNotApplyWhen: [
            "The creature is merely durable without a real guard, intercept, or line-holding identity.",
            "The stronger fit is brute_combatant or another non-guardian role."
          ],
          adjacentTags: [
            "brute_combatant",
            "commander_combatant"
          ]
        },
        {
          tag: "support_combatant",
          description: "Built to heal, buff, protect, command, or otherwise enable allied creatures more than acting as the primary damage source.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved because it keeps allies alive, stronger, better positioned, or otherwise more dangerous.",
            "Ally enablement matters more than its own direct kill pressure."
          ],
          doesNotApplyWhen: [
            "The creature has one incidental buff or heal but is otherwise a brute, artillery, or controller.",
            "The stronger identity is commander_combatant because leadership and coordination outweigh broad support."
          ],
          adjacentTags: [
            "commander_combatant",
            "controller_combatant"
          ]
        },
        {
          tag: "ambusher_combatant",
          description: "Built around stealth openings, surprise rounds, sudden strike pressure, or hidden attack vectors.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved for hidden approach, surprise attack, trapdoor positioning, or burst from concealment.",
            "Opening from stealth is central to how it functions in an encounter."
          ],
          doesNotApplyWhen: [
            "The creature only uses stealth incidentally before acting as another clearer combat role.",
            "The stronger identity is skirmisher_combatant or artillery_combatant rather than surprise-predator play."
          ],
          adjacentTags: [
            "skirmisher_combatant",
            "brute_combatant"
          ]
        },
        {
          tag: "commander_combatant",
          description: "Built to coordinate, direct, or elevate allied creatures through leadership, tactics, or pack-control abilities.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "The creature is naturally retrieved because it commands minions, improves allied action quality, or orchestrates group tactics.",
            "Its encounter identity depends on leadership or coordination more than on solo offense."
          ],
          doesNotApplyWhen: [
            "The creature only has generic support effects with no real command, minion, or tactical leadership profile.",
            "The stronger fit is support_combatant or artillery_combatant rather than leader play."
          ],
          adjacentTags: [
            "support_combatant",
            "artillery_combatant"
          ]
        }
      ]
    },
    scene_role: {
      axis: "scene_role",
      description: "Creature immediate-scenario tags for how a role-defined NPC or humanoid is used in an active scene, separate from broader job, office, or institutional identity.",
      tags: [
        {
          tag: "civic_npc",
          description: "Fits the civic or social fabric of a scene and usually isn't the primary monster answer.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature belongs to the civic, domestic, institutional, or everyday social fabric of the scene.",
            "A GM would plausibly retrieve it as a socially embedded NPC rather than as a combat-forward foe.",
            "This tag answers the creature's immediate scene slot, even if a separate world_role tag explains its profession or office."
          ],
          doesNotApplyWhen: [
            "The record is primarily a hostile combatant or raider.",
            "The creature is only profession-labeled without clear social embeddedness, or the profession or office itself is the main retrieval hook."
          ],
          adjacentTags: [
            "profession_npc",
            "combatant_npc"
          ]
        },
        {
          tag: "combatant_npc",
          description: "Scene-slot humanoid combatant such as a soldier, bandit, mercenary, or cult enforcer.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is presented as a role-defined humanoid adversary or martial operative.",
            "Retrieval value comes from combat function rather than civic embedding.",
            "This tag answers how the creature functions in the immediate scene, even if it separately has a profession, rank, or institutional role."
          ],
          doesNotApplyWhen: [
            "The record is mainly a social or occupational NPC.",
            "The creature is better captured by profession_npc or civic_npc without a strong combat-forward role.",
            "The office, profession, or contact role is the stronger retrieval hook and combat readiness is only secondary."
          ],
          adjacentTags: [
            "authority_npc",
            "civic_npc"
          ]
        },
        {
          tag: "infiltrator_npc",
          description: "Scene-slot spy, saboteur, replacer, or quiet-entry specialist whose immediate scenario value comes from infiltration more than a straight fight.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved as an embedded infiltrator, saboteur, impostor, spy, or quiet-entry specialist.",
            "This tag answers the creature's immediate scenario function rather than its broader profession, faction post, or criminal affiliation."
          ],
          doesNotApplyWhen: [
            "The creature is mainly a combatant_npc or civic_npc and only uses stealth or deception incidentally.",
            "The stronger retrieval hook is criminal_npc or another world_role tag because the world-facing identity matters more than the scene slot."
          ],
          adjacentTags: [
            "combatant_npc",
            "criminal_npc"
          ]
        },
        {
          tag: "guardian_npc",
          description: "Immediate-scenario guard, jailer, doorkeeper, bodyguard, or other posted protector whose scene value is holding or protecting a person, threshold, or place.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved because it is posted to guard, hold, jail, or protect a specific person, threshold, route, or space.",
            "The protective station matters more than a broader office, profession, or general combat readiness."
          ],
          doesNotApplyWhen: [
            "The creature is merely a combat-ready NPC without a clear posted protection or guard duty.",
            "The stronger fit is authority_npc or combatant_npc because command status or generic martial opposition matters more than guarding."
          ],
          adjacentTags: [
            "combatant_npc",
            "authority_npc"
          ]
        },
        {
          tag: "watcher_npc",
          description: "Immediate-scenario lookout, sentry, observer, or patrol-point watcher whose scene value is warning, spotting, or noticing intruders.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved as a lookout, sentry, rooftop watcher, scout-on-post, or other early-warning presence.",
            "Observation and alarm value matter more than direct command, social office, or frontline melee identity."
          ],
          doesNotApplyWhen: [
            "The creature is simply a guard or combatant without a strong surveillance or lookout function.",
            "The stronger fit is guide_npc or civic_npc because travel knowledge or social embedding matters more than active watch duty."
          ],
          adjacentTags: [
            "guardian_npc",
            "infiltrator_npc"
          ]
        },
        {
          tag: "escort_npc",
          description: "Immediate-scenario escort, courier companion, guide-on-mission, ward mover, or other figure whose scene value is accompanying or moving someone through danger.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved because it escorts, transports, shepherds, or accompanies another figure through a scene or route.",
            "Its scene function is accompaniment or safe transit rather than only office, profession, or combat duty."
          ],
          doesNotApplyWhen: [
            "The creature merely knows the route or protects an area without actively accompanying someone.",
            "The stronger fit is guide_npc or guardian_npc because world-facing pathfinding or posted guard duty matters more than escorting."
          ],
          adjacentTags: [
            "guide_npc",
            "guardian_npc"
          ]
        },
        {
          tag: "captive_npc",
          description: "Immediate-scenario prisoner, hostage, detained witness, sacrifice target, or other constrained figure whose scene value comes from being held or imperiled.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved because it is held, imprisoned, restrained, threatened, or otherwise scenically constrained.",
            "Its immediate scenario function is being rescued, questioned, transported, or guarded rather than acting freely in the scene."
          ],
          doesNotApplyWhen: [
            "The creature is merely vulnerable, socially subordinate, or under pressure without actually being a captive or detained figure.",
            "The stronger fit is civic_npc or contact_npc because social embeddedness or information exchange matters more than captivity."
          ],
          adjacentTags: [
            "guardian_npc",
            "contact_npc"
          ]
        },
        {
          tag: "contact_npc",
          description: "Immediate-scenario informant, broker, fixer, negotiator, witness, or other social lead whose scene value is what they know, arrange, or reveal right now.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved because it provides information, access, leverage, bargaining, or a social lead in the immediate scenario.",
            "The scene value comes from contact utility rather than profession label, office, or direct combat role."
          ],
          doesNotApplyWhen: [
            "The creature is mainly defined by its broader office, trade, or faction identity without a specific contact-facing scene function.",
            "The stronger fit is civic_npc or profession_npc because ongoing social embedding or world role matters more than immediate lead value."
          ],
          adjacentTags: [
            "civic_npc",
            "profession_npc"
          ]
        }
      ]
    },
    world_role: {
      axis: "world_role",
      description: "Creature world-facing role tags for jobs, offices, institutions, and social identities that matter outside one immediate encounter slot.",
      tags: [
        {
          tag: "profession_npc",
          description: "World-facing role-defined NPC such as a captain, guard, merchant, priest, or commoner.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is primarily presented through a social role, job, office, or profession label.",
            "Retrieval value comes from it being a role-defined NPC rather than a species-driven monster.",
            "This tag answers what the creature is in the world, even if a separate scene_role tag explains how it functions in the scene."
          ],
          doesNotApplyWhen: [
            "The role label is incidental to a stronger monster or combat identity.",
            "The creature is better modeled only as combatant_npc, infiltrator_npc, or civic_npc because the scene slot matters more than the job or office."
          ],
          adjacentTags: [
            "authority_npc",
            "merchant_npc"
          ]
        },
        {
          tag: "authority_npc",
          description: "Presented as an officer, magistrate, noble, administrator, or other figure of formal social authority.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's retrieval value comes from official office, rank, command, or institutional authority.",
            "A GM would plausibly seek it as a leader, official, or governing figure.",
            "Formal office or rank is the main retrieval hook, even if the creature also serves as a civic_npc or combatant_npc in the scene."
          ],
          doesNotApplyWhen: [
            "The record is only a generic combatant without meaningful office or status.",
            "The stronger fit is profession_npc, civic_npc, or combatant_npc because status is incidental."
          ],
          adjacentTags: [
            "profession_npc",
            "civic_npc"
          ]
        },
        {
          tag: "religious_npc",
          description: "Presented as a priest, shrine keeper, cult officiant, monastic figure, or other explicitly religious role-holder.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Religious office, ritual duty, or custodianship of a faith space is central to the creature's world-facing identity.",
            "The creature is naturally retrieved as clergy, cult staff, or sacred-site personnel."
          ],
          doesNotApplyWhen: [
            "The creature merely has divine powers without a role-defined religious identity.",
            "Temple placement alone is better captured by temple_setting."
          ],
          adjacentTags: [
            "profession_npc",
            "temple_setting"
          ]
        },
        {
          tag: "criminal_npc",
          description: "Presented as a thief, smuggler, assassin, gang operative, fence, or other explicitly underworld-coded role.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is defined by illicit trade, organized crime, covert violence, or other criminal social function.",
            "A GM would retrieve it as an underworld contact or criminal adversary rather than a generic soldier."
          ],
          doesNotApplyWhen: [
            "The creature is merely hostile without underworld or crime-scene framing.",
            "The stronger fit is combatant_npc without a distinct criminal identity."
          ],
          adjacentTags: [
            "combatant_npc",
            "infiltrator_npc"
          ]
        },
        {
          tag: "scholar_npc",
          description: "Presented as a sage, researcher, teacher, archivist, alchemist, or other knowledge-centered role-holder.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Research, teaching, scholarship, or recordkeeping is central to the creature's world-facing identity.",
            "The creature is naturally retrieved as an academic or knowledge-scene NPC."
          ],
          doesNotApplyWhen: [
            "Intelligence is incidental to a stronger combat or monster identity.",
            "The record only implies general competence without a knowledge-centered role."
          ],
          adjacentTags: [
            "profession_npc",
            "civic_npc"
          ]
        },
        {
          tag: "healer_npc",
          description: "Presented as a physician, battlefield medic, herbalist, chirurgeon, caretaker, or other explicitly healing-facing role-holder.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Medical treatment, recovery support, or caretaker duty is central to the creature's world-facing identity.",
            "The record is naturally used as a healer, medic, apothecary, or restorative support NPC rather than only a generic scholar."
          ],
          doesNotApplyWhen: [
            "The creature merely has healing magic or restorative abilities without a role-defined caregiving identity.",
            "The stronger fit is scholar_npc, religious_npc, or civic_npc without a real healer-facing job."
          ],
          adjacentTags: [
            "scholar_npc",
            "religious_npc"
          ]
        },
        {
          tag: "merchant_npc",
          description: "Presented as a trader, broker, shopkeeper, caravan factor, or other commerce-facing role-holder.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Trade, selling, bargaining, or inventory-handling is central to the creature's world-facing identity.",
            "The creature naturally fills a market, shop, caravan, or supply-scene role."
          ],
          doesNotApplyWhen: [
            "The record only implies wealth or possessions without an actual merchant role.",
            "The stronger fit is civic_npc or profession_npc without a clear commerce function."
          ],
          adjacentTags: [
            "profession_npc",
            "civic_npc"
          ]
        },
        {
          tag: "artisan_npc",
          description: "Presented as a smith, craftsperson, builder, artisan, or other maker-facing role-holder tied to production or skilled labor.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Craft production, repair labor, or skilled making work is central to the creature's world-facing identity.",
            "A GM would plausibly retrieve it as a blacksmith, mason, tailor, shipwright, or similarly maker-facing contact."
          ],
          doesNotApplyWhen: [
            "The creature only sells goods or manages trade without being defined by making or repair work.",
            "The stronger fit is merchant_npc, profession_npc, or civic_npc without a clear craft-labor role."
          ],
          adjacentTags: [
            "merchant_npc",
            "profession_npc"
          ]
        },
        {
          tag: "guide_npc",
          description: "Presented as a scout, tracker, ferryman, caravan guide, wilderness pathfinder, or other route-leading specialist.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Leading others through terrain, routes, borders, or dangerous travel spaces is central to the creature's world-facing identity.",
            "The record is naturally used as a scout, pathfinder, navigator, or local guide rather than only a generic outdoors person."
          ],
          doesNotApplyWhen: [
            "The creature merely knows the area or has survival competence without a role-defined guiding function.",
            "The stronger fit is scholar_npc, civic_npc, or combatant_npc rather than travel-leading expertise."
          ],
          adjacentTags: [
            "profession_npc",
            "rural_setting"
          ]
        },
        {
          tag: "performer_npc",
          description: "Presented as a musician, actor, dancer, herald, jester, or other entertainment-facing role-holder.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Performance, spectacle, or entertainment labor is central to the creature's world-facing identity.",
            "The creature would be retrieved for theater, carnival, court entertainment, or tavern-stage scenes."
          ],
          doesNotApplyWhen: [
            "The creature is only whimsical or colorful without an explicit performer role.",
            "The stronger semantic is carnival_show rather than a role-defined entertainer."
          ],
          adjacentTags: [
            "profession_npc",
            "carnival_show"
          ]
        }
      ]
    },
    ontology_cluster: {
      axis: "specialization",
      description: "Creature semantic groupings that aggregate fragmented native ontology when exact traits are too narrow.",
      tags: [
        {
          tag: "undead_adjacent",
          description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals"
        },
        {
          tag: "sinspawn_family",
          description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
          assignmentMode: "hybrid",
          nativeOntologyPolicy: "aggregates_native_signals"
        }
      ]
    },
    casting_profile: {
      axis: "specialization",
      description: "Creature prep-driving casting tags for encounter planning, tradition-aware counterplay, and shortlist searches.",
      tags: [
        {
          tag: "dragon_spellcaster",
          description: "Dragon or archdragon variant with an explicit spellcaster stat block or named spellcaster presentation.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Use when a dragon or archdragon variant explicitly presents meaningful spellcasting as part of its encounter identity.",
            "The spellcasting matters for prep and counterplay beyond incidental magical flavor."
          ],
          doesNotApplyWhen: [
            "The dragon only has innate magical flavor, one-off magical actions, or a few utility effects without real spellcaster framing.",
            "The stronger fit is only a tradition-specific spellcaster tag without a dragon-specific spellcaster presentation."
          ],
          adjacentTags: [
            "arcane_spellcaster",
            "ritualist_creature"
          ]
        },
        {
          tag: "arcane_spellcaster",
          description: "Creature whose spellcasting is substantially framed through arcane traditions, wizardry, runes, or similarly arcane technique.",
          assignmentMode: "hybrid"
        },
        {
          tag: "divine_spellcaster",
          description: "Creature whose spellcasting is substantially framed through divine prayer, sacred miracles, or deity-facing magic.",
          assignmentMode: "hybrid"
        },
        {
          tag: "occult_spellcaster",
          description: "Creature whose spellcasting is substantially framed through occult lore, spirits, emotion, dreams, or esoteric mental power.",
          assignmentMode: "hybrid"
        },
        {
          tag: "primal_spellcaster",
          description: "Creature whose spellcasting is substantially framed through nature, elemental power, druidic force, or instinctive primal magic.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ritualist_creature",
          description: "Creature strongly associated with ritual casting, ceremonial magic, or extended occult or divine preparations.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Use when ceremonial, circle-based, sacrificial, or extended-casting magic is a major reason to retrieve the creature.",
            "The creature is naturally used as a ritual leader, ritual threat, or ritual-supporting encounter element."
          ],
          doesNotApplyWhen: [
            "The creature merely casts normal encounter spells without a meaningful ritual identity.",
            "The stronger fit is a tradition spellcaster tag and ritual work is only incidental flavor."
          ],
          adjacentTags: [
            "divine_spellcaster",
            "occult_spellcaster"
          ]
        }
      ]
    },
    threat_profile: {
      axis: "encounter",
      description: "Creature prep-driving threat patterns based on behavior, counterplay, or encounter consequence rather than type.",
      tags: [
        {
          tag: "possession_threat",
          description: "Can possess, body-snatch, or take control of a victim from within.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Use when entering, riding, replacing, or controlling a host body is a major reason to retrieve the creature.",
            "The possession dynamic matters more than ordinary charm, domination, or haunting flavor."
          ],
          doesNotApplyWhen: [
            "The creature only compels, frightens, or mentally influences targets without true body-occupying takeover.",
            "The stronger fit is curse_threat or summoner_commander because possession is not central to encounter prep."
          ],
          adjacentTags: [
            "curse_threat",
            "summoner_commander"
          ]
        },
        {
          tag: "life_drain_threat",
          description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
          assignmentMode: "hybrid"
        },
        {
          tag: "spawn_creator",
          description: "Creates additional threats through infestation, spawn-making, conversion, or implanted offspring.",
          assignmentMode: "hybrid"
        },
        {
          tag: "petrification_threat",
          description: "Threat defined by petrifying victims or turning them to stone.",
          assignmentMode: "hybrid"
        },
        {
          tag: "regeneration_threat",
          description: "Regenerates or requires special suppression or finishing countermeasures.",
          assignmentMode: "hybrid"
        },
        {
          tag: "ambush_grabber",
          description: "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
          assignmentMode: "hybrid"
        },
        {
          tag: "prey_control_threat",
          description: "Threat defined by holding prey in place through grabs, constriction, webbing, swallowing, or other ongoing body-control pressure.",
          assignmentMode: "hybrid",
          adjacentTags: [
            "ambush_grabber",
            "terrain_control_threat"
          ]
        },
        {
          tag: "poison_threat",
          description: "Threat defined by venom, toxic excretions, poisoned weapons, or other recurring poison delivery.",
          assignmentMode: "hybrid"
        },
        {
          tag: "disease_vector",
          description: "Threat defined by spreading disease, curse-plague conditions, or infectious aftermath beyond immediate damage.",
          assignmentMode: "hybrid"
        },
        {
          tag: "curse_threat",
          description: "Threat defined by curses, doom effects, or other lingering supernatural afflictions imposed on victims.",
          assignmentMode: "hybrid"
        },
        {
          tag: "terrain_control_threat",
          description: "Threat defined by webs, walls, zones, hazards, or other space-shaping control that changes battlefield movement.",
          assignmentMode: "hybrid"
        },
        {
          tag: "summoner_commander",
          description: "Threat defined by calling reinforcements, commanding minions, or dramatically improving allied creatures.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Use when the creature's main prep significance is that it adds bodies, coordinates allies, or sharply elevates nearby minions.",
            "The encounter meaningfully changes because of its leadership, summoning, or reinforcement engine."
          ],
          doesNotApplyWhen: [
            "The creature only has one minor ally-facing buff or an incidental summon without changing encounter structure.",
            "The stronger fit is support_combatant or spawn_creator because command and reinforcement are not the real threat hook."
          ],
          adjacentTags: [
            "spawn_creator",
            "commander_combatant"
          ]
        },
        {
          tag: "minion_commander",
          description: "Threat defined by directing subordinate creatures, pack members, or summoned help as an encounter engine rather than acting alone.",
          assignmentMode: "hybrid",
          adjacentTags: [
            "summoner_commander",
            "commander_combatant"
          ]
        },
        {
          tag: "infiltration_threat",
          description: "Threat defined by disguise, replacement, infiltration, or remaining embedded among victims before the danger fully reveals itself.",
          assignmentMode: "hybrid",
          adjacentTags: [
            "disguised_pretender",
            "possession_threat"
          ]
        },
        {
          tag: "death_burst_threat",
          description: "Threat defined by explosive death effects, cursed aftermath, or punishing consequences when the creature is dropped.",
          assignmentMode: "hybrid"
        }
      ]
    },
    visual_motif: {
      axis: "presentation",
      description: "Creature presentation tags for recurring visual motifs, object imagery, and striking appearance-driven hooks that players and GMs often retrieve by look and iconography.",
      tags: [
        {
          tag: "living_toy",
          description: "Strongly associated with dolls, puppets, mannequins, or other animated playthings.",
          assignmentMode: "hybrid"
        },
        {
          tag: "living_artwork",
          description: "Strongly associated with paintings, graffiti, murals, portraits, or other artworks brought to life.",
          assignmentMode: "hybrid"
        },
        {
          tag: "mask_motif",
          description: "Strongly associated with masks, veils, ceremonial face-coverings, or deliberately obscured presentation.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Masks, veils, or deliberate face-covering are a salient presentation motif.",
            "The obscured face is part of the creature's recurring visual identity."
          ],
          doesNotApplyWhen: [
            "A mask appears once as minor equipment.",
            "The stronger semantic is faceless_horror or disguised_pretender rather than mask imagery."
          ],
          adjacentTags: [
            "disguised_pretender",
            "faceless_horror"
          ]
        },
        {
          tag: "faceless_horror",
          description: "Strongly associated with missing, hidden, stolen, or featureless faces.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Missing, stolen, hidden, or featureless faces are central to the creature's horror identity.",
            "Face absence or erasure is a recurring motif, not just a disguise tactic."
          ],
          doesNotApplyWhen: [
            "The creature is mainly about impersonation or disguise rather than facial absence.",
            "A covered face appears only as costume or gear."
          ],
          adjacentTags: [
            "disguised_pretender",
            "mask_motif"
          ]
        },
        {
          tag: "mirror_motif",
          description: "Strongly associated with mirrors, reflections, duplicated selves, or reflective surfaces as a core visual or horror identity.",
          assignmentMode: "hybrid"
        },
        {
          tag: "stitched_horror",
          description: "Strongly associated with sutures, patchwork flesh, sewn bodies, or visibly assembled corpse craftsmanship.",
          assignmentMode: "hybrid"
        }
      ]
    },
    genre_motif: {
      axis: "presentation",
      description: "Creature presentation tags for genre-tone, mood, and atmosphere-driven motifs that players and GMs often retrieve by horror flavor or story vibe.",
      tags: [
        {
          tag: "carnival_show",
          description: "Strongly associated with carnivals, circuses, clowns, jesters, or sideshow-style presentation.",
          assignmentMode: "hybrid"
        },
        {
          tag: "trickster_mischief",
          description: "Strongly associated with pranks, capricious humor, gleeful sabotage, or explicit trickster behavior.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
            "The creature is naturally sought for playful-but-dangerous chaos rather than generic destruction or villainy."
          ],
          doesNotApplyWhen: [
            "The creature is merely chaotic, unpredictable, or destructive without a prankster or mischief-facing identity.",
            "Whimsical presentation appears only as surface flavor and another presentation tag or combat role better explains why the creature is being retrieved."
          ],
          adjacentTags: [
            "carnival_show",
            "disguised_pretender"
          ]
        },
        {
          tag: "dream_nightmare",
          description: "Strongly associated with dreams, nightmares, sleep-haunting, surreal unreality, or subconscious dread.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Dream logic, nightmare intrusion, sleep visitation, or surreal unreality is central to the creature's story identity.",
            "A GM would plausibly retrieve the creature for dreamscapes, night terrors, or oneiric scenes even outside a literal Dreamlands setting."
          ],
          doesNotApplyWhen: [
            "The creature merely casts sleep or fear effects without a real dream or nightmare presentation theme.",
            "Dreamlands placement alone is better captured by dreamlands_setting."
          ],
          adjacentTags: [
            "dreamlands_setting",
            "cosmic_dread"
          ]
        },
        {
          tag: "folk_horror",
          description: "Strongly associated with rural superstition, old customs, harvest dread, village taboos, or uncanny folklore menace.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
            "Its retrieval value comes from uncanny communal belief and traditional dread, not only from being outdoors or rural."
          ],
          doesNotApplyWhen: [
            "The creature is merely found in fields, forests, or villages without a real folklore or superstition-facing motif.",
            "The stronger fit is only rural_setting, swamp_setting, or another location tag."
          ],
          adjacentTags: [
            "rural_setting",
            "funerary_mourning"
          ]
        },
        {
          tag: "seductive_temptation",
          description: "Strongly associated with allure, seduction, dangerous invitation, or temptation into vice, doom, or compromise.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's presentation centers on allure, invitation, enchantment through desire, or baiting victims into a doomed choice.",
            "Retrieval value comes from temptation or dangerous attraction, not just social interaction or mechanical charm effects."
          ],
          doesNotApplyWhen: [
            "The creature is merely attractive, charismatic, or capable of charm without temptation being a real story hook.",
            "The stronger fit is disguised_pretender or courtly_pageantry rather than luring desire."
          ],
          adjacentTags: [
            "disguised_pretender",
            "courtly_pageantry"
          ]
        },
        {
          tag: "predatory_seduction",
          description: "Strongly associated with deliberate luring, honey-trap menace, erotic predation, or invitation used explicitly as a hunting tactic.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's presentation centers on luring prey through desire, intimacy, or false safety before the attack or betrayal lands.",
            "Use when the hunting or consuming dynamic matters more than general temptation or glamour."
          ],
          doesNotApplyWhen: [
            "The creature is alluring or corruptive without a strong predator-lure structure.",
            "The stronger fit is seductive_temptation because dangerous attraction matters more than an explicit hunt pattern."
          ],
          adjacentTags: [
            "seductive_temptation",
            "disguised_pretender"
          ]
        },
        {
          tag: "cosmic_dread",
          description: "Strongly associated with void vastness, star-born dread, incomprehensible revelation, or insignificance before the cosmos.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes existential terror, starry abyssal scale, unknowable revelation, or the feeling of minds breaking before the universe.",
            "A GM would plausibly retrieve it for eldritch omen, cosmic terror, or revelation-of-the-void scenes."
          ],
          doesNotApplyWhen: [
            "The creature is merely alien, aberrant, or extraplanar without a strong cosmic-horror presentation.",
            "Astral or outer-planar placement alone is better captured by setting tags."
          ],
          adjacentTags: [
            "astral_setting",
            "dream_nightmare"
          ]
        },
        {
          tag: "body_horror",
          description: "Strongly associated with warped anatomy, invasive flesh transformation, surgical grotesquerie, or visceral physical corruption.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Distorted flesh, invasive alteration, exposed anatomy, or grotesque physical transformation is central to the creature's horror identity.",
            "A GM would retrieve the creature specifically for visceral corruption, mutation, or flesh-warp scenes."
          ],
          doesNotApplyWhen: [
            "The creature is merely monstrous, bloody, or physically powerful without a strong corruption-of-the-body motif.",
            "The stronger fit is stitched_horror or disease_vector because constructed patchwork or infection aftermath matters more than bodily grotesquerie as presentation."
          ],
          adjacentTags: [
            "stitched_horror",
            "disease_vector"
          ]
        },
        {
          tag: "innocence_twisted",
          description: "Strongly associated with childish innocence, nursery imagery, or comforting domestic symbols turned uncanny, cruel, or threatening.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's presentation depends on childlike, gentle, or comforting imagery becoming eerie, cruel, or dangerous.",
            "A GM would plausibly retrieve it for nursery horror, storybook menace, or innocence-corrupted scenes."
          ],
          doesNotApplyWhen: [
            "The creature is merely small, playful, or associated with toys without innocence-curdled-into-menace being central.",
            "The stronger fit is living_toy or carnival_show because animated playthings or spectacle explain the retrieval better."
          ],
          adjacentTags: [
            "living_toy",
            "carnival_show"
          ]
        },
        {
          tag: "industrial_grotesque",
          description: "Strongly associated with smoke, gears, furnaces, exploitation, mutilating machinery, or dehumanizing industrial corruption.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes factory horror, machine-maimed labor, furnace dread, or industrial systems turning flesh and society into raw material.",
            "Retrieval value comes from industrial corruption and mechanized degradation, not just from being a construct or urban creature."
          ],
          doesNotApplyWhen: [
            "The creature merely uses technology, lives in a city, or is a construct without industrial corruption or machine-horror presentation.",
            "The stronger fit is body_horror or urban_setting because visceral corruption or city placement matters more than industrial atmosphere."
          ],
          adjacentTags: [
            "body_horror",
            "urban_setting"
          ]
        },
        {
          tag: "maritime_superstition",
          description: "Strongly associated with cursed voyages, sea omens, sailor folklore, drowned portents, or nautical dread shaped by legend and taboo.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved for ghost-ship rumor, sailor taboo, drowned prophecy, or sea-legend scenes where folklore matters as much as location.",
            "Use when nautical superstition and omen-laden seafaring culture are central to the creature's presentation."
          ],
          doesNotApplyWhen: [
            "The creature is merely aquatic, coastal, or ship-linked without a real folklore-and-omen maritime motif.",
            "The stronger fit is nautical_setting or folk_horror because placement or generic rural superstition matters more than sailor legend."
          ],
          adjacentTags: [
            "nautical_setting",
            "folk_horror"
          ]
        }
      ]
    },
    story_motif: {
      axis: "presentation",
      description: "Creature presentation tags for narrative framing, social-scene premises, and plot-driving motifs that players and GMs often retrieve by story hook rather than habitat or tactics.",
      tags: [
        {
          tag: "prophecy_omen",
          description: "Strongly associated with foretelling, omen-bearing, apocalyptic signs, destiny, or a creature's role as a herald of what is to come.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Portents, prophecy, omen-reading, or the creature's arrival as a sign of coming change is a central retrieval hook.",
            "A GM would plausibly retrieve the creature for foretold doom, chosen destiny, or fate-haunted story beats."
          ],
          doesNotApplyWhen: [
            "The creature merely predicts events, has divination magic, or is important to the plot without a strong omen-facing presentation.",
            "The stronger fit is apocalypse_ruin or ancestral_legacy because destiny-sign imagery is not the main presentation hook."
          ],
          adjacentTags: [
            "apocalypse_ruin",
            "ancestral_legacy"
          ]
        },
        {
          tag: "ritual_ceremony",
          description: "Strongly associated with rites, sacrifices, processions, ceremonial observance, or cultic staging as a scene identity.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Ceremonial staging, sacrificial ritual, processional presentation, or formal observance is a major retrieval hook.",
            "The creature is naturally used in scenes defined by rites, altars, chants, offerings, or public ceremony."
          ],
          doesNotApplyWhen: [
            "The creature merely has ritual magic or divine powers without a strong ceremonial scene identity.",
            "Temple placement or religious office alone is better captured by temple_setting or religious_npc."
          ],
          adjacentTags: [
            "religious_npc",
            "ritualist_creature"
          ]
        },
        {
          tag: "corrupted_sacred",
          description: "Strongly associated with profaned sanctity, fallen holiness, blasphemous devotion, or sacred imagery twisted into menace.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's presentation depends on desecrated ritual, fallen holiness, saintly imagery gone wrong, or sanctity turned threatening.",
            "Retrieval value comes from sacred symbolism being violated, inverted, or corrupted."
          ],
          doesNotApplyWhen: [
            "The creature is merely evil, undead, or hostile in a temple without sacred corruption being central to its identity.",
            "The stronger fit is ritual_ceremony or religious_npc because the creature is ceremonial or clerical without profaned sanctity."
          ],
          adjacentTags: [
            "ritual_ceremony",
            "religious_npc"
          ]
        },
        {
          tag: "vengeful_tragedy",
          description: "Strongly associated with betrayal, grief, injustice, or a sorrowful wrong returning as vengeance.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is framed around betrayal, mourning, loss, or an unresolved wrong curdling into vengeance.",
            "A GM would retrieve it for tragic revenants, wronged spirits, or revenge-driven story beats rather than generic hostility."
          ],
          doesNotApplyWhen: [
            "The creature is simply angry, hostile, or undead without a strong tragic-injustice presentation.",
            "The stronger fit is funerary_mourning because grief and ritual loss matter more than vengeance."
          ],
          adjacentTags: [
            "funerary_mourning",
            "disguised_pretender"
          ]
        },
        {
          tag: "paranoia_surveillance",
          description: "Strongly associated with hidden watchers, constant scrutiny, being monitored, or the collapse of trust under observation.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes stalking observation, unseen witnesses, informer networks, or dread rooted in being watched.",
            "Retrieval value comes from suspicion, surveillance, or omnipresent scrutiny rather than only stealth or infiltration."
          ],
          doesNotApplyWhen: [
            "The creature merely scouts, spies, or infiltrates without a broader atmosphere of surveillance and distrust.",
            "The stronger fit is occult_conspiracy or disguised_pretender because hidden coordination or impersonation matters more than the watched feeling."
          ],
          adjacentTags: [
            "occult_conspiracy",
            "disguised_pretender"
          ]
        },
        {
          tag: "disguised_pretender",
          description: "Strongly associated with false identities, impersonation, infiltration, shapeshifting, or replacement.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Impersonation, replacement, disguise, or false identity is a central retrieval hook.",
            "The creature is framed around passing as someone or something else."
          ],
          doesNotApplyWhen: [
            "It merely uses stealth or deception without identity substitution.",
            "The stronger presentation tag is mask_motif or faceless_horror rather than impersonation."
          ],
          adjacentTags: [
            "faceless_horror",
            "mask_motif"
          ]
        },
        {
          tag: "courtly_pageantry",
          description: "Strongly associated with nobles, heraldry, formal spectacle, masquerade grandeur, or ceremonial court presentation.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Court spectacle, heraldic pomp, ballroom tension, formal procession, or aristocratic display is central to the creature's presentation.",
            "A GM would plausibly retrieve the creature for palace intrigue, masquerades, or noble ceremonial scenes rather than for office alone."
          ],
          doesNotApplyWhen: [
            "The creature merely holds rank or authority without strong pageantry, splendor, or court-scene presentation.",
            "The stronger fit is authority_npc or performer_npc rather than courtly spectacle."
          ],
          adjacentTags: [
            "authority_npc",
            "mask_motif"
          ]
        },
        {
          tag: "decadence_decline",
          description: "Strongly associated with faded luxury, aristocratic rot, opulent ruin, or beauty collapsing into moral and material decay.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes indulgent splendor gone rotten, noble decline, decadent excess, or crumbling beauty concealing corruption.",
            "A GM would plausibly retrieve it for decaying courts, ruined salons, or luxury-turned-horror scenes."
          ],
          doesNotApplyWhen: [
            "The creature is merely rich, noble, or associated with ruins without a clear decline-and-rot presentation.",
            "The stronger fit is courtly_pageantry or revelry_excess because spectacle or celebration matters more than decay."
          ],
          adjacentTags: [
            "courtly_pageantry",
            "revelry_excess"
          ]
        },
        {
          tag: "ancestral_legacy",
          description: "Strongly associated with bloodline burdens, inherited duty, dynastic memory, haunting lineage, or the weight of family legacy.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's presentation depends on lineage, inheritance, dynastic fate, ancestral memory, or family burden carried into the present.",
            "Retrieval value comes from inherited role, curse, or legacy rather than only social rank or prophecy."
          ],
          doesNotApplyWhen: [
            "The creature merely belongs to a noble house, species line, or ancestry without legacy pressure being central.",
            "The stronger fit is prophecy_omen or courtly_pageantry because omen-bearing destiny or aristocratic presentation matters more than inherited burden."
          ],
          adjacentTags: [
            "prophecy_omen",
            "courtly_pageantry"
          ]
        },
        {
          tag: "revelry_excess",
          description: "Strongly associated with feasts, drunken revels, riotous celebration, gluttony, or ecstatic overindulgence.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved for bacchanals, cursed feasts, debauched parties, or scenes of ecstatic excess.",
            "Celebration curdling into danger is part of the creature's recurring narrative identity."
          ],
          doesNotApplyWhen: [
            "The creature only appears near taverns or festivals without excess, indulgence, or revelry being central.",
            "The stronger fit is carnival_show or seductive_temptation rather than feast-and-excess presentation."
          ],
          adjacentTags: [
            "carnival_show",
            "seductive_temptation"
          ]
        },
        {
          tag: "seasonal_festival",
          description: "Strongly associated with solstice rites, harvest festivals, holiday customs, masked processions, or recurring calendar-bound celebration.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved for midsummer revels, winter rites, harvest pageants, or holiday scenes defined by recurring festal tradition.",
            "Calendar-bound custom or festival atmosphere is a central part of the creature's presentation rather than incidental backdrop."
          ],
          doesNotApplyWhen: [
            "The creature merely appears during a feast, fair, or celebration without a recurring seasonal or ritual festival identity.",
            "The stronger fit is revelry_excess, carnival_show, or folk_horror because indulgence, spectacle, or folklore carries the retrieval weight."
          ],
          adjacentTags: [
            "revelry_excess",
            "folk_horror"
          ]
        },
        {
          tag: "apocalypse_ruin",
          description: "Strongly associated with end-times, civilizational collapse, world-ending omen, or the sense that the creature heralds broad unraveling.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature evokes cataclysm, final collapse, prophesied ending, or ruin on a society-shaping scale.",
            "A GM would retrieve it for last-days storytelling, omens of collapse, or world-unmaking scenes rather than only for big threat level."
          ],
          doesNotApplyWhen: [
            "The creature is simply powerful, destructive, or extraplanar without a real end-times or collapse-of-order presentation.",
            "The stronger fit is cosmic_dread or prophecy_omen because existential scale or foretold signs matter more than ruin itself."
          ],
          adjacentTags: [
            "cosmic_dread",
            "prophecy_omen"
          ]
        },
        {
          tag: "forbidden_knowledge",
          description: "Strongly associated with taboo lore, dangerous revelation, blasphemous truth, or learning that should not be uncovered.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is framed around hidden texts, proscribed secrets, mind-breaking truths, or knowledge pursued at terrible cost.",
            "Retrieval value comes from the danger of revelation, not merely from scholarship or intelligence."
          ],
          doesNotApplyWhen: [
            "The creature is simply smart, scholarly, occult, or mysterious without dangerous-knowledge presentation being central.",
            "The stronger fit is occult_conspiracy or cosmic_dread because hidden cabals or existential terror matters more than taboo learning itself."
          ],
          adjacentTags: [
            "occult_conspiracy",
            "cosmic_dread"
          ]
        },
        {
          tag: "funerary_mourning",
          description: "Strongly associated with grief, funeral rites, mourning processions, memorial haunting, or death-ritual solemnity.",
          assignmentMode: "editorial",
          appliesWhen: [
            "Funeral symbolism, grief-haunting, mourning observance, or memorial ritual is central to the creature's presentation.",
            "The creature is naturally retrieved for funerary scenes, dirges, wakes, or death-ritual storytelling rather than only because it is undead."
          ],
          doesNotApplyWhen: [
            "The creature is merely undead, ghostly, or graveyard-linked without a meaningful mourning or funerary presentation.",
            "The stronger fit is graveyard_setting, boneyard_setting, or undead_adjacent without a real ritualized grief presentation."
          ],
          adjacentTags: [
            "ritual_ceremony",
            "mask_motif"
          ]
        },
        {
          tag: "cursed_transformation",
          description: "Strongly associated with involuntary metamorphosis, curse-driven loss of self, or identity eroded by becoming something else.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature's story identity centers on an afflicting change, monstrous becoming, or transformation that threatens personhood.",
            "A GM would retrieve it for cursed metamorphosis, bestial change, or body-and-identity corruption rather than only raw mutation."
          ],
          doesNotApplyWhen: [
            "The creature merely transforms, shapeshifts, or mutates as an ability without cursed or tragic transformation being the presentation hook.",
            "The stronger fit is body_horror or disguised_pretender because physical grotesquerie or impersonation matters more than involuntary becoming."
          ],
          adjacentTags: [
            "body_horror",
            "ancestral_legacy"
          ]
        },
        {
          tag: "obsession_fixation",
          description: "Strongly associated with compulsive pursuit, jealous fixation, collecting mania, perfectionism, or a single consuming desire.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is framed around monomania, possessive attention, compulsive collecting, or a need it cannot release.",
            "Retrieval value comes from unhealthy fixation driving the story, not simply from a preference or goal."
          ],
          doesNotApplyWhen: [
            "The creature has a mission, desire, or recurring target without obsessive compulsion being central to its identity.",
            "The stronger fit is forbidden_knowledge or predatory_seduction because taboo learning or lure-based predation matters more than fixation."
          ],
          adjacentTags: [
            "forbidden_knowledge",
            "predatory_seduction"
          ]
        },
        {
          tag: "occult_conspiracy",
          description: "Strongly associated with secret circles, hidden masters, esoteric cabals, or ritual networks manipulating events from the shadows.",
          assignmentMode: "editorial",
          appliesWhen: [
            "The creature is naturally retrieved for hidden cult cells, secret masters, conspiratorial rites, or layered occult plotting.",
            "Its presentation depends on covert structure and esoteric collusion, not only on individual deception or ritual practice."
          ],
          doesNotApplyWhen: [
            "The creature merely participates in a ritual, infiltrates a group, or knows occult lore without a real cabal or conspiracy presentation.",
            "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation."
          ],
          adjacentTags: [
            "paranoia_surveillance",
            "forbidden_knowledge"
          ]
        },
      ]
    },
    bound_object: {
      axis: "presentation",
      description: "Creature tags for animated objects, statues, and other bound physical forms.",
      tags: [
        {
          tag: "animated_object",
          description: "Strongly associated with animated objects, furniture, tools, or other constructed items.",
          assignmentMode: "hybrid"
        },
        {
          tag: "animated_statue",
          description: "Strongly associated with animated statues, effigies, idols, or monuments.",
          assignmentMode: "hybrid"
        },
        {
          tag: "possessed_object",
          description: "Strongly associated with an inhabiting spirit or curse animating an otherwise mundane object or suit of equipment.",
          assignmentMode: "hybrid",
          appliesWhen: [
            "Use when a spirit, ghost, curse, or other external presence is explicitly what animates the object.",
            "The inhabiting presence matters more than the object's construction, material, or generic animation."
          ],
          doesNotApplyWhen: [
            "The object is simply animated by magic, clockwork, or sculpted animation with no real possessing force.",
            "The stronger fit is animated_object or animated_statue because possession is not central."
          ],
          adjacentTags: [
            "animated_object",
            "possession_threat"
          ]
        }
      ]
    }
  }
} satisfies DerivedTagAuthoredCategoryOntology<"creature">;
