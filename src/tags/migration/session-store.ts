import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  DerivedTagMigrationRecordDecision,
  DerivedTagMigrationSession,
  DerivedTagMigrationSessionManifest,
  DerivedTagMigrationSessionRecord,
  DerivedTagMigrationSessionReviewState,
} from "./types.js";

function sessionRoot(rootPath: string): string {
  return path.join(rootPath, "scratch", "migration-sessions");
}

export function migrationSessionDirectory(rootPath: string, sessionId: string): string {
  return path.join(sessionRoot(rootPath), sessionId);
}

function lineSeparatedJson<T>(records: T[]): string {
  return records.map((record) => JSON.stringify(record)).join("\n") + (records.length > 0 ? "\n" : "");
}

function parseJsonLines<T>(value: string): T[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => JSON.parse(line) as T);
}

export async function writeDerivedTagMigrationSession(
  rootPath: string,
  session: DerivedTagMigrationSession,
): Promise<void> {
  const directory = migrationSessionDirectory(rootPath, session.manifest.id);
  await mkdir(directory, { recursive: true });

  await writeFile(path.join(directory, "manifest.json"), JSON.stringify(session.manifest, null, 2) + "\n", "utf8");
  await writeFile(path.join(directory, "records.jsonl"), lineSeparatedJson(session.records), "utf8");
  await writeFile(path.join(directory, "decisions.jsonl"), lineSeparatedJson(session.decisions), "utf8");
  await writeFile(path.join(directory, "review-state.json"), JSON.stringify(session.reviewState, null, 2) + "\n", "utf8");
}

export async function readDerivedTagMigrationSession(
  rootPath: string,
  sessionId: string,
): Promise<DerivedTagMigrationSession> {
  const directory = migrationSessionDirectory(rootPath, sessionId);
  const [manifestRaw, recordsRaw, decisionsRaw, reviewStateRaw] = await Promise.all([
    readFile(path.join(directory, "manifest.json"), "utf8"),
    readFile(path.join(directory, "records.jsonl"), "utf8"),
    readFile(path.join(directory, "decisions.jsonl"), "utf8"),
    readFile(path.join(directory, "review-state.json"), "utf8"),
  ]);

  return {
    manifest: JSON.parse(manifestRaw) as DerivedTagMigrationSessionManifest,
    records: parseJsonLines<DerivedTagMigrationSessionRecord>(recordsRaw),
    decisions: parseJsonLines<DerivedTagMigrationRecordDecision>(decisionsRaw),
    reviewState: JSON.parse(reviewStateRaw) as DerivedTagMigrationSessionReviewState,
  };
}
