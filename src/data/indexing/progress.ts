import { formatInteger } from "../format.js";

export type IndexingProgressReporter = {
  progressLogger?: (message: string) => void;
  progressStatusLogger?: (message: string) => void;
};

export const PACK_PROGRESS_BAR_WIDTH = 24;
export const PACK_PROGRESS_LOG_INTERVAL_MS = 5_000;
export const RESOLUTION_PROGRESS_BAR_WIDTH = 24;

type ProgressCounter = {
  shouldReport(processed: number, now?: number): boolean;
};

function renderProgressBar(completed: number, total: number, width = PACK_PROGRESS_BAR_WIDTH): string {
  if (total <= 0) {
    return `[${"-".repeat(width)}]`;
  }

  const boundedCompleted = Math.max(0, Math.min(completed, total));
  const filled = Math.max(0, Math.min(width, Math.round((boundedCompleted / total) * width)));
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

function formatPercentage(completed: number, total: number): string {
  if (total <= 0) {
    return "  0%";
  }

  return `${Math.round((Math.max(0, Math.min(completed, total)) / total) * 100)}`.padStart(3, " ") + "%";
}

function createProgressCounter(input: {
  total: number;
  interval: number;
  includeFirst?: boolean;
  initialLoggedTime?: number;
}): ProgressCounter {
  const { total, interval, includeFirst = false, initialLoggedTime = 0 } = input;
  let lastLoggedProcessed = 0;
  let lastLoggedTime = initialLoggedTime;

  return {
    shouldReport(processed: number, now = Date.now()): boolean {
      const shouldReport =
        (includeFirst && processed === 1) ||
        processed === total ||
        processed - lastLoggedProcessed >= interval ||
        now - lastLoggedTime >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldReport) {
        lastLoggedProcessed = processed;
        lastLoggedTime = now;
      }

      return shouldReport;
    },
  };
}

export function createPackScanProgressCounter(totalFiles: number): ProgressCounter {
  return createProgressCounter({
    total: totalFiles,
    interval: Math.max(100, Math.ceil(totalFiles / 10)),
    includeFirst: true,
  });
}

export function createResolutionProgressCounter(totalRecords: number): ProgressCounter {
  return createProgressCounter({
    total: totalRecords,
    interval: Math.max(100, Math.ceil(Math.max(totalRecords, 1) / 20)),
    initialLoggedTime: Date.now(),
  });
}

export function createRecordWriteProgressCounter(totalRecords: number): ProgressCounter {
  return createProgressCounter({
    total: totalRecords,
    interval: Math.max(100, Math.ceil(totalRecords / 10)),
  });
}

export function createEmbeddingProgressCounter(totalEmbeddings: number): ProgressCounter {
  return createProgressCounter({
    total: totalEmbeddings,
    interval: Math.max(25, Math.ceil(Math.max(totalEmbeddings, 1) / 10)),
  });
}

function formatRecordProgress(processed: number, total: number, noun: string, width = PACK_PROGRESS_BAR_WIDTH): string {
  return `${renderProgressBar(processed, total, width)} ${formatPercentage(processed, total)} (${formatInteger(processed)}/${formatInteger(total)} ${noun})`;
}

export function reportIndexBuildStarted(progress: IndexingProgressReporter, input: { packCount: number }): void {
  progress.progressLogger?.(`Building SQLite index from ${formatInteger(input.packCount)} PF2E packs.`);
}

export function reportPackScanProgress(
  progress: IndexingProgressReporter,
  input: {
    processedPackCount: number;
    totalPackCount: number;
    packLabel: string;
    processedFiles: number;
    totalFiles: number;
    discoveredRecordCount: number;
  },
): void {
  progress.progressStatusLogger?.(
    `[scan ${input.processedPackCount}/${input.totalPackCount}] ${input.packLabel} ${renderProgressBar(input.processedFiles, input.totalFiles)} ${formatPercentage(input.processedFiles, input.totalFiles)} (${formatInteger(input.processedFiles)}/${formatInteger(input.totalFiles)} files, ${formatInteger(input.discoveredRecordCount)} records discovered total).`,
  );
}

export function reportSourceScanCompleted(progress: IndexingProgressReporter): void {
  progress.progressLogger?.("Finished scanning pack files. Resolving verified remaster aliases.");
}

export function reportDerivedTagResolutionStarted(
  progress: IndexingProgressReporter,
  input: { totalRecords: number },
): void {
  progress.progressLogger?.(
    `Finished resolving aliases and references. Deriving tags for ${formatInteger(input.totalRecords)} records.`,
  );
}

export function reportDerivedTagResolutionProgress(
  progress: IndexingProgressReporter,
  input: { resolvedRecords: number; totalRecords: number },
): void {
  if (input.totalRecords <= 0) {
    return;
  }

  progress.progressStatusLogger?.(
    `[resolve] Derived tags ${formatRecordProgress(input.resolvedRecords, input.totalRecords, "records", RESOLUTION_PROGRESS_BAR_WIDTH)}.`,
  );
}

export function reportReferenceResolutionSummary(
  progress: IndexingProgressReporter,
  input: { aliasCount: number; legacyLinkCount: number; canonicalDerivedAfflictionCount: number },
): void {
  progress.progressLogger?.(
    `Resolved ${formatInteger(input.aliasCount)} verified aliases, ${formatInteger(input.legacyLinkCount)} legacy-to-remaster links, and ${formatInteger(input.canonicalDerivedAfflictionCount)} derived affliction canonicals.`,
  );
}

export function reportRecordWritingStarted(progress: IndexingProgressReporter): void {
  progress.progressLogger?.("Writing indexed records and search metadata.");
}

export function reportRecordWriteProgress(
  progress: IndexingProgressReporter,
  input: { writtenRecords: number; totalRecords: number },
): void {
  progress.progressStatusLogger?.(
    `[write] Stored records ${formatRecordProgress(input.writtenRecords, input.totalRecords, "records")}.`,
  );
}

export function reportEmbeddingWritingStarted(
  progress: IndexingProgressReporter,
  input: { batchSize: number },
): void {
  progress.progressLogger?.(`Processing canonical embeddings in batches of ${input.batchSize}.`);
}

export function reportEmbeddingProgress(
  progress: IndexingProgressReporter,
  input: {
    processedEmbeddings: number;
    totalEmbeddings: number;
    reusedEmbeddings: number;
    regeneratedEmbeddings: number;
  },
): void {
  progress.progressStatusLogger?.(
    `[embed] Canonical embeddings ${renderProgressBar(input.processedEmbeddings, input.totalEmbeddings)} ${formatPercentage(input.processedEmbeddings, input.totalEmbeddings)} (${formatInteger(input.processedEmbeddings)}/${formatInteger(input.totalEmbeddings)} embeddings, reused ${formatInteger(input.reusedEmbeddings)}, regenerated ${formatInteger(input.regeneratedEmbeddings)}).`,
  );
}

export function reportEmbeddingReuseSummary(
  progress: IndexingProgressReporter,
  input: { reusedEmbeddings: number; regeneratedEmbeddings: number },
): void {
  progress.progressLogger?.(
    `Canonical embedding reuse summary: reused ${formatInteger(input.reusedEmbeddings)}, regenerated ${formatInteger(input.regeneratedEmbeddings)}.`,
  );
}
