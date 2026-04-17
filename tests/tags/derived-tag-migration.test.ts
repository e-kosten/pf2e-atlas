import { mkdtemp } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import type { AuthoredDerivedTagRule, DerivedTagExemplarCategory, DerivedTagExemplarReviewCategory } from "../../src/types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
} from "../../src/tags/runtime/assignments.js";
import {
  getCurrentDerivedTagMigrationAuthoredState,
  setCurrentDerivedTagMigrationAuthoredState,
  writeDerivedTagMigrationAuthoredState,
} from "../../src/tags/migration/authored-state.js";
import {
  applyMigrationSessionToAssignments,
  applyMigrationSessionToAssignmentMemory,
  applyMigrationSessionToAssignmentReviews,
  applyMigrationSessionToAuthoredRules,
  applyMigrationSessionToExemplars,
  applyMigrationSessionToExemplarReviews,
} from "../../src/tags/migration/importer.js";
import { lintDerivedTagMigrationSession } from "../../src/tags/migration/linter.js";
import { summarizeCurrentDerivedTagReviewQueue } from "../../src/tags/migration/runtime-state.js";
import { renderDerivedTagMigrationReviewItem } from "../../src/tags/migration/render.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  summarizeDerivedTagMigrationReviewProgress,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "../../src/tags/migration/review-session.js";
import { moveSelection } from "../../src/tags/migration/terminal-ui.js";
import type { DerivedTagMigrationSession } from "../../src/tags/migration/types.js";

describe("derived tag migration tooling", () => {
  it("imports assignment review outcomes into live assignments, pending review, and rejected memory", () => {
    const assignments: AuthoredDerivedTagAssignment[] = [];
    const assignmentReviews: DerivedTagAssignmentReviewCategory = {
      category: "equipment",
      decisions: [
        {
          name: "Watch Bell",
          recordKey: "equipment:bell",
          family: "security",
          tag: "alarm",
          mode: "include",
          confidence: "medium",
          rationale: "Pending manual review.",
        },
      ],
    };
    const assignmentMemory: DerivedTagAssignmentMemoryCategory = {
      category: "equipment",
      decisions: [],
    };

    const sessionDecisions = [
      {
        recordKey: "equipment:bell",
        name: "Watch Bell",
        category: "equipment" as const,
        resolutionStatus: "complete" as const,
        decisions: [
          {
            kind: "assignment" as const,
            family: "security",
            tag: "alarm",
            mode: "include" as const,
            status: "approved" as const,
            confidence: "high" as const,
            rationale: "Confirmed by review.",
            source: "llm" as const,
          },
          {
            kind: "assignment" as const,
            family: "infiltration",
            tag: "disguise",
            mode: "exclude" as const,
            status: "rejected" as const,
            confidence: "low" as const,
            rationale: "Rejected negative proposal should remain non-live.",
            source: "llm" as const,
          },
        ],
      },
    ];

    expect(applyMigrationSessionToAssignments(assignments, sessionDecisions)).toEqual([
      {
        name: "Watch Bell",
        recordKey: "equipment:bell",
        applied: {
          security: [
            {
              tag: "alarm",
              source: "llm_reviewed",
              confidence: "high",
              rationale: "Confirmed by review.",
            },
          ],
        },
      },
    ]);
    expect(applyMigrationSessionToAssignmentReviews(assignmentReviews, sessionDecisions)).toEqual({
      category: "equipment",
      decisions: [],
    });
    expect(applyMigrationSessionToAssignmentMemory(assignmentMemory, sessionDecisions)).toEqual({
      category: "equipment",
      decisions: [
        {
          name: "Watch Bell",
          recordKey: "equipment:bell",
          family: "infiltration",
          tag: "disguise",
          mode: "exclude",
          confidence: "low",
          rationale: "Rejected negative proposal should remain non-live.",
          source: "llm",
        },
      ],
    });
  });

  it("imports auto-applied assignment proposals directly into live assignments", () => {
    const assignments: AuthoredDerivedTagAssignment[] = [];

    expect(applyMigrationSessionToAssignments(assignments, [
      {
        recordKey: "equipment:mask",
        name: "Masquerade Mask",
        category: "equipment",
        resolutionStatus: "complete",
        decisions: [
          {
            kind: "assignment",
            family: "infiltration",
            tag: "social_infiltration",
            mode: "include",
            status: "auto_applied",
            confidence: "high",
            rationale: "High-confidence direct tagging call.",
            source: "llm",
          },
        ],
      },
    ])).toEqual([
      {
        name: "Masquerade Mask",
        recordKey: "equipment:mask",
        applied: {
          infiltration: [
            {
              tag: "social_infiltration",
              source: "llm_auto",
              confidence: "high",
              rationale: "High-confidence direct tagging call.",
            },
          ],
        },
      },
    ]);
  });

  it("resolves exemplar review decisions into live exemplars and review backlog", () => {
    const exemplars: DerivedTagExemplarCategory = {
      category: "creature",
      exemplars: [
        {
          tag: "urban_setting",
          positives: [{ recordKey: "creature:old", name: "Old Urban Example" }],
          negatives: [],
        },
      ],
    };
    const exemplarReviews: DerivedTagExemplarReviewCategory = {
      category: "creature",
      decisions: [
        {
          name: "Old Urban Example",
          recordKey: "creature:old",
          tag: "urban_setting",
          proposedPolarity: "drop",
          currentPolarity: "positive",
          status: "needs_review",
          rationale: "Might not be the strongest exemplar anymore.",
        },
      ],
    };
    const sessionDecisions = [
      {
        recordKey: "creature:old",
        name: "Old Urban Example",
        category: "creature" as const,
        resolutionStatus: "complete" as const,
        decisions: [
          {
            kind: "exemplar" as const,
            tag: "urban_setting",
            polarity: "positive" as const,
            action: "drop" as const,
            status: "approved" as const,
            rationale: "Confirmed drop from exemplar set.",
            currentPolarity: "positive" as const,
          },
        ],
      },
    ];

    expect(applyMigrationSessionToExemplars(exemplars, sessionDecisions)).toEqual({
      category: "creature",
      exemplars: [],
    });
    expect(applyMigrationSessionToExemplarReviews(exemplarReviews, sessionDecisions)).toEqual({
      category: "creature",
      decisions: [],
    });
  });

  it("imports approved authored rule proposals", () => {
    const rules: AuthoredDerivedTagRule[] = [];

    const sessionDecisions = [
      {
        recordKey: "creature:new",
        name: "New Urban Example",
        category: "creature" as const,
        resolutionStatus: "complete" as const,
        decisions: [
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

  it("reports candidate and actionable review progress separately", () => {
    const session: DerivedTagMigrationSession = {
      manifest: {
        id: "test",
        mode: "proposal_review",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 3,
      },
      records: [],
      decisions: [
        {
          recordKey: "creature:one",
          name: "Creature One",
          category: "creature",
          resolutionStatus: "needs_review",
          decisions: [],
        },
        {
          recordKey: "creature:two",
          name: "Creature Two",
          category: "creature",
          resolutionStatus: "complete",
          decisions: [
            {
              kind: "assignment",
              family: "alarm",
              tag: "alarm",
              mode: "include",
              status: "approved",
              rationale: "Approved.",
            },
          ],
        },
        {
          recordKey: "spell:one",
          name: "Spell One",
          category: "spell",
          resolutionStatus: "needs_review",
          decisions: [
            {
              kind: "assignment",
              family: "alarm",
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

    expect(summarizeDerivedTagMigrationReviewProgress(session)).toEqual({
      candidateRecordCount: 3,
      actionableRecordCount: 2,
      resolvedActionableRecordCount: 1,
      visibleItemCount: 1,
    });
  });

  it("includes record context in rendered review items", () => {
    const session: DerivedTagMigrationSession = {
      manifest: {
        id: "test",
        mode: "proposal_review",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        {
          recordKey: "creature:aluum",
          name: "Spiritbound Aluum",
          category: "creature",
          subcategory: null,
          packName: "creature-pack",
          level: 16,
          traits: ["incorporeal", "undead"],
          families: [],
          currentDerivedTags: ["urban_setting"],
          currentSources: {
            urban_setting: "assignment:human",
          },
          descriptionText: "A spiritbound aluum stalks city streets and abandoned alleys in search of trespassers and old grudges.",
          blurbText: "An undead sentinel bound to a haunted district.",
          selectionReasons: [
            {
              source: "exemplar_cleanup",
              tag: "urban_setting",
              note: "Current exemplar is part of an oversized exemplar set and needs review.",
            },
          ],
        },
      ],
      decisions: [
        {
          recordKey: "creature:aluum",
          name: "Spiritbound Aluum",
          category: "creature",
          resolutionStatus: "needs_review",
          decisions: [
            {
              kind: "exemplar",
              tag: "urban_setting",
              polarity: "positive",
              action: "keep",
              status: "needs_review",
              confidence: "medium",
              rationale: "Review whether this creature remains a strong positive exemplar for \"urban_setting\".",
              source: "llm",
              currentPolarity: "positive",
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

    const rendered = renderDerivedTagMigrationReviewItem(session, 0);

    expect(rendered.indexOf("Rationale: Review whether this creature remains a strong positive exemplar for \"urban_setting\".")).toBeLessThan(
      rendered.indexOf("Traits: incorporeal, undead"),
    );
    expect(rendered).toContain("Traits: incorporeal, undead");
    expect(rendered).toContain("Current source for urban_setting: assignment:human");
    expect(rendered).toContain("Blurb: An undead sentinel bound to a haunted district.");
    expect(rendered).toContain("Description: A spiritbound aluum stalks city streets and abandoned alleys in search of trespassers and old grudges.");
  });

  it("clamps keyboard selection movement to valid bounds", () => {
    expect(moveSelection(0, -1, 3)).toBe(0);
    expect(moveSelection(0, 1, 3)).toBe(1);
    expect(moveSelection(2, 1, 3)).toBe(2);
    expect(moveSelection(5, 0, 3)).toBe(2);
    expect(moveSelection(0, 1, 0)).toBe(0);
  });

  it("refreshes in-process authored state after writing migration outputs", async () => {
    const initialState = getCurrentDerivedTagMigrationAuthoredState();
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "derived-tag-migration-state-"));

    try {
      const nextState = structuredClone(initialState);
      nextState.assignments.equipment = [
        {
          name: "Watch Bell",
          recordKey: "equipment:bell",
          applied: {
            security: [
              {
                tag: "alarm",
                source: "human",
                rationale: "Confirmed during migration review.",
              },
            ],
          },
        },
      ];
      nextState.assignmentReviews.equipment = {
        category: "equipment",
        decisions: [
          {
            name: "Smoke Veil",
            recordKey: "equipment:veil",
            family: "infiltration",
            tag: "social_infiltration",
            mode: "include",
            rationale: "Pending manual confirmation.",
          },
        ],
      };
      nextState.assignmentMemory.equipment = {
        category: "equipment",
        decisions: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            family: "infiltration",
            tag: "disguise",
            mode: "exclude",
            rationale: "Rejected earlier proposal.",
          },
        ],
      };
      nextState.exemplarReviews.equipment = {
        category: "equipment",
        decisions: [
          {
            name: "Watch Bell",
            recordKey: "equipment:bell",
            tag: "alarm",
            proposedPolarity: "positive",
            status: "needs_review",
            rationale: "Maybe a good positive teaching example.",
          },
        ],
      };

      await writeDerivedTagMigrationAuthoredState(tempRoot, nextState, ["equipment"]);

      expect(getCurrentDerivedTagMigrationAuthoredState().assignments.equipment).toEqual(nextState.assignments.equipment);
      expect(getCurrentDerivedTagMigrationAuthoredState().assignmentReviews.equipment).toEqual(nextState.assignmentReviews.equipment);
      expect(getCurrentDerivedTagMigrationAuthoredState().assignmentMemory.equipment).toEqual(nextState.assignmentMemory.equipment);
      expect(getCurrentDerivedTagMigrationAuthoredState().exemplarReviews.equipment).toEqual(nextState.exemplarReviews.equipment);
      expect(summarizeCurrentDerivedTagReviewQueue()).toEqual(expect.arrayContaining([
        expect.objectContaining({
          kind: "assignment",
          category: "equipment",
          family: "infiltration",
          tag: "social_infiltration",
          count: 1,
        }),
        expect.objectContaining({
          kind: "exemplar",
          category: "equipment",
          tag: "alarm",
          count: 1,
        }),
      ]));
    } finally {
      setCurrentDerivedTagMigrationAuthoredState(initialState);
    }
  });

  it("sorts the review queue by kind, confidence, count, managed category order, and normalized labels", () => {
    const initialState = getCurrentDerivedTagMigrationAuthoredState();

    try {
      const nextState = structuredClone(initialState);
      nextState.assignmentReviews.affliction = {
        category: "affliction",
        decisions: [
          {
            name: "Affliction Alpha",
            recordKey: "affliction:alpha",
            family: "tracking",
            tag: "zeta_watch",
            mode: "include",
            confidence: "low",
            rationale: "Low-confidence review item.",
          },
          {
            name: "Affliction Beta",
            recordKey: "affliction:beta",
            family: "tracking",
            tag: "zeta_watch",
            mode: "include",
            confidence: "medium",
            rationale: "Creates a mixed-confidence bucket.",
          },
        ],
      };
      nextState.assignmentReviews.creature = {
        category: "creature",
        decisions: [
          {
            name: "Creature Alpha",
            recordKey: "creature:alpha",
            family: "alarm",
            tag: "alarm_2",
            mode: "include",
            confidence: "low",
            rationale: "Lower numbered normalized label should sort first.",
          },
          {
            name: "Creature Beta",
            recordKey: "creature:beta",
            family: "alarm",
            tag: "alarm_10",
            mode: "include",
            confidence: "low",
            rationale: "Higher numbered normalized label should sort later.",
          },
        ],
      };
      nextState.assignmentReviews.equipment = {
        category: "equipment",
        decisions: [
          {
            name: "Equipment Alpha",
            recordKey: "equipment:alpha",
            family: "infiltration",
            tag: "social_infiltration",
            mode: "include",
            confidence: "high",
            rationale: "High-confidence assignment should sort after lower-confidence items.",
          },
        ],
      };
      nextState.exemplarReviews.affliction = {
        category: "affliction",
        decisions: [
          {
            name: "Affliction Exemplar",
            recordKey: "affliction:exemplar",
            tag: "alarm",
            proposedPolarity: "positive",
            status: "needs_review",
            confidence: "low",
            rationale: "Exemplar items should sort after assignments.",
          },
        ],
      };
      setCurrentDerivedTagMigrationAuthoredState(nextState);

      expect(summarizeCurrentDerivedTagReviewQueue().slice(0, 5)).toEqual([
        {
          kind: "assignment",
          category: "affliction",
          family: "tracking",
          tag: "zeta_watch",
          count: 2,
          confidence: "mixed",
        },
        {
          kind: "assignment",
          category: "creature",
          family: "alarm",
          tag: "alarm_2",
          count: 1,
          confidence: "low",
        },
        {
          kind: "assignment",
          category: "creature",
          family: "alarm",
          tag: "alarm_10",
          count: 1,
          confidence: "low",
        },
        {
          kind: "assignment",
          category: "equipment",
          family: "infiltration",
          tag: "social_infiltration",
          count: 1,
          confidence: "high",
        },
        {
          kind: "exemplar",
          category: "affliction",
          tag: "alarm",
          count: 1,
          confidence: "low",
        },
      ]);
    } finally {
      setCurrentDerivedTagMigrationAuthoredState(initialState);
    }
  });
});
