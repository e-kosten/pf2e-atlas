import type { SimilarityOptions } from "../../generated/atlas";
import type { SimilarityPreferences } from "./searchPreferences";

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

export function similarityLabel(options: SimilarityOptions): string {
  switch (options.method) {
    case "passages":
      return `Passage matches, decay ${options.decay}`;
    case "content-average":
      return "Content average";
    case "baseline":
      return "Current baseline";
  }
}
