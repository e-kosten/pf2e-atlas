import { describe, expect, it } from "vitest";

import type { AuthoredDerivedTagRule, DerivedTagExemplarCategory } from "../../src/types.js";
import type { AuthoredDerivedTagAssignment } from "../../src/tags/runtime/assignments.js";
import {
  applyMigrationSessionToAssignments,
  applyMigrationSessionToAuthoredRules,
  applyMigrationSessionToExemplars,
} from "../../src/tags/migration/importer.js";
import { lintDerivedTagMigrationSession } from "../../src/tags/migration/linter.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "../../src/tags/migration/review-session.js";
import { moveSelection } from "../../src/tags/migration/terminal-ui.js";
import type { DerivedTagMigrationSession } from "../../src/tags/migration/types.js";

describe("derived tag migration tooling", () => {
  it("imports assignment review outcomes into live and non-live states", () => {
    const assignments: AuthoredDerivedTagAssignment[] = [
      {
        name: "Watch Bell",
        recordKey: "equipment:bell",
        review: {
          security: {
            alarm: {
              mode: "include",
              status: "needs_review",
              confidence: "medium",
              rationale: "Pending manual review.",
            },
          },
        },
      },
    ];

    const migrated = applyMigrationSessionToAssignments(assignments, [
      {
        recordKey: "equipment:bell",
        name: "Watch Bell",
        category: "equipment",
        resolutionStatus: "complete",
        decisions: [
          {
            kind: "assignment",
            family: "security",
            tag: "alarm",
            mode: "include",
            status: "approved",
            confidence: "high",
            rationale: "Confirmed by review.",
          },
          {
            kind: "assignment",
            family: "infiltration",
            tag: "disguise",
            mode: "exclude",
            status: "rejected",
            confidence: "low",
            rationale: "Rejected negative proposal should remain non-live.",
          },
        ],
      },
    ]);

    expect(migrated).toEqual([
      {
        name: "Watch Bell",
        recordKey: "equipment:bell",
        applied: {
          security: ["alarm"],
        },
        review: {
          infiltration: {
            disguise: {
              mode: "exclude",
              status: "rejected",
              confidence: "low",
              rationale: "Rejected negative proposal should remain non-live.",
            },
          },
          security: {
            alarm: {
              mode: "include",
              status: "approved",
              confidence: "high",
              rationale: "Confirmed by review.",
            },
          },
        },
      },
    ]);
  });

  it("imports approved exemplar and authored rule proposals", () => {
    const exemplars: DerivedTagExemplarCategory = {
      category: "creature",
      exemplars: [
        {
          tag: "urban_setting",
          positives: [{ recordKey: "creature:old", name: "Old Urban Example" }],
        },
      ],
    };
    const rules: AuthoredDerivedTagRule[] = [];

    const sessionDecisions = [
      {
        recordKey: "creature:new",
        name: "New Urban Example",
        category: "creature" as const,
        resolutionStatus: "complete" as const,
        decisions: [
          {
            kind: "exemplar" as const,
            tag: "urban_setting",
            polarity: "positive" as const,
            action: "keep" as const,
            status: "approved" as const,
            rationale: "Strong exemplar.",
          },
          {
            kind: "rule" as const,
            tag: "urban_setting",
            decision: "recreate_authored" as const,
            status: "approved" as const,
            rationale: "Safe deterministic slice.",
            authoredRules: [
              {
                tag: "urban_setting",
                category: "creature",
                intent: "deterministic",
                kind: "trait_match",
                when: { traitsAll: ["clockwork"] },
              },
            ],
          },
        ],
      },
    ];

    expect(applyMigrationSessionToExemplars(exemplars, sessionDecisions)).toEqual({
      category: "creature",
      exemplars: [
        {
          tag: "urban_setting",
          positives: [
            { recordKey: "creature:new", name: "New Urban Example" },
            { recordKey: "creature:old", name: "Old Urban Example" },
          ],
          negatives: [],
        },
      ],
    });
    expect(applyMigrationSessionToAuthoredRules(rules, sessionDecisions)).toEqual([
      {
        tag: "urban_setting",
        category: "creature",
        intent: "deterministic",
        kind: "trait_match",
        when: { traitsAll: ["clockwork"] },
      },
    ]);
  });

  it("lints contradictory migration sessions", () => {
    const session: DerivedTagMigrationSession = {
      manifest: {
        id: "test",
        mode: "review_queue",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        {
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          subcategory: "gear",
          packName: "equipment",
          level: 1,
          traits: [],
          families: [],
          currentDerivedTags: [],
          currentSources: {},
          descriptionText: null,
          blurbText: null,
          selectionReasons: [],
        },
      ],
      decisions: [
        {
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          resolutionStatus: "needs_review",
          decisions: [
            {
              kind: "assignment",
              family: "security",
              tag: "alarm",
              mode: "include",
              status: "needs_review",
              rationale: "Needs review.",
            },
            {
              kind: "assignment",
              family: "security",
              tag: "alarm",
              mode: "exclude",
              status: "approved",
              rationale: "Conflicts with include.",
            },
          ],
        },
      ],
      reviewState: {
        currentIndex: 0,
        unresolvedOnly: true,
        updatedAt: "2026-04-16T00:00:00.000Z",
      },
    };

    expect(() => lintDerivedTagMigrationSession(session)).toThrow(/both include and exclude/);
  });

  it("supports scratch review navigation and status updates", () => {
    let session: DerivedTagMigrationSession = {
      manifest: {
        id: "test",
        mode: "review_queue",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        {
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          subcategory: "gear",
          packName: "equipment",
          level: 1,
          traits: [],
          families: [],
          currentDerivedTags: [],
          currentSources: {},
          descriptionText: null,
          blurbText: null,
          selectionReasons: [],
        },
      ],
      decisions: [
        {
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          resolutionStatus: "needs_review",
          decisions: [
            {
              kind: "assignment",
              family: "security",
              tag: "alarm",
              mode: "include",
              status: "needs_review",
              rationale: "Needs review.",
            },
          ],
        },
      ],
      reviewState: {
        currentIndex: 0,
        unresolvedOnly: true,
        updatedAt: "2026-04-16T00:00:00.000Z",
      },
    };

    expect(getDerivedTagMigrationReviewItems(session)).toHaveLength(1);
    session = updateDerivedTagMigrationDecisionStatus(session, { recordIndex: 0, decisionIndex: 0 }, "approved");
    expect(session.decisions[0]?.resolutionStatus).toBe("complete");
    session = toggleDerivedTagMigrationUnresolvedOnly(session);
    session = clampDerivedTagMigrationReviewIndex(session);
    expect(getDerivedTagMigrationReviewItems(session)).toHaveLength(1);
  });

  it("clamps keyboard selection movement to valid bounds", () => {
    expect(moveSelection(0, -1, 3)).toBe(0);
    expect(moveSelection(0, 1, 3)).toBe(1);
    expect(moveSelection(2, 1, 3)).toBe(2);
    expect(moveSelection(5, 0, 3)).toBe(2);
    expect(moveSelection(0, 1, 0)).toBe(0);
  });
});
