import { describe, expect, it } from "vitest";

import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../src/types.js";
import {
  buildDerivedTagSeedIndex,
  buildDerivedTagSeedLookup,
  deriveCatalogTagDerivation,
  groupDerivedTagOntology,
  publishDerivedTagOntology,
} from "../../src/tags/catalog-utils.js";

describe("derived tag ontology publication and composition", () => {
  const families: DerivedTagOntologyFamily[] = [
    {
      category: "spell",
      family: "transformation",
      description: "Spells that alter a target's body or form.",
    },
    {
      category: "equipment",
      subcategories: ["ammo"],
      family: "ammunition_payload",
      description: "Ammunition payload effects.",
    },
  ];

  const tags: DerivedTagOntologyTag[] = [
    {
      category: "spell",
      family: "transformation",
      tag: "transformation",
      description: "Spells that alter a target's body or form.",
      compositeOfAnyTags: ["battle_form", "animal_form"],
      assignmentMode: "composite",
      adjacentTags: ["battle_form", "animal_form"],
    },
    {
      category: "spell",
      family: "transformation",
      tag: "battle_form",
      description: "Combat-oriented form change.",
      assignmentMode: "deterministic",
    },
    {
      category: "spell",
      family: "transformation",
      tag: "animal_form",
      description: "Animal or beast shapechange.",
      assignmentMode: "deterministic",
    },
    {
      category: "equipment",
      family: "ammunition_payload",
      tag: "elemental_payload",
      description: "Deals elemental payload damage.",
      assignmentMode: "deterministic",
    },
  ];

  it("publishes explicit ontology records and groups them without auto-promoting the family name", () => {
    const grouped = groupDerivedTagOntology(publishDerivedTagOntology(families, tags));
    expect(grouped).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "spell",
        family: "transformation",
        tags: expect.arrayContaining([
          expect.objectContaining({
            value: "transformation",
            assignmentMode: "composite",
            compositeOfAnyTags: ["battle_form", "animal_form"],
          }),
          expect.objectContaining({
            value: "battle_form",
            assignmentMode: "deterministic",
          }),
          expect.objectContaining({
            value: "animal_form",
            assignmentMode: "deterministic",
          }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "ammunition_payload",
        tags: expect.not.arrayContaining([
          expect.objectContaining({ value: "ammunition_payload" }),
        ]),
      }),
    ]));
  });

  it("derives composite tags from matching child tags without family promotion", () => {
    const ontology = publishDerivedTagOntology(families, tags);
    const derivation = deriveCatalogTagDerivation(
      ontology,
      buildDerivedTagSeedIndex(ontology, buildDerivedTagSeedLookup([])),
      { recordKey: null, category: "spell", subcategory: null },
      ["animal_form"],
    );

    expect(derivation.tags).toEqual(["animal_form", "transformation"]);
    expect(derivation.sources.get("animal_form")).toBe("rule");
    expect(derivation.sources.get("transformation")).toBe("rule");
  });
});
