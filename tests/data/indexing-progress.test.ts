import { describe, expect, it } from "vitest";

import {
  createRecordWriteProgressCounter,
  reportEmbeddingProgress,
  reportPackScanProgress,
  reportRecordWriteProgress,
} from "../../src/data/indexing/progress.js";

describe("indexing progress helpers", () => {
  it("centralizes progress status message formatting", () => {
    const statuses: string[] = [];
    const progress = { progressStatusLogger: (message: string) => statuses.push(message) };

    reportPackScanProgress(progress, {
      processedPackCount: 1,
      totalPackCount: 2,
      packLabel: "Spells",
      processedFiles: 5,
      totalFiles: 10,
      discoveredRecordCount: 3,
    });
    reportRecordWriteProgress(progress, { writtenRecords: 2, totalRecords: 4 });
    reportEmbeddingProgress(progress, {
      processedEmbeddings: 3,
      totalEmbeddings: 6,
      reusedEmbeddings: 1,
      regeneratedEmbeddings: 2,
    });

    expect(statuses).toEqual([
      "[scan 1/2] Spells [############------------]  50% (5/10 files, 3 records discovered total).",
      "[write] Stored records [############------------]  50% (2/4 records).",
      "[embed] Canonical embeddings [############------------]  50% (3/6 embeddings, reused 1, regenerated 2).",
    ]);
  });

  it("centralizes progress timing thresholds", () => {
    const progress = createRecordWriteProgressCounter(1_000);

    expect(progress.shouldReport(50, 1)).toBe(false);
    expect(progress.shouldReport(100, 2)).toBe(true);
    expect(progress.shouldReport(150, 3)).toBe(false);
    expect(progress.shouldReport(150, 5_003)).toBe(true);
    expect(progress.shouldReport(1_000, 5_004)).toBe(true);
  });
});
