import { describe, expect, it } from "vitest";

import type {
  DerivedTagLegacySeedMigrationCategory,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
} from "../../src/types.js";
import { buildDerivedTagExplicitAssignmentIndex } from "../../src/tags/runtime/assignments.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  buildDerivedTagSeedLookup,
  deriveCatalogTagDerivation,
  listConfiguredDerivedTagLegacySeedMigrations,
  publishDerivedTagOntology,
} from "../../src/tags/runtime/catalog-utils.js";

const families: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    family: "infiltration",
    description: "Equipment that helps infiltration.",
  },
];

const tags: DerivedTagOntologyTag[] = [
  {
    category: "equipment",
    family: "infiltration",
    tag: "disguise",
    description: "Masks or alters appearance.",
    assignmentMode: "hybrid",
  },
  {
    category: "equipment",
    family: "infiltration",
    tag: "concealment",
    description: "Provides concealment or concealability.",
    assignmentMode: "hybrid",
  },
];

const ontology = publishDerivedTagOntology(families, tags);
const seedLookup = buildDerivedTagSeedLookup([
  { recordKey: "equipment:mask", pack: "equipment-srd", name: "Mask" },
  { recordKey: "equipment:veil", pack: "equipment-srd", name: "Veil" },
  { recordKey: "equipment:blocked", pack: "equipment-srd", name: "Blocked Mask" },
]);

describe("derived tag legacy seed migrations", () => {
  it("keeps migrated live tags separate from exemplars while exposing a reviewable tag list", () => {
    const migrations: DerivedTagLegacySeedMigrationCategory[] = [
      {
        category: "equipment",
        tags: [
          {
            tag: "disguise",
            includeRecords: [
              { pack: "equipment-srd", name: "Mask" },
              { pack: "equipment-srd", name: "Veil" },
            ],
            excludeRecords: [{ pack: "equipment-srd", name: "Blocked Mask" }],
          },
        ],
      },
    ];

    const migrationIndex = buildDerivedTagLegacySeedMigrationIndex(ontology, seedLookup, migrations);

    expect(listConfiguredDerivedTagLegacySeedMigrations(migrationIndex)).toEqual([
      {
        category: "equipment",
        tag: "disguise",
        recordKeys: ["equipment:mask", "equipment:veil"],
        subcategories: undefined,
      },
    ]);

    const migratedMask = deriveCatalogTagDerivation(
      ontology,
      { recordKey: "equipment:mask", category: "equipment", subcategory: null },
      ["concealment"],
      undefined,
      migrationIndex,
    );
    expect(migratedMask.tags).toEqual(["concealment", "disguise"]);
    expect(migratedMask.sources.get("concealment")).toBe("legacy_rule");
    expect(migratedMask.sources.get("disguise")).toBe("seed_migration");

    const blockedMask = deriveCatalogTagDerivation(
      ontology,
      { recordKey: "equipment:blocked", category: "equipment", subcategory: null },
      [],
      undefined,
      migrationIndex,
    );
    expect(blockedMask.tags).toEqual([]);
  });

  it("rejects unknown migration tags and lets explicit assignment exclusions override migrated tags", () => {
    expect(() => buildDerivedTagLegacySeedMigrationIndex(ontology, seedLookup, [
      {
        category: "equipment",
        tags: [
          {
            tag: "unknown_tag",
            includeRecords: [{ pack: "equipment-srd", name: "Mask" }],
          },
        ],
      },
    ])).toThrow(/does not exist in the published ontology/);

    const migrationIndex = buildDerivedTagLegacySeedMigrationIndex(ontology, seedLookup, [
      {
        category: "equipment",
        tags: [
          {
            tag: "disguise",
            includeRecords: [{ pack: "equipment-srd", name: "Mask" }],
          },
        ],
      },
    ]);
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(ontology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Mask",
            recordKey: "equipment:mask",
            excluded: { infiltration: ["disguise"] },
            review: {
              infiltration: {
                disguise: {
                  mode: "exclude",
                  status: "approved",
                  rationale: "This record is being intentionally suppressed during review.",
                },
              },
            },
          },
        ],
      },
    ]);

    const derivation = deriveCatalogTagDerivation(
      ontology,
      { recordKey: "equipment:mask", category: "equipment", subcategory: null },
      [],
      assignmentIndex,
      migrationIndex,
    );

    expect(derivation.tags).toEqual([]);
    expect(derivation.sources.get("disguise")).toBeUndefined();
  });
});
