import { createContext, useContext } from "react";
import type {
  ContentLocationView,
  SimilarityPartsView,
  SimilaritySeed,
} from "../../generated/atlas";

export type SourceSectionSelection = {
  inventory: SimilarityPartsView;
  selected: string[] | null;
};

type SourceSectionScope = {
  seed: SimilaritySeed;
  generation: string;
  selection: SourceSectionSelection | null;
};

export const SourceSectionSelectionContext = createContext<SourceSectionScope | null>(
  null,
);

export type SourceContentLocation = ContentLocationView | { trait_key: string };

/** Selection is presentation state only; canonical section ownership comes from Rust. */
export function useSourceSectionSelection(
  location?: SourceContentLocation,
  sourceFingerprint?: string | null,
) {
  const scope = useContext(SourceSectionSelectionContext);
  const matches =
    scope &&
    location &&
    (scope.seed.kind === "record"
      ? "record" in location && scope.seed.record_key === location.record
      : "trait_key" in location && scope.seed.trait_key === location.trait_key);
  if (!matches) return null;
  const selection = scope.selection;
  const inventory = selection?.inventory;
  if (
    !selection ||
    inventory?.artifact_generation !== scope.generation ||
    JSON.stringify(inventory.seed) !== JSON.stringify(scope.seed)
  )
    // Keep the same source rendering while a profile's inventory is pending.
    // Eligibility emphasis waits for an authoritative seed/generation match.
    return { selected: [] as number[], excluded: [] as number[] };
  const parts = inventory.parts.filter(
    (part) =>
      part.has_vectors &&
      ("record" in location
        ? part.field === location.field &&
          part.navigation?.source_fingerprint === sourceFingerprint &&
          JSON.stringify(part.owners) === JSON.stringify(location.embedded_path)
        : part.field === "/definition" && part.owners.length === 0),
  );
  return {
    selected: parts
      .filter(
        (part) =>
          selection.selected === null || selection.selected.includes(part.selector),
      )
      .map((part) => part.section_ordinal),
    excluded: parts
      .filter(
        (part) =>
          selection.selected !== null && !selection.selected.includes(part.selector),
      )
      .map((part) => part.section_ordinal),
  };
}
