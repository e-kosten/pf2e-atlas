import { describe, expect, it } from "vitest";

import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../src/types.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  buildDerivedTagPendingAssignmentViews,
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "../../src/tags/assignments.js";
import {
  publishDerivedTagOntology,
  buildDerivedTagSeedIndex,
  buildDerivedTagSeedLookup,
  deriveCatalogTagDerivation,
} from "../../src/tags/catalog-utils.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "../../src/tags/ontology/creature.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../../src/tags/ontology/utils.js";
import { deriveRecordTagDerivation } from "../../src/tags/index.js";

const assignmentFamilies: DerivedTagOntologyFamily[] = [
  {
    category: "equipment",
    family: "infiltration",
    description: "Equipment that helps infiltration.",
  },
  {
    category: "equipment",
    family: "security",
    description: "Equipment that supports security.",
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
];

const assignmentOntology = publishDerivedTagOntology(assignmentFamilies, assignmentTags);
const flattenedCreatureOntology = flattenDerivedTagAuthoredCategoryOntology(CREATURE_DERIVED_TAG_ONTOLOGY);
const creatureOntology = publishDerivedTagOntology(
  flattenedCreatureOntology.families,
  flattenedCreatureOntology.tags,
);

describe("derived tag explicit assignments", () => {
  it("flattens applied and excluded assignments into concrete include and exclude tags", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { infiltration: ["social_infiltration"] },
            excluded: { infiltration: ["disguise"] },
            review: {
              infiltration: {
                social_infiltration: {
                  mode: "include",
                  status: "approved",
                  confidence: "high",
                  rationale: "Tailored for moving through social spaces while disguised.",
                },
                disguise: {
                  mode: "exclude",
                  status: "approved",
                  confidence: "high",
                  rationale: "This record is using a narrower social fit than a broad disguise bucket.",
                },
              },
            },
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

  it("rejects unknown families and wrong-family tags", () => {
    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { stealth: ["social_infiltration"] },
          },
        ],
      },
    ])).toThrow(/does not exist/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { security: ["social_infiltration"] },
          },
        ],
      },
    ])).toThrow(/does not belong to family/);
  });

  it("validates canonical name drift only when the record is present in the build", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { infiltration: ["social_infiltration"] },
            review: {
              infiltration: {
                social_infiltration: {
                  mode: "include",
                  status: "approved",
                  rationale: "Core assignment for the masquerade entry.",
                },
              },
            },
          },
        ],
      },
    ]);

    expect(() => validateDerivedTagExplicitAssignmentsAgainstRecords([], assignmentIndex)).not.toThrow();
    expect(() => validateDerivedTagExplicitAssignmentsAgainstRecords([
      { recordKey: "equipment:mask", name: "Masquerade Mask", category: "equipment" },
    ], assignmentIndex)).not.toThrow();
    expect(() => validateDerivedTagExplicitAssignmentsAgainstRecords([
      { recordKey: "equipment:mask", name: "Different Name", category: "equipment" },
    ], assignmentIndex)).toThrow(/expected name/);
  });

  it("derives pending assignments from review metadata without making them live", () => {
    const groups = [
      {
        category: "equipment" as const,
        assignments: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            review: {
              security: {
                alarm: {
                  mode: "include",
                  status: "needs_review",
                  confidence: "medium",
                  rationale: "Likely security-oriented, but the signal may be too weak to auto-apply.",
                },
              },
            },
          },
        ],
      },
    ];

    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, groups);
    expect(assignmentIndex.assignmentsByRecordKey.get("equipment:bell")).toEqual({
      category: "equipment",
      name: "Watch Bell",
      includeTags: [],
      excludeTags: [],
    });

    expect(buildDerivedTagPendingAssignmentViews(assignmentOntology, groups)).toEqual([
      {
        name: "Watch Bell",
        recordKey: "equipment:bell",
        pending: {
          security: ["alarm"],
        },
      },
    ]);
  });

  it("throws on illegal live assignment states", () => {
    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { infiltration: ["social_infiltration"] },
          },
        ],
      },
    ])).toThrow(/missing review metadata/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { infiltration: ["social_infiltration"] },
            excluded: { infiltration: ["social_infiltration"] },
            review: {
              infiltration: {
                social_infiltration: {
                  mode: "include",
                  status: "approved",
                  rationale: "Conflicting live placement should be rejected.",
                },
              },
            },
          },
        ],
      },
    ])).toThrow(/both applied and excluded/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            applied: { security: ["alarm"] },
            review: {
              security: {
                alarm: {
                  mode: "include",
                  status: "needs_review",
                  rationale: "Needs review items must not be live.",
                },
              },
            },
          },
        ],
      },
    ])).toThrow(/review status is "needs_review"/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            review: {
              security: {
                alarm: {
                  mode: "include",
                  status: "auto_applied",
                  rationale: "Live auto-applied entries must appear in applied.",
                },
              },
            },
          },
        ],
      },
    ])).toThrow(/missing from applied/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            excluded: { infiltration: ["disguise"] },
            review: {
              infiltration: {
                disguise: {
                  mode: "include",
                  status: "approved",
                  rationale: "Review mode must agree with live excluded placement.",
                },
              },
            },
          },
        ],
      },
    ])).toThrow(/marks excluded tag "infiltration\.disguise" with review mode "include"/);
  });

  it("merges explicit assignments into derivation and applies exclusions without family promotion", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: { infiltration: ["social_infiltration"] },
            excluded: { infiltration: ["disguise"] },
            review: {
              infiltration: {
                social_infiltration: {
                  mode: "include",
                  status: "auto_applied",
                  rationale: "Live high-confidence assignment.",
                },
                disguise: {
                  mode: "exclude",
                  status: "approved",
                  rationale: "Explicitly keeping the broader disguise tag off this record.",
                },
              },
            },
          },
        ],
      },
    ]);
    const seedIndex = buildDerivedTagSeedIndex(assignmentOntology, buildDerivedTagSeedLookup([]));

    const derivation = deriveCatalogTagDerivation(
      assignmentOntology,
      seedIndex,
      { recordKey: "equipment:mask", category: "equipment", subcategory: null },
      ["disguise"],
      assignmentIndex,
    );

    expect(derivation.tags).toEqual(["social_infiltration"]);
    expect(derivation.sources.get("social_infiltration")).toBe("assignment");
    expect(derivation.tags).not.toContain("disguise");
  });

  it("keeps the real authored creature assignments in a legal state", () => {
    expect(() => createDerivedTagExplicitAssignmentIndex(creatureOntology)).not.toThrow();
    expect(buildDerivedTagPendingAssignmentViews(creatureOntology)).toEqual([]);
  });

  it("applies configured creature assignments to live record keys", () => {
    const departmentalChair = deriveRecordTagDerivation({
      recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
      name: "Departmental Chair",
      category: "creature",
      subcategory: null,
      descriptionText: "An overworked academic reluctantly handling university emergencies.",
      traits: ["human", "humanoid"],
    });
    expect(departmentalChair.tags).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));
    expect(["assignment", "rule_assignment"]).toContain(departmentalChair.sources.get("profession_npc"));
    expect(["assignment", "rule_assignment"]).toContain(departmentalChair.sources.get("civic_npc"));

    const falsePriest = deriveRecordTagDerivation({
      recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
      name: "False Priest",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["human", "humanoid"],
    });
    expect(falsePriest.tags).toEqual(expect.arrayContaining(["profession_npc", "combatant_npc"]));
    expect(falsePriest.tags).not.toContain("civic_npc");

    const spiritboundAluum = deriveRecordTagDerivation({
      recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
      name: "Spiritbound Aluum",
      category: "creature",
      subcategory: null,
      descriptionText: null,
      traits: ["construct", "mindless", "soulbound"],
    });
    expect(spiritboundAluum.tags).toContain("urban_setting");
    expect(spiritboundAluum.sources.get("urban_setting")).toBe("assignment");

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

    const conspiratorDragon = deriveRecordTagDerivation({
      recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
      name: "Conspirator Dragon (Adult)",
      category: "creature",
      subcategory: null,
      descriptionText: "Hidden among the shadows and upper echelons of society are the conspirator dragons. However, as most conspirator dragons meet others while in disguise, they do their best to maintain their disguise.",
      traits: ["dragon", "occult"],
    });
    expect(conspiratorDragon.tags).toEqual(expect.arrayContaining(["disguised_pretender", "urban_setting"]));
    expect(["assignment", "rule_assignment"]).toContain(conspiratorDragon.sources.get("disguised_pretender"));
    expect(["assignment", "rule_assignment"]).toContain(conspiratorDragon.sources.get("urban_setting"));
  });
});
