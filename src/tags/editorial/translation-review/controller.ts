import { buildPublishedDerivedTagTranslationsByKey } from "../../translations/index.js";
import {
  cloneDerivedTagTranslationOverride,
  normalizeDerivedTagTranslationOverride,
} from "../../translations/record-utils.js";
import { getCurrentDerivedTagTranslationOverrides } from "../../translations/state.js";
import { listCurrentDerivedTagTranslationQueueItems } from "../state/runtime-state.js";
import type {
  DerivedTagTranslationReviewFilterStatus,
  DerivedTagTranslationReviewRow,
  DerivedTagTranslationReviewSession,
} from "../types.js";
import type { SearchCategory } from "../../../domain/search-types.js";
import type { DerivedTagTranslationRecord } from "../../../domain/derived-tag-types.js";

function createTranslationSessionId(now: Date): string {
  return `translation-${now.toISOString().replace(/[:.]/g, "-")}`;
}

export function createDerivedTagTranslationReviewSession(options: {
  categoryFilter?: SearchCategory | "all";
  statusFilter?: DerivedTagTranslationReviewFilterStatus;
  rows?: DerivedTagTranslationRecord[];
} = {}): DerivedTagTranslationReviewSession {
  const now = new Date();
  const queueItems =
    options.rows ??
    listCurrentDerivedTagTranslationQueueItems({
      ...(options.categoryFilter && options.categoryFilter !== "all" ? { category: options.categoryFilter } : {}),
      ...(options.statusFilter && options.statusFilter !== "all" ? { statuses: [options.statusFilter] } : {}),
    });
  const baseTranslationsByKey = buildPublishedDerivedTagTranslationsByKey({
    includeOverrides: false,
  });
  const currentOverrides = getCurrentDerivedTagTranslationOverrides();

  const rows: DerivedTagTranslationReviewRow[] = queueItems.map((row) => {
    const key = `${row.currentCategory}:${row.currentTag}` as const;
    const base = baseTranslationsByKey.get(key);
    if (!base) {
      throw new Error(`Missing base translation row for ${key}.`);
    }
    const currentOverride = cloneDerivedTagTranslationOverride(currentOverrides.get(key));
    return {
      key,
      base,
      currentOverride,
      draftOverride: cloneDerivedTagTranslationOverride(currentOverride),
    };
  });

  return {
    manifest: {
      id: createTranslationSessionId(now),
      createdAt: now.toISOString(),
      rowCount: rows.length,
    },
    rows,
    reviewState: {
      currentIndex: 0,
      categoryFilter: options.categoryFilter ?? "all",
      statusFilter: options.statusFilter ?? "all",
      imported: false,
      updatedAt: now.toISOString(),
    },
  };
}

export function updateDerivedTagTranslationRowOverride(
  session: DerivedTagTranslationReviewSession,
  rowKey: string,
  nextOverride: DerivedTagTranslationReviewRow["draftOverride"],
): DerivedTagTranslationReviewSession {
  const next = structuredClone(session);
  const row = next.rows.find((entry) => entry.key === rowKey);
  if (!row) {
    return next;
  }
  row.draftOverride = normalizeDerivedTagTranslationOverride(nextOverride);
  next.reviewState.updatedAt = new Date().toISOString();
  return next;
}

export function setDerivedTagTranslationReviewImported(
  session: DerivedTagTranslationReviewSession,
  imported: boolean,
): DerivedTagTranslationReviewSession {
  const next = structuredClone(session);
  next.reviewState.imported = imported;
  next.reviewState.updatedAt = new Date().toISOString();
  return next;
}
