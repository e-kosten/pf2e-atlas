import { describe, expect, it } from "vitest";

import { buildDerivedTagMigrationReviewViewModel } from "../../src/tags/editorial/review-screen-model.js";
import { createInitialDerivedTagMigrationReviewScreenState } from "../../src/tags/editorial/review-screen-state.js";
import type { DerivedTagMigrationSession } from "../../src/tags/editorial/types.js";

function createSession(): DerivedTagMigrationSession {
  return {
    manifest: {
      id: "session-1",
      mode: "review_queue",
      createdAt: "2026-04-18T08:00:00.000Z",
      recordCount: 1,
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
        selectionReasons: [
          {
            source: "authored_review_queue",
            family: "security",
            tag: "alarm",
            note: "Pending assignment review.",
          },
        ],
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
    ],
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: false,
      updatedAt: "2026-04-18T08:30:00.000Z",
    },
  };
}

describe("review screen model", () => {
  it("builds the split review screen and help content from pure state", () => {
    const state = createInitialDerivedTagMigrationReviewScreenState(createSession());
    const model = buildDerivedTagMigrationReviewViewModel({
      persistError: null,
      size: { width: 120, height: 30 },
      state,
    });

    expect(model.screen.kind).toBe("two-pane");
    if (model.screen.kind !== "two-pane") {
      throw new Error("expected two-pane screen");
    }
    expect(model.screen.props.subtitle).toContain("1 visible item");
    expect(model.screen.props.left.lines[0]?.text).toContain("Alarm Ward | needs_review | security.alarm include");
    expect(
      model.screen.props.right.lines.some((line) => line.text === "Selection reasons: Pending assignment review."),
    ).toBe(true);
    expect(model.helpLines.some((line) => line.text.includes("Current Action Rail"))).toBe(true);
  });
});
