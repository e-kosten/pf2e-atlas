import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import {
  createDerivedTagTranslationReviewSession,
  getVisibleDerivedTagOntology,
  importDerivedTagTranslationReviewSession,
  listCurrentDerivedTagTranslationQueueItems,
} from "../../src/tags/editorial.js";
import {
  getCurrentDerivedTagTranslationOverrides,
  getCurrentDerivedTagFamilyTranslationDefaults,
  setCurrentDerivedTagFamilyTranslationDefaults,
  setCurrentDerivedTagTranslationOverrides,
} from "../../src/tags/translations/state.js";

const initialOverrides = getCurrentDerivedTagTranslationOverrides();
const initialFamilyDefaults = getCurrentDerivedTagFamilyTranslationDefaults();
const tempRoots: string[] = [];

afterEach(async () => {
  setCurrentDerivedTagTranslationOverrides(initialOverrides);
  setCurrentDerivedTagFamilyTranslationDefaults(initialFamilyDefaults);
  await Promise.all(
    tempRoots.splice(0, tempRoots.length).map(async (root) => {
      await rm(root, { recursive: true, force: true });
    }),
  );
});

describe("translation review session import", () => {
  it("writes override changes back to tag-overrides.ts and refreshes the unresolved queue", async () => {
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "pf2e-translation-session-"));
    tempRoots.push(tempRoot);

    const session = createDerivedTagTranslationReviewSession();
    const targetRow = session.rows.find(
      (row) => row.base.translationStatus === "provisional" || row.base.translationStatus === "unmapped",
    );
    expect(targetRow).toBeDefined();
    if (!targetRow) {
      return;
    }

    targetRow.draftOverride = {
      ...targetRow.draftOverride,
      translationStatus: "mapped",
    };

    await importDerivedTagTranslationReviewSession(tempRoot, session);

    const queueKeys = new Set(
      listCurrentDerivedTagTranslationQueueItems().map((entry) => `${entry.currentCategory}:${entry.currentTag}`),
    );
    expect(queueKeys.has(targetRow.key)).toBe(false);

    const visibleOntology = getVisibleDerivedTagOntology();
    expect(
      visibleOntology.tags.some(
        (tag) => tag.category === targetRow.base.currentCategory && tag.tag === targetRow.base.currentTag,
      ),
    ).toBe(true);

    const writtenFile = await readFile(
      path.join(tempRoot, "src", "tags", "translations", "tag-overrides.ts"),
      "utf8",
    );
    expect(writtenFile).toContain(targetRow.key);
    expect(writtenFile).toContain('translationStatus: "mapped"');
  });

  it("promotes uniform family-wide edits into family-defaults.ts", async () => {
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "pf2e-translation-family-defaults-"));
    tempRoots.push(tempRoot);

    const session = createDerivedTagTranslationReviewSession();
    const planarRows = session.rows.filter(
      (row) => row.base.currentCategory === "creature" && row.base.currentFamily === "planar_setting",
    );
    expect(planarRows.length).toBeGreaterThan(1);

    for (const row of planarRows) {
      row.draftOverride = {
        ...row.draftOverride,
        translationStatus: "mapped",
      };
    }

    await importDerivedTagTranslationReviewSession(tempRoot, session);

    const queueEntries = listCurrentDerivedTagTranslationQueueItems({
      category: "creature",
      statuses: ["provisional", "unmapped"],
    });
    expect(queueEntries.some((entry) => entry.currentFamily === "planar_setting")).toBe(false);

    const familyDefaultsFile = await readFile(
      path.join(tempRoot, "src", "tags", "translations", "family-defaults.ts"),
      "utf8",
    );
    expect(familyDefaultsFile).toContain('"creature:planar_setting"');
    expect(familyDefaultsFile).toContain('translationStatus: "mapped"');

  });
});
