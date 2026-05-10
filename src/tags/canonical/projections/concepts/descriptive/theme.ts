import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveThemeProjectionDeclarations = [
  defineConceptProjections("ancestral_legacy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with bloodline burdens, inherited duty, dynastic memory, haunting lineage, or the weight of family legacy.",
      appliesWhen: [
        "The creature's presentation depends on lineage, inheritance, dynastic fate, ancestral memory, or family burden carried into the present.",
        "Retrieval value comes from inherited role, curse, or legacy rather than only social rank or prophecy.",
      ],
      doesNotApplyWhen: [
        "The creature merely belongs to a noble house, species line, or ancestry without legacy pressure being central.",
        "The stronger fit is prophecy_omen or courtly_pageantry because omen-bearing destiny or aristocratic presentation matters more than inherited burden.",
      ],
      adjacentTags: ["prophecy_omen", "courtly_pageantry"],
    },
  }),
  defineConceptProjections("apocalypse_ruin", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with end-times, civilizational collapse, world-ending omen, or the sense that the creature heralds broad unraveling.",
      appliesWhen: [
        "The creature evokes cataclysm, final collapse, prophesied ending, or ruin on a society-shaping scale.",
        "A GM would retrieve it for last-days storytelling, omens of collapse, or world-unmaking scenes rather than only for big threat level.",
      ],
      doesNotApplyWhen: [
        "The creature is simply powerful, destructive, or extraplanar without a real end-times or collapse-of-order presentation.",
        "The stronger fit is cosmic_dread or prophecy_omen because existential scale or foretold signs matter more than ruin itself.",
      ],
      adjacentTags: ["cosmic_dread", "prophecy_omen"],
    },
  }),
  defineConceptProjections("battlefield_disruption", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Haunt that reshapes the scene with barriers, violent manifestations, or other encounter-disrupting effects.",
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Creates area denial, difficult terrain, barriers, or other battlefield obstacles.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("blight_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by ecological blight, withering nature, corrupted groves, or land-sickened wilderness.",
      adjacentTags: ["wasteland_setting", "body_horror"],
    },
  }),
  defineConceptProjections("body_horror", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with warped anatomy, invasive flesh transformation, surgical grotesquerie, or visceral physical corruption.",
      appliesWhen: [
        "Distorted flesh, invasive alteration, exposed anatomy, or grotesque physical transformation is central to the creature's horror identity.",
        "A GM would retrieve the creature specifically for visceral corruption, mutation, or flesh-warp scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely monstrous, bloody, or physically powerful without a strong corruption-of-the-body motif.",
        "The stronger fit is stitched_horror or disease_vector because constructed patchwork or infection aftermath matters more than bodily grotesquerie as presentation.",
      ],
      adjacentTags: ["stitched_horror", "disease_vector"],
    },
  }),
  defineConceptProjections("carnival_show", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with carnivals, circuses, clowns, jesters, or sideshow-style presentation.",
    },
  }),
  defineConceptProjections("corrupted_sacred", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with profaned sanctity, fallen holiness, blasphemous devotion, or sacred imagery twisted into menace.",
      appliesWhen: [
        "The creature's presentation depends on desecrated ritual, fallen holiness, saintly imagery gone wrong, or sanctity turned threatening.",
        "Retrieval value comes from sacred symbolism being violated, inverted, or corrupted.",
      ],
      doesNotApplyWhen: [
        "The creature is merely evil, undead, or hostile in a temple without sacred corruption being central to its identity.",
        "The stronger fit is ritual_ceremony or religious_npc because the creature is ceremonial or clerical without profaned sanctity.",
      ],
      adjacentTags: ["ritual_ceremony", "religious_npc"],
    },
  }),
  defineConceptProjections("cosmic_dread", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with void vastness, star-born dread, incomprehensible revelation, or insignificance before the cosmos.",
      appliesWhen: [
        "The creature evokes existential terror, starry abyssal scale, unknowable revelation, or the feeling of minds breaking before the universe.",
        "A GM would plausibly retrieve it for eldritch omen, cosmic terror, or revelation-of-the-void scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely alien, aberrant, or extraplanar without a strong cosmic-horror presentation.",
        "Astral or outer-planar placement alone is better captured by setting tags.",
      ],
      adjacentTags: ["astral_setting", "dream_nightmare"],
    },
  }),
  defineConceptProjections("courtly_pageantry", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with nobles, heraldry, formal spectacle, masquerade grandeur, or ceremonial court presentation.",
      appliesWhen: [
        "Court spectacle, heraldic pomp, ballroom tension, formal procession, or aristocratic display is central to the creature's presentation.",
        "A GM would plausibly retrieve the creature for palace intrigue, masquerades, or noble ceremonial scenes rather than for office alone.",
      ],
      doesNotApplyWhen: [
        "The creature merely holds rank or authority without strong pageantry, splendor, or court-scene presentation.",
        "The stronger fit is authority_npc or performer_npc rather than courtly spectacle.",
      ],
      adjacentTags: ["authority_npc", "mask_motif"],
    },
  }),
  defineConceptProjections("cursed_transformation", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with involuntary metamorphosis, curse-driven loss of self, or identity eroded by becoming something else.",
      appliesWhen: [
        "The creature's story identity centers on an afflicting change, monstrous becoming, or transformation that threatens personhood.",
        "A GM would retrieve it for cursed metamorphosis, bestial change, or body-and-identity corruption rather than only raw mutation.",
      ],
      doesNotApplyWhen: [
        "The creature merely transforms, shapeshifts, or mutates as an ability without cursed or tragic transformation being the presentation hook.",
        "The stronger fit is body_horror or disguised_pretender because physical grotesquerie or impersonation matters more than involuntary becoming.",
      ],
      adjacentTags: ["body_horror", "ancestral_legacy"],
    },
  }),
  defineConceptProjections("cursewarped", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by curse-driven distortion, doom-warping, or being transformed into its current state by a curse.",
      adjacentTags: ["curse_threat", "cursed_transformation"],
    },
  }),
  defineConceptProjections("decadence_decline", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with faded luxury, aristocratic rot, opulent ruin, or beauty collapsing into moral and material decay.",
      appliesWhen: [
        "The creature evokes indulgent splendor gone rotten, noble decline, decadent excess, or crumbling beauty concealing corruption.",
        "A GM would plausibly retrieve it for decaying courts, ruined salons, or luxury-turned-horror scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely rich, noble, or associated with ruins without a clear decline-and-rot presentation.",
        "The stronger fit is courtly_pageantry or revelry_excess because spectacle or celebration matters more than decay.",
      ],
      adjacentTags: ["courtly_pageantry", "revelry_excess"],
    },
  }),
  defineConceptProjections("disguised_pretender", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with false identities, impersonation, infiltration, shapeshifting, or replacement.",
      appliesWhen: [
        "Impersonation, replacement, disguise, or false identity is a central retrieval hook.",
        "The creature is framed around passing as someone or something else.",
      ],
      doesNotApplyWhen: [
        "It merely uses stealth or deception without identity substitution.",
        "The stronger presentation tag is mask_motif or faceless_horror rather than impersonation.",
      ],
      adjacentTags: ["faceless_horror", "mask_motif"],
    },
  }),
  defineConceptProjections("dream_nightmare", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with dreams, nightmares, sleep-haunting, surreal unreality, or subconscious dread.",
      appliesWhen: [
        "Dream logic, nightmare intrusion, sleep visitation, or surreal unreality is central to the creature's story identity.",
        "A GM would plausibly retrieve the creature for dreamscapes, night terrors, or oneiric scenes even outside a literal Dreamlands setting.",
      ],
      doesNotApplyWhen: [
        "The creature merely casts sleep or fear effects without a real dream or nightmare presentation theme.",
        "Dreamlands placement alone is better captured by dreamlands_setting.",
      ],
      adjacentTags: ["dreamlands_setting", "cosmic_dread"],
    },
  }),
  defineConceptProjections("epidemic_pestilence", {
    affliction: {
      axis: "disease_model",
      family: "epidemiological_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "A named plague-, fever-, pox-, or pestilence-style disease with explicit outbreak or contagion framing.",
    },
  }),
  defineConceptProjections("faceless_horror", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with missing, hidden, stolen, or featureless faces.",
      appliesWhen: [
        "Missing, stolen, hidden, or featureless faces are central to the creature's horror identity.",
        "Face absence or erasure is a recurring motif, not just a disguise tactic.",
      ],
      doesNotApplyWhen: [
        "The creature is mainly about impersonation or disguise rather than facial absence.",
        "A covered face appears only as costume or gear.",
      ],
      adjacentTags: ["disguised_pretender", "mask_motif"],
    },
  }),
  defineConceptProjections("folk_horror", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with rural superstition, old customs, harvest dread, village taboos, or uncanny folklore menace.",
      appliesWhen: [
        "The creature evokes old-country fear, harvest rites gone wrong, scarecrow dread, witchcraft omen, or taboo-laden local folklore.",
        "Its retrieval value comes from uncanny communal belief and traditional dread, not only from being outdoors or rural.",
      ],
      doesNotApplyWhen: [
        "The creature is merely found in fields, forests, or villages without a real folklore or superstition-facing motif.",
        "The stronger fit is only rural_setting, swamp_setting, or another location tag.",
      ],
      adjacentTags: ["rural_setting", "funerary_mourning"],
    },
  }),
  defineConceptProjections("forbidden_knowledge", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with taboo lore, dangerous revelation, blasphemous truth, or learning that should not be uncovered.",
      appliesWhen: [
        "The creature is framed around hidden texts, proscribed secrets, mind-breaking truths, or knowledge pursued at terrible cost.",
        "Retrieval value comes from the danger of revelation, not merely from scholarship or intelligence.",
      ],
      doesNotApplyWhen: [
        "The creature is simply smart, scholarly, occult, or mysterious without dangerous-knowledge presentation being central.",
        "The stronger fit is occult_conspiracy or cosmic_dread because hidden cabals or existential terror matters more than taboo learning itself.",
      ],
      adjacentTags: ["occult_conspiracy", "cosmic_dread"],
    },
  }),
  defineConceptProjections("funerary_mourning", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with grief, funeral rites, mourning processions, memorial haunting, or death-ritual solemnity.",
      appliesWhen: [
        "Funeral symbolism, grief-haunting, mourning observance, or memorial ritual is central to the creature's presentation.",
        "The creature is naturally retrieved for funerary scenes, dirges, wakes, or death-ritual storytelling rather than only because it is undead.",
      ],
      doesNotApplyWhen: [
        "The creature is merely undead, ghostly, or graveyard-linked without a meaningful mourning or funerary presentation.",
        "The stronger fit is graveyard_setting, boneyard_setting, or undead_adjacent without a real ritualized grief presentation.",
      ],
      adjacentTags: ["ritual_ceremony", "mask_motif"],
    },
  }),
  defineConceptProjections("fungal_infested", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by mycelium, spores, mushroom overgrowth, or fungus-driven bodily infestation.",
      adjacentTags: ["disease_vector", "body_horror"],
    },
  }),
  defineConceptProjections("industrial_grotesque", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with smoke, gears, furnaces, exploitation, mutilating machinery, or dehumanizing industrial corruption.",
      appliesWhen: [
        "The creature evokes factory horror, machine-maimed labor, furnace dread, or industrial systems turning flesh and society into raw material.",
        "Retrieval value comes from industrial corruption and mechanized degradation, not just from being a construct or urban creature.",
      ],
      doesNotApplyWhen: [
        "The creature merely uses technology, lives in a city, or is a construct without industrial corruption or machine-horror presentation.",
        "The stronger fit is body_horror or urban_setting because visceral corruption or city placement matters more than industrial atmosphere.",
      ],
      adjacentTags: ["body_horror", "urban_setting"],
    },
  }),
  defineConceptProjections("innocence_twisted", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with childish innocence, nursery imagery, or comforting domestic symbols turned uncanny, cruel, or threatening.",
      appliesWhen: [
        "The creature's presentation depends on childlike, gentle, or comforting imagery becoming eerie, cruel, or dangerous.",
        "A GM would plausibly retrieve it for nursery horror, storybook menace, or innocence-corrupted scenes.",
      ],
      doesNotApplyWhen: [
        "The creature is merely small, playful, or associated with toys without innocence-curdled-into-menace being central.",
        "The stronger fit is living_toy or carnival_show because animated playthings or spectacle explain the retrieval better.",
      ],
      adjacentTags: ["living_toy", "carnival_show"],
    },
  }),
  defineConceptProjections("judgment_haunt", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Haunt that condemns trespassers through accusation, punishment, curse-like verdicts, or moral reckoning.",
    },
  }),
  defineConceptProjections("life_drain_hazard", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that drains life force, vitality, or souls from victims.",
    },
  }),
  defineConceptProjections("living_artwork", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with paintings, graffiti, murals, portraits, or other artworks brought to life.",
    },
  }),
  defineConceptProjections("living_toy", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with dolls, puppets, mannequins, or other animated playthings.",
    },
  }),
  defineConceptProjections("lure_compulsion", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that beckons, lures, or compels creatures into moving or acting against their judgment.",
    },
  }),
  defineConceptProjections("maritime_superstition", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with cursed voyages, sea omens, sailor folklore, drowned portents, or nautical dread shaped by legend and taboo.",
      appliesWhen: [
        "The creature is naturally retrieved for ghost-ship rumor, sailor taboo, drowned prophecy, or sea-legend scenes where folklore matters as much as location.",
        "Use when nautical superstition and omen-laden seafaring culture are central to the creature's presentation.",
      ],
      doesNotApplyWhen: [
        "The creature is merely aquatic, coastal, or ship-linked without a real folklore-and-omen maritime motif.",
        "The stronger fit is nautical_setting or folk_horror because placement or generic rural superstition matters more than sailor legend.",
      ],
      adjacentTags: ["nautical_setting", "folk_horror"],
    },
  }),
  defineConceptProjections("mask_motif", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with masks, veils, ceremonial face-coverings, or deliberately obscured presentation.",
      appliesWhen: [
        "Masks, veils, or deliberate face-covering are a salient presentation motif.",
        "The obscured face is part of the creature's recurring visual identity.",
      ],
      doesNotApplyWhen: [
        "A mask appears once as minor equipment.",
        "The stronger semantic is faceless_horror or disguised_pretender rather than mask imagery.",
      ],
      adjacentTags: ["disguised_pretender", "faceless_horror"],
    },
  }),
  defineConceptProjections("mirror_motif", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with mirrors, reflections, duplicated selves, or reflective surfaces as a core visual or horror identity.",
    },
  }),
  defineConceptProjections("nightmare_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by dream corruption, sleep-haunting influence, or oneiric pollution leaking into the waking world.",
      adjacentTags: ["dream_nightmare", "dreamlands_setting"],
    },
  }),
  defineConceptProjections("obsession_fixation", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with compulsive pursuit, jealous fixation, collecting mania, perfectionism, or a single consuming desire.",
      appliesWhen: [
        "The creature is framed around monomania, possessive attention, compulsive collecting, or a need it cannot release.",
        "Retrieval value comes from unhealthy fixation driving the story, not simply from a preference or goal.",
      ],
      doesNotApplyWhen: [
        "The creature has a mission, desire, or recurring target without obsessive compulsion being central to its identity.",
        "The stronger fit is forbidden_knowledge or predatory_seduction because taboo learning or lure-based predation matters more than fixation.",
      ],
      adjacentTags: ["forbidden_knowledge", "predatory_seduction"],
    },
  }),
  defineConceptProjections("occult_conspiracy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with secret circles, hidden masters, esoteric cabals, or ritual networks manipulating events from the shadows.",
      appliesWhen: [
        "The creature is naturally retrieved for hidden cult cells, secret masters, conspiratorial rites, or layered occult plotting.",
        "Its presentation depends on covert structure and esoteric collusion, not only on individual deception or ritual practice.",
      ],
      doesNotApplyWhen: [
        "The creature merely participates in a ritual, infiltrates a group, or knows occult lore without a real cabal or conspiracy presentation.",
        "The stronger fit is ritual_ceremony, paranoia_surveillance, or forbidden_knowledge rather than hidden-network manipulation.",
      ],
      adjacentTags: ["paranoia_surveillance", "forbidden_knowledge"],
    },
  }),
  defineConceptProjections("paranoia_surveillance", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with hidden watchers, constant scrutiny, being monitored, or the collapse of trust under observation.",
      appliesWhen: [
        "The creature evokes stalking observation, unseen witnesses, informer networks, or dread rooted in being watched.",
        "Retrieval value comes from suspicion, surveillance, or omnipresent scrutiny rather than only stealth or infiltration.",
      ],
      doesNotApplyWhen: [
        "The creature merely scouts, spies, or infiltrates without a broader atmosphere of surveillance and distrust.",
        "The stronger fit is occult_conspiracy or disguised_pretender because hidden coordination or impersonation matters more than the watched feeling.",
      ],
      adjacentTags: ["occult_conspiracy", "disguised_pretender"],
    },
  }),
  defineConceptProjections("parasite_ridden", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by hosting burrowing larvae, parasitic colonies, implanted broods, or other invasive life within the body.",
      adjacentTags: ["spawn_creator", "plaguebearing"],
    },
  }),
  defineConceptProjections("phantom_assailants", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that manifests ghostly, spectral, or phantom attackers as separate assailants.",
    },
  }),
  defineConceptProjections("plaguebearing", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by carrying, spreading, or embodying pestilence, fever, or outbreak-causing corruption.",
      adjacentTags: ["disease_vector", "parasite_ridden"],
    },
  }),
  defineConceptProjections("possession_haunt", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that enters, rides, or briefly controls a victim rather than only attacking them externally.",
    },
  }),
  defineConceptProjections("predatory_seduction", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with deliberate luring, honey-trap menace, erotic predation, or invitation used explicitly as a hunting tactic.",
      appliesWhen: [
        "The creature's presentation centers on luring prey through desire, intimacy, or false safety before the attack or betrayal lands.",
        "Use when the hunting or consuming dynamic matters more than general temptation or glamour.",
      ],
      doesNotApplyWhen: [
        "The creature is alluring or corruptive without a strong predator-lure structure.",
        "The stronger fit is seductive_temptation because dangerous attraction matters more than an explicit hunt pattern.",
      ],
      adjacentTags: ["seductive_temptation", "disguised_pretender"],
    },
  }),
  defineConceptProjections("prophecy_omen", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with foretelling, omen-bearing, apocalyptic signs, destiny, or a creature's role as a herald of what is to come.",
      appliesWhen: [
        "Portents, prophecy, omen-reading, or the creature's arrival as a sign of coming change is a central retrieval hook.",
        "A GM would plausibly retrieve the creature for foretold doom, chosen destiny, or fate-haunted story beats.",
      ],
      doesNotApplyWhen: [
        "The creature merely predicts events, has divination magic, or is important to the plot without a strong omen-facing presentation.",
        "The stronger fit is apocalypse_ruin or ancestral_legacy because destiny-sign imagery is not the main presentation hook.",
      ],
      adjacentTags: ["apocalypse_ruin", "ancestral_legacy"],
    },
  }),
  defineConceptProjections("replayed_tragedy", {
    hazard: {
      axis: "haunt",
      family: "haunt_manifestation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Haunt that re-enacts a murder, betrayal, execution, disaster, or other fixed traumatic event.",
      appliesWhen: [
        "The haunt is naturally retrieved because it replays a specific past calamity, crime, execution, or emotional flashpoint as its core manifestation.",
        "The narrative repetition of an old event matters more than generic life drain, possession, or battlefield disruption.",
      ],
      doesNotApplyWhen: [
        "The haunt is only sad, angry, or spiritually active without reenacting a fixed historical scene.",
        "The stronger fit is judgment_haunt or lure_compulsion because the recurring tragedy itself is not the central hook.",
      ],
      adjacentTags: ["judgment_haunt", "appeasement_countermeasure"],
    },
  }),
  defineConceptProjections("revelry_excess", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with feasts, drunken revels, riotous celebration, gluttony, or ecstatic overindulgence.",
      appliesWhen: [
        "The creature is naturally retrieved for bacchanals, cursed feasts, debauched parties, or scenes of ecstatic excess.",
        "Celebration curdling into danger is part of the creature's recurring narrative identity.",
      ],
      doesNotApplyWhen: [
        "The creature only appears near taverns or festivals without excess, indulgence, or revelry being central.",
        "The stronger fit is carnival_show or seductive_temptation rather than feast-and-excess presentation.",
      ],
      adjacentTags: ["carnival_show", "seductive_temptation"],
    },
  }),
  defineConceptProjections("ritual_ceremony", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with rites, sacrifices, processions, ceremonial observance, or cultic staging as a scene identity.",
      appliesWhen: [
        "Ceremonial staging, sacrificial ritual, processional presentation, or formal observance is a major retrieval hook.",
        "The creature is naturally used in scenes defined by rites, altars, chants, offerings, or public ceremony.",
      ],
      doesNotApplyWhen: [
        "The creature merely has ritual magic or divine powers without a strong ceremonial scene identity.",
        "Temple placement or religious office alone is better captured by temple_setting or religious_npc.",
      ],
      adjacentTags: ["religious_npc", "ritualist_creature"],
    },
  }),
  defineConceptProjections("seasonal_festival", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with solstice rites, harvest festivals, holiday customs, masked processions, or recurring calendar-bound celebration.",
      appliesWhen: [
        "The creature is naturally retrieved for midsummer revels, winter rites, harvest pageants, or holiday scenes defined by recurring festal tradition.",
        "Calendar-bound custom or festival atmosphere is a central part of the creature's presentation rather than incidental backdrop.",
      ],
      doesNotApplyWhen: [
        "The creature merely appears during a feast, fair, or celebration without a recurring seasonal or ritual festival identity.",
        "The stronger fit is revelry_excess, carnival_show, or folk_horror because indulgence, spectacle, or folklore carries the retrieval weight.",
      ],
      adjacentTags: ["revelry_excess", "folk_horror"],
    },
  }),
  defineConceptProjections("seductive_temptation", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description:
        "Strongly associated with allure, seduction, dangerous invitation, or temptation into vice, doom, or compromise.",
      appliesWhen: [
        "The creature's presentation centers on allure, invitation, enchantment through desire, or baiting victims into a doomed choice.",
        "Retrieval value comes from temptation or dangerous attraction, not just social interaction or mechanical charm effects.",
      ],
      doesNotApplyWhen: [
        "The creature is merely attractive, charismatic, or capable of charm without temptation being a real story hook.",
        "The stronger fit is disguised_pretender or courtly_pageantry rather than luring desire.",
      ],
      adjacentTags: ["disguised_pretender", "courtly_pageantry"],
    },
  }),
  defineConceptProjections("stitched_horror", {
    creature: {
      axis: "presentation",
      family: "visual_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with sutures, patchwork flesh, sewn bodies, or visibly assembled corpse craftsmanship.",
    },
  }),
  defineConceptProjections("trickster_mischief", {
    creature: {
      axis: "presentation",
      family: "genre_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with pranks, capricious humor, gleeful sabotage, or explicit trickster behavior.",
      appliesWhen: [
        "Pranks, baiting humor, whimsical menace, or deliberate trickster conduct are a central retrieval hook.",
        "The creature is naturally sought for playful-but-dangerous chaos rather than generic destruction or villainy.",
      ],
      doesNotApplyWhen: [
        "The creature is merely chaotic, unpredictable, or destructive without a prankster or mischief-facing identity.",
        "Whimsical presentation appears only as surface flavor and another presentation tag or combat role better explains why the creature is being retrieved.",
      ],
      adjacentTags: ["carnival_show", "disguised_pretender"],
    },
  }),
  defineConceptProjections("vengeful_tragedy", {
    creature: {
      axis: "presentation",
      family: "story_motif",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.EDITORIAL,
      description: "Strongly associated with betrayal, grief, injustice, or a sorrowful wrong returning as vengeance.",
      appliesWhen: [
        "The creature is framed around betrayal, mourning, loss, or an unresolved wrong curdling into vengeance.",
        "A GM would retrieve it for tragic revenants, wronged spirits, or revenge-driven story beats rather than generic hostility.",
      ],
      doesNotApplyWhen: [
        "The creature is simply angry, hostile, or undead without a strong tragic-injustice presentation.",
        "The stronger fit is funerary_mourning because grief and ritual loss matter more than vengeance.",
      ],
      adjacentTags: ["funerary_mourning", "disguised_pretender"],
    },
  }),
  defineConceptProjections("void_tainted", {
    creature: {
      axis: "specialization",
      family: "corruption_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creature strongly defined by void corruption, cosmic hollowness, or metaphysical pollution that feels alien, cold, or reality-thinning.",
      adjacentTags: ["cosmic_dread", "forbidden_knowledge"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];
