import { describe, expect, it } from "vitest";

import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../src/domain/derived-tag-types.js";
import {
  deriveCatalogTagDerivation,
  groupDerivedTagOntology,
  publishDerivedTagOntology,
} from "../../src/tags/runtime/publication/catalog.js";

function matcherArrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function matcherObjectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

function matcherNotArrayContaining(values: unknown[]): unknown {
  return expect.not.arrayContaining(values);
}

describe("derived tag ontology publication and composition", () => {
  const families: DerivedTagOntologyFamily[] = [
    {
      category: "spell",
      family: "transformation",
      axis: "transformation",
      description: "Spells that alter a target's body or form.",
    },
    {
      category: "equipment",
      subcategories: ["ammo"],
      family: "ammunition_payload",
      axis: "item_mechanical",
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
    expect(grouped).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          category: "spell",
          family: "transformation",
          axis: "transformation",
          tags: matcherArrayContaining([
            matcherObjectContaining({
              value: "transformation",
              assignmentMode: "composite",
              compositeOfAnyTags: ["battle_form", "animal_form"],
            }),
            matcherObjectContaining({
              value: "battle_form",
              assignmentMode: "deterministic",
            }),
            matcherObjectContaining({
              value: "animal_form",
              assignmentMode: "deterministic",
            }),
          ]),
        }),
        matcherObjectContaining({
          category: "equipment",
          family: "ammunition_payload",
          axis: "item_mechanical",
          tags: matcherNotArrayContaining([matcherObjectContaining({ value: "ammunition_payload" })]),
        }),
      ]),
    );
  });

  it("derives composite tags from matching child tags without family promotion", () => {
    const ontology = publishDerivedTagOntology(families, tags);
    const derivation = deriveCatalogTagDerivation(ontology, { recordKey: null, category: "spell", subcategory: null }, [
      "animal_form",
    ]);

    expect(derivation.tags).toEqual(["animal_form", "transformation"]);
    expect(derivation.sources.get("animal_form")).toBe("legacy_rule");
    expect(derivation.sources.get("transformation")).toBe("legacy_rule");
  });
});
