import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: equipment", () => {
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

    expect(deriveRecordTags({
      name: "Navigator's Fortune",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "When you drink this cordial, you roll twice on your next Sense Direction check and take the better result as a fortune effect.",
      traits: ["consumable", "magical"],
    })).toEqual(expect.arrayContaining(["beneficial", "fortune_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Fortune's Coin",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When activated, this coin becomes a powerful agent of fortune. If you fail a check or attack, flip the coin. On a lucky result, you get a 12 on the die instead.",
      traits: ["invested", "magical"],
    })).toContain("fortune_support");

    expect(deriveRecordTags({
      name: "Brooch of Inspiration",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When you Recall Knowledge with Lore, the brooch grants a sudden inspiration. On this check, you roll twice and take the higher result.",
      traits: ["invested", "magical"],
    })).toContain("fortune_support");
  });

  it("derives access-system equipment tags", () => {
    expect(deriveRecordTags({
      name: "Spacious Pouch (Type I)",
      category: "equipment",
      subcategory: "backpack",
      descriptionText: "A spacious pouch opens into a magical space larger than its outside dimensions.",
      traits: ["extradimensional", "magical"],
    })).toEqual(expect.arrayContaining(["carry_support", "extradimensional_storage"]));

    expect(deriveRecordTags({
      name: "Gunner's Bandolier",
      category: "equipment",
      subcategory: "backpack",
      descriptionText: "This spacious bandolier can hold up to 4 one-handed crossbows or firearms, and you can draw the weapon after empowering it.",
      traits: ["invested", "magical"],
    })).toContain("weapon_staging");

    expect(deriveRecordTags({
      name: "Repeating Crossbow Magazine",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "",
      traits: ["consumable"],
    })).toContain("ammo_management");

    expect(deriveRecordTags({
      name: "Trail Pack",
      category: "equipment",
      subcategory: "backpack",
      descriptionText: "A roomy backpack for carrying supplies and weathering long wilderness travel.",
      traits: [],
    })).not.toContain("extradimensional_storage");

    expect(deriveRecordTags({
      name: "Cane Pistol",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "This fashionable cane's handle hides a dueling pistol fired through the painted cap at the bottom of the cane.",
      traits: [],
    })).not.toContain("weapon_staging");

    expect(deriveRecordTags({
      name: "Beacon Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated beacon shot hits a target, it embeds itself and spews sparks for 1 minute.",
      traits: ["ammunition"],
    })).not.toContain("ammo_management");
  });

  it("derives expanded gear-purpose and communication tags", () => {
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
      name: "Sunsteel Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "When raised, the shield sheds bright light and illuminates the area around you.",
      traits: ["magical"],
    })).toContain("illumination");

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
      name: "Alarm Snare",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "You create an alarm snare by rigging one or more noisy objects to a trip wire or pressure plate. When a Small or larger creature enters the square, the snare makes a noise loud enough that it can be heard by all creatures in the range you designated.",
      traits: ["auditory", "consumable", "mechanical", "snare", "trap"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Warning Snare",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Using materials specific to the area, you connect a sound-making component to a trip wire or a pressure plate. This snare is like an alarm snare, but its subtle sound blends into ambient noise.",
      traits: ["auditory", "consumable", "mechanical", "snare", "trap"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Sentry Fulu",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "A sentry fulu depicts an armed guard. When you activate the fulu, it takes the shape of a Tiny humanoid guard made of paper and keeps watch over an area. If a creature enters the area without giving the password, the sentry creates either an audible or mental alarm.",
      traits: ["consumable", "fulu", "magical"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Floorbell",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When the amount of weight you specify is placed on the floorbell, it emits an ear-piercing wail clearly audible to a range of 150 feet. A floorbell can also ring an alarm if a weight you specify is removed from the floorbell.",
      traits: [],
    })).toContain("alarm");

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
      name: "Lawbringer's Lasso",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This enchanted lasso can be used to Grapple creatures up to 30 feet away and has an Escape DC of 18.",
      traits: ["lawful", "magical"],
      references: [
        {
          recordKey: "actionspf2e:grapple-1",
          packName: "actionspf2e",
          name: "Grapple",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
      ],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Lawbringer's Lasso",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This enchanted lasso can be used to Grapple creatures up to 30 feet away and has an Escape DC of 18.",
      traits: ["lawful", "magical"],
      references: [
        {
          recordKey: "actionspf2e:grapple-1",
          packName: "actionspf2e",
          name: "Grapple",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
      ],
    })).not.toContain("restraint_escape");

    expect(deriveRecordTags({
      name: "Injigo's Loving Embrace",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Injigo's Loving Embrace functions as a typical net. You gain a +1 item bonus to Athletics checks to Grapple with the net. The creature must succeed at a DC 25 check to Escape the net.",
      traits: ["magical"],
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
      name: "Injigo's Loving Embrace",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Injigo's Loving Embrace functions as a typical net. You gain a +1 item bonus to Athletics checks to Grapple with the net. The creature must succeed at a DC 25 check to Escape the net.",
      traits: ["magical"],
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
    })).not.toContain("restraint_escape");

    expect(deriveRecordTags({
      name: "False Manacles",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "These manacles are nearly indistinguishable from real manacles upon inspection, but contain a hidden release.",
      traits: [],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Manacles of Persuasion",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When the manacles are locked around an immobilized creature's wrists, they begin to sap the life out of the victim.",
      traits: ["magical"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Titan's Grasp",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "You gain a +3 item bonus to Athletics checks to Grapple. If you successfully Grapple an enemy larger than you, the gauntlets dig into it.",
      traits: ["invested", "magical"],
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
    })).not.toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Bracers of Strength",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "You gain a +3 item bonus to Athletics checks and a +2 circumstance bonus to Athletics checks to Escape and Force Open.",
      traits: ["invested", "magical"],
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
    })).toContain("restraint_escape");

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

  it("derives armor-purpose tags for mobility, carry support, stealth, and survival", () => {
    expect(deriveRecordTags({
      name: "Dragonaut's Wingsuit",
      category: "equipment",
      subcategory: "armor",
      descriptionText: "This +1 resilient leather armor includes wing-like membranes that connect the limbs to one another. If you don't have a fly Speed, you gain a 20-foot fly Speed for Fly actions from this activation, and the armor helps you glide safely to earth.",
      traits: ["armor", "invested", "magical"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Crafting Leathers",
      category: "equipment",
      subcategory: "armor",
      descriptionText: "This simple leather armor is adorned with a series of pockets and pouches, all within easy reach. Each pocket or pouch contains a specific tool required for specialized crafting.",
      traits: ["armor"],
    })).toContain("carry_support");

    expect(deriveRecordTags({
      name: "Elven Chain (High-Grade)",
      category: "equipment",
      subcategory: "armor",
      descriptionText: "Created by elven artisans employing ancient crafting techniques, elven chain is exceptionally quiet. Unlike other chain shirts-even other dawnsilver chain shirts-elven chain does not have the noisy trait.",
      traits: ["armor"],
    })).toContain("stealth_support");

    expect(deriveRecordTags({
      name: "Deep Sea Plate",
      category: "equipment",
      subcategory: "armor",
      descriptionText: "The interior of this heavy, brass full plate is lined with waterproof fabric, especially covering the seams between plates. When worn, it provides a sealed environment that protects you from drowning as well as allowing you to move more freely while underwater. The armor enables you to breathe underwater and gives you a swim Speed equal to half your land Speed.",
      traits: ["armor", "invested", "magical"],
    })).toEqual(expect.arrayContaining(["mobility", "survival", "hazard_shielding"]));
  });

  it("derives concealment, scouting, and disguise support on gear and weapons", () => {
    expect(deriveRecordTags({
      name: "Bloodknuckles",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "These thin cloth wraps disguise themselves as simple bandages and take on the appearance of harmless wraps.",
      traits: ["magical"],
    })).toContain("disguise");

    expect(deriveRecordTags({
      name: "Bloodknuckles",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "These thin cloth wraps blend into the skin and are disguised as simple bandages.",
      traits: ["magical"],
    })).not.toContain("social_infiltration");

    expect(deriveRecordTags({
      name: "Batsbreath Cane",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "A resonant pulse through the cane lets you gain hearing as a precise sense while you hold it.",
      traits: ["magical"],
    })).toContain("scouting");

    expect(deriveRecordTags({
      name: "Blink Blade",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "The space you leave and the one you appear in are filled with puffs of smoke that make anyone within concealed until they leave the smoke.",
      traits: ["magical"],
    })).toContain("concealment");

    expect(deriveRecordTags({
      name: "Aeon Stone (Flickering)",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "The stone draws you slightly out of sync with the flow of time, causing you to flicker in and out of existence. You become concealed for 1 minute.",
      traits: ["magical"],
    })).toContain("concealment");

    expect(deriveRecordTags({
      name: "Quick-Change Outfit",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Two separate outfits sewn together let you switch quickly between the two outfits.",
      traits: [],
    })).not.toContain("concealment");

    expect(deriveRecordTags({
      name: "Cane Pistol",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "This fashionable cane's handle hides a dueling pistol fired through the thin, painted cap at the bottom of the cane.",
      traits: [],
    })).toContain("concealable");

    expect(deriveRecordTags({
      name: "False-Bottomed Mug",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This mug looks ordinary, but the bottom part unscrews to reveal a velvet-lined chamber for sneaking messages and small objects.",
      traits: [],
    })).toContain("concealable");
  });

  it("derives ammo payload, signaling, mobility, and restraint tags", () => {
    expect(deriveRecordTags({
      name: "Beacon Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "The shaft of a beacon shot is studded with tiny flecks of glimmering gemstones. When an activated beacon shot hits a target, it embeds itself into that target and spews sparks for 1 minute.",
      traits: ["ammunition"],
    })).toEqual(expect.arrayContaining(["illumination", "signaling"]));

    expect(deriveRecordTags({
      name: "Shining Ammunition",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "Shining ammunition gives off a faint glow. When shot, it sheds bright light in a 20-foot radius for 10 minutes.",
      traits: ["ammunition"],
    })).toContain("illumination");

    expect(deriveRecordTags({
      name: "Climbing Bolt",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "The shaft of this bolt is wrapped with fine twine. When the bolt strikes a solid surface, the twine unwinds and enlarges into a 50-foot-long rope, securely fastened to the surface the bolt struck.",
      traits: ["ammunition"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Burrowing Bolt",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "These arrows have tips grooved like a drill bit and angled fletching, causing them to spin quickly about their shaft when fired. When striking a structure or object of Hardness 14 or less within your first range increment, an activated burrowing bolt tunnels into the surface silently.",
      traits: ["ammunition"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Dimension Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "Dimension shot is deep blue black, but motes of light play upon it like stars in the night sky. The activated ammunition allows you to teleport to a location near where the ammunition hits.",
      traits: ["ammunition"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Antler Arrow",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated antler arrow hits a target, bone antlers extend to pin it down. The target must succeed at a Reflex save or become stuck to the surface.",
      traits: ["ammunition"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Bola Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated bola shot hits a target, it deals nonlethal bludgeoning damage. Critical Success The target falls prone and is stunned 1.",
      traits: ["ammunition"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Garrote Bolt",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When a garrote bolt hits a target, it transforms into a silver garrote and wraps around one of the target's appendages.",
      traits: ["ammunition"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Garrote Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When the activated ammunition hits a target, it transforms into a silvery garrote that wraps around one of the target's appendages.",
      traits: ["ammunition"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Black Tendril Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When the activated ammunition hits a target, it exudes tendrils that encase the target until the target Escapes.",
      traits: ["ammunition"],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Bane Ammunition",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "Monster hunters favor bane ammunition that contains a capsule of reagents tailored to a particular type of creature.",
      traits: ["ammunition"],
    })).toContain("creature_bane");

    expect(deriveRecordTags({
      name: "Elemental Ammunition",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When activated, the reservoir of alchemical reagents in elemental ammunition atomizes on impact, dealing persistent acid damage to the target.",
      traits: ["ammunition"],
    })).toContain("elemental_payload");

    expect(deriveRecordTags({
      name: "Explosive Ammunition",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When activated explosive ammunition hits a target, the missile explodes in a burst, dealing fire damage to each creature in the area.",
      traits: ["ammunition"],
    })).toContain("explosive_payload");

    expect(deriveRecordTags({
      name: "Meteor Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When you fire an activated meteor shot, it explodes into a small swarm of meteors as it reaches its target.",
      traits: ["ammunition"],
    })).toContain("explosive_payload");

    expect(deriveRecordTags({
      name: "Freeze Ammunition",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "Freeze ammunition carries chilling reagents that activate on contact with the target. A creature hit by activated freeze ammunition takes cold damage instead of the weapon's normal damage type, plus 2 cold splash damage.",
      traits: ["ammunition"],
    })).toContain("elemental_payload");

    expect(deriveRecordTags({
      name: "Depth Charge I",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "This ammunition explodes if it hits a target underwater, dealing bludgeoning damage in a burst to all creatures nearby.",
      traits: ["ammunition"],
    })).toContain("explosive_payload");

    expect(deriveRecordTags({
      name: "Disintegration Bolt",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated disintegration bolt hits a target, it is subject to a disintegrate spell requiring a Fortitude save.",
      traits: ["ammunition"],
    })).toContain("spell_payload");

    expect(deriveRecordTags({
      name: "Viper Arrow",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "After an activated viper arrow hits a target, the arrow transforms into a viper and the target is affected by the viper's poison.",
      traits: ["ammunition"],
    })).not.toContain("spell_payload");
  });

  it("derives hostile equipment impact tags for ammo and consumables", () => {
    expect(deriveRecordTags({
      name: "Glue Bullet",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated glue bullet hits a target, the creature becomes immobilized and stuck to the surface until it Escapes.",
      traits: ["ammunition"],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Blindpepper Bolt",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "When an activated blindpepper bolt hits, the creature must attempt a save or become blinded by the caustic pepper cloud.",
      traits: ["ammunition"],
    })).toContain("sensory_impairment");

    expect(deriveRecordTags({
      name: "Mindlock Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "A creature struck by this shot becomes frightened 2 and stupefied 1 as panic grips its mind.",
      traits: ["ammunition"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Slumber Wine",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This ingested poison leaves the target drowsy before it falls asleep and becomes unconscious.",
      traits: ["consumable", "poison"],
    })).toEqual(expect.arrayContaining(["offensive", "ingested_offense", "sedation"]));

    expect(deriveRecordTags({
      name: "Debilitating Draught",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "A creature that drinks the draught must attempt a Fortitude save. On a failed save, the target becomes sickened 2 and fatigued.",
      traits: ["consumable"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Dreamer's Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This restorative tonic helps recover from mental conditions and steady the emotions.",
      traits: ["alchemical", "consumable"],
    })).not.toContain("mental_impairment");
  });

  it("derives expedition tags for mounted support, sustenance, and aquatic support", () => {
    expect(deriveRecordTags({
      name: "War Saddle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Each war saddle is specifically fitted to a mount's body type and has numerous straps that can secure you on your mount.",
      traits: [],
    })).toContain("mounted_support");

    expect(deriveRecordTags({
      name: "Cavalry Commander's Lance",
      category: "equipment",
      subcategory: "weapon",
      descriptionText: "When mounted and wielding a cavalry commander's lance, you can direct allies with the pennant while charging.",
      traits: ["magical"],
    })).toContain("mounted_support");

    expect(deriveRecordTags({
      name: "Portable Weapon Mount",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This tripod can be mounted on a wall bracket to stabilize a firearm in defensive positions.",
      traits: [],
    })).not.toContain("mounted_support");

    expect(deriveRecordTags({
      name: "Ration Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Drinking a ration tonic magically nourishes you with the equivalent of a day's worth of food and water.",
      traits: ["consumable", "magical", "potion"],
    })).toContain("sustenance");

    expect(deriveRecordTags({
      name: "Prankster's Perpetual Pieplate",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Although the simple pies that fill this plate every minute are edible, they don't last long enough to sate hunger or provide any real nutritive value.",
      traits: ["magical"],
    })).not.toContain("sustenance");

    expect(deriveRecordTags({
      name: "Sailor's Collar",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "It can even save your life if you fall overboard, and while wearing it you gain a swim Speed.",
      traits: ["invested", "magical"],
    })).toContain("aquatic_support");

    expect(deriveRecordTags({
      name: "Diving Harness",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This reinforced diving suit keeps you afloat and lets you breathe underwater during long underwater travel.",
      traits: ["magical"],
    })).toContain("aquatic_support");

    expect(deriveRecordTags({
      name: "Feather Token (Puddle)",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "When activated, the feather transforms into two gallons of fresh, clear water.",
      traits: ["consumable", "magical"],
    })).toContain("sustenance");

    expect(deriveRecordTags({
      name: "Feather Token (Puddle)",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "When activated, the feather transforms into two gallons of fresh, clear water.",
      traits: ["consumable", "magical"],
    })).not.toContain("aquatic_support");
  });

  it("avoids equipment false positives while preserving shared tags", () => {
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
      name: "Conduit Shot",
      category: "equipment",
      subcategory: "ammo",
      descriptionText: "Fine lines of djezet sparkle in conduit shot. When you Activate it, you name up to four creatures, in addition to you, that the ammunition's magic works for.",
      traits: ["ammunition"],
    })).not.toEqual(expect.arrayContaining(["illumination", "signaling", "mobility", "restraint_capture"]));

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
      name: "Glorious Plate",
      category: "equipment",
      subcategory: "armor",
      descriptionText: "This elegant full plate sheds bright light in a 10-foot radius, which enemies see as a blinding halo that obscures creatures other than the wearer.",
      traits: ["armor", "magical"],
    })).not.toEqual(expect.arrayContaining(["mobility", "carry_support", "stealth_support", "survival"]));

    expect(deriveRecordTags({
      name: "Net Launcher",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This wide tube fires an unattached net at much greater range than one can be thrown. A net fired with a net launcher can target a Medium or smaller creature within 40 feet.",
      traits: [],
    })).not.toContain("restraint_capture");

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
      name: "Practice Target",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "These sturdy paper targets are excellent for tracking a gunslinger's progress over time and keeping score.",
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
      name: "Eye of the Moonwarden",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This pendant warns the wearer of danger. When a hostile creature comes within 30 feet of you, the stone glows with moonlight only you can see.",
      traits: ["invested", "magical"],
    })).not.toContain("alarm");

    expect(deriveRecordTags({
      name: "Signal Whistle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When sounded, a signal whistle can be heard clearly up to half a mile away across open terrain.",
      traits: [],
    })).toContain("signaling");

    expect(deriveRecordTags({
      name: "Signal Whistle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "When sounded, a signal whistle can be heard clearly up to half a mile away across open terrain.",
      traits: [],
    })).not.toContain("alarm");

    expect(deriveRecordTags({
      name: "Flare Beacon",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "Flare beacons create an incredibly bright light for a brief period of time. They are often used to signal others to the beacon's location, to coordinate assaults, to request rescue, or for other similar reasons.",
      traits: ["consumable", "gadget"],
    })).toContain("signaling");

    expect(deriveRecordTags({
      name: "Communication Bangle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "While decorative, this serves a cover for the bangle's function as a message bearer. Messages can be coded into the band and read later by the intended recipient.",
      traits: [],
    })).toContain("message_delivery");

    expect(deriveRecordTags({
      name: "Countering Charm",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Spellcasters can cast spells into countering charms that they've invested. The spell's effect doesn't occur; the spell's power is instead stored within the charm.",
      traits: ["invested", "magical"],
    })).toContain("countermagic");

    expect(deriveRecordTags({
      name: "Countering Charm",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Spellcasters can cast spells into countering charms that they've invested. The spell's effect doesn't occur; the spell's power is instead stored within the charm.",
      traits: ["invested", "magical"],
    })).not.toContain("magic_protection");

    expect(deriveRecordTags({
      name: "Antimagic Oil",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This oil contains energy that repels nearly all types of magic. When you apply this oil to armor, the creature wearing the armor becomes immune to all spells, effects of magic items, and effects with the magical trait for 1 minute.",
      traits: ["consumable", "magical", "oil"],
    })).toEqual(expect.arrayContaining(["countermagic", "magic_protection"]));
  });

  it("derives shield defense tags", () => {
    expect(deriveRecordTags({
      name: "Amaranthine Pavise",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "When your allies have cover from the amaranthine pavise, the circumstance bonus they gain from cover to Reflex saves against area effects also applies to Fortitude and Will saves against demons' area effects.",
      traits: ["shield"],
    })).toContain("ally_cover");

    expect(deriveRecordTags({
      name: "Shining Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "While raised, a shining shield moves itself to provide cover for your allies. If you have this shield raised and would provide cover to an ally against a foe by standing between them, the cover you provide increases one step.",
      traits: ["shield"],
    })).toContain("ally_cover");

    expect(deriveRecordTags({
      name: "Arrow-Catching Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "Trigger A ranged weapon Strike targets a creature within 15 feet of you when you have this shield raised. The triggering Strike targets you instead of its normal target.",
      traits: ["shield"],
    })).toContain("projectile_defense");

    expect(deriveRecordTags({
      name: "Turnabout Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "Trigger A ranged Strike using ammunition such as arrows, bolts, or bullets misses you. The ammunition enters the shield and is redirected with the same force with which it was originally fired.",
      traits: ["shield"],
    })).toContain("projectile_defense");

    expect(deriveRecordTags({
      name: "Reflecting Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "This shield functions as a spellguard shield that can also reflect spells. Trigger You're targeted by a spell. The shield attempts to counteract the spell, with a counteract rank of 9th.",
      traits: ["shield"],
    })).toContain("countermagic");

    expect(deriveRecordTags({
      name: "Cursebreak Bulwark",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "While you have this shield raised, you gain a bonus against spells that target you, increasing against curses. It can destroy a curse before it can affect you.",
      traits: ["shield"],
    })).toContain("magic_protection");

    expect(deriveRecordTags({
      name: "Dragonslayer's Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "While raised, this shield grants its circumstance bonus to Reflex saves against area effects. The shield has resistance 10 against the damage type corresponding to the dragon breath of the dragon whose hide was used in its creation.",
      traits: ["shield"],
    })).toContain("hazard_shielding");

    expect(deriveRecordTags({
      name: "Tower Shield",
      category: "equipment",
      subcategory: "shield",
      descriptionText: "These massive shields can be used to provide cover to nearly the entire body. AC increases to +4 if you are using the Take Cover action.",
      traits: ["shield"],
    })).not.toContain("ally_cover");
  });
});
