import { validateConfiguredDerivedTagAssignments } from "../../tags/runtime.js";
import type { BuildSourceEntry, IndexedBuildSourceEntry } from "../index-types.js";
import { assignVariantFamilies } from "../variant-families.js";
import { toIndexedBuildSourceEntry } from "./stage-artifacts.js";

export type FamilyAssignmentStageResult = {
  indexedEntries: IndexedBuildSourceEntry[];
};

export function assignIndexFamilies(sourceEntries: BuildSourceEntry[]): FamilyAssignmentStageResult {
  const indexedEntries = sourceEntries
    .map((entry) => toIndexedBuildSourceEntry(entry))
    .filter((entry): entry is IndexedBuildSourceEntry => entry !== null);

  assignVariantFamilies(indexedEntries);
  validateConfiguredDerivedTagAssignments(
    indexedEntries.map((entry) => ({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      category: entry.record.category,
    })),
  );

  return { indexedEntries };
}
