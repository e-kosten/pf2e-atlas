import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import { normalizeText } from "../../shared/utils.js";
import type {
  PendingCanonicalEmbeddingWithHash,
  ReusableEmbeddingLookup,
  ReusableEmbeddingRow,
} from "../index-types.js";
import {
  createEmbeddingProgressCounter,
  reportEmbeddingProgress,
  reportEmbeddingReuseSummary,
  reportEmbeddingWritingStarted,
  type IndexingProgressReporter,
} from "./progress.js";

const VEC_TEXT_NONE = "";
const VEC_INT_NONE = -1n;
const EMBEDDING_BATCH_SIZE = 64;

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function normalizeVecText(value: string | null | undefined): string {
  return normalizeText(value ?? "") || VEC_TEXT_NONE;
}

function normalizeVecInteger(value: number | null | undefined): bigint {
  return value === null || value === undefined ? VEC_INT_NONE : BigInt(value);
}

export function buildReusableEmbeddingLookup(db: DatabaseSync): ReusableEmbeddingLookup {
  const selectReusableEmbedding = db.prepare(`
    SELECT
      semantic_input_hash AS semanticInputHash,
      dimensions,
      vector_blob AS vectorBlob
    FROM embeddings
    WHERE record_key = ?
  `);

  return {
    get(recordKey: string): ReusableEmbeddingRow | null {
      return selectReusableEmbedding.get(recordKey) as ReusableEmbeddingRow | null;
    },
  };
}

export type EmbeddingWritingStageResult = {
  embeddingGenerationDurationMs: number;
  vecInsertDurationMs: number;
  reusedCanonicalEmbeddingCount: number;
  regeneratedCanonicalEmbeddingCount: number;
};

export async function writeIndexEmbeddings(input: {
  db: DatabaseSync;
  embeddingProvider: EmbeddingProvider;
  pendingCanonicalEmbeddings: PendingCanonicalEmbeddingWithHash[];
  reusableEmbeddingLookup?: ReusableEmbeddingLookup | null;
  progress?: IndexingProgressReporter;
}): Promise<EmbeddingWritingStageResult> {
  const { db, embeddingProvider, pendingCanonicalEmbeddings, reusableEmbeddingLookup, progress = {} } = input;
  const insertEmbedding = db.prepare(`
    INSERT INTO embeddings (record_key, dimensions, semantic_input_hash, vector_blob) VALUES (?, ?, ?, ?)
  `);
  const insertVecEmbedding = db.prepare(`
    INSERT INTO record_embeddings (
      record_key,
      embedding,
      category,
      subcategory,
      pack_name,
      pack_label,
      document_type,
      record_type,
      level,
      rarity,
      source_category,
      publication_title,
      publication_remaster,
      has_description,
      is_unique,
      size,
      item_category,
      price_cp,
      action_cost
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const canonicalEmbeddingCount = pendingCanonicalEmbeddings.length;
  const embeddingProgress = createEmbeddingProgressCounter(canonicalEmbeddingCount);
  let embeddingGenerationDurationMs = 0;
  let vecInsertDurationMs = 0;
  let reusedCanonicalEmbeddingCount = 0;
  let regeneratedCanonicalEmbeddingCount = 0;
  let processedCanonicalEmbeddingCount = 0;

  reportEmbeddingWritingStarted(progress, { batchSize: EMBEDDING_BATCH_SIZE });

  const insertVectorRows = (entry: PendingCanonicalEmbeddingWithHash, vectorBlob: Buffer | Uint8Array): void => {
    const record = entry.record;
    insertEmbedding.run(record.recordKey, embeddingProvider.identity.dimensions, entry.semanticInputHash, vectorBlob);
    insertVecEmbedding.run(
      record.recordKey,
      vectorBlob,
      normalizeVecText(record.category),
      normalizeVecText(record.subcategory),
      normalizeVecText(record.packName),
      normalizeVecText(record.packLabel),
      normalizeVecText(record.documentType),
      normalizeVecText(record.type),
      normalizeVecInteger(record.level),
      normalizeVecText(record.rarity),
      normalizeVecText(record.sourceCategory),
      normalizeVecText(record.publicationTitle),
      BigInt(record.publicationRemaster ? 1 : 0),
      BigInt(record.hasDescription ? 1 : 0),
      BigInt(record.isUnique ? 1 : 0),
      normalizeVecText(record.size),
      normalizeVecText(record.itemCategory),
      normalizeVecInteger(record.priceCp),
      normalizeVecInteger(record.actionCost),
    );
  };

  for (let index = 0; index < pendingCanonicalEmbeddings.length; index += EMBEDDING_BATCH_SIZE) {
    const batch = pendingCanonicalEmbeddings.slice(index, index + EMBEDDING_BATCH_SIZE);
    const pendingRegeneration: PendingCanonicalEmbeddingWithHash[] = [];

    const vecInsertStartTime = Date.now();
    for (const entry of batch) {
      const reusableEmbedding = reusableEmbeddingLookup?.get(entry.record.recordKey) ?? null;

      if (
        reusableEmbedding &&
        reusableEmbedding.semanticInputHash === entry.semanticInputHash &&
        reusableEmbedding.dimensions === embeddingProvider.identity.dimensions
      ) {
        insertVectorRows(entry, reusableEmbedding.vectorBlob);
        reusedCanonicalEmbeddingCount += 1;
        continue;
      }

      pendingRegeneration.push(entry);
    }

    if (pendingRegeneration.length > 0) {
      const embeddingStartTime = Date.now();
      const embeddings = await embeddingProvider.embedMany(
        pendingRegeneration.map((entry) => entry.encodedEmbeddingInput),
      );
      embeddingGenerationDurationMs += Date.now() - embeddingStartTime;

      for (const [batchIndex, entry] of pendingRegeneration.entries()) {
        const embedding = embeddings[batchIndex] ?? new Float32Array(embeddingProvider.identity.dimensions);
        insertVectorRows(entry, encodeVector(embedding));
        regeneratedCanonicalEmbeddingCount += 1;
      }
    }
    vecInsertDurationMs += Date.now() - vecInsertStartTime;

    processedCanonicalEmbeddingCount += batch.length;
    if (embeddingProgress.shouldReport(processedCanonicalEmbeddingCount)) {
      reportEmbeddingProgress(progress, {
        processedEmbeddings: processedCanonicalEmbeddingCount,
        totalEmbeddings: canonicalEmbeddingCount,
        reusedEmbeddings: reusedCanonicalEmbeddingCount,
        regeneratedEmbeddings: regeneratedCanonicalEmbeddingCount,
      });
    }
  }

  reportEmbeddingReuseSummary(progress, {
    reusedEmbeddings: reusedCanonicalEmbeddingCount,
    regeneratedEmbeddings: regeneratedCanonicalEmbeddingCount,
  });

  return {
    embeddingGenerationDurationMs,
    vecInsertDurationMs,
    reusedCanonicalEmbeddingCount,
    regeneratedCanonicalEmbeddingCount,
  };
}
