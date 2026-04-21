import { describe, expect, it, vi } from "vitest";

import {
  importDerivedTagReviewSession,
  renderDerivedTagReviewSummary,
} from "../../src/tags/editorial/ui/review-controller.js";
import type { DerivedTagReviewSession } from "../../src/tags/editorial/types.js";

function createSession(): DerivedTagReviewSession {
  return {
    manifest: {
      id: "session-1",
      mode: "review_queue",
      createdAt: "2026-04-17T08:00:00.000Z",
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
        resolutionStatus: "complete",
        decisions: [
          {
            kind: "assignment",
            family: "security",
            tag: "alarm",
            mode: "include",
            status: "approved",
            rationale: "Confirmed by review.",
            confidence: "high",
            source: "llm",
          },
        ],
      },
    ],
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: false,
      updatedAt: "2026-04-17T08:30:00.000Z",
    },
  };
}

describe("derived tag review controller", () => {
  it("imports and persists the reviewed session through controller services", async () => {
    const session = createSession();
    const calls: string[] = [];
    const services = {
      lintSession: vi.fn(() => {
        calls.push("lint");
      }),
      importSession: vi.fn(() => {
        calls.push("import");
        return Promise.resolve();
      }),
      writeSession: vi.fn((_rootPath: string, _session: DerivedTagReviewSession) => {
        calls.push("writeSession");
        return Promise.resolve();
      }),
      writeSummary: vi.fn((_rootPath: string, _sessionId: string, summary: string) => {
        calls.push("writeSummary");
        expect(summary).toContain("Session: session-1");
        expect(summary).toContain("Actionable records resolved: 1/1");
        return Promise.resolve();
      }),
    };

    await importDerivedTagReviewSession("/tmp/review-controller", session, services);

    expect(calls).toEqual(["lint", "import", "writeSession", "writeSummary"]);
    expect(services.writeSession).toHaveBeenCalledWith("/tmp/review-controller", session);
    expect(services.writeSummary).toHaveBeenCalledWith(
      "/tmp/review-controller",
      "session-1",
      expect.stringContaining("Mode: review_queue"),
    );
  });

  it("renders a review summary with progress details", () => {
    const summary = renderDerivedTagReviewSummary(createSession());

    expect(summary).toContain("Candidate records: 1");
    expect(summary).toContain("Visible review items: 1");
    expect(summary).toContain("Updated at: 2026-04-17T08:30:00.000Z");
  });
});
