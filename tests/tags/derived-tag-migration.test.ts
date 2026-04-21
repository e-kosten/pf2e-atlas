import { mkdtemp, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarReviewCategory,
  SearchCategory,
  SearchSubcategory,
} from "../../src/domain/derived-tag-types.js";
import type { OntologyExplorerEntityRecord } from "../../src/app/ontology/entity-record.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
} from "../../src/tags/runtime/derivation/assignments.js";
import {
  getCurrentDerivedTagAuthoredState,
  setCurrentDerivedTagAuthoredState,
} from "../../src/tags/editorial/state/authored-state.js";
import { writeDerivedTagAuthoredState } from "../../src/tags/editorial/writeback/authored-state-writer.js";
import {
  applyMigrationSessionToAssignments,
  applyMigrationSessionToAssignmentMemory,
  applyMigrationSessionToAssignmentReviews,
  applyMigrationSessionToAuthoredRules,
  applyMigrationSessionToExemplars,
  applyMigrationSessionToExemplarReviews,
} from "../../src/tags/editorial/writeback/importer.js";
import { lintDerivedTagReviewSession } from "../../src/tags/editorial/writeback/linter.js";
import {
  deriveCurrentTagSources,
  summarizeCurrentDerivedTagReviewQueue,
} from "../../src/tags/editorial/state/runtime-state.js";
import { renderDerivedTagReviewItem } from "../../src/tags/editorial/ui/render.js";
import {
  clampDerivedTagReviewIndex,
  getDerivedTagReviewItems,
  summarizeDerivedTagReviewProgress,
  toggleDerivedTagReviewUnresolvedOnly,
  updateDerivedTagReviewDecisionStatus,
} from "../../src/tags/editorial/sessions/review-session.js";
import { moveSelection } from "../../src/tui/terminal-ui.js";
import type { DerivedTagReviewSession } from "../../src/tags/editorial/types.js";

function createEntityRecord(
  overrides: Partial<OntologyExplorerEntityRecord> &
    Pick<OntologyExplorerEntityRecord, "recordKey" | "name" | "category">,
): OntologyExplorerEntityRecord {
  return {
    recordKey: overrides.recordKey,
    packName: overrides.packName ?? overrides.recordKey.split(":")[0] ?? "",
    name: overrides.name,
    type: overrides.type ?? "unknown",
    category: overrides.category,
    subcategory: overrides.subcategory ?? null,
    documentType: overrides.documentType ?? "unknown",
    level: overrides.level ?? null,
    rarity: overrides.rarity ?? null,
    traits: overrides.traits ?? [],
    derivedTags: overrides.derivedTags ?? [],
    families: overrides.families ?? [],
    descriptionText: overrides.descriptionText ?? null,
    blurbText: overrides.blurbText ?? null,
    sourceCategory: overrides.sourceCategory ?? "unknown",
    publicationTitle: overrides.publicationTitle ?? null,
    publicationRemaster: overrides.publicationRemaster ?? false,
    isUnique: overrides.isUnique ?? false,
    size: overrides.size ?? null,
    languages: overrides.languages ?? [],
    speedTypes: overrides.speedTypes ?? [],
    senses: overrides.senses ?? [],
    immunities: overrides.immunities ?? [],
    resistances: overrides.resistances ?? [],
    weaknesses: overrides.weaknesses ?? [],
    itemCategory: overrides.itemCategory ?? null,
    baseItem: overrides.baseItem ?? null,
    priceCp: overrides.priceCp ?? null,
    actionCost: overrides.actionCost ?? null,
    usage: overrides.usage ?? null,
    hands: overrides.hands ?? null,
    damageTypes: overrides.damageTypes ?? [],
    weaponGroup: overrides.weaponGroup ?? null,
    armorGroup: overrides.armorGroup ?? null,
    traditions: overrides.traditions ?? [],
    spellKinds: overrides.spellKinds ?? [],
    saveType: overrides.saveType ?? null,
    areaType: overrides.areaType ?? null,
    rangeText: overrides.rangeText ?? null,
    durationText: overrides.durationText ?? null,
    targetText: overrides.targetText ?? null,
    areaValue: overrides.areaValue ?? null,
    sustained: overrides.sustained ?? false,
    basicSave: overrides.basicSave ?? false,
    disableText: overrides.disableText ?? null,
    disableSkills: overrides.disableSkills ?? [],
    isComplex: overrides.isComplex ?? false,
  };
}

function createSessionRecord(options: {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory?: SearchSubcategory | null;
  packName?: string;
  level?: number | null;
  traits?: string[];
  families?: string[];
  derivedTags?: string[];
  descriptionText?: string | null;
  blurbText?: string | null;
  currentSources?: DerivedTagReviewSession["records"][number]["currentSources"];
  selectionReasons?: DerivedTagReviewSession["records"][number]["selectionReasons"];
}): DerivedTagReviewSession["records"][number] {
  return {
    entityRecord: createEntityRecord({
      recordKey: options.recordKey,
      packName: options.packName,
      name: options.name,
      category: options.category,
      subcategory: options.subcategory ?? null,
      level: options.level ?? null,
      traits: options.traits ?? [],
      derivedTags: options.derivedTags ?? [],
      families: options.families ?? [],
      descriptionText: options.descriptionText ?? null,
      blurbText: options.blurbText ?? null,
    }),
    currentSources: options.currentSources ?? {},
    selectionReasons: options.selectionReasons ?? [],
  };
}

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

    expect(
      applyMigrationSessionToAssignments(assignments, [
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
      ]),
    ).toEqual([
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
    const session: DerivedTagReviewSession = {
      manifest: {
        id: "test",
        mode: "review_queue",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        createSessionRecord({
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          subcategory: "gear",
          packName: "equipment",
          level: 1,
        }),
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

    expect(() => lintDerivedTagReviewSession(session)).toThrow(/both include and exclude/);
  });

  it("supports scratch review navigation and status updates", () => {
    let session: DerivedTagReviewSession = {
      manifest: {
        id: "test",
        mode: "review_queue",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        createSessionRecord({
          recordKey: "equipment:bell",
          name: "Watch Bell",
          category: "equipment",
          subcategory: "gear",
          packName: "equipment",
          level: 1,
        }),
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

    expect(getDerivedTagReviewItems(session)).toHaveLength(1);
    session = updateDerivedTagReviewDecisionStatus(session, { recordIndex: 0, decisionIndex: 0 }, "approved");
    expect(session.decisions[0]?.resolutionStatus).toBe("complete");
    session = toggleDerivedTagReviewUnresolvedOnly(session);
    session = clampDerivedTagReviewIndex(session);
    expect(getDerivedTagReviewItems(session)).toHaveLength(1);
  });

  it("reports candidate and actionable review progress separately", () => {
    const session: DerivedTagReviewSession = {
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

    expect(summarizeDerivedTagReviewProgress(session)).toEqual({
      candidateRecordCount: 3,
      actionableRecordCount: 2,
      resolvedActionableRecordCount: 1,
      visibleItemCount: 1,
    });
  });

  it("includes record context in rendered review items", () => {
    const session: DerivedTagReviewSession = {
      manifest: {
        id: "test",
        mode: "proposal_review",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        createSessionRecord({
          recordKey: "creature:aluum",
          name: "Spiritbound Aluum",
          category: "creature",
          packName: "creature-pack",
          level: 16,
          traits: ["incorporeal", "undead"],
          derivedTags: ["urban_setting"],
          currentSources: {
            urban_setting: "assignment:human",
          },
          descriptionText:
            "A spiritbound aluum stalks city streets and abandoned alleys in search of trespassers and old grudges.",
          blurbText: "An undead sentinel bound to a haunted district.",
          selectionReasons: [
            {
              source: "exemplar_cleanup",
              tag: "urban_setting",
              note: "Current exemplar is part of an oversized exemplar set and needs review.",
            },
          ],
        }),
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
              rationale: 'Review whether this creature remains a strong positive exemplar for "urban_setting".',
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

    const rendered = renderDerivedTagReviewItem(session, 0);

    expect(
      rendered.indexOf(
        'Rationale: Review whether this creature remains a strong positive exemplar for "urban_setting".',
      ),
    ).toBeLessThan(rendered.indexOf("Identity"));
    expect(rendered).toContain("Identity");
    expect(rendered).toContain("Retrieval");
    expect(rendered).toContain("Traits: incorporeal, undead");
    expect(rendered).toContain("Derived tags: urban_setting");
    expect(rendered).toContain("Blurb");
    expect(rendered).toContain("An undead sentinel bound to a haunted district.");
    expect(rendered).toContain("Description");
    expect(rendered).toContain(
      "A spiritbound aluum stalks city streets and abandoned alleys in search of trespassers and old grudges.",
    );
    expect(rendered).not.toContain("Current source for urban_setting");
  });

  it("does not truncate long record context once detail scrolling is available", () => {
    const longDescription = [
      "Whereas most aluums are animated by the souls of volunteers loyal to Katapesh,",
      "the Pactmasters created a handful of more capable and deadly aluums powered by",
      "the souls of a dozen or more dangerous criminals. These spiritbound aluums are",
      "rarely used as peacekeepers, instead serving as assassins, elite bodyguards, or",
      "riot control during times of martial law.",
    ].join(" ");
    const session: DerivedTagReviewSession = {
      manifest: {
        id: "test",
        mode: "exemplar_cleanup",
        createdAt: "2026-04-16T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        createSessionRecord({
          recordKey: "creature:aluum",
          name: "Spiritbound Aluum",
          category: "creature",
          packName: "creature-pack",
          level: 16,
          traits: ["construct", "mindless", "soulbound"],
          derivedTags: ["urban_setting"],
          currentSources: {
            urban_setting: "assignment",
          },
          descriptionText: longDescription,
          selectionReasons: [
            {
              source: "exemplar_cleanup",
              tag: "urban_setting",
              note: "Current exemplar is part of an oversized exemplar set and needs review.",
            },
          ],
        }),
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
              rationale: 'Review whether this creature remains a strong positive exemplar for "urban_setting".',
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

    const rendered = renderDerivedTagReviewItem(session, 0);

    expect(rendered).toContain(longDescription);
    expect(rendered).not.toContain("that...");
  });

  it("clamps keyboard selection movement to valid bounds", () => {
    expect(moveSelection(0, -1, 3)).toBe(0);
    expect(moveSelection(0, 1, 3)).toBe(1);
    expect(moveSelection(2, 1, 3)).toBe(2);
    expect(moveSelection(5, 0, 3)).toBe(2);
    expect(moveSelection(0, 1, 0)).toBe(0);
  });

  it("refreshes in-process authored state after writing migration outputs", async () => {
    const initialState = getCurrentDerivedTagAuthoredState();
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "derived-tag-migration-state-"));

    try {
      const nextState = structuredClone(initialState);
      nextState.assignments.equipment = [
        {
          name: "Plain Widget",
          recordKey: "equipment:plain-widget",
          applied: {
            infiltration: [
              {
                tag: "social_infiltration",
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

      await writeDerivedTagAuthoredState(tempRoot, nextState, ["equipment"]);

      expect(getCurrentDerivedTagAuthoredState().assignments.equipment).toEqual(
        nextState.assignments.equipment,
      );
      expect(getCurrentDerivedTagAuthoredState().assignmentReviews.equipment).toEqual(
        nextState.assignmentReviews.equipment,
      );
      expect(getCurrentDerivedTagAuthoredState().assignmentMemory.equipment).toEqual(
        nextState.assignmentMemory.equipment,
      );
      expect(getCurrentDerivedTagAuthoredState().exemplarReviews.equipment).toEqual(
        nextState.exemplarReviews.equipment,
      );
      expect(
        deriveCurrentTagSources({
          recordKey: "equipment:plain-widget",
          name: "Plain Widget",
          category: "equipment",
          subcategory: null,
          descriptionText: null,
          traits: [],
        }),
      ).toMatchObject({
        social_infiltration: "assignment",
      });
      await expect(
        readFile(path.join(tempRoot, "src/tags/reviews/assignment-reviews/index.ts"), "utf8"),
      ).resolves.toContain(
        "DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY",
      );
      await expect(
        readFile(path.join(tempRoot, "src/tags/reviews/assignment-memory/index.ts"), "utf8"),
      ).resolves.toContain(
        "DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY",
      );
      await expect(
        readFile(path.join(tempRoot, "src/tags/reviews/exemplar-reviews/index.ts"), "utf8"),
      ).resolves.toContain(
        "DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY",
      );
      expect(summarizeCurrentDerivedTagReviewQueue()).toEqual(
        expect.arrayContaining([
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
        ]),
      );
    } finally {
      setCurrentDerivedTagAuthoredState(initialState);
    }
  });

  it("sorts the review queue by kind, confidence, count, managed category order, and normalized labels", () => {
    const initialState = getCurrentDerivedTagAuthoredState();

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
      setCurrentDerivedTagAuthoredState(nextState);

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
      setCurrentDerivedTagAuthoredState(initialState);
    }
  });
});
