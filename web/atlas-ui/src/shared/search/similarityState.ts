import type { CoreSectionFamily, SimilarityOptions } from "../../generated/atlas";
import {
  DEFAULT_SOURCE_SECTIONS,
  type SearchPreferences,
  type SimilarityPreferences,
} from "./searchPreferences";

export function similarityFromPreferences(
  preferences: SimilarityPreferences,
  trait = false,
  parts: string[] | null = null,
): SimilarityOptions {
  if (preferences.method === "baseline") return { method: "baseline" };
  const candidate_parts = trait ? "any" : preferences.candidateParts;
  return preferences.method === "passages"
    ? { method: "passages", decay: preferences.decay, parts, candidate_parts }
    : { method: "content-average", parts, candidate_parts };
}

export function selectedSimilarityParts(options?: SimilarityOptions): string[] | null {
  return options && options.method !== "baseline" ? (options.parts ?? null) : null;
}

export function similarityPreferences(
  options: SimilarityOptions,
  previous: SimilarityPreferences,
): SimilarityPreferences {
  return {
    method: options.method,
    decay: options.method === "passages" ? options.decay : previous.decay,
    candidateParts:
      options.method === "baseline" ? previous.candidateParts : options.candidate_parts,
  };
}

export function similarityLabel(options: SimilarityOptions, trait = false): string {
  switch (options.method) {
    case "passages":
      return `Passage matches, decay ${options.decay}`;
    case "content-average":
      return "Content average";
    case "baseline":
      return trait ? "Trait definition" : "Name and identity";
  }
}

/** A remembered semantic preset never contains record-specific selectors. */
export function preferredSourceSections(
  preferences: SearchPreferences,
  family: CoreSectionFamily,
): "all" | "core" {
  return preferences.sourceSectionsByFamily[family] ?? DEFAULT_SOURCE_SECTIONS;
}
export function hasCoreSourcePreference(preferences: SearchPreferences): boolean {
  return Object.values(preferences.sourceSectionsByFamily).some(
    (preset) => preset === "core",
  );
}
