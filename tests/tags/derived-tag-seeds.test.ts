import { describe, expect, it } from "vitest";

import type { DerivedTagExemplarCategory, DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../src/domain/index.js";
import { publishDerivedTagOntology } from "../../src/tags/runtime/catalog-utils.js";
import {
  publishDerivedTagExemplars,
  resolveDerivedTagExemplarRecordKeys,
  validateDerivedTagExemplarsAgainstRecords,
} from "../../src/tags/runtime/exemplar-utils.js";
import {
  deriveRecordTagDerivation,
  getDerivedTagExemplarRecordKeys,
  getDerivedTagLegacySeedMigrationRecordKeys,
  listDerivedTagLegacySeedMigrations,
} from "../../src/tags/index.js";

function matcherArrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function matcherObjectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

const exemplarFamilies: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    family: "infiltration",
    axis: "utility",
    description: "Equipment that helps infiltration.",
  },
  {
    category: "spell",
    family: "infiltration",
    axis: "utility",
    description: "Spells that help infiltration.",
  },
  {
    category: "creature",
    family: "visual_motif",
    axis: "presentation",
    description: "Creature visual motifs.",
  },
];

const exemplarTags: DerivedTagOntologyTag[] = [
  {
    category: "equipment",
    family: "infiltration",
    tag: "disguise",
    description: "Masks or alters appearance.",
    assignmentMode: "hybrid",
  },
  {
    family: "infiltration",
    category: "equipment",
    tag: "concealment",
    description: "Provides concealment or concealability.",
    assignmentMode: "hybrid",
  },
  {
    category: "spell",
    family: "infiltration",
    tag: "disguise",
    description: "Spell-based disguise support.",
    assignmentMode: "hybrid",
  },
  {
    category: "creature",
    family: "visual_motif",
    tag: "mask_motif",
    description: "Mask-centric creature imagery.",
    assignmentMode: "hybrid",
  },
];

const exemplarOntology = publishDerivedTagOntology(exemplarFamilies, exemplarTags);
const authoredExemplars: DerivedTagExemplarCategory[] = [
  {
    category: "equipment",
    exemplars: [
      {
        tag: "disguise",
        positives: [
          { name: "Mask", recordKey: "equipment:mask" },
          { name: "Veil", recordKey: "equipment:veil" },
        ],
        negatives: [{ name: "Blocked Mask", recordKey: "equipment:blocked" }],
      },
    ],
  },
  {
    category: "spell",
    exemplars: [
      {
        tag: "disguise",
        positives: [{ name: "Illusory Disguise", recordKey: "spell:illusory-disguise" }],
      },
    ],
  },
  {
    category: "creature",
    exemplars: [
      {
        tag: "mask_motif",
        positives: [{ name: "Masked Priest", recordKey: "creature:masked-priest" }],
      },
    ],
  },
];

describe("derived tag exemplars and legacy seed migrations", () => {
  it("publishes positive exemplar pools by tag and category scope", () => {
    const exemplars = publishDerivedTagExemplars(exemplarOntology, authoredExemplars);

    expect(resolveDerivedTagExemplarRecordKeys(exemplars, "disguise", { category: "equipment" })).toEqual([
      "equipment:mask",
      "equipment:veil",
    ]);
    expect(resolveDerivedTagExemplarRecordKeys(exemplars, "disguise", { category: "spell" })).toEqual([
      "spell:illusory-disguise",
    ]);
    expect(resolveDerivedTagExemplarRecordKeys(exemplars, "mask_motif", { category: "creature" })).toEqual([
      "creature:masked-priest",
    ]);
  });

  it("fails fast when an exemplar references an unknown ontology tag", () => {
    expect(() =>
      publishDerivedTagExemplars(exemplarOntology, [
        {
          category: "equipment",
          exemplars: [
            {
              tag: "unknown_tag",
              positives: [{ name: "Mask", recordKey: "equipment:mask" }],
            },
          ],
        },
      ]),
    ).toThrow(/does not exist in the published ontology/);
  });

  it("rejects duplicate and conflicting exemplar records", () => {
    expect(() =>
      publishDerivedTagExemplars(exemplarOntology, [
        {
          category: "equipment",
          exemplars: [
            {
              tag: "disguise",
              positives: [
                { name: "Mask", recordKey: "equipment:mask" },
                { name: "Mask", recordKey: "equipment:mask" },
              ],
            },
          ],
        },
      ]),
    ).toThrow(/repeats record/);

    expect(() =>
      publishDerivedTagExemplars(exemplarOntology, [
        {
          category: "equipment",
          exemplars: [
            {
              tag: "disguise",
              positives: [{ name: "Mask", recordKey: "equipment:mask" }],
              negatives: [{ name: "Mask", recordKey: "equipment:mask" }],
            },
          ],
        },
      ]),
    ).toThrow(/both positive and negative/);
  });

  it("validates exemplar name drift against canonical records when present", () => {
    const exemplars = publishDerivedTagExemplars(exemplarOntology, authoredExemplars);

    expect(() =>
      validateDerivedTagExemplarsAgainstRecords(
        [
          { recordKey: "equipment:mask", name: "Mask", category: "equipment" },
          { recordKey: "equipment:veil", name: "Veil", category: "equipment" },
          { recordKey: "equipment:blocked", name: "Blocked Mask", category: "equipment" },
          { recordKey: "spell:illusory-disguise", name: "Illusory Disguise", category: "spell" },
          { recordKey: "creature:masked-priest", name: "Masked Priest", category: "creature" },
        ],
        exemplars,
      ),
    ).not.toThrow();

    expect(() =>
      validateDerivedTagExemplarsAgainstRecords(
        [{ recordKey: "equipment:mask", name: "Different Mask", category: "equipment" }],
        exemplars,
      ),
    ).toThrow(/expected name/);
  });

  it("exposes configured exemplars separately from live tag derivation", () => {
    expect(getDerivedTagExemplarRecordKeys("urban_setting", { category: "creature" })).toEqual([
      "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
      "pathfinder-monster-core:TGYELuImcTcuX0aH",
    ]);

    const spiritboundAluumDerivation = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
      name: "Spiritbound Aluum",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["construct", "mindless", "soulbound"],
    });
    expect(spiritboundAluumDerivation.tags).toContain("urban_setting");
    expect(spiritboundAluumDerivation.sources.get("urban_setting")).toBe("assignment");
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
      touchedHazardTags.flatMap((tag) => getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "hazard" })),
    );

    expect(seededHazardRecords.size).toBeGreaterThanOrEqual(50);
    expect(getDerivedTagLegacySeedMigrationRecordKeys("ward_trigger", { category: "hazard" })).toEqual(
      expect.arrayContaining([
        "agents-of-edgewatch-bestiary:qy53ECS2agScE7G3",
        "extinction-curse-bestiary:1CjTIaMYUvQUkQI2",
        "season-of-ghosts-bestiary:DueMGlf6tX1bqwSS",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("fire_hazard", { category: "hazard" })).toEqual(
      expect.arrayContaining([
        "hazards:O0qA1ElCOgYGEBtL",
        "stolen-fate-bestiary:UX7QKytewemOnNeX",
        "blood-lords-bestiary:lycxuueclDmiIAOF",
      ]),
    );

    const mukradiDerivation = deriveRecordTagDerivation({
      recordKey: "extinction-curse-bestiary:1CjTIaMYUvQUkQI2",
      name: "Mukradi Summoning Runes",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(mukradiDerivation.tags).toEqual(expect.arrayContaining(["spawned_attackers", "ward_trigger"]));
    expect(mukradiDerivation.sources.get("spawned_attackers")).toBe("seed_migration");
    expect(mukradiDerivation.sources.get("ward_trigger")).toBe("seed_migration");

    const gasChamberDerivation = deriveRecordTagDerivation({
      recordKey: "outlaws-of-alkenstar-bestiary:QQ2Ci8E2lkxG8QIV",
      name: "Subduing Gas Chamber",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(gasChamberDerivation.tags).toEqual(
      expect.arrayContaining(["barrier_lockdown", "poison_hazard", "respiratory_hazard"]),
    );
    expect(gasChamberDerivation.sources.get("barrier_lockdown")).toBe("seed_migration");
    expect(gasChamberDerivation.sources.get("poison_hazard")).toBe("seed_migration");
    expect(gasChamberDerivation.sources.get("respiratory_hazard")).toBe("seed_migration");

    const timeRiftDerivation = deriveRecordTagDerivation({
      recordKey: "blood-lords-bestiary:I83vD5fNYIC1s3Xg",
      name: "Time Rift",
      category: "hazard",
      subcategory: "trap",
      descriptionText: null,
      traits: [],
    });
    expect(timeRiftDerivation.tags).toContain("planar_breach");
    expect(timeRiftDerivation.sources.get("planar_breach")).toBe("seed_migration");
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
      (count, tag) => count + getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "spell" }).length,
      0,
    );
    const seededSpellRecords = new Set(
      touchedSpellTags.flatMap((tag) => getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "spell" })),
    );

    expect(rawSeedAdds).toBeGreaterThanOrEqual(120);
    expect(seededSpellRecords.size).toBeGreaterThanOrEqual(90);
    expect(getDerivedTagLegacySeedMigrationRecordKeys("persistent_damage", { category: "spell" })).toEqual(
      expect.arrayContaining([
        "spells-srd:f8hRqLJaxBVhF1u0",
        "spells-srd:Z3kJty995FkrsZRb",
        "spells-srd:A16eFTRh82xIjMu8",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("initiative_support", { category: "spell" })).toEqual(
      expect.arrayContaining([
        "spells-srd:EUMjrJJwSgsqNidi",
        "spells-srd:dqaCLzINHBiKjh4J",
        "spells-srd:I8CPe9Pp7GABqOyB",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("eidolon_support", { category: "spell" })).toEqual(
      expect.arrayContaining([
        "spells-srd:HStu2Yhw3iQER9tY",
        "spells-srd:AfOpnnwdZwHi2Tnc",
        "spells-srd:TYbCj4dgXDOZou9k",
      ]),
    );

    const acidArrowDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:f8hRqLJaxBVhF1u0",
      name: "Acid Arrow",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(acidArrowDerivation.tags).toContain("persistent_damage");
    expect(acidArrowDerivation.sources.get("persistent_damage")).toBe("seed_migration");

    const anticipatePerilDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:EUMjrJJwSgsqNidi",
      name: "Anticipate Peril",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(anticipatePerilDerivation.tags).toContain("initiative_support");
    expect(anticipatePerilDerivation.sources.get("initiative_support")).toBe("seed_migration");

    const protectCompanionDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:AfOpnnwdZwHi2Tnc",
      name: "Protect Companion",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(protectCompanionDerivation.tags).toContain("eidolon_support");
    expect(protectCompanionDerivation.sources.get("eidolon_support")).toBe("seed_migration");

    const airWalkDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:b5sGjGlBf58f8jn0",
      name: "Air Walk",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(airWalkDerivation.tags).toContain("mobility");
    expect(airWalkDerivation.sources.get("mobility")).toBe("seed_migration");

    const returnBeaconDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:ru3YdXajUREbKQDV",
      name: "Return Beacon",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(returnBeaconDerivation.tags).toContain("navigation");
    expect(returnBeaconDerivation.sources.get("navigation")).toBe("seed_migration");

    const veilOfPrivacyDerivation = deriveRecordTagDerivation({
      recordKey: "spells-srd:EoKBlgf6Smt8opaU",
      name: "Veil of Privacy",
      category: "spell",
      subcategory: null,
      descriptionText: null,
      traits: [],
    });
    expect(veilOfPrivacyDerivation.tags).toContain("countermagic");
    expect(veilOfPrivacyDerivation.sources.get("countermagic")).toBe("seed_migration");

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
    const encounterRoleTags = ["profession_npc", "civic_npc", "enforcer_npc"];
    const encounterRoleRawAdds = encounterRoleTags.reduce(
      (count, tag) => count + getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "creature" }).length,
      0,
    );
    const encounterRoleRecords = new Set(
      encounterRoleTags.flatMap((tag) => getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "creature" })),
    );

    expect(encounterRoleRawAdds).toBeGreaterThanOrEqual(180);
    expect(encounterRoleRecords.size).toBeGreaterThanOrEqual(95);
    expect(getDerivedTagLegacySeedMigrationRecordKeys("profession_npc", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
        "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("civic_npc", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
        "gatewalkers-bestiary:kneoApQfhlRhhp1R",
        "sky-kings-tomb-bestiary:OWVg3LYOdGHOYUHt",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("enforcer_npc", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
        "triumph-of-the-tusk-bestiary:xd35No1x2n1MDVCm",
        "battlecry-bestiary:R1Ukw41ygDmnAmJk",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("urban_setting", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "battlecry-bestiary:AHV0FTrbuPljLndw",
        "blood-lords-bestiary:EqO67DHLlB88vSJZ",
        "book-of-the-dead-bestiary:ol2lji9lH7PXh1uw",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("nautical_setting", { category: "creature" })).toEqual(
      expect.arrayContaining(["agents-of-edgewatch-bestiary:d3TzpCuRJF78xHZK"]),
    );
    expect(
      getDerivedTagLegacySeedMigrationRecordKeys("organized_undead_society_setting", { category: "creature" }),
    ).toEqual(
      expect.arrayContaining([
        "blood-lords-bestiary:IXPZR1DTdT7Tu7UG",
        "blood-lords-bestiary:KQkouk6tku8akpmU",
        "blood-lords-bestiary:k8NnItW7Hp79bg26",
        "blood-lords-bestiary:vUv8SR7Pa5fONiuN",
      ]),
    );
    expect(
      getDerivedTagLegacySeedMigrationRecordKeys("undead_war_torn_region_setting", { category: "creature" }),
    ).toEqual(
      expect.arrayContaining([
        "book-of-the-dead-bestiary:gXo04F7O4pwOY698",
        "claws-of-the-tyrant-bestiary:0mllbU5aBEcVUj3E",
        "claws-of-the-tyrant-bestiary:jSE16N2ASzN3qlN1",
        "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
        "claws-of-the-tyrant-bestiary:vdywUTHF4uhA7cyh",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("tian_xia_setting", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "fists-of-the-ruby-phoenix-bestiary:HqN4nUnl75foBKLZ",
        "fists-of-the-ruby-phoenix-bestiary:YVP3pM7jxY9Gyouy",
        "fists-of-the-ruby-phoenix-bestiary:koRKlywSbwttifEq",
        "fists-of-the-ruby-phoenix-bestiary:zNOSSDaaCozimqaS",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("island_setting", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "age-of-ashes-bestiary:5dSVk2y88SLsPPON",
        "age-of-ashes-bestiary:6AN7eagk2WrWc4im",
        "age-of-ashes-bestiary:X6TTBlHIfJZ43OqR",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("battlefield_setting", { category: "creature" })).toEqual(
      expect.arrayContaining(["battlecry-bestiary:pC4qg7AarAty1K7K", "battlecry-bestiary:R1Ukw41ygDmnAmJk"]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("desert_setting", { category: "creature" })).toEqual(
      expect.arrayContaining(["battlecry-bestiary:egHaHp1lqQBZdKdR", "book-of-the-dead-bestiary:8qB0gj8salw8746I"]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("rural_setting", { category: "creature" })).toContain(
      "book-of-the-dead-bestiary:7WqlOvjoqURmeorA",
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("rural_setting", { category: "creature" })).toContain(
      "fall-of-plaguestone-bestiary:BgBTntoz1qQ3h1X5",
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("small_settlement_setting", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "extinction-curse-bestiary:OCrQtfKDFpLedE13",
        "extinction-curse-bestiary:YS8UvVGFgHP2TrY3",
      ]),
    );

    const starwatchCommandoDerivation = deriveRecordTagDerivation({
      recordKey: "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
      name: "Starwatch Commando",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(starwatchCommandoDerivation.tags).toEqual(
      expect.arrayContaining(["profession_npc", "civic_npc", "enforcer_npc"]),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      starwatchCommandoDerivation.sources.get("profession_npc"),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      starwatchCommandoDerivation.sources.get("civic_npc"),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      starwatchCommandoDerivation.sources.get("enforcer_npc"),
    );

    const falsePriestDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      name: "False Priest",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid"],
    });
    expect(falsePriestDerivation.tags).toEqual(expect.arrayContaining(["profession_npc", "enforcer_npc"]));
    expect(["assignment", "legacy_rule+assignment"]).toContain(falsePriestDerivation.sources.get("profession_npc"));
    expect(["assignment", "legacy_rule+assignment"]).toContain(falsePriestDerivation.sources.get("enforcer_npc"));

    const commanderArsiellaDerivation = deriveRecordTagDerivation({
      recordKey: "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      name: "Commander Arsiella Dei",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aiuvarin", "elf", "human", "humanoid"],
    });
    expect(commanderArsiellaDerivation.tags).toEqual(
      expect.arrayContaining(["profession_npc", "civic_npc", "enforcer_npc"]),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      commanderArsiellaDerivation.sources.get("profession_npc"),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      commanderArsiellaDerivation.sources.get("civic_npc"),
    );
    expect(["seed_migration", "legacy_rule+seed_migration"]).toContain(
      commanderArsiellaDerivation.sources.get("enforcer_npc"),
    );

    const spiritboundAluumDerivation = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
      name: "Spiritbound Aluum",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["construct", "mindless", "soulbound"],
    });
    expect(spiritboundAluumDerivation.tags).toContain("urban_setting");
    expect(spiritboundAluumDerivation.sources.get("urban_setting")).toBe("assignment");

    const blackWhaleGuardDerivation = deriveRecordTagDerivation({
      recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
      name: "Black Whale Guard",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(blackWhaleGuardDerivation.tags).toContain("nautical_setting");
    expect(blackWhaleGuardDerivation.sources.get("nautical_setting")).toBe("assignment");

    const mengkareDerivation = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:6AN7eagk2WrWc4im",
      name: "Mengkare",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "evil", "fire", "lawful"],
    });
    expect(mengkareDerivation.tags).toContain("island_setting");
    expect(mengkareDerivation.sources.get("island_setting")).toBe("seed_migration");

    const dromaarCompanyDerivation = deriveRecordTagDerivation({
      recordKey: "battlecry-bestiary:R1Ukw41ygDmnAmJk",
      name: "Dromaar Company",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dromaar", "human", "humanoid", "orc", "troop"],
    });
    expect(dromaarCompanyDerivation.tags).toEqual(expect.arrayContaining(["battlefield_setting", "enforcer_npc"]));
    expect(dromaarCompanyDerivation.sources.get("battlefield_setting")).toBe("legacy_rule+seed_migration");
    expect(dromaarCompanyDerivation.sources.get("enforcer_npc")).toBe("seed_migration");

    const qadiranCamelCorpsDerivation = deriveRecordTagDerivation({
      recordKey: "battlecry-bestiary:egHaHp1lqQBZdKdR",
      name: "Qadiran Camel Corps",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["animal", "human", "humanoid", "troop"],
    });
    expect(qadiranCamelCorpsDerivation.tags).toContain("desert_setting");
    expect(qadiranCamelCorpsDerivation.sources.get("desert_setting")).toBe("seed_migration");

    const chargharDerivation = deriveRecordTagDerivation({
      recordKey: "blood-lords-bestiary:vUv8SR7Pa5fONiuN",
      name: "Charghar",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["evil", "incorporeal", "spirit", "undead", "unholy"],
    });
    expect(chargharDerivation.tags).toContain("organized_undead_society_setting");
    expect(chargharDerivation.sources.get("organized_undead_society_setting")).toBe("seed_migration");

    const commanderArsiellaSettingDerivation = deriveRecordTagDerivation({
      recordKey: "claws-of-the-tyrant-bestiary:tMqtId1TKVUXe4tN",
      name: "Commander Arsiella Dei",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aiuvarin", "elf", "human", "humanoid"],
    });
    expect(commanderArsiellaSettingDerivation.tags).toContain("undead_war_torn_region_setting");
    expect(commanderArsiellaSettingDerivation.sources.get("undead_war_torn_region_setting")).toBe("seed_migration");

    const deathCoachDerivation = deriveRecordTagDerivation({
      recordKey: "book-of-the-dead-bestiary:7WqlOvjoqURmeorA",
      name: "Death Coach",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["evil", "incorporeal", "spirit", "undead", "unholy"],
    });
    expect(deathCoachDerivation.tags).toContain("rural_setting");
    expect(deathCoachDerivation.sources.get("rural_setting")).toBe("seed_migration");

    const naiYanFeiDerivation = deriveRecordTagDerivation({
      recordKey: "fists-of-the-ruby-phoenix-bestiary:HqN4nUnl75foBKLZ",
      name: "Nai Yan Fei",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(naiYanFeiDerivation.tags).toContain("tian_xia_setting");
    expect(naiYanFeiDerivation.sources.get("tian_xia_setting")).toBe("seed_migration");

    const drunkenFarmerDerivation = deriveRecordTagDerivation({
      recordKey: "fall-of-plaguestone-bestiary:BgBTntoz1qQ3h1X5",
      name: "Drunken Farmer",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["good", "human", "humanoid"],
    });
    expect(drunkenFarmerDerivation.tags).toContain("rural_setting");
    expect(drunkenFarmerDerivation.sources.get("rural_setting")).toBe("seed_migration");

    const shoonyHierarchDerivation = deriveRecordTagDerivation({
      recordKey: "extinction-curse-bestiary:YS8UvVGFgHP2TrY3",
      name: "Shoony Hierarch",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["good", "humanoid", "shoony"],
    });
    expect(shoonyHierarchDerivation.tags).toContain("small_settlement_setting");
    expect(shoonyHierarchDerivation.sources.get("small_settlement_setting")).toBe("seed_migration");

    const touchedCreatureTags = ["dragon_spellcaster", "disguised_pretender", "faceless_horror", "regeneration_threat"];
    const rawSeedAdds = touchedCreatureTags.reduce(
      (count, tag) => count + getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "creature" }).length,
      0,
    );
    const seededCreatureRecords = new Set(
      touchedCreatureTags.flatMap((tag) => getDerivedTagLegacySeedMigrationRecordKeys(tag, { category: "creature" })),
    );

    expect(rawSeedAdds).toBeGreaterThanOrEqual(120);
    expect(seededCreatureRecords.size).toBeGreaterThanOrEqual(110);
    expect(getDerivedTagLegacySeedMigrationRecordKeys("dragon_spellcaster", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "pathfinder-bestiary:pFmaszqtsA2yt7dv",
        "pathfinder-monster-core:T0OAOkmk4xz0wvjJ",
        "lost-omens-bestiary:IG23I5XqPXeICaOH",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("disguised_pretender", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
        "pathfinder-monster-core:T0OAOkmk4xz0wvjJ",
        "lost-omens-bestiary:E4qscYn7U3jHoCia",
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("faceless_horror", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "season-of-ghosts-bestiary:QSa1PbcvbgDv8Zpr",
        "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
        "season-of-ghosts-bestiary:3KNblm2fWM6XLiS7",
      ]),
    );
    expect(listDerivedTagLegacySeedMigrations({ category: "creature" })).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          category: "creature",
          tag: "faceless_horror",
          recordKeys: matcherArrayContaining([
            "season-of-ghosts-bestiary:QSa1PbcvbgDv8Zpr",
            "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
            "season-of-ghosts-bestiary:3KNblm2fWM6XLiS7",
          ]),
        }),
      ]),
    );
    expect(getDerivedTagLegacySeedMigrationRecordKeys("regeneration_threat", { category: "creature" })).toEqual(
      expect.arrayContaining([
        "pathfinder-monster-core-2:yHduMu4VBVUHnssz",
        "stolen-fate-bestiary:KgwkUtJ8czIC0KFj",
        "pathfinder-monster-core-2:1LBt5H8GekcuzDHw",
      ]),
    );

    const redDragonDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-bestiary:pFmaszqtsA2yt7dv",
      name: "Red Dragon (Adult, Spellcaster)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "fire", "evil"],
    });
    expect(redDragonDerivation.tags).toContain("dragon_spellcaster");
    expect(redDragonDerivation.sources.get("dragon_spellcaster")).toBe("seed_migration");

    const conspiratorDragonDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
      name: "Conspirator Dragon (Adult)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["dragon", "occult"],
    });
    expect(conspiratorDragonDerivation.tags).toContain("disguised_pretender");
    expect(["assignment", "legacy_rule+assignment"]).toContain(
      conspiratorDragonDerivation.sources.get("disguised_pretender"),
    );

    const nopperaBoDivineDerivation = deriveRecordTagDerivation({
      recordKey: "season-of-ghosts-bestiary:dqsQutshiegWaFPQ",
      name: "Noppera-Bo Impersonator (Divine)",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["aberration", "chaotic", "evil"],
    });
    expect(nopperaBoDivineDerivation.tags).toEqual(expect.arrayContaining(["disguised_pretender", "faceless_horror"]));
    expect(nopperaBoDivineDerivation.sources.get("disguised_pretender")).toBe("seed_migration");
    expect(nopperaBoDivineDerivation.sources.get("faceless_horror")).toBe("seed_migration");

    const cavernTrollDerivation = deriveRecordTagDerivation({
      recordKey: "pathfinder-monster-core-2:yHduMu4VBVUHnssz",
      name: "Cavern Troll",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["earth", "giant", "humanoid", "troll"],
    });
    expect(cavernTrollDerivation.tags).toContain("regeneration_threat");
    expect(cavernTrollDerivation.sources.get("regeneration_threat")).toBe("seed_migration");
  });
});
