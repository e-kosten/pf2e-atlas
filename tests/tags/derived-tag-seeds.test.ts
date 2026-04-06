import { describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry } from "../../src/types.js";
import {
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
        seedRecordKeys: ["equipment:mask", "equipment:veil"],
        excludeSeedRecordKeys: ["equipment:blocked"],
      },
      {
        value: "concealment",
        description: "Provides concealment or concealability.",
        seedRecordKeys: ["equipment:cloak"],
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
        seedRecordKeys: ["spell:illusory-disguise"],
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
        seedRecordKeys: ["creature:masked-priest"],
      },
    ],
  },
];

describe("derived tag seeds", () => {
  it("publishes family tags without copying child seeds onto the promoted family", () => {
    expect(publishDerivedTagCatalog(seedCatalog)).toEqual(expect.arrayContaining([
      expect.objectContaining({
        family: "infiltration",
        tags: expect.arrayContaining([
          expect.objectContaining({
            value: "disguise",
            seedRecordKeys: ["equipment:mask", "equipment:veil"],
            excludeSeedRecordKeys: ["equipment:blocked"],
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
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog);

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
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog);
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
    const seedIndex = buildDerivedTagSeedIndex(seedCatalog);
    const derivation = deriveCatalogTagDerivation(seedCatalog, seedIndex, {
      recordKey: "equipment:blocked",
      category: "equipment",
      subcategory: null,
    }, ["disguise"]);

    expect(derivation.tags).toEqual(["disguise", "infiltration"]);
    expect(derivation.sources.get("disguise")).toBe("rule");
    expect(derivation.sources.get("infiltration")).toBe("rule");
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
});
