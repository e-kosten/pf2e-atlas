import { describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry } from "../../src/types.js";
import {
  buildDerivedTagSeedLookup,
  buildDerivedTagSeedIndex,
  deriveCatalogTagDerivation,
  publishDerivedTagCatalog,
  resolveCatalogSeedRecordKeys,
} from "../../src/tags/catalog-utils.js";
import {
  deriveRecordTagDerivation,
  getDerivedTagSeedRecordKeys,
} from "../../src/tags/index.js";

const seedCatalog: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    family: "infiltration",
    description: "Equipment that helps infiltration.",
    promoteFamilyToTag: true,
    tags: [
      {
        value: "disguise",
        description: "Masks or alters appearance.",
        seedRecords: [
          { pack: "equipment-srd", name: "Mask" },
          { pack: "equipment-srd", name: "Veil" },
        ],
        excludeSeedRecords: [{ pack: "equipment-srd", name: "Blocked Mask" }],
      },
      {
        value: "concealment",
        description: "Provides concealment or concealability.",
        seedRecords: [{ pack: "equipment-srd", name: "Cloak" }],
      },
    ],
  },
  {
    category: "spell",
    family: "infiltration",
    description: "Spells that help infiltration.",
    tags: [
      {
        value: "disguise",
        description: "Spell-based disguise support.",
        seedRecords: [{ pack: "spells-srd", name: "Illusory Disguise" }],
      },
    ],
  },
  {
    category: "creature",
    family: "motif",
    description: "Creature visual motifs.",
    tags: [
      {
        value: "mask_motif",
        description: "Mask-centric creature imagery.",
        seedRecords: [{ pack: "pathfinder-monster-core", name: "Masked Priest" }],
      },
    ],
  },
];

const seedLookup = buildDerivedTagSeedLookup([
  { recordKey: "equipment:mask", pack: "equipment-srd", name: "Mask" },
  { recordKey: "equipment:veil", pack: "equipment-srd", name: "Veil" },
  { recordKey: "equipment:blocked", pack: "equipment-srd", name: "Blocked Mask" },
  { recordKey: "equipment:cloak", pack: "equipment-srd", name: "Cloak" },
  { recordKey: "spell:illusory-disguise", pack: "spells-srd", name: "Illusory Disguise" },
  { recordKey: "creature:masked-priest", pack: "pathfinder-monster-core", name: "Masked Priest" },
]);

describe("derived tag seeds", () => {
  it("publishes family tags without copying child seeds onto the promoted family", () => {
    expect(publishDerivedTagCatalog(seedCatalog)).toEqual(expect.arrayContaining([
      expect.objectContaining({
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({
            value: "disguise",
            seedRecords: [
              { pack: "equipment-srd", name: "Mask" },
              { pack: "equipment-srd", name: "Veil" },
            ],
            excludeSeedRecords: [{ pack: "equipment-srd", name: "Blocked Mask" }],
          }),
          expect.objectContaining({
            value: "infiltration",
            description: "Equipment that helps infiltration.",
          }),
        ]),
      }),
    ]));
  });

  it("resolves seed pools by tag and category scope, including promoted families", () => {
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog, seedLookup);

    expect(resolveCatalogSeedRecordKeys(seedIndex, "disguise", { category: "equipment" })).toEqual([
      "equipment:mask",
      "equipment:veil",
    ]);
    expect(resolveCatalogSeedRecordKeys(seedIndex, "disguise", { category: "spell" })).toEqual([
      "spell:illusory-disguise",
    ]);
    expect(resolveCatalogSeedRecordKeys(seedIndex, "infiltration", { category: "equipment" })).toEqual([
      "equipment:cloak",
      "equipment:mask",
      "equipment:veil",
    ]);
    expect(resolveCatalogSeedRecordKeys(seedIndex, "mask_motif", { category: "creature" })).toEqual([
      "creature:masked-priest",
    ]);
  });

  it("unions rule and seed tags and keeps provenance internally", () => {
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog, seedLookup);
    const derivation = deriveCatalogTagDerivation(seedCatalog, seedIndex, {
      recordKey: "equipment:mask",
      category: "equipment",
      subcategory: null,
    }, ["concealment"]);

    expect(derivation.tags).toEqual(["concealment", "disguise", "infiltration"]);
    expect(derivation.sources.get("concealment")).toBe("rule");
    expect(derivation.sources.get("disguise")).toBe("seed");
    expect(derivation.sources.get("infiltration")).toBe("both");
  });

  it("lets explicit seed exclusions block only seeded membership, not rule matches", () => {
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog, seedLookup);
    const derivation = deriveCatalogTagDerivation(seedCatalog, seedIndex, {
      recordKey: "equipment:blocked",
      category: "equipment",
      subcategory: null,
    }, ["disguise"]);

    expect(derivation.tags).toEqual(["disguise", "infiltration"]);
    expect(derivation.sources.get("disguise")).toBe("rule");
    expect(derivation.sources.get("infiltration")).toBe("rule");
  });

  it("fails fast when a seed reference does not resolve", () => {
    expect(() => buildDerivedTagSeedIndex([
      {
        category: "equipment",
        family: "infiltration",
        description: "Equipment that helps infiltration.",
        tags: [
          {
            value: "disguise",
            seedRecords: [{ pack: "equipment-srd", name: "Missing Mask" }],
          },
        ],
      },
    ], seedLookup)).toThrow(/did not resolve/);
  });

  it("fails fast when a seed reference resolves ambiguously", () => {
    const ambiguousLookup = buildDerivedTagSeedLookup([
      { recordKey: "equipment:mask-one", pack: "equipment-srd", name: "Mask" },
      { recordKey: "equipment:mask-two", pack: "equipment-srd", name: "Mask" },
    ]);

    expect(() => buildDerivedTagSeedIndex([
      {
        category: "equipment",
        family: "infiltration",
        description: "Equipment that helps infiltration.",
        tags: [
          {
            value: "disguise",
            seedRecords: [{ pack: "equipment-srd", name: "Mask" }],
          },
        ],
      },
    ], ambiguousLookup)).toThrow(/ambiguously/);
  });

  it("exposes the hazard manual seed pass and applies representative seeded records", () => {
    const touchedHazardTags = [
      "ward_trigger",
      "threshold_lockdown",
      "planar_breach",
      "restraint_capture",
      "barrier_lockdown",
      "spawned_attackers",
      "phantom_assailants",
      "fire_hazard",
      "poison_hazard",
      "respiratory_hazard",
    ];
    const seededHazardRecords = new Set(
      touchedHazardTags.flatMap((tag) => getDerivedTagSeedRecordKeys(tag, { category: "hazard" })),
    );

    expect(seededHazardRecords.size).toBeGreaterThanOrEqual(50);
    expect(getDerivedTagSeedRecordKeys("ward_trigger", { category: "hazard" })).toEqual(expect.arrayContaining([
      "agents-of-edgewatch-bestiary:qy53ECS2agScE7G3",
      "extinction-curse-bestiary:1CjTIaMYUvQUkQI2",
      "season-of-ghosts-bestiary:DueMGlf6tX1bqwSS",
    ]));
    expect(getDerivedTagSeedRecordKeys("fire_hazard", { category: "hazard" })).toEqual(expect.arrayContaining([
      "hazards:O0qA1ElCOgYGEBtL",
      "stolen-fate-bestiary:UX7QKytewemOnNeX",
      "blood-lords-bestiary:lycxuueclDmiIAOF",
    ]));

    const mukradiDerivation = deriveRecordTagDerivation({
      recordKey: "extinction-curse-bestiary:1CjTIaMYUvQUkQI2",
      name: "Mukradi Summoning Runes",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(mukradiDerivation.tags).toEqual(expect.arrayContaining(["spawned_attackers", "ward_trigger"]));
    expect(mukradiDerivation.sources.get("spawned_attackers")).toBe("seed");
    expect(mukradiDerivation.sources.get("ward_trigger")).toBe("seed");

    const gasChamberDerivation = deriveRecordTagDerivation({
      recordKey: "outlaws-of-alkenstar-bestiary:QQ2Ci8E2lkxG8QIV",
      name: "Subduing Gas Chamber",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(gasChamberDerivation.tags).toEqual(expect.arrayContaining([
      "barrier_lockdown",
      "poison_hazard",
      "respiratory_hazard",
    ]));
    expect(gasChamberDerivation.sources.get("barrier_lockdown")).toBe("seed");
    expect(gasChamberDerivation.sources.get("poison_hazard")).toBe("seed");
    expect(gasChamberDerivation.sources.get("respiratory_hazard")).toBe("seed");

    const timeRiftDerivation = deriveRecordTagDerivation({
      recordKey: "blood-lords-bestiary:I83vD5fNYIC1s3Xg",
      name: "Time Rift",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(timeRiftDerivation.tags).toContain("planar_breach");
    expect(timeRiftDerivation.sources.get("planar_breach")).toBe("seed");
  });

  it("exposes the spell manual seed pass and applies representative seeded records", () => {
    const touchedSpellTags = [
      "scouting",
      "navigation",
      "mobility",
      "healing_support",
      "condition_support",
      "affliction_cleanup",
      "protective_ward",
      "resistance_support",
      "countermagic",
      "persistent_damage",
      "initiative_support",
      "eidolon_support",
    ];
    const rawSeedAdds = touchedSpellTags.reduce(
      (count, tag) => count + getDerivedTagSeedRecordKeys(tag, { category: "spell" }).length,
      0,
    );
    const seededSpellRecords = new Set(
      touchedSpellTags.flatMap((tag) => getDerivedTagSeedRecordKeys(tag, { category: "spell" })),
    );

    expect(rawSeedAdds).toBeGreaterThanOrEqual(120);
    expect(seededSpellRecords.size).toBeGreaterThanOrEqual(90);
    expect(getDerivedTagSeedRecordKeys("persistent_damage", { category: "spell" })).toEqual(expect.arrayContaining([
      "spells-srd:f8hRqLJaxBVhF1u0",
      "spells-srd:Z3kJty995FkrsZRb",
      "spells-srd:A16eFTRh82xIjMu8",
    ]));
    expect(getDerivedTagSeedRecordKeys("initiative_support", { category: "spell" })).toEqual(expect.arrayContaining([
      "spells-srd:EUMjrJJwSgsqNidi",
      "spells-srd:dqaCLzINHBiKjh4J",
      "spells-srd:I8CPe9Pp7GABqOyB",
    ]));
    expect(getDerivedTagSeedRecordKeys("eidolon_support", { category: "spell" })).toEqual(expect.arrayContaining([
      "spells-srd:HStu2Yhw3iQER9tY",
      "spells-srd:AfOpnnwdZwHi2Tnc",
      "spells-srd:TYbCj4dgXDOZou9k",
    ]));

    const acidArrowDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:f8hRqLJaxBVhF1u0",
      name: "Acid Arrow",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(acidArrowDerivation.tags).toContain("persistent_damage");
    expect(acidArrowDerivation.sources.get("persistent_damage")).toBe("seed");

    const anticipatePerilDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:EUMjrJJwSgsqNidi",
      name: "Anticipate Peril",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(anticipatePerilDerivation.tags).toContain("initiative_support");
    expect(anticipatePerilDerivation.sources.get("initiative_support")).toBe("seed");

    const protectCompanionDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:AfOpnnwdZwHi2Tnc",
      name: "Protect Companion",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(protectCompanionDerivation.tags).toContain("eidolon_support");
    expect(protectCompanionDerivation.sources.get("eidolon_support")).toBe("seed");

    const airWalkDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:b5sGjGlBf58f8jn0",
      name: "Air Walk",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(airWalkDerivation.tags).toContain("mobility");
    expect(airWalkDerivation.sources.get("mobility")).toBe("seed");

    const returnBeaconDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:ru3YdXajUREbKQDV",
      name: "Return Beacon",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(returnBeaconDerivation.tags).toContain("navigation");
    expect(returnBeaconDerivation.sources.get("navigation")).toBe("seed");

    const veilOfPrivacyDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:EoKBlgf6Smt8opaU",
      name: "Veil of Privacy",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(veilOfPrivacyDerivation.tags).toContain("countermagic");
    expect(veilOfPrivacyDerivation.sources.get("countermagic")).toBe("seed");

    const cauterizeWoundsDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:sBSalosrt7C4IXas",
      name: "Cauterize Wounds",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(cauterizeWoundsDerivation.tags).not.toContain("persistent_damage");
  });

  it("exposes the creature manual seed pass and applies representative seeded records", () => {
    const encounterRoleTags = [
      "profession_npc",
      "civic_npc",
      "combatant_npc",
    ];
    const encounterRoleRawAdds = encounterRoleTags.reduce(
      (count, tag) => count + getDerivedTagSeedRecordKeys(tag, { category: "creature" }).length,
      0,
    );
    const encounterRoleRecords = new Set(
      encounterRoleTags.flatMap((tag) => getDerivedTagSeedRecordKeys(tag, { category: "creature" })),
    );

    expect(encounterRoleRawAdds).toBeGreaterThanOrEqual(180);
    expect(encounterRoleRecords.size).toBeGreaterThanOrEqual(95);
    expect(getDerivedTagSeedRecordKeys("profession_npc", { category: "creature" })).toEqual(expect.arrayContaining([
      "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
      "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
    ]));
    expect(getDerivedTagSeedRecordKeys("civic_npc", { category: "creature" })).toEqual(expect.arrayContaining([
      "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
      "gatewalkers-bestiary:kneoApQfhlRhhp1R",
      "sky-kings-tomb-bestiary:OWVg3LYOdGHOYUHt",
    ]));
    expect(getDerivedTagSeedRecordKeys("combatant_npc", { category: "creature" })).toEqual(expect.arrayContaining([
      "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      "triumph-of-the-tusk-bestiary:xd35No1x2n1MDVCm",
      "battlecry-bestiary:R1Ukw41ygDmnAmJk",
    ]));
    expect(getDerivedTagSeedRecordKeys("urban_setting", { category: "creature" })).toContain(
      "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    );
    expect(getDerivedTagSeedRecordKeys("urban_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "battlecry-bestiary:AHV0FTrbuPljLndw",
      "blood-lords-bestiary:EqO67DHLlB88vSJZ",
      "book-of-the-dead-bestiary:ol2lji9lH7PXh1uw",
    ]));
    expect(getDerivedTagSeedRecordKeys("nautical_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
      "agents-of-edgewatch-bestiary:d3TzpCuRJF78xHZK",
    ]));
    expect(getDerivedTagSeedRecordKeys("geb_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "blood-lords-bestiary:IXPZR1DTdT7Tu7UG",
      "blood-lords-bestiary:KQkouk6tku8akpmU",
      "blood-lords-bestiary:k8NnItW7Hp79bg26",
      "blood-lords-bestiary:vUv8SR7Pa5fONiuN",
    ]));
    expect(getDerivedTagSeedRecordKeys("gravelands_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "book-of-the-dead-bestiary:gXo04F7O4pwOY698",
      "claws-of-the-tyrant-bestiary:0mllbU5aBEcVUj3E",
      "claws-of-the-tyrant-bestiary:jSE16N2ASzN3qlN1",
      "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      "claws-of-the-tyrant-bestiary:vdywUTHF4uhA7cyh",
    ]));
    expect(getDerivedTagSeedRecordKeys("island_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "age-of-ashes-bestiary:5dSVk2y88SLsPPON",
      "age-of-ashes-bestiary:6AN7eagk2WrWc4im",
      "age-of-ashes-bestiary:X6TTBlHIfJZ43OqR",
    ]));
    expect(getDerivedTagSeedRecordKeys("battlefield_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "battlecry-bestiary:pC4qg7AarAty1K7K",
      "battlecry-bestiary:R1Ukw41ygDmnAmJk",
    ]));
    expect(getDerivedTagSeedRecordKeys("desert_setting", { category: "creature" })).toEqual(expect.arrayContaining([
      "battlecry-bestiary:egHaHp1lqQBZdKdR",
      "book-of-the-dead-bestiary:8qB0gj8salw8746I",
    ]));
    expect(getDerivedTagSeedRecordKeys("rural_setting", { category: "creature" })).toContain(
      "book-of-the-dead-bestiary:7WqlOvjoqURmeorA",
    );

    const starwatchCommandoDerivation = deriveRecordTagDerivation({
      recordKey: "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
      name: "Starwatch Commando",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(starwatchCommandoDerivation.tags).toEqual(expect.arrayContaining([
      "profession_npc",
      "civic_npc",
      "combatant_npc",
    ]));
    expect(["seed", "both"]).toContain(starwatchCommandoDerivation.sources.get("profession_npc"));
    expect(["seed", "both"]).toContain(starwatchCommandoDerivation.sources.get("civic_npc"));
    expect(["seed", "both"]).toContain(starwatchCommandoDerivation.sources.get("combatant_npc"));

    const falsePriestDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      name: "False Priest",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid"],
    });
    expect(falsePriestDerivation.tags).toEqual(expect.arrayContaining([
      "profession_npc",
      "combatant_npc",
    ]));
    expect(["seed", "both"]).toContain(falsePriestDerivation.sources.get("profession_npc"));
    expect(["seed", "both"]).toContain(falsePriestDerivation.sources.get("combatant_npc"));

    const commanderArsiellaDerivation = deriveRecordTagDerivation({
      recordKey: "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      name: "Commander Arsiella Dei",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aiuvarin", "elf", "human", "humanoid"],
    });
    expect(commanderArsiellaDerivation.tags).toEqual(expect.arrayContaining([
      "profession_npc",
      "civic_npc",
      "combatant_npc",
    ]));
    expect(["seed", "both"]).toContain(commanderArsiellaDerivation.sources.get("profession_npc"));
    expect(["seed", "both"]).toContain(commanderArsiellaDerivation.sources.get("civic_npc"));
    expect(["seed", "both"]).toContain(commanderArsiellaDerivation.sources.get("combatant_npc"));

    const spiritboundAluumDerivation = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
      name: "Spiritbound Aluum",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["construct", "mindless", "soulbound"],
    });
    expect(spiritboundAluumDerivation.tags).toContain("urban_setting");
    expect(spiritboundAluumDerivation.sources.get("urban_setting")).toBe("seed");

    const blackWhaleGuardDerivation = deriveRecordTagDerivation({
      recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
      name: "Black Whale Guard",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(blackWhaleGuardDerivation.tags).toContain("nautical_setting");
    expect(blackWhaleGuardDerivation.sources.get("nautical_setting")).toBe("seed");

    const mengkareDerivation = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:6AN7eagk2WrWc4im",
      name: "Mengkare",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "evil", "fire", "lawful"],
    });
    expect(mengkareDerivation.tags).toContain("island_setting");
    expect(mengkareDerivation.sources.get("island_setting")).toBe("seed");

    const dromaarCompanyDerivation = deriveRecordTagDerivation({
      recordKey: "battlecry-bestiary:R1Ukw41ygDmnAmJk",
      name: "Dromaar Company",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dromaar", "human", "humanoid", "orc", "troop"],
    });
    expect(dromaarCompanyDerivation.tags).toEqual(expect.arrayContaining(["battlefield_setting", "combatant_npc"]));
    expect(dromaarCompanyDerivation.sources.get("battlefield_setting")).toBe("both");
    expect(dromaarCompanyDerivation.sources.get("combatant_npc")).toBe("seed");

    const qadiranCamelCorpsDerivation = deriveRecordTagDerivation({
      recordKey: "battlecry-bestiary:egHaHp1lqQBZdKdR",
      name: "Qadiran Camel Corps",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["animal", "human", "humanoid", "troop"],
    });
    expect(qadiranCamelCorpsDerivation.tags).toContain("desert_setting");
    expect(qadiranCamelCorpsDerivation.sources.get("desert_setting")).toBe("seed");

    const chargharDerivation = deriveRecordTagDerivation({
      recordKey: "blood-lords-bestiary:vUv8SR7Pa5fONiuN",
      name: "Charghar",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["evil", "incorporeal", "spirit", "undead", "unholy"],
    });
    expect(chargharDerivation.tags).toContain("geb_setting");
    expect(chargharDerivation.sources.get("geb_setting")).toBe("seed");

    const commanderArsiellaSettingDerivation = deriveRecordTagDerivation({
      recordKey: "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      name: "Commander Arsiella Dei",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aiuvarin", "elf", "human", "humanoid"],
    });
    expect(commanderArsiellaSettingDerivation.tags).toContain("gravelands_setting");
    expect(commanderArsiellaSettingDerivation.sources.get("gravelands_setting")).toBe("seed");

    const deathCoachDerivation = deriveRecordTagDerivation({
      recordKey: "book-of-the-dead-bestiary:7WqlOvjoqURmeorA",
      name: "Death Coach",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["evil", "incorporeal", "spirit", "undead", "unholy"],
    });
    expect(deathCoachDerivation.tags).toContain("rural_setting");
    expect(deathCoachDerivation.sources.get("rural_setting")).toBe("seed");

    const touchedCreatureTags = [
      "dragon_spellcaster",
      "disguised_pretender",
      "faceless_horror",
      "regeneration_threat",
    ];
    const rawSeedAdds = touchedCreatureTags.reduce(
      (count, tag) => count + getDerivedTagSeedRecordKeys(tag, { category: "creature" }).length,
      0,
    );
    const seededCreatureRecords = new Set(
      touchedCreatureTags.flatMap((tag) => getDerivedTagSeedRecordKeys(tag, { category: "creature" })),
    );

    expect(rawSeedAdds).toBeGreaterThanOrEqual(120);
    expect(seededCreatureRecords.size).toBeGreaterThanOrEqual(110);
    expect(getDerivedTagSeedRecordKeys("dragon_spellcaster", { category: "creature" })).toEqual(expect.arrayContaining([
      "pathfinder-bestiary:pFmaszqtsA2yt7dv",
      "pathfinder-monster-core:T0OAOkmk4xz0wvjJ",
      "lost-omens-bestiary:IG23I5XqPXeICaOH",
    ]));
    expect(getDerivedTagSeedRecordKeys("disguised_pretender", { category: "creature" })).toEqual(expect.arrayContaining([
      "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
      "pathfinder-monster-core:TGYELuImcTcuX0aH",
      "lost-omens-bestiary:E4qscYn7U3jHoCia",
    ]));
    expect(getDerivedTagSeedRecordKeys("faceless_horror", { category: "creature" })).toEqual(expect.arrayContaining([
      "season-of-ghosts-bestiary:QSa1PbcvbgDv8Zpr",
      "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
      "season-of-ghosts-bestiary:3KNblm2fWM6XLiS7",
    ]));
    expect(getDerivedTagSeedRecordKeys("regeneration_threat", { category: "creature" })).toEqual(expect.arrayContaining([
      "pathfinder-monster-core-2:yHduMu4VBVUHnssz",
      "stolen-fate-bestiary:KgwkUtJ8czIC0KFj",
      "pathfinder-monster-core-2:1LBt5H8GekcuzDHw",
    ]));

    const redDragonDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-bestiary:pFmaszqtsA2yt7dv",
      name: "Red Dragon (Adult, Spellcaster)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "fire", "evil"],
    });
    expect(redDragonDerivation.tags).toContain("dragon_spellcaster");
    expect(redDragonDerivation.sources.get("dragon_spellcaster")).toBe("seed");

    const conspiratorDragonDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
      name: "Conspirator Dragon (Adult)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "occult"],
    });
    expect(conspiratorDragonDerivation.tags).toContain("disguised_pretender");
    expect(conspiratorDragonDerivation.sources.get("disguised_pretender")).toBe("seed");

    const nopperaBoDivineDerivation = deriveRecordTagDerivation({
      recordKey: "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
      name: "Noppera-Bo Impersonator (Divine)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aberration", "chaotic", "evil"],
    });
    expect(nopperaBoDivineDerivation.tags).toEqual(expect.arrayContaining([
      "disguised_pretender",
      "faceless_horror",
    ]));
    expect(nopperaBoDivineDerivation.sources.get("disguised_pretender")).toBe("seed");
    expect(nopperaBoDivineDerivation.sources.get("faceless_horror")).toBe("seed");

    const cavernTrollDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-monster-core-2:yHduMu4VBVUHnssz",
      name: "Cavern Troll",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["earth", "giant", "humanoid", "troll"],
    });
    expect(cavernTrollDerivation.tags).toContain("regeneration_threat");
    expect(cavernTrollDerivation.sources.get("regeneration_threat")).toBe("seed");
  });
});
