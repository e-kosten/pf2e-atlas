import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import type { BuildIndexResult, ReusableEmbeddingLookup } from "../index-types.js";
import { canonicalizeIndexRecords } from "./canonicalization.js";
import {
  populateMetricCatalog,
  writeAliasCatalogRows,
  writeIndexMetadata,
  writeIndexPacks,
} from "./catalog-writer.js";
import { writeIndexEmbeddings } from "./embedding-writer.js";
import { assignIndexFamilies } from "./family-assignment.js";
import type { IndexingProgressReporter } from "./progress.js";
import { reportSourceScanCompleted } from "./progress.js";
import { createRecordWriteModel } from "./record-write-model.js";
import { writeIndexRecords } from "./record-writer.js";
import { resolveIndexReferences } from "./reference-resolution.js";
import { loadIndexSources } from "./source-loading.js";

export async function buildIndex(
  db: DatabaseSync,
  rootPath: string,
  manifestPath: string,
  embeddingProvider: EmbeddingProvider,
  sourceSignature: string,
  progressLogger?: (message: string) => void,
  progressStatusLogger?: (message: string) => void,
  reusableEmbeddingLookup?: ReusableEmbeddingLookup | null,
): Promise<BuildIndexResult> {
  const progress: IndexingProgressReporter = { progressLogger, progressStatusLogger };

  const scanStartTime = Date.now();

  db.exec("BEGIN");
  try {
    writeIndexMetadata(db, sourceSignature, embeddingProvider);

    const sourceStage = await loadIndexSources(rootPath, manifestPath, progress);
    writeIndexPacks(db, sourceStage.packs);
    const scanNormalizationDurationMs = Date.now() - scanStartTime;

    reportSourceScanCompleted(progress);
    const resolutionStartTime = Date.now();
    const familyStage = assignIndexFamilies(sourceStage.sourceEntries);
    const referenceStage = await resolveIndexReferences({
      indexedEntries: familyStage.indexedEntries,
      sourceEntries: sourceStage.sourceEntries,
      packs: sourceStage.packs,
      rootPath,
    });
    const canonicalStage = canonicalizeIndexRecords(referenceStage.referencedEntries, referenceStage, progress);
    const resolutionDurationMs = Date.now() - resolutionStartTime;

    const writeModel = createRecordWriteModel({
      indexedEntries: canonicalStage.canonicalEntries,
      derivedAfflictions: canonicalStage.derivedAfflictions,
      aliasRows: referenceStage.aliasRows,
      legacyLinkRows: referenceStage.legacyLinkRows,
    });

    const recordCount = writeModel.writableEntries.length;
    const recordWritingStage = writeIndexRecords(
      db,
      writeModel.writableEntries,
      canonicalStage.derivedAfflictions,
      progress,
    );
    const recordStorageDurationMs = recordWritingStage.durationMs;

    const embeddingStage = await writeIndexEmbeddings({
      db,
      embeddingProvider,
      pendingCanonicalEmbeddings: recordWritingStage.pendingCanonicalEmbeddings,
      reusableEmbeddingLookup,
      progress,
    });
    writeAliasCatalogRows({
      db,
      aliasRows: referenceStage.aliasRows,
      legacyLinkRows: referenceStage.legacyLinkRows,
    });
    populateMetricCatalog(db);

    db.exec("COMMIT");

    const packs = [...sourceStage.packs].sort((left, right) => left.label.localeCompare(right.label));
    return {
      packs,
      warnings: sourceStage.warnings,
      recordCount,
      reusedCanonicalEmbeddingCount: embeddingStage.reusedCanonicalEmbeddingCount,
      regeneratedCanonicalEmbeddingCount: embeddingStage.regeneratedCanonicalEmbeddingCount,
      stageTimings: [
        { label: "Scan and normalize records", durationMs: scanNormalizationDurationMs },
        { label: "Resolve families, references, tags, and aliases", durationMs: resolutionDurationMs },
        { label: "Write records and lexical search metadata", durationMs: recordStorageDurationMs },
        { label: "Generate canonical embeddings", durationMs: embeddingStage.embeddingGenerationDurationMs },
        { label: "Insert vector rows", durationMs: embeddingStage.vecInsertDurationMs },
      ],
    };
  } catch (error) {
    db.exec("ROLLBACK");
    throw error;
  }
}
