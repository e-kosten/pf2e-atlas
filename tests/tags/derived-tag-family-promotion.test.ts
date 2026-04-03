import { describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry } from "../../src/types.js";
import { applyPromotedFamilyTags, publishDerivedTagCatalog } from "../../src/tags/catalog-utils.js";

describe("derived tag family promotion", () => {
  const catalog: DerivedTagCatalogEntry[] = [
    {
      category: "spell",
      family: "transformation",
      description: "Spells that alter a target's body or form.",
      promoteFamilyToTag: true,
      tags: [
        { value: "battle_form", description: "Combat-oriented form change." },
        { value: "animal_form", description: "Animal or beast shapechange." },
      ],
    },
    {
      category: "equipment",
      subcategories: ["ammo"],
      family: "ammunition_payload",
      description: "Ammunition payload effects.",
      promoteFamilyToTag: true,
      tags: [
        { value: "elemental_payload", description: "Deals elemental payload damage." },
      ],
    },
    {
      category: "creature",
      family: "motif",
      description: "Creature motif tags.",
      tags: [
        { value: "mask_motif", description: "Masked or mask-centric creature." },
      ],
    },
  ];

  it("publishes promoted family tags alongside child tags", () => {
    expect(publishDerivedTagCatalog(catalog)).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "spell",
        family: "transformation",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "battle_form" }),
          expect.objectContaining({ value: "animal_form" }),
          expect.objectContaining({
            value: "transformation",
            description: "Spells that alter a target's body or form.",
          }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "motif",
        tags: expect.not.arrayContaining([
          expect.objectContaining({ value: "motif" }),
        ]),
      }),
    ]));
  });

  it("adds promoted family tags when a child tag matched", () => {
    expect(applyPromotedFamilyTags(catalog, {
      category: "spell",
      subcategory: null,
    }, ["animal_form"])).toEqual(["animal_form", "transformation"]);
  });

  it("respects category and subcategory boundaries for family promotion", () => {
    expect(applyPromotedFamilyTags(catalog, {
      category: "equipment",
      subcategory: "ammo",
    }, ["elemental_payload"])).toEqual(["ammunition_payload", "elemental_payload"]);

    expect(applyPromotedFamilyTags(catalog, {
      category: "equipment",
      subcategory: "weapon",
    }, ["elemental_payload"])).toEqual(["elemental_payload"]);
  });
});
