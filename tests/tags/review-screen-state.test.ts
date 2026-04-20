import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  createInitialDerivedTagMigrationReviewScreenState,
  reduceDerivedTagMigrationReviewScreenState,
} from "../../src/tags/editorial/review-screen-state.js";
import type { DerivedTagMigrationSession } from "../../src/tags/editorial/types.js";

function createSession(): DerivedTagMigrationSession {
  return {
    manifest: {
      id: "session-1",
      mode: "review_queue",
      createdAt: "2026-04-18T08:00:00.000Z",
      recordCount: 2,
    },
    records: [
      {
        entityRecord: {
          recordKey: "spell:alarm-ward",
          packName: "spell",
          name: "Alarm Ward",
          type: "spell",
          category: "spell",
          subcategory: null,
          documentType: "Item",
          level: 1,
          rarity: null,
          traits: [],
          derivedTags: ["alarm"],
          families: ["security"],
          descriptionText: "Warns against intrusion.",
          blurbText: null,
          sourceCategory: "core",
          publicationTitle: null,
          publicationRemaster: false,
          isUnique: false,
          size: null,
          languages: [],
          speedTypes: [],
          senses: [],
          immunities: [],
          resistances: [],
          weaknesses: [],
          itemCategory: null,
          baseItem: null,
          priceCp: null,
          usage: null,
          hands: null,
          damageTypes: [],
          weaponGroup: null,
          armorGroup: null,
          traditions: ["arcane"],
          spellKinds: ["spell"],
          saveType: null,
          areaType: null,
          rangeText: "30 feet",
          durationText: "1 minute",
          targetText: "1 creature",
          areaValue: null,
          sustained: false,
          basicSave: false,
          disableText: null,
          disableSkills: [],
          isComplex: false,
        },
        currentSources: {},
        selectionReasons: [{ source: "authored_review_queue", family: "security", tag: "alarm", note: "First." }],
      },
      {
        entityRecord: {
          recordKey: "spell:warding-glyph",
          packName: "spell",
          name: "Warding Glyph",
          type: "spell",
          category: "spell",
          subcategory: null,
          documentType: "Item",
          level: 3,
          rarity: null,
          traits: [],
          derivedTags: ["alarm"],
          families: ["security"],
          descriptionText: "Defensive glyph.",
          blurbText: null,
          sourceCategory: "core",
          publicationTitle: null,
          publicationRemaster: false,
          isUnique: false,
          size: null,
          languages: [],
          speedTypes: [],
          senses: [],
          immunities: [],
          resistances: [],
          weaknesses: [],
          itemCategory: null,
          baseItem: null,
          priceCp: null,
          usage: null,
          hands: null,
          damageTypes: [],
          weaponGroup: null,
          armorGroup: null,
          traditions: ["arcane"],
          spellKinds: ["spell"],
          saveType: null,
          areaType: null,
          rangeText: "touch",
          durationText: "sustained",
          targetText: "1 object",
          areaValue: null,
          sustained: true,
          basicSave: false,
          disableText: null,
          disableSkills: [],
          isComplex: false,
        },
        currentSources: {},
        selectionReasons: [{ source: "authored_review_queue", family: "security", tag: "alarm", note: "Second." }],
      },
    ],
    decisions: [
      {
        recordKey: "spell:alarm-ward",
        name: "Alarm Ward",
        category: "spell",
        resolutionStatus: "needs_review",
        decisions: [
          {
            kind: "assignment",
            family: "security",
            tag: "alarm",
            mode: "include",
            status: "needs_review",
            rationale: "Pending confirmation.",
            confidence: "high",
            source: "llm",
          },
        ],
      },
      {
        recordKey: "spell:warding-glyph",
        name: "Warding Glyph",
        category: "spell",
        resolutionStatus: "needs_review",
        decisions: [
          {
            kind: "assignment",
            family: "security",
            tag: "alarm",
            mode: "include",
            status: "approved",
            rationale: "Already reviewed.",
            confidence: "high",
            source: "llm",
          },
        ],
      },
    ],
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: false,
      updatedAt: "2026-04-18T08:30:00.000Z",
    },
  };
}

describe("review screen state", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-19T09:45:00.000Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("wraps list navigation and resets detail scroll", () => {
    const initial = createInitialDerivedTagMigrationReviewScreenState(createSession());
    const next = reduceDerivedTagMigrationReviewScreenState(
      {
        ...initial,
        detailScroll: 6,
        session: {
          ...initial.session,
          reviewState: {
            ...initial.session.reviewState,
            currentIndex: 0,
          },
        },
      },
      { type: "move_list_wrapped", delta: -1, itemCount: 2 },
    );

    expect(next.detailScroll).toBe(0);
    expect(next.session.reviewState.currentIndex).toBe(1);
  });

  it("toggles unresolved-only mode and clamps the queue to visible items", () => {
    const initial = createInitialDerivedTagMigrationReviewScreenState({
      ...createSession(),
      reviewState: {
        currentIndex: 1,
        unresolvedOnly: false,
        updatedAt: "2026-04-18T08:30:00.000Z",
      },
    });

    const next = reduceDerivedTagMigrationReviewScreenState(initial, { type: "toggle_unresolved" });

    expect(next.detailScroll).toBe(0);
    expect(next.session.reviewState.unresolvedOnly).toBe(true);
    expect(next.session.reviewState.currentIndex).toBe(0);
    expect(next.session.reviewState.updatedAt).toBe("2026-04-19T09:45:00.000Z");
  });
});
