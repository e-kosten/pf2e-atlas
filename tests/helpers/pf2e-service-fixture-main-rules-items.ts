import { writeFile } from "node:fs/promises";
import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";

export async function writeRulesAndItemsFixtureData(root: string, packRoot: string): Promise<void> {
  await writeJson(path.join(packRoot, "actions", "raise-a-shield.json"), {
    _id: "shield1",
    name: "Raise a Shield",
    type: "action",
    system: {
      description: {
        value: "<p>You protect yourself.</p>",
      },
      level: {
        value: 1,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["defensive"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "seek.json"), {
    _id: "Seek",
    name: "Seek",
    type: "action",
    system: {
      description: {
        value: "<p>You scan for things. Success can reveal @UUID[Compendium.pf2e.conditionitems.Item.Hidden]{Hidden} creatures.</p>",
      },
      level: {
        value: 1,
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["concentrate"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "refocus.json"), {
    _id: "action-refocus-1",
    name: "Refocus",
    type: "action",
    system: {
      description: {
        value: "<p>You spend 10 minutes restoring your magical connection.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["concentrate", "exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "reactive-strike.json"), {
    _id: "reactive1",
    name: "Reactive Strike",
    type: "action",
    system: {
      description: {
        value: "<p>You lash out when a foe drops their guard.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["fighter"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actions", "attack-of-opportunity.json"), {
    _id: "aoo1",
    name: "Attack of Opportunity",
    type: "action",
    system: {
      description: {
        value: "<p>You punish a nearby opening.</p>",
      },
      publication: {
        title: "Pathfinder Core Rulebook",
        remaster: false,
      },
      traits: {
        rarity: "common",
        value: ["fighter"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "impersonate.json"), {
    _id: "action-impersonate-1",
    name: "Impersonate",
    type: "action",
    system: {
      description: {
        value: "<p>You create a disguise to pass yourself off as someone else.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["skill"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "escape.json"), {
    _id: "action-escape-1",
    name: "Escape",
    type: "action",
    system: {
      description: {
        value: "<p>You attempt to break free from being @UUID[Compendium.pf2e.conditionitems.Item.Grabbed]{Grabbed} or @UUID[Compendium.pf2e.conditionitems.Item.Restrained]{Restrained}.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["attack"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "grapple.json"), {
    _id: "action-grapple-1",
    name: "Grapple",
    type: "action",
    system: {
      description: {
        value: "<p>You grab a creature and hold it in place.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["attack"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "balance.json"), {
    _id: "action-balance-1",
    name: "Balance",
    type: "action",
    system: {
      description: {
        value: "<p>You move across a narrow surface or uneven ground.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "high-jump.json"), {
    _id: "action-high-jump-1",
    name: "High Jump",
    type: "action",
    system: {
      description: {
        value: "<p>You leap high into the air.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "long-jump.json"), {
    _id: "action-long-jump-1",
    name: "Long Jump",
    type: "action",
    system: {
      description: {
        value: "<p>You leap forward in a long arc.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["move"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "sense-direction.json"), {
    _id: "action-sense-direction-1",
    name: "Sense Direction",
    type: "action",
    system: {
      description: {
        value: "<p>You use your surroundings to determine direction.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "track.json"), {
    _id: "action-track-1",
    name: "Track",
    type: "action",
    system: {
      description: {
        value: "<p>You follow the trail left by a creature or group.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "actionspf2e", "cover-tracks.json"), {
    _id: "action-cover-tracks-1",
    name: "Cover Tracks",
    type: "action",
    system: {
      description: {
        value: "<p>You obscure the trail you leave behind.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["exploration"],
      },
    },
  });

  await writeJson(path.join(packRoot, "classfeatures", "meditative-well.json"), {
    _id: "classfeature1",
    name: "Meditative Well",
    type: "feat",
    system: {
      description: {
        value: "<p>Your training deepens whenever you @UUID[Compendium.pf2e.actions.Item.Refocus]{Refocus}.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "blinded.json"), {
    _id: "Blinded",
    name: "Blinded",
    type: "condition",
    system: {
      description: {
        value: "<p>You cannot see. Blinded overrides @UUID[Compendium.pf2e.conditionitems.Item.Dazzled]. You can @UUID[Compendium.pf2e.actions.Item.Seek]{Seek} to find creatures.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "dazzled.json"), {
    _id: "Dazzled",
    name: "Dazzled",
    type: "condition",
    system: {
      description: {
        value: "<p>Bright lights and sparkles impede your vision.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "grabbed.json"), {
    _id: "Grabbed",
    name: "Grabbed",
    type: "condition",
    system: {
      description: {
        value: "<p>You are held in place and can use @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} to break free.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "restrained.json"), {
    _id: "Restrained",
    name: "Restrained",
    type: "condition",
    system: {
      description: {
        value: "<p>You are tightly bound and usually need to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "off-guard.json"), {
    _id: "Off-Guard",
    name: "Off-Guard",
    type: "condition",
    system: {
      description: {
        value: "<p>You take a penalty to AC.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "conditionitems", "flat-footed.json"), {
    _id: "Flat-Footed",
    name: "Flat-Footed",
    type: "condition",
    system: {
      description: {
        value: "<p>You take a penalty to AC.</p>",
      },
      publication: {
        title: "Pathfinder Core Rulebook",
        remaster: false,
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "spacious-pouch-type-i.json"), {
    _id: "pouch1",
    name: "Spacious Pouch (Type I)",
    type: "backpack",
    system: {
      description: {
        value: "<p>A roomy extradimensional pouch.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
      price: {
        value: {
          gp: 75,
        },
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-chest.json"), {
    _id: "miniChest1",
    name: "Marvelous Miniature (Chest)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny chest that unfolds to full size.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-ladder.json"), {
    _id: "miniLadder1",
    name: "Marvelous Miniature (Ladder)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny ladder that unfolds to full size.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "marvelous-miniature-boat.json"), {
    _id: "miniBoat1",
    name: "Marvelous Miniature (Boat)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A tiny boat that unfolds on the water.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "surging-serum-lesser.json"), {
    _id: "serum1",
    name: "Surging Serum (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A crackling restorative elixir.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "elixir"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "sightless-tincture.json"), {
    _id: "tincture1",
    name: "Sightless Tincture",
    type: "consumable",
    system: {
      description: {
        value: "<p>A toxin that clouds the victim's vision.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault (Remastered)",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "elixir-of-life-minor.json"), {
    _id: "elixirLife1",
    name: "Elixir of Life (Minor)",
    type: "consumable",
    system: {
      description: {
        value: "<p>A healing elixir that restores hit points.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "elixir", "healing"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "antidote-lesser.json"), {
    _id: "antidote1",
    name: "Antidote (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This antidote bolsters the drinker against poison.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "antiplague-lesser.json"), {
    _id: "antiplague1",
    name: "Antiplague (Lesser)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This antiplague helps ward off disease.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "bottled-catharsis-serenity.json"), {
    _id: "catharsis1",
    name: "Bottled Catharsis (Serenity)",
    type: "consumable",
    system: {
      description: {
        value: "<p>This bottled catharsis steadies the emotions and helps recover from mental conditions.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault (Remastered)",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "potion-cold-resistance-moderate.json"), {
    _id: "potion-cold-resistance-1",
    name: "Potion of Cold Resistance (Moderate)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Drinking this thick, fortifying potion grants resistance 10 against cold damage for 1 hour.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "potion"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "bloodhound-mask-greater.json"), {
    _id: "bloodhound-mask-1",
    name: "Bloodhound Mask (Greater)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Once activated, the mask sharpens odors, giving you imprecise scent with a 60-foot range.</p><p>When you use Survival to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} a creature by its scent, the mask grants you a +3 item bonus to your Survival check.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "aroma-concealer.json"), {
    _id: "aroma-concealer-1",
    name: "Aroma Concealer",
    type: "consumable",
    system: {
      description: {
        value: "<p>This oily mix can be applied to a creature to reduce and cover any ordinary odors they produce. The creature receives a +2 item bonus to Stealth checks to @UUID[Compendium.pf2e.actionspf2e.Item.Hide]{Hide} or @UUID[Compendium.pf2e.actionspf2e.Item.Sneak]{Sneak} against creatures using primarily smell. This bonus also applies to the DC to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} the creature by scent.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "ichthyosis-mutagen.json"), {
    _id: "ichthyosis-mutagen-1",
    name: "Ichthyosis Mutagen",
    type: "consumable",
    system: {
      description: {
        value: "<p>Benefit You gain fast healing 2.</p><p>Drawback Any creature attempting to @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track} you in the next 24 hours gains a +4 circumstance bonus to their check.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable", "mutagen"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "escape-fulu.json"), {
    _id: "escape-fulu-1",
    name: "Escape Fulu",
    type: "consumable",
    system: {
      description: {
        value: "<p>Trigger You attempt to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p><p>The escape fulu is a charm worn in case of kidnapping. When you activate this fulu, you gain a +2 status bonus to checks to Escape for 1 minute.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Tian Xia Character Guide",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "fulu", "magical", "talisman"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "travelers-fulu.json"), {
    _id: "travelers-fulu-1",
    name: "Traveler's Fulu",
    type: "consumable",
    system: {
      description: {
        value: "<p>Trigger You attempt to @UUID[Compendium.pf2e.actionspf2e.Item.Sense Direction]{Sense Direction}.</p><p>This fulu shows constellations and arrows across the night sky. Your attempt to Sense Direction functions as if you have a compass, and you use the outcome one degree of success better than the result of your Survival check.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Tian Xia Character Guide",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "fulu", "magical", "talisman"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "trackers-stew.json"), {
    _id: "trackers-stew-1",
    name: "Tracker's Stew",
    type: "consumable",
    system: {
      description: {
        value: "<p>Once you've eaten the stew, it improves your ability to sense and follow tracks for 24 hours. You gain a +1 item bonus to Survival checks to @UUID[Compendium.pf2e.actionspf2e.Item.Cover Tracks]{Cover Tracks} and @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "consumable"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "ration-tonic.json"), {
    _id: "ration-tonic-1",
    name: "Ration Tonic",
    type: "consumable",
    system: {
      description: {
        value: "<p>This slender vial appears to hold clean, clear water with a faintly fruity scent. Drinking a ration tonic magically nourishes you with the equivalent of a day's worth of food and water.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "potion"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "potion-of-disguise-moderate.json"), {
    _id: "potion-disguise-1",
    name: "Potion of Disguise (Moderate)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Upon imbibing this potion, you take on the appearance of a specific type of creature for 2d12 hours.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "polymorph", "potion"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "climbing-kit.json"), {
    _id: "climbKit1",
    name: "Climbing Kit",
    type: "equipment",
    system: {
      description: {
        value: "<p>A compact climbing kit with rope and pitons for climbing sheer walls and rappelling safely.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "boots-of-free-running-greater.json"), {
    _id: "boots-free-running-1",
    name: "Boots of Free Running (Greater)",
    type: "equipment",
    system: {
      description: {
        value: "<p>These comfortable and practical boots slip on easily and fill you with boundless energy. The treads of these boots provide exceptional traction, with improved grip on surfaces you would traditionally have difficulty traversing. While wearing the boots, you gain a +3 item bonus to Acrobatics checks to @UUID[Compendium.pf2e.actionspf2e.Item.Balance]{Balance} and to Athletics checks to @UUID[Compendium.pf2e.actionspf2e.Item.High Jump]{High Jump} and @UUID[Compendium.pf2e.actionspf2e.Item.Long Jump]{Long Jump}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "concealable-thieves-tools.json"), {
    _id: "concealTools1",
    name: "Concealable Thieves' Tools",
    type: "equipment",
    system: {
      description: {
        value: "<p>Slim lockpicks and hidden tools for bypassing locks without drawing attention.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "war-saddle.json"), {
    _id: "war-saddle-1",
    name: "War Saddle",
    type: "equipment",
    system: {
      description: {
        value: "<p>Each war saddle is specifically fitted to a mount's body type and has numerous straps that can secure you on your mount.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "sailors-collar.json"), {
    _id: "sailors-collar-1",
    name: "Sailor's Collar",
    type: "equipment",
    system: {
      description: {
        value: "<p>Veteran sailors like to wear this jaunty blue collar. It can even save your life if you fall overboard, and while wearing it you gain a swim Speed.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "ring-of-sustenance.json"), {
    _id: "ring-sustenance-1",
    name: "Ring of Sustenance",
    type: "equipment",
    system: {
      description: {
        value: "<p>This polished wooden ring constantly refreshes your body and mind. While wearing it, you need to eat and drink only once per week.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "trackers-goggles.json"), {
    _id: "trackers-goggles-1",
    name: "Tracker's Goggles",
    type: "equipment",
    system: {
      description: {
        value: "<p>While wearing these goggles, you gain a +1 bonus to Survival checks to @UUID[Compendium.pf2e.actionspf2e.Item.Sense Direction]{Sense Direction} and @UUID[Compendium.pf2e.actionspf2e.Item.Track]{Track}. If you fail a check to Track, you can try again after 30 minutes rather than an hour.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "tracking-tag.json"), {
    _id: "tracking-tag-1",
    name: "Tracking Tag",
    type: "equipment",
    system: {
      description: {
        value: "<p>Tracking tags are attached to wild animals to track their movements and identify individual creatures among a herd.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "trackless.json"), {
    _id: "trackless-1",
    name: "Trackless",
    type: "equipment",
    system: {
      description: {
        value: "<p>Trackless footwear is favored by anyone fleeing pursuit. While wearing it, you gain a +4 item bonus to the DC to track you.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "shadow-shroud.json"), {
    _id: "shadow-shroud-1",
    name: "Shadow Shroud",
    type: "armor",
    system: {
      description: {
        value: "<p>This cloak-like armor muffles stray noise, remains exceptionally quiet, and helps you avoid notice in dim corridors.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "alarm-snare.json"), {
    _id: "equip-alarm-snare-1",
    name: "Alarm Snare",
    type: "consumable",
    system: {
      description: {
        value: "<p>You create an alarm snare by rigging one or more noisy objects to a trip wire or pressure plate.</p><p>When a Small or larger creature enters the square, the snare makes a noise loud enough that it can be heard by all creatures in the range you designated.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["auditory", "consumable", "mechanical", "snare", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "warning-snare.json"), {
    _id: "equip-warning-snare-1",
    name: "Warning Snare",
    type: "consumable",
    system: {
      description: {
        value: "<p>Using materials specific to the area, you connect a sound-making component to a trip wire or a pressure plate.</p><p>This snare is like an alarm snare, but its subtle sound blends into ambient noise.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: ["auditory", "consumable", "mechanical", "snare", "trap"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "sentry-fulu.json"), {
    _id: "equip-sentry-fulu-1",
    name: "Sentry Fulu",
    type: "consumable",
    system: {
      description: {
        value: "<p>A sentry fulu depicts an armed guard.</p><p>When you activate the fulu, it keeps watch over an area in a 20-foot burst. If a creature enters the area without giving the password, the sentry creates either an audible or mental alarm.</p>",
      },
      publication: {
        title: "Pathfinder Tian Xia Character Guide",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "fulu", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "floorbell.json"), {
    _id: "equip-floorbell-1",
    name: "Floorbell",
    type: "equipment",
    system: {
      description: {
        value: "<p>This sturdy tile resembles a pressure plate.</p><p>When the amount of weight you specify is placed on the floorbell, it emits an ear-piercing wail. A floorbell can also ring an alarm if a weight you specify is removed from it.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "flare-beacon-moderate.json"), {
    _id: "equip-flare-beacon-1",
    name: "Flare Beacon (Moderate)",
    type: "consumable",
    system: {
      description: {
        value: "<p>Flare beacons create an incredibly bright light for a brief period of time.</p><p>They are often used to signal others to the beacon's location, to coordinate assaults, to request rescue, or for other similar reasons.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "gadget"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "beacon-shot.json"), {
    _id: "ammo-beacon-shot-1",
    name: "Beacon Shot",
    type: "ammo",
    system: {
      description: {
        value: "<p>The shaft of a beacon shot is studded with tiny flecks of glimmering gemstones.</p><p>When an activated beacon shot hits a target, it embeds itself into that target and spews sparks for 1 minute.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "elemental-ammunition.json"), {
    _id: "ammo-elemental-ammunition-1",
    name: "Elemental Ammunition",
    type: "ammo",
    system: {
      description: {
        value: "<p>When activated, the reservoir of alchemical reagents in elemental ammunition atomizes on impact, dealing persistent acid damage to the target.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "bola-shot.json"), {
    _id: "ammo-bola-shot-1",
    name: "Bola Shot",
    type: "ammo",
    system: {
      description: {
        value: "<p>When an activated bola shot hits a target, it deals nonlethal bludgeoning damage.</p><p>Critical Success The target falls prone and is stunned 1.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "glue-bullet.json"), {
    _id: "ammo-glue-bullet-1",
    name: "Glue Bullet",
    type: "ammo",
    system: {
      description: {
        value: "<p>When an activated glue bullet hits a target, the creature becomes immobilized and stuck to the surface until it @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}s.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "blindpepper-bolt.json"), {
    _id: "ammo-blindpepper-bolt-1",
    name: "Blindpepper Bolt",
    type: "ammo",
    system: {
      description: {
        value: "<p>When an activated blindpepper bolt hits a target, the creature must attempt a save or become blinded by the caustic pepper cloud.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "mindlock-shot.json"), {
    _id: "ammo-mindlock-shot-1",
    name: "Mindlock Shot",
    type: "ammo",
    system: {
      description: {
        value: "<p>A creature struck by this shot becomes frightened 2 and stupefied 1 as panic grips its mind.</p>",
      },
      publication: {
        title: "Pathfinder Guns & Gears",
      },
      traits: {
        rarity: "common",
        value: ["ammunition"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "slumber-wine.json"), {
    _id: "consumable-slumber-wine-1",
    name: "Slumber Wine",
    type: "consumable",
    system: {
      description: {
        value: "<p>This ingested poison leaves the target drowsy before it falls asleep and becomes unconscious.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "poison"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "countering-charm.json"), {
    _id: "equip-countering-charm-1",
    name: "Countering Charm",
    type: "equipment",
    system: {
      description: {
        value: "<p>Spellcasters can cast spells into countering charms that they've invested or that are invested by a willing creature.</p><p>The spell's effect doesn't occur; the spell's power is instead stored within the charm.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["invested", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "antimagic-oil.json"), {
    _id: "equip-antimagic-oil-1",
    name: "Antimagic Oil",
    type: "consumable",
    system: {
      description: {
        value: "<p>This oil contains energy that repels nearly all types of magic.</p><p>When you apply this oil to armor, the creature wearing the armor becomes immune to all spells, effects of magic items, and effects with the magical trait for 1 minute.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["consumable", "magical", "oil"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "disguise-kit.json"), {
    _id: "equip-disguise-kit-1",
    name: "Disguise Kit",
    type: "equipment",
    system: {
      description: {
        value: "<p>You usually need a disguise kit to set up a disguise in order to @UUID[Compendium.pf2e.actionspf2e.Item.Impersonate]{Impersonate} someone.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "quick-change-outfit.json"), {
    _id: "equip-quick-change-1",
    name: "Quick-Change Outfit",
    type: "equipment",
    system: {
      description: {
        value: "<p>A quick-change outfit is in fact two separate outfits sewn together, allowing you to switch quickly between the two outfits.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Firebrands",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "masquerade-scarf.json"), {
    _id: "equip-masquerade-scarf-1",
    name: "Masquerade Scarf",
    type: "equipment",
    system: {
      description: {
        value: "<p>You arrange the scarf over your lower face, and it casts @UUID[Compendium.pf2e.spells-srd.Item.Illusory Disguise]{Illusory Disguise} on you.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "swallow-spike.json"), {
    _id: "equip-swallow-spike-1",
    name: "Swallow-Spike",
    type: "equipment",
    system: {
      description: {
        value: "<p>Your armor responds to your desire to break free of a creature grabbing you by growing spikes.</p><p>Trigger You become @UUID[Compendium.pf2e.conditionitems.Item.Grabbed]{Grabbed}, @UUID[Compendium.pf2e.conditionitems.Item.Restrained]{Restrained}, or another immobilizing effect that would hold you until you @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape}.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "shacklebreaker.json"), {
    _id: "equip-shacklebreaker-1",
    name: "Shacklebreaker",
    type: "equipment",
    system: {
      description: {
        value: "<p>This bracelet has three charms depicting a dagger, a shield, and a rose. Whenever you roll a success to free someone from manacles, it counts as two successes.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "handcuffs-average.json"), {
    _id: "equip-handcuffs-average-1",
    name: "Handcuffs (Average)",
    type: "equipment",
    system: {
      description: {
        value: "<p>These handcuffs possess a ratcheting lock system in each cuff that allows them to be quickly cinched down on a captive's limbs, even if they're actively resisting.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "catch-pole.json"), {
    _id: "equip-catch-pole-1",
    name: "Catch Pole",
    type: "equipment",
    system: {
      description: {
        value: "<p>This sturdy pole has a rope attached to one end in a loop. You can pull the handle side of the rope to tighten the loop. Using this loop, you can @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} without having a free hand.</p>",
      },
      publication: {
        title: "Pathfinder GM Core",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "lawbringers-lasso.json"), {
    _id: "equip-lawbringers-lasso-1",
    name: "Lawbringer's Lasso",
    type: "equipment",
    system: {
      description: {
        value: "<p>This enchanted lasso is a @UUID[Compendium.pf2e.equipment-srd.Item.Net]{Net} that can be used to @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} creatures up to 30 feet away and has an @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} DC of 18.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Knights of Lastwall",
      },
      traits: {
        rarity: "common",
        value: ["lawful", "magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "injigos-loving-embrace.json"), {
    _id: "equip-injigos-loving-embrace-1",
    name: "Injigo's Loving Embrace",
    type: "equipment",
    system: {
      description: {
        value: "<p>Injigo's Loving Embrace functions as a typical net. You gain a +1 item bonus to Athletics checks to @UUID[Compendium.pf2e.actionspf2e.Item.Grapple]{Grapple} with the net.</p><p>The creature must succeed at a DC 25 check to @UUID[Compendium.pf2e.actionspf2e.Item.Escape]{Escape} the net.</p>",
      },
      publication: {
        title: "Pathfinder Treasure Vault",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "false-manacles.json"), {
    _id: "equip-false-manacles-1",
    name: "False Manacles",
    type: "equipment",
    system: {
      description: {
        value: "<p>These manacles are nearly indistinguishable from real manacles upon inspection, but contain a hidden release that enables a wearer who knows the location of the release to free themselves with a single Interact action.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Impossible Lands",
      },
      traits: {
        rarity: "common",
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment-srd", "manacles-of-persuasion.json"), {
    _id: "equip-manacles-of-persuasion-1",
    name: "Manacles of Persuasion",
    type: "equipment",
    system: {
      description: {
        value: "<p>When the manacles are locked around an immobilized creature's wrists, they begin to sap the life out of the victim.</p>",
      },
      publication: {
        title: "Pathfinder Lost Omens Gods & Magic",
      },
      traits: {
        rarity: "common",
        value: ["magical"],
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "ghost-charge-prototype.json"), {
    _id: "weapon-bomb-1",
    name: "Ghost Charge Prototype",
    type: "weapon",
    system: {
      category: "martial",
      group: "bomb",
      damage: {
        damageType: "positive",
      },
      description: {
        value: "<p>A prototype bomb that bursts with cleansing light.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["alchemical", "bomb"],
      },
      usage: {
        value: "held-in-one-hand",
      },
    },
  });

  await writeJson(path.join(packRoot, "equipment", "practice-sword.json"), {
    _id: "weapon-sword-1",
    name: "Practice Sword",
    type: "weapon",
    system: {
      category: "martial",
      group: "sword",
      damage: {
        damageType: "slashing",
      },
      description: {
        value: "<p>A balanced practice blade for drilling sword forms.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        rarity: "common",
        value: ["martial"],
      },
      usage: {
        value: "held-in-one-hand",
      },
    },
  });

  await writeJson(path.join(packRoot, "heritages", "nephilim.json"), {
    _id: "neph1",
    name: "Nephilim",
    type: "heritage",
    system: {
      description: {
        value: "<p>You are touched by an outer plane.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["nephilim"],
      },
    },
  });

  await writeJson(path.join(packRoot, "heritages", "naari.json"), {
    _id: "naari1",
    name: "Naari",
    type: "heritage",
    system: {
      description: {
        value: "<p>You are touched by elemental fire.</p>",
      },
      publication: {
        title: "Pathfinder Player Core 2",
        remaster: true,
      },
      traits: {
        rarity: "common",
        value: ["naari"],
      },
    },
  });

  await writeJson(path.join(packRoot, "journals", "remaster-changes.json"), {
    _id: "journal1",
    name: "Remaster Changes",
    pages: [
      {
        _id: "page1",
        name: "Remaster Changes",
        type: "text",
        text: {
          content: `
            <ul>
              <li>Aasimar, Aphorite, Ganzi, and Tiefling are merged into @UUID[Compendium.pf2e.heritages.Item.neph1]{Nephilim}.</li>
              <li>Ifrit are now @UUID[Compendium.pf2e.heritages.Item.naari1]{Naari}.</li>
            </ul>
          `,
          format: 1,
        },
      },
      {
        _id: "page2",
        name: "Class Features",
        type: "text",
        text: {
          content: `
            <table><tbody>
              <tr><td>Attack of Opportunity</td><td>Multiple</td><td>Renamed</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr>
              <tr><td>Strike Back</td><td>Multiple</td><td>Replaced</td><td>@UUID[Compendium.pf2e.actions.Item.Reactive Strike]{Reactive Strike}</td></tr>
            </tbody></table>
          `,
          format: 1,
        },
      },
      {
        _id: "page3",
        name: "Equipment",
        type: "text",
        text: {
          content: `
            <table><tbody>
              <tr>
                <td>Feather Token (Chest, Ladder, Swan Boat)</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>
                  Marvelous Miniatures (
                  @UUID[Compendium.pf2e.equipment.Item.miniChest1]{Chest},
                  @UUID[Compendium.pf2e.equipment.Item.miniLadder1]{Ladder},
                  @UUID[Compendium.pf2e.equipment.Item.miniBoat1]{Boat}
                  )
                </td>
              </tr>
              <tr>
                <td>Bag of Holding</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>@UUID[Compendium.pf2e.equipment.Item.pouch1]{Spacious Pouch}</td>
              </tr>
              <tr>
                <td>Sight-Theft Grit</td>
                <td>Multiple</td>
                <td>Renamed</td>
                <td>@UUID[Compendium.pf2e.equipment.Item.serum1]{Sightless Tincture}</td>
              </tr>
            </tbody></table>
          `,
          format: 1,
        },
      },
    ],
  });

  await writeFile(
    path.join(root, "src", "module", "migration", "migrations", "850-flat-footed-to-off-guard.ts"),
    `/** Rename all uses and mentions of "flat-footed" to "off-guard" */\n`,
  );

  await writeJson(path.join(packRoot, "conditionitems", "hidden.json"), {
    _id: "Hidden",
    name: "Hidden",
    type: "condition",
    system: {
      description: {
        value: "<p>The observer knows your space but not your exact location.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });

  await writeJson(path.join(packRoot, "feats-srd", "deep-focus.json"), {
    _id: "feat1",
    name: "Deep Focus",
    type: "feat",
    system: {
      description: {
        value: "<p>Whenever you @UUID[Compendium.pf2e.actions.Item.Refocus]{Refocus}, your resolve sharpens.</p>",
      },
      publication: {
        title: "Pathfinder Player Core",
      },
      traits: {
        value: [],
      },
    },
  });
}
