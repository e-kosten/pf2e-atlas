import { describe, expect, it } from "vitest";

import { assignVariantFamilies } from "../../src/data/variant-families.js";
import type {
  BuildSourceEntry,
  NormalizedIndexRecord,
  PackBuildInfo,
} from "../../src/data/index-types.js";

function createRecord(input: {
  recordKey: string;
  name: string;
  category?: NormalizedIndexRecord["category"];
  subcategory?: NormalizedIndexRecord["subcategory"];
  packName?: string;
  sourcePath: string;
  descriptionText?: string | null;
}): NormalizedIndexRecord {
  return {
    recordKey: input.recordKey,
    id: input.recordKey,
    name: input.name,
    normalizedName: input.name.toLowerCase(),
    type: "equipment",
    category: input.category ?? "equipment",
    subcategory: input.subcategory ?? "gear",
    packName: input.packName ?? "equipment-srd",
    packLabel: "Equipment",
    documentType: "Item",
    level: null,
    rarity: "common",
    traits: [],
    derivedTags: [],
    publicationTitle: "Pathfinder Treasure Vault",
    publicationRemaster: true,
    descriptionText: input.descriptionText ?? null,
    hasDescription: Boolean(input.descriptionText),
    descriptionSnippet: input.descriptionText ?? null,
    sourceCategory: "rules",
    folderId: null,
    families: [],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: input.sourcePath,
    isUnique: false,
    size: null,
    itemCategory: "wand",
    priceCp: null,
    bulkValue: null,
    actionCost: null,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: [],
    spellKinds: [],
    saveType: null,
    areaType: null,
    rangeText: null,
    durationText: null,
    durationUnit: null,
    targetText: null,
    areaValue: null,
    sustained: false,
    basicSave: false,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    disableText: null,
    disableSkills: [],
    isComplex: false,
    actorMetrics: {},
    itemMetrics: {},
    rangeValue: null,
    searchText: input.name,
  };
}

function createEntry(input: {
  recordKey: string;
  name: string;
  sourcePath: string;
  descriptionText?: string | null;
}): BuildSourceEntry & { record: NormalizedIndexRecord } {
  const pack: PackBuildInfo = {
    name: "equipment-srd",
    label: "Equipment",
    documentType: "Item",
    declaredPath: "packs/pf2e/equipment",
    resolvedPath: "/tmp/equipment",
  };

  return {
    pack,
    filePath: input.sourcePath,
    raw: {
      _id: input.recordKey,
      name: input.name,
      type: "consumable",
      system: {},
    },
    record: createRecord({
      recordKey: input.recordKey,
      name: input.name,
      sourcePath: input.sourcePath,
      descriptionText: input.descriptionText,
    }),
    actorData: null,
    itemData: null,
    spellData: null,
    references: [],
    resolvedReferences: [],
  };
}

const HAG_EYE_SHARED_DESCRIPTION = [
  "This item appears to be an ordinary semiprecious stone and is typically mounted on a brooch or ring, but the stone is, in fact, an eyeball.",
  "The hag eye produces no direct benefit for the wearer, but allows the hag who created it, or any member of her coven, can peer through the eye using the Seek action.",
  "Any damage dealt to the eye destroys it. If this happens while a hag is looking through it, the hag is Blinded for 1 hour.",
].join("\n");

describe("variant family normalization", () => {
  it("groups hag eyes from shared suffix titles and opening descriptions", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:hag-eye",
        name: "Hag Eye",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/hag-eye.json",
        descriptionText: HAG_EYE_SHARED_DESCRIPTION,
      }),
      createEntry({
        recordKey: "equipment:smoky-hag-eye",
        name: "Smoky Hag Eye",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/smoky-hag-eye.json",
        descriptionText: `${HAG_EYE_SHARED_DESCRIPTION}\nA smoky hag eye is named after the black vapor that constantly emanates from it.`,
      }),
      createEntry({
        recordKey: "equipment:stony-hag-eye",
        name: "Stony Hag Eye",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/stony-hag-eye.json",
        descriptionText: `${HAG_EYE_SHARED_DESCRIPTION}\nA stony hag eye is fashioned from the eye of a creature with a petrifying gaze.`,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Hag Eye");
    expect(entries[0].record.variantLabel).toBeNull();
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantLabel).toBe("Smoky");
    expect(entries[2].record.variantLabel).toBe("Stony");
    expect(entries[1].record.variantSource).toBe("composite");
  });

  it("groups arbitrary parenthetical variants such as Wondrous Figurines from shared lead text", () => {
    const sharedLead = [
      "Each one of these statuettes is 1 inch in height, carved from a specific material and taking the shape of a particular animal or animals.",
      "Activate 2 command, Interact",
      "Effect You activate the statue by placing it on solid ground and then speaking its name, causing the statuette to transform into a living creature or creatures.",
    ].join("\n");
    const entries = [
      createEntry({
        recordKey: "equipment:figurine-bear",
        name: "Wondrous Figurine (Rubber Bear)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-rubber-bear.json",
        descriptionText: `${sharedLead}\nThe figurine becomes a bear-shaped construct.`,
      }),
      createEntry({
        recordKey: "equipment:figurine-lions",
        name: "Wondrous Figurine (Golden Lions)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-golden-lions.json",
        descriptionText: `${sharedLead}\nThe figurine becomes two golden lions.`,
      }),
      createEntry({
        recordKey: "equipment:figurine-dog",
        name: "Wondrous Figurine (Onyx Dog)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-onyx-dog.json",
        descriptionText: `${sharedLead}\nThe figurine becomes a vigilant dog.`,
      }),
      createEntry({
        recordKey: "equipment:figurine-serpent",
        name: "Wondrous Figurine (Jade Serpent)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-jade-serpent.json",
        descriptionText: `${sharedLead}\nThe figurine becomes a giant serpent.`,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Wondrous Figurine");
    expect(entries[1].record.variantBaseName).toBe("Wondrous Figurine");
    expect(entries[0].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[0].record.variantLabel).toBe("Rubber Bear");
    expect(entries[1].record.variantLabel).toBe("Golden Lions");
    expect(entries[0].record.variantSource).toBe("composite");
  });

  it("collapses stacked parenthetical variants into one family", () => {
    const sharedLead = [
      "Puzzle boxes are games of logic and tactile experimentation.",
      "A creature can spend 10 minutes attempting to solve the box.",
      "Opening the box reveals the compartment inside.",
    ].join("\n");
    const entries = [
      createEntry({
        recordKey: "equipment:puzzle-simple",
        name: "Puzzle Box (Simple)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/puzzle-box-simple.json",
        descriptionText: `${sharedLead}\nThis simple box uses sliding wooden catches.`,
      }),
      createEntry({
        recordKey: "equipment:puzzle-simple-hollow",
        name: "Puzzle Box (Simple) (Hollow)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/puzzle-box-simple-hollow.json",
        descriptionText: `${sharedLead}\nThis simple box uses sliding wooden catches and hides a hollow chamber.`,
      }),
      createEntry({
        recordKey: "equipment:puzzle-complex-hollow",
        name: "Puzzle Box (Complex) (Hollow)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/puzzle-box-complex-hollow.json",
        descriptionText: `${sharedLead}\nThis complex box hides a hollow chamber behind nested catches.`,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Puzzle Box");
    expect(entries[1].record.variantBaseName).toBe("Puzzle Box");
    expect(entries[2].record.variantBaseName).toBe("Puzzle Box");
    expect(entries[1].record.variantLabel).toBe("Simple, Hollow");
    expect(entries[2].record.variantLabel).toBe("Complex, Hollow");
    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
  });

  it("keeps a rank ladder as a fallback family even without shared description text", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:wand-base",
        name: "Wand of Choking Mist",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wand-of-choking-mist.json",
        descriptionText: "A base wand description that does not line up with the rank-specific variants.",
      }),
      createEntry({
        recordKey: "equipment:wand-2",
        name: "Wand of Choking Mist (2nd-Rank)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wand-of-choking-mist-2nd-rank.json",
        descriptionText: "The 2nd-rank version has a shorter custom description.",
      }),
      createEntry({
        recordKey: "equipment:wand-4",
        name: "Wand of Choking Mist (4th-Rank)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wand-of-choking-mist-4th-rank.json",
        descriptionText: "The 4th-rank version has a different custom description.",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Wand of Choking Mist");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantAxes).toEqual(["rank"]);
    expect(entries[2].record.variantAxes).toEqual(["rank"]);
    expect(entries[1].record.variantSource).toBe("composite");
  });

  it("groups title variants when lead text matches strongly but label parsing is loose", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:ring-wiz-1",
        name: "Ring of Wizardry (Type I)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/ring-of-wizardry-type-i.json",
        descriptionText: [
          "This ring is made from the purest platinum and is covered in esoteric arcane symbols. It does nothing unless you have a spellcasting class feature with the arcane tradition. While wearing the ring of wizardry, you gain a +1 item bonus to Arcana checks and have two additional 1st-rank arcane spell slots each day. You prepare spells in these slots or cast from them spontaneously, just as you normally cast your spells.",
          "If you take off the ring for any reason, you lose the additional spell slots. You can't gain spell slots from more than one ring of wizardry per day.",
          "If you can cast arcane spells in a variety of different ways, you can divide the spell slots as you wish among your various sources of arcane spells.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:ring-wiz-4",
        name: "Ring of Wizardry (Type IV)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/ring-of-wizardry-type-iv.json",
        descriptionText: [
          "This ring is made from the purest platinum and is covered in esoteric arcane symbols. It does nothing unless you have a spellcasting class feature with the arcane tradition. While wearing the ring of wizardry, you gain a +2 item bonus to Arcana checks and have two additional 4th-rank and one additional 3rd-rank arcane spell slot each day. You prepare spells in these slots or cast from them spontaneously, just as you normally cast your spells.",
          "If you take off the ring for any reason, you lose the additional spell slots. You can't gain spell slots from more than one ring of wizardry per day.",
          "If you can cast arcane spells in a variety of different ways, you can divide the spell slots as you wish among your various sources of arcane spells.",
        ].join("\n"),
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Ring of Wizardry");
    expect(entries[1].record.variantBaseName).toBe("Ring of Wizardry");
    expect(entries[0].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[0].record.variantSource).toBe("composite");
  });

  it("groups potency rune ladders despite numeric-only differences in lead text", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:armor-potency-1",
        name: "Armor Potency (+1)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/armor-potency-1.json",
        descriptionText: [
          "Magic wards deflect attacks. Increase the armor's item bonus to AC by 1. The armor can be etched with one property rune.",
          "You can upgrade the armor potency rune already etched on a suit of armor using the normal process for upgrading items and runes.",
          "Craft Requirements You are an expert in Crafting.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:armor-potency-3",
        name: "Armor Potency (+3)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/armor-potency-3.json",
        descriptionText: [
          "Magic wards deflect attacks. Increase the armor's item bonus to AC by 3. The armor can be etched with three property runes.",
          "You can upgrade the armor potency rune already etched on a suit of armor using the normal process for upgrading items and runes.",
          "Craft Requirements You are legendary in Crafting.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:weapon-potency-1",
        name: "Weapon Potency (+1)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/weapon-potency-1.json",
        descriptionText: [
          "Magical enhancements make this weapon strike true. Attack rolls with this weapon gain a +1 item bonus, and the weapon can be etched with one property rune.",
          "You can upgrade the weapon potency rune already etched on a weapon to a stronger version, increasing the values of the existing rune to those of the new rune.",
          "Craft Requirements You are an expert in Crafting.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:weapon-potency-3",
        name: "Weapon Potency (+3)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/weapon-potency-3.json",
        descriptionText: [
          "Magical enhancements make this weapon strike true. Attack rolls with this weapon gain a +3 item bonus, and the weapon can be etched with three property runes.",
          "You can upgrade the weapon potency rune already etched on a weapon to a stronger version, increasing the values of the existing rune to those of the new rune.",
          "Craft Requirements You are legendary in Crafting.",
        ].join("\n"),
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Armor Potency");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantBaseName).toBe("Weapon Potency");
    expect(entries[3].record.variantFamilyKey).toBe(entries[2].record.variantFamilyKey);
    expect(entries[0].record.variantFamilyKey).not.toBe(entries[2].record.variantFamilyKey);
  });

  it("groups specialist rings from shared intro text and shared title subsequence", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:specialist-base",
        name: "Specialist's Ring",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/specialists-ring.json",
        descriptionText: [
          "Each specialist's ring is dedicated to a single school of magic, and the ring is covered in symbols and glyphs related to that school according to the creator's arcane studies. A specialist's ring has the trait corresponding to its school of magic. You gain a +2 item bonus to Arcana checks, and a +1 circumstance bonus to recognize magical effects and items of the specific school of magic.",
          "Activate f envision",
          "Effect You gain 1 Focus Point, which you can use only to cast a wizard school spell of the corresponding school.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:specialist-conjuration",
        name: "Specialist's Ring (Conjuration)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/specialists-ring-conjuration.json",
        descriptionText: [
          "This specialist's ring is dedicated to the conjuration school of magic, and the ring is covered in symbols and glyphs related to the conjuration school according to the creator's arcane studies. You gain a +2 item bonus to Arcana checks, and a +1 circumstance bonus to recognize magical effects and items of the conjuration school of magic.",
          "Activate f envision",
          "Effect You gain 1 Focus Point, which you can use only to cast a wizard school spell of the conjuration school.",
        ].join("\n"),
      }),
      createEntry({
        recordKey: "equipment:specialist-evocation",
        name: "Specialist's Ring (Evocation)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/specialists-ring-evocation.json",
        descriptionText: [
          "This specialist's ring is dedicated to the evocation school of magic, and the ring is covered in symbols and glyphs related to the evocation school according to the creator's arcane studies. You gain a +2 item bonus to Arcana checks, and a +1 circumstance bonus to recognize magical effects and items of the evocation school of magic.",
          "Activate f envision",
          "Effect You gain 1 Focus Point, which you can use only to cast a wizard school spell of the evocation school.",
        ].join("\n"),
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Specialist's Ring");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantLabel).toBe("Conjuration");
    expect(entries[2].record.variantLabel).toBe("Evocation");
  });

  it("groups Highhelm drill marks from shared intro text and ordered title tokens", () => {
    const sharedLead = [
      "Not to be outdone by their cousins in Dongun Hold, the artificers of Highhelm have developed a handheld magical equivalent to their clockwork drilling constructs and vehicles. The device is still in the testing phases, but early versions have been released to fund more development. Appearing as an unassuming yellow box with two handles normally, when the command word is spoken, a spiraling drill made of force emerges from its top. An active Highhelm drill can be used as an improvised weapon, dealing damage on a Strike as though it had been used on a surface for one round with no additional damage from other sources.",
      "Activate 2 Interact",
    ].join("\n");
    const entries = [
      createEntry({
        recordKey: "equipment:drill-1",
        name: "Highhelm Drill Mark I",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/highhelm-drill-mark-i.json",
        descriptionText: `${sharedLead}\nEffect The force drill appears and begins turning, dealing 5 force damage per round.`,
      }),
      createEntry({
        recordKey: "equipment:drill-2",
        name: "Highhelm Drill Mark II",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/highhelm-drill-mark-ii.json",
        descriptionText: `${sharedLead}\nEffect The force drill appears and begins turning, dealing 10 force damage per round.`,
      }),
      createEntry({
        recordKey: "equipment:drill-3",
        name: "Highhelm Drill Mark III",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/highhelm-drill-mark-iii.json",
        descriptionText: `${sharedLead}\nEffect The force drill appears and begins turning, dealing 15 force damage per round.`,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Highhelm Drill");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantLabel).toBe("Mark III");
  });

  it("groups lock qualities from identical intro templates and shared title token", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:lock-poor",
        name: "Lock (Poor)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/lock-poor.json",
        descriptionText: "Picking a poor lock requires one successful Check[thievery|dc:15|traits:action:pick-a-lock|immutable:true] check.",
      }),
      createEntry({
        recordKey: "equipment:lock-average",
        name: "Lock (Average)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/lock-average.json",
        descriptionText: "Picking an average lock requires four successful Check[thievery|dc:25|traits:action:pick-a-lock|immutable:true] checks.",
      }),
      createEntry({
        recordKey: "equipment:lock-superior",
        name: "Lock (Superior)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/lock-superior.json",
        descriptionText: "Picking a superior lock requires six successful Check[thievery|dc:40|traits:action:pick-a-lock|immutable:true] checks.",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Lock");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantLabel).toBe("Superior");
  });

  it("does not group parenthetical title lines without shared opening descriptions", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:aeon-agate",
        name: "Aeon Stone (Agate Ellipsoid)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/aeon-stone-agate-ellipsoid.json",
        descriptionText: "This aeon stone allows you to cast Augury as a divine innate spell once per day.",
      }),
      createEntry({
        recordKey: "equipment:aeon-amber",
        name: "Aeon Stone (Amber Sphere)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/aeon-stone-amber-sphere.json",
        descriptionText: "If you are undead, your body regains much of the appearance it had in life.",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantFamilyKey).toBeNull();
    expect(entries[1].record.variantFamilyKey).toBeNull();
  });

  it("requires at least two siblings before assigning a variant family", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:mask",
        name: "Mask (Ordinary)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/mask-ordinary.json",
        descriptionText: "An ordinary mask with no sibling entries in the sample set.",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantFamilyKey).toBeNull();
    expect(entries[0].record.variantBaseName).toBeNull();
    expect(entries[0].record.variantSource).toBe("none");
  });
});
