import { describe, expect, it } from "vitest";

import type {
  AuthoredDerivedTagRule,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
} from "../../src/types.js";
import {
  deriveCatalogTagDerivation,
  publishDerivedTagOntology,
} from "../../src/tags/runtime/catalog-utils.js";
import { compileAuthoredDerivedTagRules } from "../../src/tags/authored-rules/compiler.js";
import { deriveRecordTagsFromRules, type DerivedTagRule } from "../../src/tags/runtime/matcher.js";

describe("authored derived tag rules", () => {
  const families: DerivedTagOntologyFamily[] = [
    {
      category: "creature",
      family: "setting",
      axis: "setting",
      description: "Creature setting tags.",
    },
    {
      category: "creature",
      family: "motif",
      axis: "presentation",
      description: "Creature motif tags.",
    },
    {
      category: "spell",
      family: "support",
      axis: "support",
      description: "Spell support tags.",
    },
  ];

  const tags: DerivedTagOntologyTag[] = [
    {
      category: "creature",
      family: "setting",
      tag: "plane_of_fire_setting",
      description: "Strongly associated with the Plane of Fire.",
      assignmentMode: "deterministic",
    },
    {
      category: "creature",
      family: "motif",
      tag: "legacy_phrase_tag",
      description: "Legacy matcher phrase coverage.",
      assignmentMode: "deterministic",
    },
    {
      category: "spell",
      family: "support",
      tag: "linked_support",
      description: "References linked support records.",
      assignmentMode: "deterministic",
    },
    {
      category: "creature",
      family: "setting",
      tag: "elemental_plane_setting",
      description: "Aggregates elemental plane settings.",
      assignmentMode: "composite",
      compositeOfAnyTags: ["plane_of_fire_setting"],
    },
  ];

  const ontology = publishDerivedTagOntology(families, tags);

  it("compiles constrained authored rules and keeps authored and legacy provenance distinct", () => {
    const authoredRules: AuthoredDerivedTagRule[] = [
      {
        tag: "plane_of_fire_setting",
        category: "creature",
        kind: "trait_match",
        intent: "deterministic",
        when: {
          traitsAll: ["elemental", "fire"],
        },
      },
      {
        tag: "linked_support",
        category: "spell",
        kind: "reference_match",
        intent: "deterministic",
        when: {
          referencesWhere: [
            {
              category: "rule",
              subcategory: "action",
              nameAny: ["Treat Wounds"],
            },
          ],
          minReferenceMatches: 1,
        },
      },
    ];
    const compiledAuthoredRules = compileAuthoredDerivedTagRules(ontology, authoredRules);
    const legacyRules: DerivedTagRule[] = [
      {
        tag: "legacy_phrase_tag",
        category: "creature",
        anyOf: [
          {
            textAny: ["old world phrase"],
          },
        ],
      },
    ];

    const creatureInput = {
      name: "Cinder Wisp",
      category: "creature" as const,
      subcategory: null,
      descriptionText: "An elemental scout still whispers the old world phrase to mark safe portals.",
      traits: ["elemental", "fire"],
    };
    const authoredRuleTags = deriveRecordTagsFromRules(compiledAuthoredRules, creatureInput);
    const legacyRuleTags = deriveRecordTagsFromRules(legacyRules, creatureInput);

    const derivation = deriveCatalogTagDerivation(
      ontology,
      { recordKey: null, category: "creature", subcategory: null },
      {
        authoredRuleTags,
        legacyRuleTags,
      },
    );

    expect(derivation.tags).toEqual([
      "elemental_plane_setting",
      "legacy_phrase_tag",
      "plane_of_fire_setting",
    ]);
    expect(derivation.sources.get("plane_of_fire_setting")).toBe("authored_rule");
    expect(derivation.sources.get("legacy_phrase_tag")).toBe("legacy_rule");
    expect(derivation.sources.get("elemental_plane_setting")).toBe("authored_rule");

    const spellTags = deriveRecordTagsFromRules(compiledAuthoredRules, {
      name: "Battlefield Triage",
      category: "spell",
      subcategory: null,
      descriptionText: "A tactical ward for triage teams.",
      traits: [],
      references: [
        {
          recordKey: "actionspf2e:treat-wounds",
          packName: "actionspf2e",
          name: "Treat Wounds",
          category: "rule",
          subcategory: "action",
          traits: ["healing"],
        },
      ],
    });
    expect(spellTags).toEqual(["linked_support"]);
  });

  it("rejects matcher pattern syntax in authored exact text rules", () => {
    const authoredRules: AuthoredDerivedTagRule[] = [
      {
        tag: "legacy_phrase_tag",
        category: "creature",
        kind: "exact_text_match",
        intent: "deterministic",
        when: {
          textAny: ["{{alt(old,new)}} phrase"],
        },
      },
    ];

    expect(() => compileAuthoredDerivedTagRules(ontology, authoredRules)).toThrow(/cannot use matcher pattern syntax/);
  });

  it("rejects authored composite rules that duplicate ontology composites", () => {
    const authoredRules: AuthoredDerivedTagRule[] = [
      {
        tag: "elemental_plane_setting",
        category: "creature",
        kind: "composite_tag",
        intent: "deterministic",
        when: {
          anyTags: ["plane_of_fire_setting"],
        },
      },
    ];

    expect(() => compileAuthoredDerivedTagRules(ontology, authoredRules)).toThrow(/duplicates ontology composite behavior/);
  });
});
