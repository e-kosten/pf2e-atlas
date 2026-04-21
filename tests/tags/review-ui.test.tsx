import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import {
  DerivedTagReviewScreen,
  type DerivedTagReviewResult,
} from "../../src/tags/editorial/ui/review-ui.js";
import type { DerivedTagReviewServices } from "../../src/tags/editorial/ui/review-controller.js";
import type { DerivedTagReviewSession } from "../../src/tags/editorial/types.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function pressLeft(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[D");
}

function createSession(): DerivedTagReviewSession {
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

function createServices(): DerivedTagReviewServices {
  return {
    importSession: vi.fn(() => Promise.resolve()),
    lintSession: vi.fn(),
    writeSession: vi.fn(() => Promise.resolve()),
    writeSummary: vi.fn(() => Promise.resolve()),
  };
}

function createOnCompleteSpy() {
  return vi.fn((_result: DerivedTagReviewResult) => undefined);
}

afterEach(() => {
  cleanup();
});

describe("review ui", () => {
  it("applies the selected review action through the shared action rail", async () => {
    const onComplete = createOnCompleteSpy();
    const app = render(
      <DerivedTagTerminalProvider>
        <DerivedTagReviewScreen
          rootPath="/tmp/review-ui"
          initialSession={createSession()}
          onComplete={onComplete}
          services={createServices()}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write(":");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Alarm Ward | approved | security.alarm include");
  });

  it("treats left arrow as action-rail navigation while the rail is focused", async () => {
    const onComplete = createOnCompleteSpy();
    const app = render(
      <DerivedTagTerminalProvider>
        <DerivedTagReviewScreen
          rootPath="/tmp/review-ui"
          initialSession={createSession()}
          onComplete={onComplete}
          services={createServices()}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write(":");
    await flushInk();
    pressLeft(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onComplete).toHaveBeenCalledTimes(1);
    const [result] = onComplete.mock.calls[0] ?? [];
    expect(result).toBeDefined();
    expect(result).toMatchObject({
      imported: false,
      session: {
        manifest: {
          id: "session-1",
        },
      },
    });
  });

  it("shows the controller-owned help dialog", async () => {
    const onComplete = createOnCompleteSpy();
    const app = render(
      <DerivedTagTerminalProvider>
        <DerivedTagReviewScreen
          rootPath="/tmp/review-ui"
          initialSession={createSession()}
          onComplete={onComplete}
          services={createServices()}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("?");
    await flushInk();

    expect(app.lastFrame()).toContain("Derived-Tag Review Help");
    expect(app.lastFrame()).toContain("Enter Actions");
  });
});
