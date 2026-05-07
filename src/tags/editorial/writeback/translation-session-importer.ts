import type { DerivedTagTranslationRecord } from "../../../domain/derived-tag-types.js";
import type { DerivedTagFamilyTranslationDefaults } from "../../translations/family-defaults.js";
import {
  getCurrentDerivedTagFamilyTranslationDefaults,
  getCurrentDerivedTagTranslationOverrides,
} from "../../translations/state.js";
import { applyDerivedTagTranslationOverride } from "../../translations/record-utils.js";
import {
  isEmptyDerivedTagTranslationOverride,
  normalizeDerivedTagTranslationOverride,
} from "../../translations/record-utils.js";
import { writeDerivedTagFamilyTranslationDefaults } from "./translation-family-defaults-writer.js";
import { writeDerivedTagTranslationOverrides } from "./translation-overrides-writer.js";
import { lintDerivedTagTranslationReviewSession } from "./translation-session-linter.js";
import type { DerivedTagTranslationReviewSession } from "../types.js";

function overridesEqual(
  left: Record<string, unknown>,
  right: Record<string, unknown>,
): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

const FAMILY_DEFAULT_FIELDS = [
  "schemaKind",
  "translationStatus",
  "primaryFacetKind",
  "primaryFacetValue",
] as const satisfies ReadonlyArray<keyof DerivedTagFamilyTranslationDefaults>;

function buildFamilyKey(row: Pick<DerivedTagTranslationRecord, "currentCategory" | "currentFamily">): string {
  return `${row.currentCategory}:${row.currentFamily}`;
}

function pickFamilyDefaultsFromRow(row: DerivedTagTranslationRecord): DerivedTagFamilyTranslationDefaults {
  return {
    schemaKind: row.schemaKind,
    translationStatus: row.translationStatus,
    ...(row.primaryFacetKind ? { primaryFacetKind: row.primaryFacetKind } : {}),
    ...(row.primaryFacetValue ? { primaryFacetValue: row.primaryFacetValue } : {}),
  };
}

function familyDefaultsEqual(
  left: DerivedTagFamilyTranslationDefaults,
  right: DerivedTagFamilyTranslationDefaults,
): boolean {
  return overridesEqual(left, right);
}

function normalizeFamilyDefaults(
  defaults: DerivedTagFamilyTranslationDefaults,
): DerivedTagFamilyTranslationDefaults {
  return {
    schemaKind: defaults.schemaKind,
    translationStatus: defaults.translationStatus,
    ...(defaults.primaryFacetKind ? { primaryFacetKind: defaults.primaryFacetKind } : {}),
    ...(defaults.primaryFacetValue ? { primaryFacetValue: defaults.primaryFacetValue } : {}),
    ...(defaults.notes ? { notes: defaults.notes } : {}),
  };
}

function stripRedundantFamilyDefaultFields(
  effectiveRow: DerivedTagTranslationRecord,
  rowOverride: Record<string, unknown>,
  familyDefaults: DerivedTagFamilyTranslationDefaults,
): Record<string, unknown> {
  const nextOverride = structuredClone(rowOverride);
  for (const field of FAMILY_DEFAULT_FIELDS) {
    if (
      nextOverride[field] !== undefined &&
      effectiveRow[field] === familyDefaults[field]
    ) {
      delete nextOverride[field];
    }
  }
  return nextOverride;
}

export async function importDerivedTagTranslationReviewSession(
  rootPath: string,
  session: DerivedTagTranslationReviewSession,
): Promise<void> {
  lintDerivedTagTranslationReviewSession(session);
  const nextFamilyDefaults = getCurrentDerivedTagFamilyTranslationDefaults();
  const nextOverrides = getCurrentDerivedTagTranslationOverrides();
  const rowsByFamily = new Map<string, DerivedTagTranslationReviewSession["rows"]>();

  for (const row of session.rows) {
    const familyKey = buildFamilyKey(row.base);
    const currentRows = rowsByFamily.get(familyKey) ?? [];
    currentRows.push(row);
    rowsByFamily.set(familyKey, currentRows);
  }

  for (const [familyKey, rows] of rowsByFamily.entries()) {
    const effectiveRows = rows.map((row) => applyDerivedTagTranslationOverride(row.base, row.draftOverride));
    const firstDefaults = pickFamilyDefaultsFromRow(effectiveRows[0]!);
    if (!effectiveRows.every((row) => familyDefaultsEqual(pickFamilyDefaultsFromRow(row), firstDefaults))) {
      continue;
    }
    const currentDefaults = nextFamilyDefaults.get(familyKey);
    if (!currentDefaults || !familyDefaultsEqual(currentDefaults, firstDefaults)) {
      nextFamilyDefaults.set(familyKey, normalizeFamilyDefaults(firstDefaults));
    }

    for (let index = 0; index < rows.length; index += 1) {
      const row = rows[index]!;
      const effectiveRow = effectiveRows[index]!;
      const strippedOverride = stripRedundantFamilyDefaultFields(
        effectiveRow,
        normalizeDerivedTagTranslationOverride(row.draftOverride),
        firstDefaults,
      );
      row.draftOverride = strippedOverride;
    }
  }

  for (const row of session.rows) {
    const originalOverride = normalizeDerivedTagTranslationOverride(row.currentOverride);
    const draftOverride = normalizeDerivedTagTranslationOverride(row.draftOverride);
    if (overridesEqual(originalOverride, draftOverride)) {
      continue;
    }
    if (isEmptyDerivedTagTranslationOverride(draftOverride)) {
      nextOverrides.delete(row.key);
    } else {
      nextOverrides.set(row.key, draftOverride);
    }
  }

  await writeDerivedTagFamilyTranslationDefaults(rootPath, nextFamilyDefaults);
  await writeDerivedTagTranslationOverrides(rootPath, nextOverrides);
}
