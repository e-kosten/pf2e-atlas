import { describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry } from "../../src/types.js";
import {
  buildDerivedTagSeedIndex,
  deriveCatalogTagDerivation,
  publishDerivedTagCatalog,
  resolveCatalogSeedRecordKeys,
} from "../../src/tags/catalog-utils.js";

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
});
