import { describe, expect, it } from "vitest";

import type {
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
  PublishedDerivedTagConceptModel,
} from "../../src/domain/derived-tag-types.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  buildDerivedTagPendingAssignmentViews,
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagAssignmentMemory,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
  type AuthoredDerivedTagAssignment,
} from "../../src/tags/runtime/derivation/assignments.js";
import { getDerivedTagCanonicalOntology } from "../../src/tags/canonical/index.js";
import { deriveRecordTagDerivation } from "../../src/tags/runtime.js";
import { publishDerivedTagOntology } from "../../src/tags/runtime/publication/catalog.js";

const assignmentFamilies: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    family: "infiltration",
    axis: "utility",
    description: "Equipment that helps infiltration.",
  },
  {
    category: "equipment",
    family: "security",
    axis: "utility",
    description: "Equipment that supports security.",
  },
  {
    category: "equipment",
    family: "reconnaissance",
    axis: "utility",
    description: "Equipment that supports scouting and tracking.",
  },
];

const assignmentTags: DerivedTagOntologyTag[] = [
  {
    category: "equipment",
    family: "infiltration",
    tag: "disguise",
    description: "Alters appearance.",
    assignmentMode: "deterministic",
  },
  {
    category: "equipment",
    family: "infiltration",
    tag: "social_infiltration",
    description: "Blends into social spaces.",
    assignmentMode: "deterministic",
  },
  {
    category: "equipment",
    family: "security",
    tag: "alarm",
    description: "Warns of intruders.",
    assignmentMode: "deterministic",
  },
  {
    category: "equipment",
    family: "reconnaissance",
    tag: "scouting",
    description: "Supports observation and recon.",
    assignmentMode: "deterministic",
  },
  {
    category: "equipment",
    family: "reconnaissance",
    tag: "tracking",
    description: "Supports following and relocating targets.",
    assignmentMode: "deterministic",
  },
  {
    category: "equipment",
    family: "reconnaissance",
    tag: "reconnaissance",
    description: "Broad scouting umbrella.",
    assignmentMode: "composite",
    compositeOfAnyTags: ["scouting", "tracking"],
  },
];

function buildTestConceptModel(tags: DerivedTagOntologyTag[]): PublishedDerivedTagConceptModel {
  const concepts = tags.map((tag) => ({
    id: `${tag.category}:${tag.tag}`,
    label: tag.tag,
    schemaKind: tag.assignmentMode === "composite" ? ("aggregate" as const) : ("descriptive" as const),
  }));
  const projections = tags.map((tag) => ({
    id: `${tag.category}:${tag.tag}`,
    conceptId: `${tag.category}:${tag.tag}`,
    category: tag.category,
    axis: assignmentFamilies.find((family) => family.category === tag.category && family.family === tag.family)?.axis ?? "utility",
    family: tag.family,
    currentTag: tag.tag,
    label: tag.label,
    description: tag.description,
    assignmentMode: tag.assignmentMode,
    compositeOfAnyTags: tag.compositeOfAnyTags,
    translationStatus: "mapped" as const,
  }));

  return {
    concepts,
    conceptById: new Map(concepts.map((concept) => [concept.id, concept])),
    projections,
    projectionsById: new Map(projections.map((projection) => [projection.id, projection])),
    projectionsByTagKey: new Map(
      projections.map((projection) => [`${projection.category}:${projection.currentTag}`, projection] as const),
    ),
    translations: [],
    translationsByTagKey: new Map(),
    relations: [],
  };
}

const assignmentOntology = publishDerivedTagOntology(
  assignmentFamilies,
  assignmentTags,
  buildTestConceptModel(assignmentTags),
);
const canonicalOntology = getDerivedTagCanonicalOntology();
const creatureOntology = publishDerivedTagOntology(
  canonicalOntology.families.filter((family) => family.category === "creature"),
  canonicalOntology.tags.filter((tag) => tag.category === "creature"),
  canonicalOntology.conceptModel,
);

function equipmentRecords(assignments: AuthoredDerivedTagAssignment[]): Array<{
  recordKey: string;
  name: string;
  category: "equipment";
}> {
  return assignments.map((assignment) => ({
    recordKey: assignment.recordKey,
    name: assignment.name,
    category: "equipment",
  }));
}

function buildEquipmentAssignmentIndex(assignments: AuthoredDerivedTagAssignment[]) {
  return buildDerivedTagExplicitAssignmentIndex(assignmentOntology, assignments, equipmentRecords(assignments));
}

describe("derived tag explicit assignments", () => {
  it("resolves tag-authored applied and excluded assignments into concrete include and exclude tags", () => {
    const assignmentIndex = buildEquipmentAssignmentIndex([
      {
        name: "Masquerade Mask",
        recordKey: "equipment:mask",
        applied: [
          {
            tag: "social_infiltration",
            source: "human",
            confidence: "high",
            rationale: "Tailored for moving through social spaces while disguised.",
          },
        ],
        excluded: [
          {
            tag: "disguise",
            source: "human",
            confidence: "high",
            rationale: "This record is using a narrower social fit than a broad disguise bucket.",
          },
        ],
      },
    ]);

    expect(assignmentIndex.assignmentsByRecordKey.get("equipment:mask")).toEqual({
      category: "equipment",
      name: "Masquerade Mask",
      includeTags: ["social_infiltration"],
      excludeTags: ["disguise"],
    });
  });

  it("rejects unknown tags, wrong-category tags, duplicate placements, conflicts, and composite tags", () => {
    expect(() =>
      buildEquipmentAssignmentIndex([
        {
          name: "Masquerade Mask",
          recordKey: "equipment:mask",
          applied: [{ tag: "does_not_exist", source: "human", rationale: "Invalid tag." }],
        },
      ]),
    ).toThrow(/does not exist/);

    expect(() =>
      buildEquipmentAssignmentIndex([
        {
          name: "Masquerade Mask",
          recordKey: "equipment:mask",
          applied: [{ tag: "urban_setting", source: "human", rationale: "Invalid category placement." }],
        },
      ]),
    ).toThrow(/does not exist in category/);

    expect(() =>
      buildEquipmentAssignmentIndex([
        {
          name: "Masquerade Mask",
          recordKey: "equipment:mask",
          applied: [{ tag: "social_infiltration", source: "human", rationale: "Conflicting placement." }],
          excluded: [{ tag: "social_infiltration", source: "human", rationale: "Conflicting placement." }],
        },
      ]),
    ).toThrow(/both applied and excluded/);

    expect(() =>
      buildEquipmentAssignmentIndex([
        {
          name: "Watch Bell",
          recordKey: "equipment:bell",
          applied: [
            { tag: "alarm", source: "human", rationale: "Duplicate live entries should be rejected." },
            { tag: "alarm", source: "human", rationale: "Duplicate live entries should be rejected." },
          ],
        },
      ]),
    ).toThrow(/repeats/);

    expect(() =>
      buildEquipmentAssignmentIndex([
        {
          name: "Spyglass Kit",
          recordKey: "equipment:spyglass-kit",
          applied: [{ tag: "reconnaissance", source: "human", rationale: "Composite tags should not be assigned." }],
        },
      ]),
    ).toThrow(/cannot target composite tag/i);
  });

  it("validates record-backed assignment names and missing record summaries", () => {
    const assignments: AuthoredDerivedTagAssignment[] = [
      {
        name: "Masquerade Mask",
        recordKey: "equipment:mask",
        applied: [{ tag: "social_infiltration", source: "human", rationale: "Core assignment." }],
      },
    ];
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, assignments);

    expect(() => validateDerivedTagExplicitAssignmentsAgainstRecords([], assignmentIndex)).not.toThrow();
    expect(() =>
      buildDerivedTagExplicitAssignmentIndex(assignmentOntology, assignments, [
        { recordKey: "equipment:mask", name: "Different Name", category: "equipment" },
      ]),
    ).toThrow(/expected name/);
    expect(() =>
      buildDerivedTagExplicitAssignmentIndex(assignmentOntology, assignments, [
        { recordKey: "equipment:other", name: "Other", category: "equipment" },
      ], { requireCompleteRecordCoverage: true }),
    ).toThrow(/Cannot resolve explicit derived tag assignment category/);
  });

  it("keeps pending assignment reviews and memory separate from live assignments", () => {
    expect(
      buildDerivedTagPendingAssignmentViews(assignmentOntology, [
        {
          category: "equipment",
          decisions: [
            {
              name: "Watch Bell",
              recordKey: "equipment:bell",
              family: "security",
              tag: "alarm",
              mode: "include",
              confidence: "medium",
              rationale: "Likely security-oriented.",
            },
          ],
        },
      ]),
    ).toEqual([
      {
        name: "Watch Bell",
        recordKey: "equipment:bell",
        pending: {
          security: ["alarm"],
        },
      },
    ]);

    expect(() =>
      validateDerivedTagAssignmentMemory(assignmentOntology, [
        {
          category: "equipment",
          decisions: [
            {
              name: "Watch Bell",
              recordKey: "equipment:bell",
              family: "security",
              tag: "alarm",
              mode: "include",
              rationale: "One memory entry.",
            },
          ],
        },
      ]),
    ).not.toThrow();
  });

  it("applies configured pack-authored assignments during live derivation", () => {
    const falsePriest = deriveRecordTagDerivation({
      recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      name: "False Priest",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid"],
    });
    expect(falsePriest.tags).toEqual(expect.arrayContaining(["profession_npc", "enforcer_npc"]));
    expect(falsePriest.tags).not.toContain("civic_npc");

    const blackWhaleGuard = deriveRecordTagDerivation({
      recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
      name: "Black Whale Guard",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid", "lawful"],
    });
    expect(blackWhaleGuard.tags).toContain("nautical_setting");
    expect(blackWhaleGuard.sources.get("nautical_setting")).toBe("assignment");
  });

  it("validates configured assignments against canonical creature records", () => {
    expect(() => createDerivedTagExplicitAssignmentIndex(creatureOntology)).not.toThrow();
  });
});
