import { describe, expect, it } from "vitest";

import { deriveRecordTagsFromRules, type DerivedTagRule } from "../../src/tags/matcher.js";

describe("derived tag matcher extensions", () => {
  it("supports minimum text matches plus negative trait and family filters", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "field_kit",
        category: "equipment",
        subcategories: ["gear"],
        anyOf: [
          {
            textAny: ["rope", "piton", "hook"],
            minTextAnyMatches: 2,
            traitsNone: ["magical"],
            familiesNone: ["construct"],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Climber's Bundle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This bundle includes rope, a hook, and other climbing supplies.",
      traits: [],
    })).toContain("field_kit");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Arcane Bundle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This bundle includes rope, a hook, and other climbing supplies.",
      traits: ["magical"],
    })).not.toContain("field_kit");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Construct Bundle",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "This bundle includes rope, a hook, and other climbing supplies.",
      traits: [],
      families: ["construct"],
    })).not.toContain("field_kit");
  });

  it("supports bounded text proximity and negative proximity blockers", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "tracking",
        category: "equipment",
        subcategories: ["gear"],
        anyOf: [
          {
            textNear: [
              {
                terms: ["track", "scent"],
                window: 4,
                scope: "description",
              },
            ],
            textNotNear: [
              {
                terms: ["track", "time"],
                window: 2,
                scope: "description",
              },
            ],
          },
        ],
      },
      {
        tag: "ordered_phrase",
        category: "spell",
        anyOf: [
          {
            textNear: [
              {
                terms: ["pass", "identity"],
                window: 4,
                ordered: true,
                scope: "description",
              },
            ],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Bloodhound Powder",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Use this powder to track a creature by scent through heavy rain.",
      traits: [],
    })).toContain("tracking");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Chronicle Dial",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "Use this dial to track time by scent marks burned into incense rings.",
      traits: [],
    })).not.toContain("tracking");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Borrowed Station",
      category: "spell",
      subcategory: null,
      descriptionText: "The spell lets you pass as a borrowed identity.",
      traits: [],
    })).toContain("ordered_phrase");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Identity Reversal",
      category: "spell",
      subcategory: null,
      descriptionText: "The spell makes your identity obvious before you can pass unnoticed.",
      traits: [],
    })).not.toContain("ordered_phrase");
  });

  it("supports structured reference predicates and minimum reference matches", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "exploration_combo",
        category: "equipment",
        subcategories: ["consumable"],
        anyOf: [
          {
            referencesWhere: [
              {
                category: "rule",
                subcategory: "action",
                packName: "actionspf2e",
                nameAny: ["Sense Direction", "Track", "Cover Tracks"],
                traitsAll: ["exploration"],
              },
            ],
            minReferenceMatches: 2,
          },
        ],
      },
      {
        tag: "attack_linked",
        category: "equipment",
        subcategories: ["consumable"],
        anyOf: [
          {
            referencesWhere: [
              {
                category: "rule",
                subcategory: "action",
                traitsAny: ["attack"],
              },
            ],
          },
        ],
      },
    ];

    const references = [
      {
        recordKey: "actionspf2e:sense-direction-1",
        packName: "actionspf2e",
        name: "Sense Direction",
        category: "rule" as const,
        subcategory: "action" as const,
        traits: ["exploration"],
      },
      {
        recordKey: "actionspf2e:track-1",
        packName: "actionspf2e",
        name: "Track",
        category: "rule" as const,
        subcategory: "action" as const,
        traits: ["exploration"],
      },
      {
        recordKey: "actionspf2e:escape-1",
        packName: "actionspf2e",
        name: "Escape",
        category: "rule" as const,
        subcategory: "action" as const,
        traits: ["attack"],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Survey Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This tonic helps with route-finding and pursuit.",
      traits: ["consumable"],
      references,
    })).toEqual(expect.arrayContaining(["exploration_combo", "attack_linked"]));

    expect(deriveRecordTagsFromRules(rules, {
      name: "Half Survey Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This tonic helps with route-finding and pursuit.",
      traits: ["consumable"],
      references: references.slice(0, 1),
    })).not.toContain("exploration_combo");
  });
});
