import { describe, expect, it } from "vitest";

import { assignVariantFamilies } from "../../src/data/variant-families.js";
import type { BuildSourceEntry, NormalizedIndexRecord, PackBuildInfo } from "../../src/data/index-types.js";

function createRecord(input: {
  recordKey: string;
  name: string;
  category?: NormalizedIndexRecord["category"];
  subcategory?: NormalizedIndexRecord["subcategory"];
  packName?: string;
  packLabel?: string;
  documentType?: string;
  recordType?: string;
  sourcePath: string;
  descriptionText?: string | null;
  traits?: string[];
  isUnique?: boolean;
}): NormalizedIndexRecord {
  const category = input.category ?? "equipment";
  return {
    recordKey: input.recordKey,
    id: input.recordKey,
    name: input.name,
    normalizedName: input.name.toLowerCase(),
    type: input.recordType ?? "equipment",
    category,
    subcategory: input.subcategory ?? "gear",
    packName: input.packName ?? "equipment-srd",
    packLabel: input.packLabel ?? "Equipment",
    documentType: input.documentType ?? "Item",
    level: null,
    rarity: "common",
    traits: input.traits ?? [],
    derivedTags: [],
    publicationTitle: "Pathfinder Treasure Vault",
    publicationRemaster: true,
    descriptionText: input.descriptionText ?? null,
    blurbText: null,
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
    isUnique: input.isUnique ?? false,
    size: null,
    itemCategory: category === "equipment" ? "wand" : null,
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
  rawBlurb?: string | null;
  category?: NormalizedIndexRecord["category"];
  subcategory?: NormalizedIndexRecord["subcategory"];
  packName?: string;
  packLabel?: string;
  documentType?: string;
  recordType?: string;
  traits?: string[];
  isUnique?: boolean;
}): BuildSourceEntry & { record: NormalizedIndexRecord } {
  const category = input.category ?? "equipment";
  const packName = input.packName ?? (category === "creature" ? "pathfinder-bestiary" : "equipment-srd");
  const packLabel = input.packLabel ?? (category === "creature" ? "Bestiary" : "Equipment");
  const documentType = input.documentType ?? (category === "creature" ? "Actor" : "Item");
  const recordType = input.recordType ?? (category === "creature" ? "npc" : "consumable");
  const pack: PackBuildInfo = {
    name: packName,
    label: packLabel,
    documentType,
    declaredPath: category === "creature" ? "packs/pf2e/pathfinder-bestiary" : "packs/pf2e/equipment",
    resolvedPath: category === "creature" ? "/tmp/creatures" : "/tmp/equipment",
  };

  return {
    pack,
    filePath: input.sourcePath,
    raw: {
      _id: input.recordKey,
      name: input.name,
      type: recordType,
      system: input.rawBlurb ? { details: { blurb: input.rawBlurb } } : {},
    },
    record: createRecord({
      recordKey: input.recordKey,
      name: input.name,
      category,
      subcategory: input.subcategory,
      packName,
      packLabel,
      documentType,
      recordType,
      sourcePath: input.sourcePath,
      descriptionText: input.descriptionText,
      traits: input.traits,
      isUnique: input.isUnique,
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
        descriptionText:
          "Picking a poor lock requires one successful Check[thievery|dc:15|traits:action:pick-a-lock|immutable:true] check.",
      }),
      createEntry({
        recordKey: "equipment:lock-average",
        name: "Lock (Average)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/lock-average.json",
        descriptionText:
          "Picking an average lock requires four successful Check[thievery|dc:25|traits:action:pick-a-lock|immutable:true] checks.",
      }),
      createEntry({
        recordKey: "equipment:lock-superior",
        name: "Lock (Superior)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/lock-superior.json",
        descriptionText:
          "Picking a superior lock requires six successful Check[thievery|dc:40|traits:action:pick-a-lock|immutable:true] checks.",
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

  it("groups creature age variants from parenthetical dragon titles", () => {
    const entries = [
      createEntry({
        recordKey: "creature:storm-young",
        name: "Storm Dragon (Young)",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/storm-dragon-young.json",
        descriptionText: "A storm dragon circles through black clouds and roaring winds.",
        traits: ["dragon", "electricity"],
      }),
      createEntry({
        recordKey: "creature:storm-adult",
        name: "Storm Dragon (Adult)",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/storm-dragon-adult.json",
        descriptionText: "A storm dragon circles through black clouds and roaring winds.",
        traits: ["dragon", "electricity"],
      }),
      createEntry({
        recordKey: "creature:storm-ancient",
        name: "Storm Dragon (Ancient)",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/storm-dragon-ancient.json",
        descriptionText: "A storm dragon circles through black clouds and roaring winds.",
        traits: ["dragon", "electricity"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Storm Dragon");
    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[0].record.variantLabel).toBe("Young");
    expect(entries[1].record.variantLabel).toBe("Adult");
    expect(entries[2].record.variantLabel).toBe("Ancient");
    expect(entries[0].record.variantAxes).toEqual(["dragonAge"]);
  });

  it("links a named unique creature to an established base creature family from its blurb", () => {
    const entries = [
      createEntry({
        recordKey: "creature:white-young",
        name: "White Dragon (Young)",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/white-dragon-young.json",
        descriptionText: "White dragons dwell on glacial mountaintops and in ice caverns.",
        traits: ["dragon", "cold"],
      }),
      createEntry({
        recordKey: "creature:white-adult",
        name: "White Dragon (Adult)",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/white-dragon-adult.json",
        descriptionText: "White dragons dwell on glacial mountaintops and in ice caverns.",
        traits: ["dragon", "cold"],
      }),
      createEntry({
        recordKey: "creature:venexus",
        name: "Venexus",
        category: "creature",
        subcategory: null,
        packName: "quest-for-the-frozen-flame-bestiary",
        packLabel: "Quest for the Frozen Flame",
        sourcePath: "vendor/pf2e/packs/pf2e/quest-for-the-frozen-flame-bestiary/venexus.json",
        descriptionText: "A unique white dragon carrying the Primordial Flame.",
        rawBlurb: "Female young white dragon",
        traits: ["dragon", "cold"],
        isUnique: true,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[2].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[2].record.variantBaseName).toBe("White Dragon");
    expect(entries[2].record.variantLabel).toBe("Venexus");
    expect(entries[2].record.variantAxes).toEqual(["dragonAge"]);
  });

  it("links a variant creature with an opaque title to its exact base creature from the blurb", () => {
    const entries = [
      createEntry({
        recordKey: "creature:scythe-tree",
        name: "Scythe Tree",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/scythe-tree.json",
        descriptionText: "Malevolent carnivorous trees pose as normal forest growth.",
        traits: ["plant"],
      }),
      createEntry({
        recordKey: "creature:tree-that-weeps",
        name: "Tree That Weeps",
        category: "creature",
        subcategory: null,
        packName: "kingmaker-bestiary",
        packLabel: "Kingmaker",
        sourcePath: "vendor/pf2e/packs/pf2e/kingmaker-bestiary/tree-that-weeps.json",
        descriptionText: "A malevolent unique scythe tree.",
        rawBlurb: "Variant scythe tree",
        traits: ["plant"],
        isUnique: true,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantBaseName).toBe("Scythe Tree");
    expect(entries[1].record.variantLabel).toBe("Tree That Weeps");
    expect(entries[1].record.variantAxes).toEqual(["specialization"]);
  });

  it("uses raw blurbs to resolve named creature links that prose-only matching missed", () => {
    const entries = [
      createEntry({
        recordKey: "creature:umasi",
        name: "Umasi",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-3/umasi.json",
        descriptionText: "A reclusive people who harvest appendages and organs from other creatures.",
        traits: ["aberration", "humanoid"],
      }),
      createEntry({
        recordKey: "creature:naiad-queen",
        name: "Naiad Queen",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/naiad-queen.json",
        descriptionText: "Naiad queens rule over pristine wildernesses centered on untouched lakes.",
        traits: ["fey", "nymph", "water"],
      }),
      createEntry({
        recordKey: "creature:bottlenose-dolphin",
        name: "Bottlenose Dolphin",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/bottlenose-dolphin.json",
        descriptionText: "Dolphins are social aquatic mammals known for intelligence.",
        traits: ["animal"],
      }),
      createEntry({
        recordKey: "creature:owlbear",
        name: "Owlbear",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/owlbear.json",
        descriptionText: "A dangerous territorial predator with the body of a bear and the senses of an owl.",
        traits: ["animal"],
      }),
      createEntry({
        recordKey: "creature:valmar",
        name: "Valmar",
        category: "creature",
        subcategory: null,
        packName: "gatewalkers-bestiary",
        packLabel: "Gatewalkers",
        sourcePath: "vendor/pf2e/packs/pf2e/gatewalkers-bestiary/valmar.json",
        descriptionText: "A unique umasi with public notes that don't begin with a subtype blurb.",
        rawBlurb: "Elite umasi",
        traits: ["aberration", "humanoid"],
        isUnique: true,
      }),
      createEntry({
        recordKey: "creature:pholebis",
        name: "Pholebis",
        category: "creature",
        subcategory: null,
        packName: "gatewalkers-bestiary",
        packLabel: "Gatewalkers",
        sourcePath: "vendor/pf2e/packs/pf2e/gatewalkers-bestiary/pholebis.json",
        descriptionText: "A unique naiad queen with longer public notes.",
        rawBlurb: "female naiad queen",
        traits: ["fey", "nymph", "water"],
        isUnique: true,
      }),
      createEntry({
        recordKey: "creature:whalesteed",
        name: "Whalesteed",
        category: "creature",
        subcategory: null,
        packName: "lost-omens-bestiary",
        packLabel: "Lost Omens Bestiary",
        sourcePath: "vendor/pf2e/packs/pf2e/lost-omens-bestiary/whalesteed.json",
        descriptionText: "A trained marine mount with public notes unrelated to its base-creature subtype.",
        rawBlurb: "Variant bottlenose dolphin",
        traits: ["animal"],
      }),
      createEntry({
        recordKey: "creature:irriseni-owlbear",
        name: "Irriseni Owlbear",
        category: "creature",
        subcategory: null,
        packName: "lost-omens-bestiary",
        packLabel: "Lost Omens Bestiary",
        sourcePath: "vendor/pf2e/packs/pf2e/lost-omens-bestiary/irriseni-owlbear.json",
        descriptionText: "An arctic-adapted owlbear with longer public notes.",
        rawBlurb: "Variant elite owlbear",
        traits: ["animal"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[4].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[4].record.variantBaseName).toBe("Umasi");
    expect(entries[5].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[5].record.variantBaseName).toBe("Naiad Queen");
    expect(entries[6].record.variantFamilyKey).toBe(entries[2].record.variantFamilyKey);
    expect(entries[6].record.variantBaseName).toBe("Bottlenose Dolphin");
    expect(entries[7].record.variantFamilyKey).toBe(entries[3].record.variantFamilyKey);
    expect(entries[7].record.variantBaseName).toBe("Owlbear");
  });

  it("does not infer named creature families from gender-only simple-species or humanoid-role blurbs", () => {
    const entries = [
      createEntry({
        recordKey: "creature:lion",
        name: "Lion",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/lion.json",
        descriptionText: "A big cat predator.",
        traits: ["animal"],
      }),
      createEntry({
        recordKey: "creature:nixie",
        name: "Nixie",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/nixie.json",
        descriptionText: "A mischievous water fey.",
        traits: ["fey", "water"],
      }),
      createEntry({
        recordKey: "creature:hunter",
        name: "Hunter",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-npc-core",
        packLabel: "NPC Core",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-npc-core/hunter.json",
        descriptionText: "A humanoid wilderness scout.",
        traits: ["humanoid"],
      }),
      createEntry({
        recordKey: "creature:elf-ranger",
        name: "Elf Ranger",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-npc-core",
        packLabel: "NPC Core",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-npc-core/elf-ranger.json",
        descriptionText: "A humanoid elven ranger.",
        traits: ["humanoid", "elf"],
      }),
      createEntry({
        recordKey: "creature:leandrus",
        name: "Leandrus",
        category: "creature",
        subcategory: null,
        packName: "extinction-curse-bestiary",
        packLabel: "Extinction Curse",
        sourcePath: "vendor/pf2e/packs/pf2e/extinction-curse-bestiary/leandrus.json",
        descriptionText: "A named circus lion.",
        rawBlurb: "Male lion",
        traits: ["animal"],
        isUnique: true,
      }),
      createEntry({
        recordKey: "creature:melianse",
        name: "Melianse",
        category: "creature",
        subcategory: null,
        packName: "kingmaker-bestiary",
        packLabel: "Kingmaker",
        sourcePath: "vendor/pf2e/packs/pf2e/kingmaker-bestiary/melianse.json",
        descriptionText: "A named nixie.",
        rawBlurb: "Female nixie",
        traits: ["fey", "water"],
        isUnique: true,
      }),
      createEntry({
        recordKey: "creature:aldori-sister",
        name: "Aldori Sister",
        category: "creature",
        subcategory: null,
        packName: "kingmaker-bestiary",
        packLabel: "Kingmaker",
        sourcePath: "vendor/pf2e/packs/pf2e/kingmaker-bestiary/aldori-sister.json",
        descriptionText: "A named humanoid hunter.",
        rawBlurb: "Female Hunter",
        traits: ["humanoid"],
        isUnique: true,
      }),
      createEntry({
        recordKey: "creature:shalelu-andosana",
        name: "Shalelu Andosana",
        category: "creature",
        subcategory: null,
        packName: "spore-war-bestiary",
        packLabel: "Spore War",
        sourcePath: "vendor/pf2e/packs/pf2e/spore-war-bestiary/shalelu-andosana.json",
        descriptionText: "A named elven ranger.",
        rawBlurb: "Female elf ranger",
        traits: ["humanoid", "elf"],
        isUnique: true,
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[4].record.variantFamilyKey).toBeNull();
    expect(entries[5].record.variantFamilyKey).toBeNull();
    expect(entries[6].record.variantFamilyKey).toBeNull();
    expect(entries[7].record.variantFamilyKey).toBeNull();
  });

  it("keeps explicit labels for exact-name creature variants resolved from raw blurbs", () => {
    const entries = [
      createEntry({
        recordKey: "creature:bulette-base",
        name: "Bulette",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-bestiary",
        packLabel: "Bestiary 1",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/bulette.json",
        descriptionText: "A burrowing landshark.",
        traits: ["animal", "beast"],
      }),
      createEntry({
        recordKey: "creature:bulette-variant",
        name: "Bulette",
        category: "creature",
        subcategory: null,
        packName: "blood-lords-bestiary",
        packLabel: "Blood Lords",
        sourcePath: "vendor/pf2e/packs/pf2e/blood-lords-bestiary/bulette.json",
        descriptionText: "A variant bulette from an adventure bestiary.",
        rawBlurb: "Variant Bulette",
        traits: ["animal", "beast"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[0].record.variantBaseName).toBe("Bulette");
    expect(entries[0].record.variantLabel).toBeNull();
    expect(entries[1].record.variantLabel).toBe("Variant");
    expect(entries[1].record.variantSource).toBe("composite");
  });

  it("prefers explicit raw-blurb subtype links over weaker parenthetical creature title grouping", () => {
    const entries = [
      createEntry({
        recordKey: "creature:hill-giant",
        name: "Hill Giant",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-bestiary",
        packLabel: "Bestiary 1",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/hill-giant.json",
        descriptionText: "A brutish giant that lairs in hills and mountains.",
        traits: ["giant"],
      }),
      createEntry({
        recordKey: "creature:tiger-lord-hill-giant",
        name: "Tiger Lord Hill Giant (TL2)",
        category: "creature",
        subcategory: null,
        packName: "kingmaker-bestiary",
        packLabel: "Kingmaker",
        sourcePath: "vendor/pf2e/packs/pf2e/kingmaker-bestiary/tiger-lord-hill-giant-tl2.json",
        descriptionText: "A tiger lord hill giant with a title-based suffix.",
        rawBlurb: "Variant hill giant",
        traits: ["giant"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantBaseName).toBe("Hill Giant");
    expect(entries[1].record.variantLabel).toBe("Tiger Lord Hill Giant (TL2)");
  });

  it("singularizes plural creature blurbs before resolving the base family", () => {
    const entries = [
      createEntry({
        recordKey: "creature:frost-drake",
        name: "Frost Drake",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-bestiary-2",
        packLabel: "Bestiary 2",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-2/frost-drake.json",
        descriptionText: "A drake adapted to icy climates.",
        traits: ["dragon"],
      }),
      createEntry({
        recordKey: "creature:drake-courser",
        name: "Drake Courser",
        category: "creature",
        subcategory: null,
        packName: "fists-of-the-ruby-phoenix-bestiary",
        packLabel: "Fists of the Ruby Phoenix",
        sourcePath: "vendor/pf2e/packs/pf2e/fists-of-the-ruby-phoenix-bestiary/drake-courser.json",
        descriptionText: "A fast drake used as a mount.",
        rawBlurb: "Variant frost drakes",
        traits: ["dragon"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantBaseName).toBe("Frost Drake");
    expect(entries[1].record.variantLabel).toBe("Drake Courser");
  });

  it("singularizes plural elite creature blurbs before resolving the base family", () => {
    const entries = [
      createEntry({
        recordKey: "creature:vampire-count",
        name: "Vampire Count",
        category: "creature",
        subcategory: null,
        packName: "pathfinder-monster-core-2",
        packLabel: "Monster Core 2",
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core-2/vampire-count.json",
        descriptionText: "An aristocratic vampire with potent occult power.",
        traits: ["undead"],
      }),
      createEntry({
        recordKey: "creature:xarbaene",
        name: "Xarbaene",
        category: "creature",
        subcategory: null,
        packName: "shades-of-blood-bestiary",
        packLabel: "Shades of Blood",
        sourcePath: "vendor/pf2e/packs/pf2e/shades-of-blood-bestiary/xarbaene.json",
        descriptionText: "A named vampire noble.",
        rawBlurb: "Elite vampire counts",
        traits: ["undead"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantBaseName).toBe("Vampire Count");
    expect(entries[1].record.variantLabel).toBe("Xarbaene");
  });

  it("falls back to exact suffix creature families for allowlisted undead subtype bases", () => {
    const entries = [
      createEntry({
        recordKey: "creature:wraith",
        name: "Wraith",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/wraith.json",
        descriptionText: "A life-draining incorporeal undead.",
        traits: ["incorporeal", "undead", "unholy", "wraith"],
      }),
      createEntry({
        recordKey: "creature:war-wraith",
        name: "War Wraith",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core-2/war-wraith.json",
        descriptionText: "A towering undead warlord spirit.",
        traits: ["incorporeal", "undead", "unholy", "wraith"],
      }),
      createEntry({
        recordKey: "creature:wight",
        name: "Wight",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/wight.json",
        descriptionText: "An intelligent spiteful undead.",
        traits: ["undead", "unholy", "wight"],
      }),
      createEntry({
        recordKey: "creature:hunter-wight",
        name: "Hunter Wight",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-2/hunter-wight.json",
        descriptionText: "A wilderness-hunting wight.",
        traits: ["evil", "lawful", "undead", "unholy", "wight"],
      }),
      createEntry({
        recordKey: "creature:ghoul",
        name: "Ghoul",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/ghoul.json",
        descriptionText: "A ravenous corpse-eating undead.",
        traits: ["chaotic", "evil", "ghoul", "undead", "unholy"],
      }),
      createEntry({
        recordKey: "creature:leng-ghoul",
        name: "Leng Ghoul",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-3/leng-ghoul.json",
        descriptionText: "A ghoul shaped by Leng.",
        traits: ["chaotic", "dream", "evil", "ghoul", "undead", "unholy"],
      }),
      createEntry({
        recordKey: "creature:ghost",
        name: "Ghost",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/ghost.json",
        descriptionText: "A common ghost.",
        traits: ["ghost", "incorporeal", "spirit", "undead", "unholy"],
      }),
      createEntry({
        recordKey: "creature:hungry-ghost",
        name: "Hungry Ghost",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-3/hungry-ghost.json",
        descriptionText: "A ghost driven by burial neglect.",
        traits: ["ghost", "incorporeal", "spirit", "undead", "unholy"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBe(entries[0].record.variantFamilyKey);
    expect(entries[1].record.variantBaseName).toBe("Wraith");
    expect(entries[1].record.variantLabel).toBe("War Wraith");
    expect(entries[1].record.variantSource).toBe("composite");

    expect(entries[3].record.variantFamilyKey).toBe(entries[2].record.variantFamilyKey);
    expect(entries[3].record.variantBaseName).toBe("Wight");
    expect(entries[3].record.variantLabel).toBe("Hunter Wight");

    expect(entries[5].record.variantFamilyKey).toBe(entries[4].record.variantFamilyKey);
    expect(entries[5].record.variantBaseName).toBe("Ghoul");
    expect(entries[5].record.variantLabel).toBe("Leng Ghoul");

    expect(entries[7].record.variantFamilyKey).toBe(entries[6].record.variantFamilyKey);
    expect(entries[7].record.variantBaseName).toBe("Ghost");
    expect(entries[7].record.variantLabel).toBe("Hungry Ghost");
  });

  it("does not use suffix fallback for disallowed or trait-mismatched creature families", () => {
    const entries = [
      createEntry({
        recordKey: "creature:ghost",
        name: "Ghost",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/ghost.json",
        descriptionText: "A common ghost.",
        traits: ["ghost", "incorporeal", "spirit", "undead", "unholy"],
      }),
      createEntry({
        recordKey: "creature:stone-ghost",
        name: "Stone Ghost",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/sky-kings-tomb-bestiary/stone-ghost.json",
        descriptionText: "A haunted oread statue.",
        traits: ["evil", "human", "humanoid", "oread"],
      }),
      createEntry({
        recordKey: "creature:troll",
        name: "Troll",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary/troll.json",
        descriptionText: "A regenerating giant menace.",
        traits: ["chaotic", "evil", "giant", "troll"],
      }),
      createEntry({
        recordKey: "creature:ice-troll",
        name: "Ice Troll",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-2/ice-troll.json",
        descriptionText: "A troll adapted to frozen regions.",
        traits: ["cold", "giant", "humanoid", "troll"],
      }),
      createEntry({
        recordKey: "creature:golem",
        name: "Golem",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-monster-core/golem.json",
        descriptionText: "A divine animated construct.",
        traits: ["construct", "earth", "holy"],
      }),
      createEntry({
        recordKey: "creature:mithral-golem",
        name: "Mithral Golem",
        category: "creature",
        subcategory: null,
        sourcePath: "vendor/pf2e/packs/pf2e/pathfinder-bestiary-3/mithral-golem.json",
        descriptionText: "A silvery construct colossus.",
        traits: ["construct", "golem", "mindless"],
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[1].record.variantFamilyKey).toBeNull();
    expect(entries[3].record.variantFamilyKey).toBeNull();
    expect(entries[5].record.variantFamilyKey).toBeNull();
  });
});
