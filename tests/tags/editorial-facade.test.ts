import { describe, expect, it } from "vitest";

import * as editorial from "../../src/tags/editorial.js";
import * as editorialUi from "../../src/tags/editorial-ui.js";

describe("editorial facade", () => {
  it("does not re-export app ontology entity-record helpers", () => {
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntityDetailLines");
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntityRecordSelectColumns");
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntitySummary");
    expect(editorial).not.toHaveProperty("mapNormalizedRecordToOntologyExplorerEntityRecord");
    expect(editorial).not.toHaveProperty("mapOntologyExplorerEntityRecordRow");
  });

  it("keeps UI-only workbench exports out of the non-UI editorial facade", () => {
    expect(editorial).toHaveProperty("createDerivedTagWorkbenchSession");
    expect(editorial).toHaveProperty("getDerivedTagWorkbenchQueueItems");
    expect(editorial).toHaveProperty("getPublishedDerivedTagConceptModel");
    expect(editorial).toHaveProperty("listPublishedDerivedTagTranslations");
    expect(editorial).toHaveProperty("summarizeCurrentDerivedTagTranslationQueue");
    expect(editorial).not.toHaveProperty("promptAndCreateDerivedTagWorkbenchSession");
    expect(editorial).not.toHaveProperty("formatDerivedTagWorkbenchModeLabel");
    expect(editorial).not.toHaveProperty("DerivedTagReviewScreen");
  });

  it("exposes a translation-review summary without requiring the UI workbench", () => {
    const summary = editorial.summarizeCurrentDerivedTagTranslationQueue();
    expect(summary.length).toBeGreaterThan(0);
    expect(summary.some((entry) => entry.translationStatus === "provisional")).toBe(true);
  });
});

describe("editorial UI facade", () => {
  it("exposes the review UI and prompt-driven workbench helpers", () => {
    expect(editorialUi).toHaveProperty("DerivedTagReviewScreen");
    expect(editorialUi).toHaveProperty("promptAndCreateDerivedTagWorkbenchSession");
    expect(editorialUi).toHaveProperty("formatDerivedTagWorkbenchModeLabel");
  });
});
