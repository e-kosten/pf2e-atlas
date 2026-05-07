import type { DerivedTagFamilyTranslationDefaults } from "./family-defaults.js";
import { DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS } from "./family-defaults.js";
import type { DerivedTagTranslationOverride } from "./tag-overrides.js";
import { DERIVED_TAG_TRANSLATION_OVERRIDES } from "./tag-overrides.js";

function cloneMap<T>(entries: ReadonlyMap<string, T>): Map<string, T> {
  return new Map(
    [...entries.entries()].map(([key, value]) => [key, structuredClone(value)] as const),
  );
}

let currentDerivedTagTranslationOverrides: Map<string, DerivedTagTranslationOverride> | null = null;
let currentDerivedTagTranslationOverridesRevision = 0;
let currentDerivedTagFamilyTranslationDefaults: Map<string, DerivedTagFamilyTranslationDefaults> | null = null;
let currentDerivedTagFamilyTranslationDefaultsRevision = 0;

function buildImportedDerivedTagTranslationOverrides(): Map<string, DerivedTagTranslationOverride> {
  return cloneMap(DERIVED_TAG_TRANSLATION_OVERRIDES);
}

function buildImportedDerivedTagFamilyTranslationDefaults(): Map<string, DerivedTagFamilyTranslationDefaults> {
  return cloneMap(DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS);
}

export function getCurrentDerivedTagTranslationOverrides(): Map<string, DerivedTagTranslationOverride> {
  if (!currentDerivedTagTranslationOverrides) {
    currentDerivedTagTranslationOverrides = buildImportedDerivedTagTranslationOverrides();
  }
  return cloneMap(currentDerivedTagTranslationOverrides);
}

export function getCurrentDerivedTagTranslationOverride(
  key: string,
): DerivedTagTranslationOverride | undefined {
  return getCurrentDerivedTagTranslationOverrides().get(key);
}

export function setCurrentDerivedTagTranslationOverrides(
  overrides: ReadonlyMap<string, DerivedTagTranslationOverride>,
): void {
  currentDerivedTagTranslationOverrides = cloneMap(overrides);
  currentDerivedTagTranslationOverridesRevision += 1;
}

export function getCurrentDerivedTagTranslationOverridesRevision(): number {
  return currentDerivedTagTranslationOverridesRevision;
}

export function getCurrentDerivedTagFamilyTranslationDefaults(): Map<string, DerivedTagFamilyTranslationDefaults> {
  if (!currentDerivedTagFamilyTranslationDefaults) {
    currentDerivedTagFamilyTranslationDefaults = buildImportedDerivedTagFamilyTranslationDefaults();
  }
  return cloneMap(currentDerivedTagFamilyTranslationDefaults);
}

export function getCurrentDerivedTagFamilyTranslationDefault(
  key: string,
): DerivedTagFamilyTranslationDefaults | undefined {
  return getCurrentDerivedTagFamilyTranslationDefaults().get(key);
}

export function setCurrentDerivedTagFamilyTranslationDefaults(
  defaults: ReadonlyMap<string, DerivedTagFamilyTranslationDefaults>,
): void {
  currentDerivedTagFamilyTranslationDefaults = cloneMap(defaults);
  currentDerivedTagFamilyTranslationDefaultsRevision += 1;
}

export function getCurrentDerivedTagFamilyTranslationDefaultsRevision(): number {
  return currentDerivedTagFamilyTranslationDefaultsRevision;
}
