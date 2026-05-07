import { readFile } from "node:fs/promises";

import type { PackInfo, PackManifestEntry } from "../../domain/record-types.js";
import { isExcludedPackName } from "../record-normalization.js";
import type { BuildSourceEntry, PackBuildInfo } from "../index-types.js";
import { resolvePackPath, walkJsonFiles } from "./source-discovery.js";
import { normalizeSourceEntry } from "./record-normalization.js";
import {
  createPackScanProgressCounter,
  reportIndexBuildStarted,
  reportPackScanProgress,
  type IndexingProgressReporter,
} from "./progress.js";

export type SourceLoadingStageResult = {
  packs: PackInfo[];
  warnings: string[];
  sourceEntries: BuildSourceEntry[];
  recordCount: number;
};

type ManifestRaw = {
  packs?: PackManifestEntry[];
};

function parseManifestPacks(serializedManifest: string): PackManifestEntry[] {
  const manifestRaw = JSON.parse(serializedManifest) as ManifestRaw;
  return Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];
}

function parseRawRecord(serializedRecord: string): Record<string, unknown> {
  return JSON.parse(serializedRecord) as Record<string, unknown>;
}

export async function loadIndexSources(
  rootPath: string,
  manifestPath: string,
  progress: IndexingProgressReporter = {},
): Promise<SourceLoadingStageResult> {
  const manifestPacks = parseManifestPacks(await readFile(manifestPath, "utf8"));
  const includedManifestPacks = manifestPacks.filter((manifestPack) => !isExcludedPackName(manifestPack.name));

  const warnings: string[] = [];
  const packs: PackInfo[] = [];
  const sourceEntries: BuildSourceEntry[] = [];
  let recordCount = 0;
  let processedPackCount = 0;

  reportIndexBuildStarted(progress, { packCount: includedManifestPacks.length });

  for (const manifestPack of manifestPacks) {
    const resolvedPath = await resolvePackPath(rootPath, manifestPack);
    if (!resolvedPath) {
      warnings.push(`Skipping pack ${manifestPack.name}: could not resolve a readable directory.`);
      continue;
    }

    const pack: PackBuildInfo = {
      name: manifestPack.name,
      label: manifestPack.label,
      documentType: manifestPack.type,
      declaredPath: manifestPack.path,
      resolvedPath,
    };

    if (isExcludedPackName(pack.name)) {
      continue;
    }

    processedPackCount += 1;

    let filePaths: string[];
    try {
      filePaths = await walkJsonFiles(pack.resolvedPath);
    } catch (error) {
      warnings.push(`Skipping pack ${pack.name}: ${(error as Error).message}`);
      continue;
    }

    let packRecordCount = 0;
    const packScanProgress = createPackScanProgressCounter(filePaths.length);

    for (const [fileIndex, filePath] of filePaths.entries()) {
      const sourceEntry = normalizeSourceEntry(pack, filePath, parseRawRecord(await readFile(filePath, "utf8")));
      sourceEntries.push(sourceEntry);

      if (sourceEntry.record !== null) {
        packRecordCount += 1;
        recordCount += 1;
      }

      const processedFiles = fileIndex + 1;
      if (packScanProgress.shouldReport(processedFiles)) {
        reportPackScanProgress(progress, {
          processedPackCount,
          totalPackCount: includedManifestPacks.length,
          packLabel: pack.label,
          processedFiles,
          totalFiles: filePaths.length,
          discoveredRecordCount: recordCount,
        });
      }
    }

    if (packRecordCount === 0) {
      continue;
    }

    packs.push({ ...pack, recordCount: packRecordCount });
  }

  return {
    packs,
    warnings,
    sourceEntries,
    recordCount,
  };
}
