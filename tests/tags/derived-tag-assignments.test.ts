import { describe, expect, it } from "vitest";

import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../src/types.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  buildDerivedTagPendingAssignmentViews,
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagAssignmentMemory,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "../../src/tags/runtime/assignments.js";
import {
  publishDerivedTagOntology,
  deriveCatalogTagDerivation,
} from "../../src/tags/runtime/catalog-utils.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "../../src/tags/ontology/creature.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../../src/tags/ontology/utils.js";
import { deriveRecordTagDerivation } from "../../src/tags/index.js";

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
            applied: {
              infiltration: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  confidence: "high",
                  rationale: "Tailored for moving through social spaces while disguised.",
                },
              ],
            },
            excluded: {
              infiltration: [
                {
                  tag: "disguise",
                  source: "human",
                  confidence: "high",
                  rationale: "This record is using a narrower social fit than a broad disguise bucket.",
                },
              ],
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
            applied: {
              stealth: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  rationale: "Invalid family.",
                },
              ],
            },
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
            applied: {
              security: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  rationale: "Invalid family placement.",
                },
              ],
            },
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
            applied: {
              infiltration: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  rationale: "Core assignment for the masquerade entry.",
                },
              ],
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

  it("derives pending assignments from assignment review files without making them live", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            applied: {
              security: [
                {
                  tag: "alarm",
                  source: "human",
                  rationale: "Live assignment stays separate from pending review state.",
                },
              ],
            },
          },
        ],
      },
    ]);
    expect(assignmentIndex.assignmentsByRecordKey.get("equipment:bell")).toEqual({
      category: "equipment",
      name: "Watch Bell",
      includeTags: ["alarm"],
      excludeTags: [],
    });

    expect(buildDerivedTagPendingAssignmentViews(assignmentOntology, [
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
            rationale: "Likely security-oriented, but the signal may be too weak to auto-apply.",
          },
        ],
      },
    ])).toEqual([
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
          },
        ],
      },
    ])).toThrow(/at least one applied or excluded tag/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: {
              infiltration: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  rationale: "Conflicting live placement should be rejected.",
                },
              ],
            },
            excluded: {
              infiltration: [
                {
                  tag: "social_infiltration",
                  source: "human",
                  rationale: "Conflicting live placement should be rejected.",
                },
              ],
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
            applied: {
              security: [
                {
                  tag: "alarm",
                  source: "human",
                  rationale: "Duplicate live entries should be rejected.",
                },
                {
                  tag: "alarm",
                  source: "human",
                  rationale: "Duplicate live entries should be rejected.",
                },
              ],
            },
          },
        ],
      },
    ])).toThrow(/repeats/);
  });

  it("throws on illegal assignment review and memory states", () => {
    expect(() => buildDerivedTagPendingAssignmentViews(assignmentOntology, [
      {
        category: "equipment",
        decisions: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            family: "security",
            tag: "alarm",
            mode: "include",
            rationale: "One pending review entry.",
          },
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            family: "security",
            tag: "alarm",
            mode: "include",
            rationale: "Duplicate pending review entry.",
          },
        ],
      },
    ])).toThrow(/repeats/);

    expect(() => validateDerivedTagAssignmentMemory(assignmentOntology, [
      {
        category: "equipment",
        decisions: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            family: "security",
            tag: "alarm",
            mode: "include",
            rationale: "Rejected once.",
          },
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            family: "security",
            tag: "alarm",
            mode: "include",
            rationale: "Rejected twice with the same identity.",
          },
        ],
      },
    ])).toThrow(/repeats/);
  });

  it("merges explicit assignments into derivation and applies exclusions without family promotion", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentOntology, [
      {
        category: "equipment",
        assignments: [
          {
            name: "Masquerade Mask",
            recordKey: "equipment:mask",
            applied: {
              infiltration: [
                {
                  tag: "social_infiltration",
                  source: "llm_auto",
                  rationale: "Live high-confidence assignment.",
                },
              ],
            },
            excluded: {
              infiltration: [
                {
                  tag: "disguise",
                  source: "human",
                  rationale: "Explicitly keeping the broader disguise tag off this record.",
                },
              ],
            },
          },
        ],
      },
    ]);
    const derivation = deriveCatalogTagDerivation(
      assignmentOntology,
      { recordKey: "equipment:mask", category: "equipment", subcategory: null },
      ["disguise"],
      assignmentIndex,
    );

    expect(derivation.tags).toEqual(["social_infiltration"]);
    expect(derivation.sources.get("social_infiltration")).toBe("assignment");
    expect(derivation.tags).not.toContain("disguise");
  });

  it("keeps the real authored creature assignments, assignment reviews, and assignment memory in a legal state", () => {
    expect(() => createDerivedTagExplicitAssignmentIndex(creatureOntology)).not.toThrow();
    expect(() => buildDerivedTagPendingAssignmentViews(creatureOntology)).not.toThrow();
    expect(() => validateDerivedTagAssignmentMemory(creatureOntology)).not.toThrow();
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
    expect(["assignment", "legacy_rule+assignment"]).toContain(departmentalChair.sources.get("profession_npc"));
    expect(["assignment", "legacy_rule+assignment"]).toContain(departmentalChair.sources.get("civic_npc"));

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
    expect(["assignment", "legacy_rule+assignment"]).toContain(conspiratorDragon.sources.get("disguised_pretender"));
    expect(["assignment", "legacy_rule+assignment"]).toContain(conspiratorDragon.sources.get("urban_setting"));
  });
});
