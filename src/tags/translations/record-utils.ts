import type { DerivedTagTranslationRecord } from "../../domain/derived-tag-types.js";
import type { DerivedTagTranslationOverride } from "./tag-overrides.js";

function cloneOverrideValue<T>(value: T): T {
  return structuredClone(value);
}

export function mergeDerivedTagTranslationNotes(
  baseNotes: string | undefined,
  overrideNotes: string | undefined,
): string | undefined {
  const merged = [baseNotes, overrideNotes].filter(Boolean).join(" ");
  return merged || undefined;
}

export function applyDerivedTagTranslationOverride(
  base: DerivedTagTranslationRecord,
  override: DerivedTagTranslationOverride | undefined,
): DerivedTagTranslationRecord {
  if (!override) {
    return structuredClone(base);
  }

  const merged = {
    ...structuredClone(base),
    ...cloneOverrideValue(override),
  } satisfies DerivedTagTranslationRecord;
  merged.notes = mergeDerivedTagTranslationNotes(base.notes, override.notes);
  return merged;
}

export function cloneDerivedTagTranslationOverride(
  override: DerivedTagTranslationOverride | undefined,
): DerivedTagTranslationOverride {
  return override ? cloneOverrideValue(override) : {};
}

export function normalizeDerivedTagTranslationOverride(
  override: DerivedTagTranslationOverride,
): DerivedTagTranslationOverride {
  return Object.fromEntries(Object.entries(cloneOverrideValue(override))) as DerivedTagTranslationOverride;
}

export function isEmptyDerivedTagTranslationOverride(override: DerivedTagTranslationOverride): boolean {
  return Object.keys(normalizeDerivedTagTranslationOverride(override)).length === 0;
}
