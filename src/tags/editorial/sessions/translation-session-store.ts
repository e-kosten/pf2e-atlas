import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  DerivedTagTranslationReviewFilterStatus,
  DerivedTagTranslationReviewRow,
  DerivedTagTranslationReviewSession,
  DerivedTagTranslationReviewSessionManifest,
  DerivedTagTranslationReviewSessionState,
} from "../types.js";
import { normalizeSearchCategory } from "../../../domain/categories.js";
import { parseSessionJson } from "./session-store.js";

type JsonObject = Record<string, unknown>;

function translationSessionRoot(rootPath: string): string {
  return path.join(rootPath, "scratch", "translation-sessions");
}

export function translationSessionDirectory(rootPath: string, sessionId: string): string {
  return path.join(translationSessionRoot(rootPath), sessionId);
}

function sessionFilePath(rootPath: string, sessionId: string): string {
  return path.join(translationSessionDirectory(rootPath, sessionId), "session.json");
}

function expectObject(value: unknown, context: string): JsonObject {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Expected ${context} to be an object.`);
  }
  return value as JsonObject;
}

function expectString(value: unknown, context: string): string {
  if (typeof value !== "string") {
    throw new Error(`Expected ${context} to be a string.`);
  }
  return value;
}

function expectNumber(value: unknown, context: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`Expected ${context} to be a number.`);
  }
  return value;
}

function expectBoolean(value: unknown, context: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Expected ${context} to be a boolean.`);
  }
  return value;
}

function expectArray(value: unknown, context: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Expected ${context} to be an array.`);
  }
  return value;
}

function parseStatusFilter(value: unknown, context: string): DerivedTagTranslationReviewFilterStatus {
  const parsed = expectString(value, context);
  if (parsed !== "all" && parsed !== "mapped" && parsed !== "provisional" && parsed !== "unmapped") {
    throw new Error(`Expected ${context} to be all, mapped, provisional, or unmapped.`);
  }
  return parsed;
}

function parseManifest(value: unknown): DerivedTagTranslationReviewSessionManifest {
  const object = expectObject(value, "translation session manifest");
  return {
    id: expectString(object.id, "translation session manifest.id"),
    createdAt: expectString(object.createdAt, "translation session manifest.createdAt"),
    rowCount: expectNumber(object.rowCount, "translation session manifest.rowCount"),
  };
}

function parseReviewState(value: unknown): DerivedTagTranslationReviewSessionState {
  const object = expectObject(value, "translation session state");
  const categoryFilterValue = expectString(object.categoryFilter, "translation session state.categoryFilter");
  const normalizedCategory = categoryFilterValue === "all" ? "all" : normalizeSearchCategory(categoryFilterValue);
  if (!normalizedCategory) {
    throw new Error(`Expected translation session state.categoryFilter to be a valid category or "all".`);
  }
  return {
    currentIndex: expectNumber(object.currentIndex, "translation session state.currentIndex"),
    categoryFilter: normalizedCategory,
    statusFilter: parseStatusFilter(object.statusFilter, "translation session state.statusFilter"),
    imported: expectBoolean(object.imported, "translation session state.imported"),
    updatedAt: expectString(object.updatedAt, "translation session state.updatedAt"),
  };
}

function parseRow(value: unknown, index: number): DerivedTagTranslationReviewRow {
  const object = expectObject(value, `translation session rows[${index}]`);
  return {
    key: expectString(object.key, `translation session rows[${index}].key`) as DerivedTagTranslationReviewRow["key"],
    base: expectObject(object.base, `translation session rows[${index}].base`) as DerivedTagTranslationReviewRow["base"],
    currentOverride: expectObject(
      object.currentOverride,
      `translation session rows[${index}].currentOverride`,
    ) as DerivedTagTranslationReviewRow["currentOverride"],
    draftOverride: expectObject(
      object.draftOverride,
      `translation session rows[${index}].draftOverride`,
    ) as DerivedTagTranslationReviewRow["draftOverride"],
  };
}

export async function writeDerivedTagTranslationReviewSession(
  rootPath: string,
  session: DerivedTagTranslationReviewSession,
): Promise<void> {
  const directory = translationSessionDirectory(rootPath, session.manifest.id);
  await mkdir(directory, { recursive: true });
  await writeFile(sessionFilePath(rootPath, session.manifest.id), `${JSON.stringify(session, null, 2)}\n`, "utf8");
}

export async function readDerivedTagTranslationReviewSession(
  rootPath: string,
  sessionId: string,
): Promise<DerivedTagTranslationReviewSession> {
  const raw = await readFile(sessionFilePath(rootPath, sessionId), "utf8");
  const parsed = parseSessionJson(raw, sessionFilePath(rootPath, sessionId));
  const object = expectObject(parsed, "translation review session");
  return {
    manifest: parseManifest(object.manifest),
    rows: expectArray(object.rows, "translation review session.rows").map((row, index) => parseRow(row, index)),
    reviewState: parseReviewState(object.reviewState),
  };
}
