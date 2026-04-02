import { describe, expect, it } from "vitest";

import { DERIVED_TAG_CATALOG, deriveRecordTags } from "../src/derived-tags.js";

describe("derived tag rules", () => {
  it("derives expanded consumable support and offense tags", () => {
    expect(deriveRecordTags({
      name: "Darkvision Elixir",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The drinker gains darkvision and a bonus to Perception for 1 hour.",
      traits: ["alchemical", "consumable", "elixir"],
    })).toEqual(expect.arrayContaining(["beneficial", "senses_support", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Dreamer's Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This restorative tonic helps recover from mental conditions and steady the emotions.",
      traits: ["alchemical", "consumable"],
    })).toEqual(expect.arrayContaining(["beneficial", "condition_support", "mental_recovery"]));

    expect(deriveRecordTags({
      name: "Fire Ward Elixir",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The drinker gains resistance to fire for 1 hour.",
      traits: ["alchemical", "consumable", "elixir"],
    })).toEqual(expect.arrayContaining(["beneficial", "energy_resistance", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Spider Venom",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This contact poison is smeared on a weapon and afflicts the target through skin contact.",
      traits: ["alchemical", "consumable", "poison"],
    })).toEqual(expect.arrayContaining(["offensive", "weapon_applied", "contact_offense"]));

    expect(deriveRecordTags({
      name: "Potion of Cold Resistance",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Drinking this thick, fortifying potion grants resistance 10 against cold damage for 1 hour.",
      traits: ["consumable", "magical", "potion"],
    })).toEqual(expect.arrayContaining(["beneficial", "energy_resistance", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Antivenom Potion",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This cloudy liquid helps protect against poisons. When you drink an antivenom potion, you can immediately attempt to end persistent poison damage.",
      traits: ["consumable", "magical", "potion"],
    })).toEqual(expect.arrayContaining(["beneficial", "anti_poison", "self_buff"]));

    expect(deriveRecordTags({
      name: "Escape Fulu",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Trigger You attempt to Escape. The escape fulu is a charm worn in case of kidnapping. When you activate this fulu, you gain a +2 status bonus to checks to Escape.",
      traits: ["consumable", "fulu", "magical", "talisman"],
      references: [
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
      ],
    })).toEqual(expect.arrayContaining(["beneficial", "escape_support", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Bloodhound Mask",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Once activated, the mask sharpens odors, giving you imprecise scent with a 60-foot range.",
      traits: ["alchemical", "consumable"],
    })).toEqual(expect.arrayContaining(["beneficial", "senses_support", "self_buff"]));
  });

  it("derives expanded gear-purpose tags", () => {
    expect(deriveRecordTags({
      name: "Disguise Kit",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "A costume and cosmetics kit used to create a false identity and pass as local nobility.",
      traits: [],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Trail Pack",
      category: "equipment",
      subcategory: "backpack",
      descriptionText: "A roomy backpack for carrying supplies and weathering long wilderness travel.",
      traits: [],
    })).toEqual(expect.arrayContaining(["carry_support", "survival"]));

    expect(deriveRecordTags({
      name: "Survey Lantern",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "A lantern with a compass hood used to illuminate ruins and track your heading underground.",
      traits: [],
    })).toEqual(expect.arrayContaining(["illumination", "navigation"]));

    expect(deriveRecordTags({
      name: "Navigator's Star",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "A star inked on the back of the hand keeps you on the right path. As you hold up your hand and align the star in view, you learn which direction you're facing.",
      traits: ["invested", "magical", "tattoo"],
    })).toContain("navigation");

    expect(deriveRecordTags({
      name: "Traveler's Fulu",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Trigger You attempt to Sense Direction. This fulu functions as if you have a compass, and your attempt to Sense Direction gains a better degree of success.",
      traits: ["consumable", "fulu", "magical", "talisman"],
      references: [
        {
          recordKey: "actionspf2e:sense-direction-1",
          packName: "actionspf2e",
          name: "Sense Direction",
          category: "rule",
          subcategory: "action",
          traits: ["exploration"],
        },
      ],
    })).toContain("navigation");

    expect(deriveRecordTags({
      name: "Navigator's Feather",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Trigger You would roll a Survival check to Sense Direction or Track. The feather lets you roll the check twice and later helps you continue to track the same creature.",
      traits: ["consumable", "magical", "talisman"],
      references: [
        {
          recordKey: "actionspf2e:sense-direction-1",
          packName: "actionspf2e",
          name: "Sense Direction",
          category: "rule",
          subcategory: "action",
          traits: ["exploration"],
        },
        {
          recordKey: "actionspf2e:track-1",
          packName: "actionspf2e",
          name: "Track",
          category: "rule",
          subcategory: "action",
          traits: ["exploration"],
        },
      ],
    })).toEqual(expect.arrayContaining(["navigation", "tracking"]));

    expect(deriveRecordTags({
      name: "Tracker's Stew",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The stew improves your ability to sense and follow tracks. You gain a +1 item bonus to Survival checks to Cover Tracks and Track.",
      traits: ["alchemical", "consumable"],
      references: [
        {
          recordKey: "actionspf2e:cover-tracks-1",
          packName: "actionspf2e",
          name: "Cover Tracks",
          category: "rule",
          subcategory: "action",
          traits: ["exploration"],
        },
        {
          recordKey: "actionspf2e:track-1",
          packName: "actionspf2e",
          name: "Track",
          category: "rule",
          subcategory: "action",
          traits: ["exploration"],
        },
      ],
    })).toContain("tracking");

    expect(deriveRecordTags({
      name: "Tracker's Goggles",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "While wearing these goggles, you gain a +1 bonus to Survival checks to Sense Direction and Track. If you fail a check to Track, you can try again after 30 minutes rather than an hour.",
      traits: ["invested", "magical"],
    })).toEqual(expect.arrayContaining(["navigation", "tracking"]));

    expect(deriveRecordTags({
      name: "Bloodhound Mask",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The mask sharpens odors. When you use Survival to Track a creature by its scent, the mask grants you a +1 item bonus to your Survival check. The GM sets the Survival DC based on the area's ability to hold scent rather than on visual clues.",
      traits: ["alchemical", "consumable"],
    })).toContain("tracking");

    expect(deriveRecordTags({
      name: "Bloodhound Mask",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The mask sharpens odors. When you use Survival to Track a creature by its scent, the mask grants you a +1 item bonus to your Survival check. The GM sets the Survival DC based on the area's ability to hold scent rather than on visual clues.",
      traits: ["alchemical", "consumable"],
    })).not.toContain("anti_tracking");

    expect(deriveRecordTags({
      name: "Tracking Tag",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "These tags are attached to wild animals to track their movements and identify individual creatures later.",
      traits: [],
    })).toContain("tracking");

    expect(deriveRecordTags({
      name: "Aroma Concealer",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This oily mix reduces and covers any ordinary odors. The bonus also applies to the DC to Track the creature by scent.",
      traits: ["alchemical", "consumable"],
    })).toContain("anti_tracking");

    expect(deriveRecordTags({
      name: "Aroma Concealer",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This oily mix reduces and covers any ordinary odors. The bonus also applies to the DC to Track the creature by scent.",
      traits: ["alchemical", "consumable"],
    })).not.toContain("tracking");

    expect(deriveRecordTags({
      name: "Trackless",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Trackless footwear grants you a +4 item bonus to the DC to track you and is favored by anyone fleeing pursuit.",
      traits: ["magical"],
    })).toContain("anti_tracking");

    expect(deriveRecordTags({
      name: "Ichthyosis Mutagen",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Any creature attempting to Track you in the next 24 hours gains a +4 circumstance bonus to their check.",
      traits: ["alchemical", "consumable", "mutagen"],
    })).not.toContain("tracking");

    expect(deriveRecordTags({
      name: "Ichthyosis Mutagen",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Any creature attempting to Track you in the next 24 hours gains a +4 circumstance bonus to their check.",
      traits: ["alchemical", "consumable", "mutagen"],
    })).not.toContain("anti_tracking");

    expect(deriveRecordTags({
      name: "Boots of Bounding",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "The springy soles of these sturdy leather boots give you a +5-foot item bonus to your Speed and a +2 item bonus to Athletics checks to High Jump and Long Jump.",
      traits: ["invested", "magical"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Boots of Bounding",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "The springy soles of these sturdy leather boots give you a +5-foot item bonus to your Speed and a +2 item bonus to Athletics checks to High Jump and Long Jump.",
      traits: ["invested", "magical"],
    })).not.toContain("climbing");

    expect(deriveRecordTags({
      name: "Boots of Free Running",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "These practical boots provide exceptional traction, with improved grip on surfaces you would traditionally have difficulty traversing.",
      traits: ["invested", "magical"],
      references: [
        {
          recordKey: "actionspf2e:balance-1",
          packName: "actionspf2e",
          name: "Balance",
          category: "rule",
          subcategory: "action",
          traits: [],
        },
        {
          recordKey: "actionspf2e:high-jump-1",
          packName: "actionspf2e",
          name: "High Jump",
          category: "rule",
          subcategory: "action",
          traits: [],
        },
        {
          recordKey: "actionspf2e:long-jump-1",
          packName: "actionspf2e",
          name: "Long Jump",
          category: "rule",
          subcategory: "action",
          traits: [],
        },
      ],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Masquerade Scarf",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "The scarf casts Illusory Disguise on you.",
      traits: [],
      references: [
        {
          recordKey: "spells-srd:i35dpZFI7jZcRoBo",
          packName: "spells-srd",
          name: "Illusory Disguise",
          category: "spell",
          subcategory: null,
          traits: ["illusion"],
        },
      ],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Quick-Change Outfit",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Two separate outfits sewn together let you switch quickly between the two outfits.",
      traits: [],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Swallow-Spike",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Your armor responds to your desire to break free of a creature grabbing you by growing spikes. Trigger You become Grabbed or Restrained.",
      traits: [],
      references: [
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "conditionitems:grabbed-1",
          packName: "conditionitems",
          name: "Grabbed",
          category: "rule",
          subcategory: "condition",
          traits: [],
        },
        {
          recordKey: "conditionitems:restrained-1",
          packName: "conditionitems",
          name: "Restrained",
          category: "rule",
          subcategory: "condition",
          traits: [],
        },
      ],
    })).toContain("restraint_escape");

    expect(deriveRecordTags({
      name: "Shacklebreaker",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This bracelet helps free someone from manacles. Whenever you roll a success to free someone from manacles, it counts as two successes.",
      traits: ["magical"],
    })).toContain("restraint_escape");

    expect(deriveRecordTags({
      name: "Catch Pole",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This sturdy pole has a rope attached to one end in a loop. You can pull the handle side of the rope to tighten the loop. Using this loop, you can Grapple without having a free hand.",
      traits: [],
      references: [
        {
          recordKey: "actionspf2e:grapple-1",
          packName: "actionspf2e",
          name: "Grapple",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
      ],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Potion of Disguise",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Upon imbibing this potion, you take on the appearance of a specific type of creature for hours.",
      traits: ["consumable", "magical", "polymorph", "potion"],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Emergency Disguise",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This ribbon helps you throw together an emergency disguise and pass as a different social station.",
      traits: ["consumable", "magical", "talisman"],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
  });

  it("derives expanded creature context tags without adding redundant composites", () => {
    expect(deriveRecordTags({
      name: "Graveyard Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "This cemetery guard patrols the crypts beneath the old city.",
      traits: [],
    })).toEqual(expect.arrayContaining(["graveyard", "underground", "urban", "profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Bog Wisp",
      category: "creature",
      subcategory: null,
      descriptionText: "A fey spirit that haunts marshy bogs and flooded mires.",
      traits: ["fey"],
    })).toEqual(expect.arrayContaining(["fey_threat", "swamp"]));

    expect(deriveRecordTags({
      name: "Icebound Mariner",
      category: "creature",
      subcategory: null,
      descriptionText: "A sailor raider from the frozen sea who prowls icy coasts and shipwrecks.",
      traits: [],
    })).toEqual(expect.arrayContaining(["nautical", "aquatic_context", "arctic"]));

    expect(deriveRecordTags({
      name: "Pelagic Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "A sleek predator built for sudden bursts of speed.",
      traits: ["aquatic", "beast"],
    })).toContain("aquatic_context");

    expect(deriveRecordTags({
      name: "Bog Prowler",
      category: "creature",
      subcategory: null,
      descriptionText: "An ambush hunter with a powerful bite.",
      traits: ["amphibious", "beast"],
    })).toContain("aquatic_context");

    expect(deriveRecordTags({
      name: "Wealthy Vigilante",
      category: "creature",
      subcategory: null,
      descriptionText: "By night, this member of the nobility dons a false identity to mete out extralegal justice.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Prophet",
      category: "creature",
      subcategory: null,
      descriptionText: "A wandering prophet shares divine dreams and advice with the faithful.",
      traits: ["human", "humanoid"],
    })).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));
  });

  it("uses glossary family evidence for obvious undead-family threat and blocker cases", () => {
    expect(deriveRecordTags({
      name: "Morlock Thrall",
      category: "creature",
      subcategory: null,
      descriptionText: "A thrall reshaped by a vampire master's curse.",
      traits: ["humanoid"],
      families: ["vampire"],
    })).toContain("undead_threat");

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_threat"]));

    expect(deriveRecordTags({
      name: "Manor Guard",
      category: "creature",
      subcategory: null,
      descriptionText: "A manor guard who patrols the estate grounds.",
      traits: ["human", "humanoid"],
      families: ["vampire"],
    })).not.toContain("scene_adjacent");

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).toEqual(expect.arrayContaining(["profession_npc", "undead_threat"]));

    expect(deriveRecordTags({
      name: "Mythic Courtier",
      category: "creature",
      subcategory: null,
      descriptionText: "A courtier sustained by impossible necromancy.",
      traits: ["humanoid"],
      families: ["mythic", "lich"],
    })).not.toContain("scene_adjacent");
  });

  it("avoids known substring false positives from the rebuilt corpus", () => {
    expect(deriveRecordTags({
      name: "Antidote (Lesser)",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "An antidote protects you against toxins. Upon drinking an antidote, you gain a +2 item bonus to Fortitude saving throws against poisons for 6 hours.",
      traits: ["alchemical", "consumable", "elixir", "healing"],
    })).toEqual(expect.arrayContaining(["beneficial", "anti_poison"]));
    expect(deriveRecordTags({
      name: "Antidote (Lesser)",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "An antidote protects you against toxins. Upon drinking an antidote, you gain a +2 item bonus to Fortitude saving throws against poisons for 6 hours.",
      traits: ["alchemical", "consumable", "elixir", "healing"],
    })).not.toEqual(expect.arrayContaining(["offensive", "thrown_offense"]));

    expect(deriveRecordTags({
      name: "Accuser Agent",
      category: "creature",
      subcategory: null,
      descriptionText: "Accuser agents might be high court advocates, official spymasters, or innocuous adjutants delivering important messages to magistrates, generals, officers, or mercenaries.",
      traits: ["human", "humanoid"],
    })).not.toContain("arctic");

    expect(deriveRecordTags({
      name: "Abandoned Zealot",
      category: "creature",
      subcategory: null,
      descriptionText: "Abandoned zealots arise from false faiths unknown to most worshippers.",
      traits: ["undead", "spirit"],
    })).not.toContain("nautical");

    expect(deriveRecordTags({
      name: "Adamantine Golem",
      category: "creature",
      subcategory: null,
      descriptionText: "Crafting an adamantine golem requires mounting a mining expedition while guardian suits stand watch.",
      traits: ["construct", "golem", "mindless"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Animated Armor",
      category: "creature",
      subcategory: null,
      descriptionText: "Animated armor serves as guardians and training partners in martial academies.",
      traits: ["construct", "mindless"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Tangle Cuffs",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "These cuffs tighten around the target. On a hit, the target becomes Restrained until it Escapes.",
      traits: [],
      references: [
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "conditionitems:restrained-1",
          packName: "conditionitems",
          name: "Restrained",
          category: "rule",
          subcategory: "condition",
          traits: [],
        },
      ],
    })).toEqual(expect.arrayContaining(["restraint_capture"]));

    expect(deriveRecordTags({
      name: "Tangle Cuffs",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "These cuffs tighten around the target. On a hit, the target becomes Restrained until it Escapes.",
      traits: [],
      references: [
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "conditionitems:restrained-1",
          packName: "conditionitems",
          name: "Restrained",
          category: "rule",
          subcategory: "condition",
          traits: [],
        },
      ],
    })).not.toContain("restraint_escape");

    expect(deriveRecordTags({
      name: "Orchestral Brooch",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Trigger You attempt a Performance check, but you have not rolled yet. This silver brooch reverberates lightly with the sound of music.",
      traits: ["consumable", "magical", "talisman"],
    })).not.toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Clockwork Dial",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This precise timepiece helps you accurately track time for coordinated plans and spell durations.",
      traits: [],
    })).not.toContain("tracking");

    expect(deriveRecordTags({
      name: "Anathema Fulu",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This fulu comes in four pieces, one placed in each cardinal direction around the target area.",
      traits: ["consumable", "fulu", "magical", "talisman"],
    })).not.toContain("navigation");

    expect(deriveRecordTags({
      name: "Vanth Guardian Flock",
      category: "creature",
      subcategory: null,
      descriptionText: "Vanth psychopomps are eternal guardians of the cycle of life and death.",
      traits: ["monitor", "psychopomp", "troop"],
    })).not.toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));
  });

  it("requires enough distinct evidence for weighted creature context tags", () => {
    expect(deriveRecordTags({
      name: "Harbor Watcher",
      category: "creature",
      subcategory: null,
      descriptionText: "A sentry posted near the harbor gates.",
      traits: [],
    })).not.toContain("nautical");

    expect(deriveRecordTags({
      name: "Harbor Mariner",
      category: "creature",
      subcategory: null,
      descriptionText: "A mariner who keeps watch over the harbor docks.",
      traits: [],
    })).toContain("nautical");
  });

  it("publishes a compact derived-tag catalog", () => {
    expect(DERIVED_TAG_CATALOG).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "equipment",
        subcategories: ["consumable"],
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_recovery", description: expect.any(String) }),
          expect.objectContaining({ value: "senses_support", description: expect.any(String) }),
          expect.objectContaining({ value: "energy_resistance", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "purpose",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "navigation", description: expect.any(String) }),
          expect.objectContaining({ value: "tracking", description: expect.any(String) }),
          expect.objectContaining({ value: "anti_tracking", description: expect.any(String) }),
          expect.objectContaining({ value: "carry_support", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_escape", description: expect.any(String) }),
          expect.objectContaining({ value: "restraint_capture", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "social_infiltration", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "context",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "swamp", description: expect.any(String) }),
          expect.objectContaining({ value: "underground", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard", description: expect.any(String) }),
        ]),
      }),
    ]));
  });
});
