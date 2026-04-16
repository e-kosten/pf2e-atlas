import { describe, expect, it } from "vitest";

import { deriveRecordTagsFromRules, type DerivedTagRule } from "../../src/tags/runtime/matcher.js";

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
                all: ["track", "scent"],
                window: 4,
                scope: "description",
              },
            ],
          },
        ],
        noneOf: [
          {
            textNear: [
              {
                all: ["track", "time"],
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
                all: ["pass", "identity"],
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

  it("supports explicit blurb-only text matching without broadening either-scope matching", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "base_species_blurb",
        category: "creature",
        anyOf: [
          {
            textAny: [{ value: "white dragon", scope: "blurb" }],
          },
        ],
      },
      {
        tag: "either_scope_should_ignore_blurb",
        category: "creature",
        anyOf: [
          {
            textAny: ["white dragon"],
          },
        ],
      },
    ];

    const tags = deriveRecordTagsFromRules(rules, {
      name: "Venexus",
      category: "creature",
      subcategory: null,
      descriptionText: "A unique dragon carrying the Primordial Flame.",
      blurbText: "Female young white dragon",
      traits: ["dragon", "cold"],
    });

    expect(tags).toContain("base_species_blurb");
    expect(tags).not.toContain("either_scope_should_ignore_blurb");
  });

  it("supports part-local token analysis on literals and mixed-length alternatives", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "noun_keep",
        category: "creature",
        anyOf: [
          {
            textAny: [{
              scope: "description",
              parts: [
                {
                  type: "literal",
                  value: "keep",
                  analysis: [{ pos: ["NOUN"] }],
                },
              ],
            }],
          },
        ],
      },
      {
        tag: "residence_form",
        category: "creature",
        anyOf: [
          {
            textAny: [{
              scope: "description",
              parts: [
                {
                  type: "alternative",
                  options: [
                    {
                      value: "take residence",
                      analysis: [{ pos: ["VERB"] }, { pos: ["NOUN"] }],
                    },
                    {
                      value: "resides",
                      analysis: [{ pos: ["VERB"] }],
                    },
                  ],
                },
              ],
            }],
          },
        ],
      },
      {
        tag: "plain_keep",
        category: "creature",
        anyOf: [
          {
            textAny: ["keep"],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Keep Guardian",
      category: "creature",
      subcategory: null,
      descriptionText: "The guardian stands within the keep walls.",
      traits: [],
    })).toEqual(expect.arrayContaining(["noun_keep", "plain_keep"]));

    expect(deriveRecordTagsFromRules(rules, {
      name: "Castle Steward",
      category: "creature",
      subcategory: null,
      descriptionText: "These guardians take residence within the old stone bastion.",
      traits: [],
    })).toEqual(expect.arrayContaining(["residence_form"]));

    expect(deriveRecordTagsFromRules(rules, {
      name: "Citadel Ghost",
      category: "creature",
      subcategory: null,
      descriptionText: "The ghost resides within the citadel walls.",
      traits: [],
    })).toEqual(expect.arrayContaining(["residence_form"]));

    expect(deriveRecordTagsFromRules(rules, {
      name: "Shadow Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "These predators keep to the deepest shadows.",
      traits: [],
    })).not.toContain("noun_keep");
  });

  it("supports optional pattern parts with token analysis", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "fortified_residence",
        category: "creature",
        anyOf: [
          {
            textAny: [{
              scope: "description",
              parts: [
                {
                  type: "optional",
                  value: "old",
                  analysis: [{ pos: ["ADJ"] }],
                },
                {
                  type: "literal",
                  value: "keep",
                  analysis: [{ pos: ["NOUN"] }],
                },
              ],
            }],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Keep Warden",
      category: "creature",
      subcategory: null,
      descriptionText: "The sentry patrols the old keep at dusk.",
      traits: [],
    })).toContain("fortified_residence");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Keep Warden",
      category: "creature",
      subcategory: null,
      descriptionText: "The sentry patrols the keep at dusk.",
      traits: [],
    })).toContain("fortified_residence");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Shadow Stalker",
      category: "creature",
      subcategory: null,
      descriptionText: "These predators keep to the deepest shadows.",
      traits: [],
    })).not.toContain("fortified_residence");
  });

  it("supports placeholder analysis", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "numbered_goblins",
        category: "creature",
        anyOf: [
          {
            textAny: [{
              scope: "description",
              parts: [
                {
                  type: "placeholder",
                  value: "number",
                  analysis: [{ pos: ["NUM"] }],
                },
                {
                  type: "literal",
                  value: "goblins",
                },
              ],
            }],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Goblin Scout",
      category: "creature",
      subcategory: null,
      descriptionText: "3 goblins patrol the ravine at dusk.",
      traits: [],
    })).toContain("numbered_goblins");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Goblin Scout",
      category: "creature",
      subcategory: null,
      descriptionText: "Three goblins patrol the ravine at dusk.",
      traits: [],
    })).not.toContain("numbered_goblins");
  });

  it("fails analysis-constrained matches closed on NLP tokenization misalignment without affecting plain matching", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "constrained_cant_keep",
        category: "creature",
        anyOf: [
          {
            textAny: [{
              scope: "description",
              parts: [
                {
                  type: "literal",
                  value: "can t keep",
                  analysis: [{ lemma: ["can"] }, { lemma: ["not"] }, { pos: ["VERB"] }],
                },
              ],
            }],
          },
        ],
      },
      {
        tag: "plain_cant_keep",
        category: "creature",
        anyOf: [
          {
            textAny: ["can t keep"],
          },
        ],
      },
    ];

    const tags = deriveRecordTagsFromRules(rules, {
      name: "Contradictory Sentry",
      category: "creature",
      subcategory: null,
      descriptionText: "The sentry can't keep a steady watch.",
      traits: [],
    });

    expect(tags).toContain("plain_cant_keep");
    expect(tags).not.toContain("constrained_cant_keep");
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

  it("supports pattern anchors for alternatives, optionals, and typed placeholders", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "healing_suppression",
        category: "affliction",
        anyOf: [
          {
            textAny: [
              { value: "{{alt(can't, cannot)}} {{alt(heal damage, be healed)}}", scope: "description" },
              { value: "{{alt(regain, regains)}} {{opt(only )}}half as many hit points from {{alt(healing, healing effects, all healing, all healing effects)}}", scope: "description" },
            ],
          },
        ],
      },
      {
        tag: "mobility",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "jump {{number}} feet", scope: "description" },
              { value: "gain a +{{number}}-foot status bonus to your speed", scope: "description" },
            ],
          },
        ],
      },
      {
        tag: "damage_signature",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "deal {{dice}} damage", scope: "description" },
            ],
          },
        ],
      },
      {
        tag: "area_effect",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "creatures in a {{range}}", scope: "description" },
            ],
          },
        ],
      },
      {
        tag: "range_alert",
        category: "spell",
        anyOf: [
          {
            textNear: [
              {
                all: [
                  { value: "within {{range}}", scope: "description" },
                  "alert",
                ],
                window: 8,
                scope: "description",
              },
            ],
          },
        ],
      },
      {
        tag: "temporary_hp_support",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{alt(gain, gains, grant, grants)}} {{gap(4)}} temporary hit points", scope: "description" },
              { value: "{{alt(gain, gains, grant, grants)}} {{gap(0, 4)}} a buffer of temporary hit points", scope: "description" },
            ],
          },
        ],
      },
    ];

    expect(deriveRecordTagsFromRules(rules, {
      name: "Blackfrost",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "This affliction cannot be healed until the curse is lifted.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Blood Rot",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "The creature regains half as many Hit Points from all healing effects for 1 day.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Jump",
      category: "spell",
      subcategory: null,
      descriptionText: "You jump 60 feet in any direction without touching the ground.",
      traits: [],
    })).toContain("mobility");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Tailwind",
      category: "spell",
      subcategory: null,
      descriptionText: "You gain a +10-foot status bonus to your Speed.",
      traits: [],
    })).toContain("mobility");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Shock Lance",
      category: "spell",
      subcategory: null,
      descriptionText: "You deal 4d8 damage to the target.",
      traits: [],
    })).toContain("damage_signature");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Shock Lance",
      category: "spell",
      subcategory: null,
      descriptionText: "You deal 10 damage to the target.",
      traits: [],
    })).not.toContain("damage_signature");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Fire Wave",
      category: "spell",
      subcategory: null,
      descriptionText: "Creatures in a 30-foot cone take fire damage.",
      traits: [],
    })).toContain("area_effect");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Alarm Pulse",
      category: "spell",
      subcategory: null,
      descriptionText: "A bell rings when creatures move within 30-foot emanation of the ward to alert nearby guards.",
      traits: [],
    })).toContain("range_alert");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Quick March",
      category: "spell",
      subcategory: null,
      descriptionText: "You gain a +1 status bonus for 1 minute.",
      traits: [],
    })).not.toContain("mobility");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Watchful Ward",
      category: "spell",
      subcategory: null,
      descriptionText: "A bell rings when creatures move within 1 minute of the ward to alert nearby guards.",
      traits: [],
    })).not.toContain("range_alert");

    expect(deriveRecordTagsFromRules(rules, {
      name: "False Vitality",
      category: "spell",
      subcategory: null,
      descriptionText: "The spell grants the target temporary hit points, surrounding them with a buffer of temporary hit points.",
      traits: [],
    })).toContain("temporary_hp_support");

    expect(deriveRecordTagsFromRules(rules, {
      name: "Stasis Coil",
      category: "spell",
      subcategory: null,
      descriptionText: "The target is trapped in a suspended state and its container has 40 Hit Points.",
      traits: [],
    })).not.toContain("temporary_hp_support");
  });

  it("rejects invalid pattern syntax", () => {
    const rules: DerivedTagRule[] = [
      {
        tag: "broken",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{alt(cannot, )}} heal damage", scope: "description" },
            ],
          },
        ],
      },
    ];

    expect(() => deriveRecordTagsFromRules(rules, {
      name: "Broken Ward",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/Invalid pattern anchor/);

    expect(() => deriveRecordTagsFromRules([
      {
        tag: "nested",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{opt(alt(cannot, can't))}} heal damage", scope: "description" },
            ],
          },
        ],
      },
    ], {
      name: "Nested Ward",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/nested expressions are not supported/);

    expect(() => deriveRecordTagsFromRules([
      {
        tag: "leading_optional",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{opt(all )}}healing", scope: "description" },
            ],
          },
        ],
      },
    ], {
      name: "Leading Optional",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/leading or trailing opt\(\.\.\.\) or gap\(\.\.\.\) is not supported/);

    expect(() => deriveRecordTagsFromRules([
      {
        tag: "trailing_optional",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "healing{{opt( effects)}}", scope: "description" },
            ],
          },
        ],
      },
    ], {
      name: "Trailing Optional",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/leading or trailing opt\(\.\.\.\) or gap\(\.\.\.\) is not supported/);

    expect(() => deriveRecordTagsFromRules([
      {
        tag: "broken_gap",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{gap(4, 2)}} ward", scope: "description" },
            ],
          },
        ],
      },
    ], {
      name: "Broken Gap",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/gap\(\.\.\.\) minimum cannot exceed maximum/);

    expect(() => deriveRecordTagsFromRules([
      {
        tag: "leading_gap",
        category: "spell",
        anyOf: [
          {
            textAny: [
              { value: "{{gap(4)}} ward", scope: "description" },
            ],
          },
        ],
      },
    ], {
      name: "Leading Gap",
      category: "spell",
      subcategory: null,
      descriptionText: "This text should never matter.",
      traits: [],
    })).toThrow(/leading or trailing opt\(\.\.\.\) or gap\(\.\.\.\) is not supported/);
  });
});
