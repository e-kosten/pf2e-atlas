import { describe, expect, it } from "vitest";

import type {
  DerivedTagLegacySeedMigrationCategory,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
  PublishedDerivedTagConceptModel,
} from "../../src/domain/derived-tag-types.js";
import { buildDerivedTagExplicitAssignmentIndex } from "../../src/tags/runtime/derivation/assignments.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  buildDerivedTagSeedLookup,
  deriveCatalogTagDerivation,
  listConfiguredDerivedTagLegacySeedMigrations,
  publishDerivedTagOntology,
} from "../../src/tags/runtime/publication/catalog.js";

const families: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    family: "infiltration",
    axis: "utility",
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

function buildTestConceptModel(tags: DerivedTagOntologyTag[]): PublishedDerivedTagConceptModel {
  const concepts = tags.map((tag) => ({
    id: `${tag.category}:${tag.tag}`,
    label: tag.tag,
    schemaKind: "descriptive" as const,
  }));
  const projections = tags.map((tag) => ({
    id: `${tag.category}:${tag.tag}`,
    conceptId: `${tag.category}:${tag.tag}`,
    category: tag.category,
    axis: "utility" as const,
    family: tag.family,
    currentTag: tag.tag,
    description: tag.description,
    assignmentMode: tag.assignmentMode,
    translationStatus: "mapped" as const,
  }));
  return {
    concepts,
    conceptById: new Map(concepts.map((concept) => [concept.id, concept])),
    projections,
    projectionsById: new Map(projections.map((projection) => [projection.id, projection])),
    projectionsByTagKey: new Map(projections.map((projection) => [`${projection.category}:${projection.currentTag}`, projection] as const)),
    translations: [],
    translationsByTagKey: new Map(),
    relations: [],
  };
}

const ontology = publishDerivedTagOntology(families, tags, buildTestConceptModel(tags));
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
    expect(() =>
      buildDerivedTagLegacySeedMigrationIndex(ontology, seedLookup, [
        {
          category: "equipment",
          tags: [
            {
              tag: "unknown_tag",
              includeRecords: [{ pack: "equipment-srd", name: "Mask" }],
            },
          ],
        },
      ]),
    ).toThrow(/does not exist in the published ontology/);

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
            excluded: [
              {
                projectionId: "equipment:disguise",
                source: "human",
                rationale: "This record is being intentionally suppressed during review.",
              },
            ],
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
