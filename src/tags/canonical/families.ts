import type { DerivedTagOntologyFamily } from "../../domain/derived-tag-types.js";

export const DERIVED_TAG_CANONICAL_FAMILIES: DerivedTagOntologyFamily[] = 
[
  {
    axis: "behavior",
    category: "affliction",
    description: "Affliction tags for forced behavior and explicit agency override.",
    family: "behavioral_override",
    label: "behavioral_override"
  },
  {
    axis: "disease_model",
    category: "affliction",
    description: "Affliction tags for how exposure or transmission enters the body, breath, bloodstream, or dreaming mind.",
    family: "delivery_profile",
    label: "delivery_profile"
  },
  {
    axis: "disease_model",
    category: "affliction",
    description: "Affliction tags for outbreak scale, vector style, and contagion-facing retrieval such as quarantine, source tracing, and settlement risk.",
    family: "epidemiological_profile",
    label: "epidemiological_profile"
  },
  {
    axis: "effect",
    category: "affliction",
    description: "Affliction impact tags for practical downstream consequences.",
    family: "impact",
    label: "impact"
  },
  {
    axis: "metaphysical",
    category: "affliction",
    description: "Affliction tags for soul-straining corruption, curse marks, and dream- or spirit-facing torment.",
    family: "metaphysical_profile",
    label: "metaphysical_profile"
  },
  {
    axis: "disease_model",
    category: "affliction",
    description: "Affliction tags for recurring infection, corruption, and body-changing disease patterns.",
    family: "pathogenesis",
    label: "pathogenesis"
  },
  {
    axis: "effect",
    category: "affliction",
    description: "Affliction tags for forced breathing failure, catastrophic body change, and organ-level disruption.",
    family: "physiology_override",
    label: "physiology_override"
  },
  {
    axis: "disease_model",
    category: "affliction",
    description: "Affliction tags for pacing, relapse patterns, and how the condition escalates toward its end state.",
    family: "progression_profile",
    label: "progression_profile"
  },
  {
    axis: "response",
    category: "affliction",
    description: "Affliction tags for the kinds of remedies, containment plans, or cleanup answers the condition naturally asks for.",
    family: "resolution_profile",
    label: "resolution_profile"
  },
  {
    axis: "response",
    category: "affliction",
    description: "Affliction tags for the GM-facing response problem: quarantine, tracing the source, outbreak management, and how urgently a cure must be found.",
    family: "response_profile",
    label: "response_profile"
  },
  {
    axis: "presentation",
    category: "creature",
    description: "Creature tags for animated objects, statues, and other bound physical forms.",
    family: "bound_object",
    label: "bound_object"
  },
  {
    axis: "specialization",
    category: "creature",
    description: "Creature prep-driving casting tags for encounter planning, tradition-aware counterplay, and shortlist searches.",
    family: "casting_profile",
    label: "casting_profile"
  },
  {
    axis: "encounter",
    category: "creature",
    description: "Creature roster-construction tags for the kinds of crews, patrols, cult cells, packs, and retinues a creature naturally belongs to in a scene.",
    family: "cohort_role",
    label: "cohort_role"
  },
  {
    axis: "encounter",
    category: "creature",
    description: "Creature combat-role tags for encounter assembly, tactical reading, and balancing mixed enemy groups.",
    family: "combat_role",
    label: "combat_role"
  },
  {
    axis: "specialization",
    category: "creature",
    description: "Creature-facing corruption and taint tags for retrieval driven by what kind of blight, infestation, curse, or metaphysical pollution defines the creature.",
    family: "corruption_profile",
    label: "corruption_profile"
  },
  {
    axis: "presentation",
    category: "creature",
    description: "Creature presentation tags for genre-tone, mood, and atmosphere-driven motifs that players and GMs often retrieve by horror flavor or story vibe.",
    family: "genre_motif",
    label: "genre_motif"
  },
  {
    axis: "setting",
    category: "creature",
    description: "Creature habitat tags for terrain, climate, and ecological placement such as aquatic, desert, sky, or underground retrieval.",
    family: "habitat_setting",
    label: "habitat_setting",
    variantInheritance: true
  },
  {
    axis: "specialization",
    category: "creature",
    description: "Creature semantic groupings that aggregate fragmented native ontology when exact traits are too narrow.",
    family: "ontology_cluster",
    label: "ontology_cluster"
  },
  {
    axis: "setting",
    category: "creature",
    description: "Creature setting tags for planes, cosmological realms, and extraplanar retrieval patterns.",
    family: "planar_setting",
    label: "planar_setting",
    variantInheritance: true
  },
  {
    axis: "setting",
    category: "creature",
    description: "Creature setting tags for broad Golarion macro-regions plus portable thematic macro-regions derived from iconic Pathfinder settings when those lenses materially affect encounter planning and retrieval.",
    family: "regional_setting",
    label: "regional_setting",
    variantInheritance: true
  },
  {
    axis: "npc_role",
    category: "creature",
    description: "Creature immediate-scenario tags for how a role-defined NPC or humanoid is used in an active scene, separate from broader job, office, or institutional identity.",
    family: "scene_role",
    label: "scene_role"
  },
  {
    axis: "setting",
    category: "creature",
    description: "Creature site and scene-placement tags for ships, settlements, ruins, temples, battlefields, graveyards, and other encounter locations.",
    family: "site_setting",
    label: "site_setting",
    variantInheritance: true
  },
  {
    axis: "npc_role",
    category: "creature",
    description: "Creature social-role tags for jobs, offices, institutions, and social identities that matter outside one immediate encounter slot.",
    family: "social_role",
    label: "social_role"
  },
  {
    axis: "presentation",
    category: "creature",
    description: "Creature presentation tags for narrative framing, social-scene premises, and plot-driving motifs that players and GMs often retrieve by story hook rather than habitat or tactics.",
    family: "story_motif",
    label: "story_motif"
  },
  {
    axis: "encounter",
    category: "creature",
    description: "Creature prep-driving threat patterns based on behavior, counterplay, or encounter consequence rather than type.",
    family: "threat_profile",
    label: "threat_profile"
  },
  {
    axis: "presentation",
    category: "creature",
    description: "Creature presentation tags for recurring visual motifs, object imagery, and striking appearance-driven hooks that players and GMs often retrieve by look and iconography.",
    family: "visual_motif",
    label: "visual_motif"
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment for technical or precision bypass of locks, traps, and secured access points without folding force-entry tools into the same bucket.",
    family: "access_bypass",
    label: "access_bypass",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "item_mechanical",
    category: "equipment",
    description: "Storage, draw, and ammunition-handling equipment that changes how gear is accessed in play.",
    family: "access_system",
    label: "access_system",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "ammo",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "item_mechanical",
    category: "equipment",
    description: "Ammunition payload tags for recurring on-hit effects and tailored projectile roles.",
    family: "ammunition_payload",
    label: "ammunition_payload",
    subcategories: [
      "ammo"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment for countering hostile magic or protecting against harmful magical effects.",
    family: "anti_magic",
    label: "anti_magic",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment built to force entry, open routes through barriers, or dismantle hardened environmental features.",
    family: "breaching",
    label: "breaching",
    subcategories: [
      "gear",
      "kit",
      "consumable",
      "weapon"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment for stowing, carrying, organizing, and transporting gear through play.",
    family: "carry_logistics",
    label: "carry_logistics",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Coordination, signaling, language-bridging, and message-relay equipment.",
    family: "communication",
    label: "communication",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ]
  },
  {
    axis: "effect",
    category: "equipment",
    description: "Consumable role tags for whether an item is primarily hostile, self-directed, or meant to aid another creature.",
    family: "consumable_role",
    label: "consumable_role",
    subcategories: [
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Tools and supplies that enable treatment, repair, paperwork, crafting, or ritual preparation.",
    family: "crafting_support",
    label: "crafting_support",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ]
  },
  {
    axis: "item_mechanical",
    category: "equipment",
    description: "Shield and armor tags for interception, cover, and protection against ranged fire, hazardous scenes, or vertical danger.",
    family: "defense_profile",
    label: "defense_profile",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "shield"
    ]
  },
  {
    axis: "effect",
    category: "equipment",
    description: "Consumable delivery tags for how a hostile item is applied, delivered, or introduced to the target.",
    family: "delivery_profile",
    label: "delivery_profile",
    subcategories: [
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Field-sustainment equipment for expedition travel, provisioning, camp life, aquatic operations, and hostile-environment endurance.",
    family: "expedition",
    label: "expedition",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable",
      "armor",
      "shield",
      "weapon"
    ]
  },
  {
    axis: "effect",
    category: "equipment",
    description: "Beneficial consumable outcome and recovery tags.",
    family: "function",
    label: "function",
    subcategories: [
      "consumable"
    ]
  },
  {
    axis: "effect",
    category: "equipment",
    description: "Hostile equipment tags for recurring target impairments and disabling outcomes.",
    family: "impact",
    label: "impact",
    subcategories: [
      "ammo",
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Appearance-changing, discreet-carry, and social-passing equipment across gear and consumables.",
    family: "infiltration",
    label: "infiltration",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment that helps move, climb, navigate, or transport creatures and cargo from place to place.",
    family: "movement_traversal",
    label: "movement_traversal",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "effect",
    category: "equipment",
    description: "Hostile consumable and ammunition tags for bombs, payloads, smoke effects, sticky control, and other offensive retrieval patterns that are not just status impairments.",
    family: "offensive_profile",
    label: "offensive_profile",
    subcategories: [
      "ammo",
      "consumable"
    ]
  },
  {
    axis: "party_role",
    category: "equipment",
    description: "Party-facing retrieval tags for loot that is valuable because of who benefits from it in a party or build.",
    family: "party_role",
    label: "party_role"
  },
  {
    axis: "party_role",
    category: "equipment",
    description: "Build- and play-pattern-facing retrieval tags for loot that is valuable because of the recurring tactics or workflows it supports.",
    family: "play_pattern",
    label: "play_pattern"
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment for recon, field observation, visibility support, evidence capture, and tracking or anti-tracking work.",
    family: "reconnaissance",
    label: "reconnaissance",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment used to break curses, support appeasement rites, sanctify spaces, trace hidden sources, contain spread, clean corrupted sites, or deal with a problem at its source.",
    family: "resolution",
    label: "resolution",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "consumable"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Equipment for capturing, binding, securing, or escaping restraints and immobilizing holds.",
    family: "restraint",
    label: "restraint",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable",
      "armor",
      "weapon"
    ]
  },
  {
    axis: "utility",
    category: "equipment",
    description: "Intrusion-warning, anti-surveillance, seal-checking, and after-the-fact security gear for camps, vaults, cargo, and protected rooms.",
    family: "security",
    label: "security",
    subcategories: [
      "gear",
      "backpack",
      "kit",
      "vehicle",
      "consumable"
    ]
  },
  {
    axis: "effect",
    category: "hazard",
    description: "Hazards categorized by how their attack is delivered into the scene.",
    family: "attack_vector",
    label: "attack_vector"
  },
  {
    axis: "resolution",
    category: "hazard",
    description: "Hazard tags for distinctive resolution patterns that matter for prep beyond a raw disable-skill list, especially when the real retrieval question is how the party solves or bypasses the hazard.",
    family: "countermeasure_profile",
    label: "countermeasure_profile"
  },
  {
    axis: "effect",
    category: "hazard",
    description: "Hazards defined by recurring elemental or toxic environmental threats.",
    family: "environmental_danger",
    label: "environmental_danger"
  },
  {
    axis: "effect",
    category: "hazard",
    description: "Hazards that drop, collapse, or forcibly reposition creatures.",
    family: "forced_position",
    label: "forced_position"
  },
  {
    axis: "encounter",
    category: "hazard",
    description: "Hazard scene-pressure tags for alerts, lockdowns, area control, ambush punishment, attrition, and guarding valuable space.",
    family: "function",
    label: "function"
  },
  {
    axis: "haunt",
    category: "hazard",
    description: "Haunt manifestations that materially change the encounter through attackers, lures, possession, replayed trauma, or judgment.",
    family: "haunt_manifestation",
    label: "haunt_manifestation"
  },
  {
    axis: "effect",
    category: "hazard",
    description: "Hazard impact tags for mental destabilization and movement-limiting effects.",
    family: "impact",
    label: "impact"
  },
  {
    axis: "mechanism",
    category: "hazard",
    description: "Hazards whose threat comes from trigger mechanisms, locking thresholds, control surfaces, or planar breaches.",
    family: "mechanism",
    label: "mechanism"
  },
  {
    axis: "effect",
    category: "hazard",
    description: "Hazards that distort routes, orientation, or the perceived layout of a space.",
    family: "perception_control",
    label: "perception_control"
  },
  {
    axis: "problem",
    category: "hazard",
    description: "Hazard prep tags for the kind of investigation, timing, and layered-solving problem the party faces before the hazard is actually neutralized.",
    family: "problem_shape",
    label: "problem_shape"
  },
  {
    axis: "setting",
    category: "hazard",
    description: "Hazard scene-placement tags for the kinds of sites and encounter spaces where the danger is most naturally retrieved.",
    family: "setting",
    label: "setting"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells that open secured access, neutralize trapped entry, bypass blocked thresholds, or precisely manipulate important mechanisms.",
    family: "access_bypass",
    label: "access_bypass"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Communication and coordination spells for messaging, psychic speech, and language-bridging.",
    family: "communication",
    label: "communication"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells used to ask for guidance, gain analytical insight, diagnose hidden problems, or consult cosmic and magical knowledge beyond raw sensory scouting.",
    family: "consultation",
    label: "consultation"
  },
  {
    axis: "battlefield",
    category: "spell",
    description: "Spells that reshape the battlefield by denying movement, sight, casting, actions, or active magic.",
    family: "control",
    label: "control"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells that support movement, travel, survival, and field logistics such as flight, route guidance, shelter, food, aquatic operations, and hostile-environment endurance.",
    family: "expedition",
    label: "expedition"
  },
  {
    axis: "effect",
    category: "spell",
    description: "Direct offensive outcome tags for harm, impairment, and taking priority targets or clustered foes out of the fight.",
    family: "impact",
    label: "impact"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Quiet-entry, appearance-changing, and social-passing spells.",
    family: "infiltration",
    label: "infiltration"
  },
  {
    axis: "influence",
    category: "spell",
    description: "Spells that charm, compel, dominate, or emotionally steer a creature's behavior.",
    family: "influence",
    label: "influence"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Reconnaissance spells for surveying areas, extending senses, and locating specific targets before engagement.",
    family: "reconnaissance",
    label: "reconnaissance"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells used to directly solve curses, hauntings, appeasement problems, contamination, outbreak containment problems, and hidden supernatural causes rather than merely endure or diagnose them.",
    family: "resolution",
    label: "resolution"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Detection and revelation spells that expose hidden truths, concealed dangers, and supernatural facts.",
    family: "revelation",
    label: "revelation"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Security spells for intrusion warning, anti-scrying, and protected spaces, including broad security umbrellas.",
    family: "security",
    label: "security"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells that improve sight, hearing, scent, or usable light so creatures can perceive and operate better in darkness or obscurity.",
    family: "sensory_support",
    label: "sensory_support"
  },
  {
    axis: "summoning",
    category: "spell",
    description: "Spells that call, create, or bind temporary creatures and servitors into the scene for combat, scouting, labor, or utility retrieval.",
    family: "summoning",
    label: "summoning"
  },
  {
    axis: "support",
    category: "spell",
    description: "Spells that restore, protect, ward, or reinforce allies and targets for combat recovery, scene resilience, and expedition safety.",
    family: "support",
    label: "support"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Spells that blink, reposition, extract, or transport creatures across long distances or planes.",
    family: "teleportation",
    label: "teleportation"
  },
  {
    axis: "transformation",
    category: "spell",
    description: "Spells that alter a creature's body, form, or battle shape.",
    family: "transformation",
    label: "transformation"
  },
  {
    axis: "utility",
    category: "spell",
    description: "Planning-oriented umbrella for spells used to find routes, reach destinations, and solve strategic navigation or travel-location problems.",
    family: "wayfinding",
    label: "wayfinding"
  }
];
