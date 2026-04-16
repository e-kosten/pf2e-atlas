import { describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry } from "../../src/types.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "../../src/tags/assignments.js";
import {
  buildDerivedTagSeedIndex,
  buildDerivedTagSeedLookup,
  deriveCatalogTagDerivation,
} from "../../src/tags/catalog-utils.js";
import { deriveRecordTagDerivation } from "../../src/tags/index.js";

const assignmentCatalog: DerivedTagCatalogEntry[] = [
  {
    category: "equipment",
    family: "infiltration",
    description: "Equipment that helps infiltration.",
    promoteFamilyToTag: true,
    tags: [
      { value: "disguise", description: "Alters appearance." },
      { value: "social_infiltration", description: "Blends into social spaces." },
    ],
  },
  {
    category: "equipment",
    family: "security",
    description: "Equipment that supports security.",
    tags: [
      { value: "alarm", description: "Warns of intruders." },
    ],
  },
];

describe("derived tag explicit assignments", () => {
  it("flattens grouped family assignments into concrete include and exclude tags", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentCatalog, [
      {
        category: "equipment",
        assignments: [
          {
            recordKey: "equipment:mask",
            name: "Masquerade Mask",
            byFamily: { infiltration: ["social_infiltration"] },
            excludeByFamily: { infiltration: ["disguise"] },
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
    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentCatalog, [
      {
        category: "equipment",
        assignments: [
          {
            recordKey: "equipment:mask",
            name: "Masquerade Mask",
            byFamily: { stealth: ["social_infiltration"] },
          },
        ],
      },
    ])).toThrow(/does not exist/);

    expect(() => buildDerivedTagExplicitAssignmentIndex(assignmentCatalog, [
      {
        category: "equipment",
        assignments: [
          {
            recordKey: "equipment:mask",
            name: "Masquerade Mask",
            byFamily: { security: ["social_infiltration"] },
          },
        ],
      },
    ])).toThrow(/does not belong to family/);
  });

  it("validates canonical name drift only when the record is present in the build", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentCatalog, [
      {
        category: "equipment",
        assignments: [
          {
            recordKey: "equipment:mask",
            name: "Masquerade Mask",
            byFamily: { infiltration: ["social_infiltration"] },
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

  it("merges explicit assignments into derivation and applies exclusions before family promotion", () => {
    const assignmentIndex = buildDerivedTagExplicitAssignmentIndex(assignmentCatalog, [
      {
        category: "equipment",
        assignments: [
          {
            recordKey: "equipment:mask",
            name: "Masquerade Mask",
            byFamily: { infiltration: ["social_infiltration"] },
            excludeByFamily: { infiltration: ["disguise"] },
          },
        ],
      },
    ]);
    const seedIndex = buildDerivedTagSeedIndex(assignmentCatalog, buildDerivedTagSeedLookup([]));

    const derivation = deriveCatalogTagDerivation(
      assignmentCatalog,
      seedIndex,
      { recordKey: "equipment:mask", category: "equipment", subcategory: null },
      ["disguise"],
      assignmentIndex,
    );

    expect(derivation.tags).toEqual(["infiltration", "social_infiltration"]);
    expect(derivation.sources.get("social_infiltration")).toBe("assignment");
    expect(derivation.sources.get("infiltration")).toBe("assignment");
    expect(derivation.tags).not.toContain("disguise");
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
