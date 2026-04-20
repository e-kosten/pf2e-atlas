import { mkdir, rename } from "node:fs/promises";
import path from "node:path";

import { DatabaseSync } from "node:sqlite";

import { createEmbeddingProvider } from "../../embeddings.js";
import { formatInteger } from "../../shared/format.js";
import { fileExists } from "../../shared/fs.js";
import type { EmbeddingConfig } from "../../domain/index.js";
import type { StageTiming } from "../index-types.js";
import { buildIndex, buildReusableEmbeddingLookup, computeSourceSignature, removeIndexFiles } from "../indexer.js";
import {
  buildMissingIndexError,
  buildStaleIndexError,
  createSchema,
  defaultEmbeddingConfig,
  defaultIndexPath,
  getEmbeddingReuseInvalidReason,
  getIndexInvalidReason,
  loadPacksFromIndex,
  openDatabase,
} from "../schema.js";
import { sqliteRowCount } from "../rows.js";
import type { Pf2eDataServiceLoadOptions, Pf2eLoadedDataRuntime } from "./types.js";

function formatDurationMs(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

async function moveIndexFiles(sourcePath: string, targetPath: string): Promise<void> {
  await rename(sourcePath, targetPath);

  for (const suffix of ["-wal", "-shm"]) {
    const sourceSidecar = `${sourcePath}${suffix}`;
    if (await fileExists(sourceSidecar)) {
      await rename(sourceSidecar, `${targetPath}${suffix}`);
    }
  }
}

export async function loadPf2eDataRuntime(
  rootPath: string,
  manifestPath: string,
  options: Pf2eDataServiceLoadOptions = {},
): Promise<Pf2eLoadedDataRuntime> {
  const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
  const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
  const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
  const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
  const embeddingProvider = embeddingRuntime.provider;
  const sourceSignature = await computeSourceSignature(rootPath, manifestPath);

  if (!(await fileExists(indexPath))) {
    throw buildMissingIndexError(indexPath);
  }

  const existingDb = openDatabase(indexPath, {
    vectorExtensionLoader: options.vectorExtensionLoader,
  });
  const invalidReason = getIndexInvalidReason(existingDb, sourceSignature, embeddingProvider);
  if (invalidReason) {
    existingDb.close();
    throw buildStaleIndexError(indexPath, invalidReason);
  }

  const packs = loadPacksFromIndex(existingDb);
  const recordCount = sqliteRowCount(
    existingDb.prepare("SELECT COUNT(*) AS total FROM records").get() as Record<string, unknown> | undefined,
  );

  return {
    db: existingDb,
    packs,
    warnings: [...embeddingRuntime.warnings, ...(options.rankingConfigStore?.warnings ?? [])],
    recordCount,
    embeddingProvider,
    rankingConfigStore: options.rankingConfigStore ?? null,
  };
}

export async function rebuildPf2eDataRuntime(
  rootPath: string,
  manifestPath: string,
  options: Pf2eDataServiceLoadOptions = {},
): Promise<Pf2eLoadedDataRuntime> {
  const rebuildStartTime = Date.now();
  const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
  const tempIndexPath = `${indexPath}.rebuild-${process.pid}-${Date.now()}`;
  const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
  const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;

  options.progressLogger?.("Loading the configured embedding provider.");
  const embeddingProviderLoadStartTime = Date.now();
  const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
  const embeddingProviderLoadDurationMs = Date.now() - embeddingProviderLoadStartTime;
  const embeddingProvider = embeddingRuntime.provider;
  options.progressLogger?.(
    `Embedding provider ready: ${embeddingProvider.identity.model} (${embeddingProvider.identity.dimensions} dimensions).`,
  );

  options.progressLogger?.("Computing the PF2E source signature.");
  const sourceSignatureStartTime = Date.now();
  const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
  const sourceSignatureDurationMs = Date.now() - sourceSignatureStartTime;

  options.progressLogger?.(`Preparing index output at ${indexPath}.`);
  const prepareOutputStartTime = Date.now();
  await mkdir(path.dirname(indexPath), { recursive: true });
  await removeIndexFiles(tempIndexPath);

  let previousDb: DatabaseSync | null = null;
  let reusableEmbeddingLookup = null;
  if (options.reuseEmbeddings) {
    if (await fileExists(indexPath)) {
      try {
        previousDb = openDatabase(indexPath, {
          vectorExtensionLoader: options.vectorExtensionLoader,
        });
        const reuseInvalidReason = getEmbeddingReuseInvalidReason(previousDb, embeddingProvider);
        if (reuseInvalidReason) {
          options.progressLogger?.(
            `Embedding reuse unavailable: ${reuseInvalidReason}. Regenerating all canonical embeddings.`,
          );
          previousDb.close();
          previousDb = null;
        } else {
          options.progressLogger?.(
            "Reusing unchanged canonical embeddings from the existing index when semantic inputs match.",
          );
          reusableEmbeddingLookup = buildReusableEmbeddingLookup(previousDb);
        }
      } catch (error) {
        options.progressLogger?.(
          `Embedding reuse unavailable: ${(error as Error).message}. Regenerating all canonical embeddings.`,
        );
        previousDb?.close();
        previousDb = null;
      }
    } else {
      options.progressLogger?.(
        "Embedding reuse unavailable: no existing index found. Regenerating all canonical embeddings.",
      );
    }
  }

  const prepareOutputDurationMs = Date.now() - prepareOutputStartTime;

  let tempDb: DatabaseSync | null = null;
  let finalDb: DatabaseSync | null = null;

  try {
    tempDb = openDatabase(tempIndexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    options.progressLogger?.("Creating SQLite schema.");
    const schemaCreationStartTime = Date.now();
    createSchema(tempDb, embeddingProvider.identity.dimensions);
    const schemaCreationDurationMs = Date.now() - schemaCreationStartTime;
    const { packs, warnings, recordCount, stageTimings } = await buildIndex(
      tempDb,
      rootPath,
      manifestPath,
      embeddingProvider,
      sourceSignature,
      options.progressLogger,
      options.progressStatusLogger,
      reusableEmbeddingLookup,
    );
    options.progressLogger?.(
      `Finished writing ${formatInteger(recordCount)} records across ${formatInteger(packs.length)} packs.`,
    );
    tempDb.close();
    tempDb = null;
    previousDb?.close();
    previousDb = null;
    await removeIndexFiles(indexPath);
    await moveIndexFiles(tempIndexPath, indexPath);

    finalDb = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    const rebuildDurationMs = Date.now() - rebuildStartTime;
    const summaryTimings: StageTiming[] = [
      { label: "Embedding provider load", durationMs: embeddingProviderLoadDurationMs },
      { label: "Source signature", durationMs: sourceSignatureDurationMs },
      { label: "Prepare index output", durationMs: prepareOutputDurationMs },
      { label: "Create SQLite schema", durationMs: schemaCreationDurationMs },
      ...stageTimings,
      { label: "Total rebuild time", durationMs: rebuildDurationMs },
    ];
    options.progressLogger?.("Index rebuild stage timings:");
    for (const timing of summaryTimings) {
      options.progressLogger?.(`- ${timing.label}: ${formatDurationMs(timing.durationMs)}`);
    }

    return {
      db: finalDb,
      packs,
      warnings: [...embeddingRuntime.warnings, ...warnings, ...(options.rankingConfigStore?.warnings ?? [])],
      recordCount,
      embeddingProvider,
      rankingConfigStore: options.rankingConfigStore ?? null,
    };
  } catch (error) {
    finalDb?.close();
    tempDb?.close();
    previousDb?.close();
    await removeIndexFiles(tempIndexPath);
    throw error;
  }
}
