import type { PackInfo } from "../../domain/record-types.js";
import type {
  BuildSourceEntry,
  IndexedBuildSourceEntry,
  RecordAliasRow,
  RecordLegacyLinkRow,
} from "../index-types.js";
import { resolveBuildReferencesAndAliases } from "../references.js";
import { cloneIndexedBuildSourceEntry } from "./stage-artifacts.js";

export type ReferenceResolutionStageResult = {
  referencedEntries: IndexedBuildSourceEntry[];
  aliasRows: RecordAliasRow[];
  legacyLinkRows: RecordLegacyLinkRow[];
};

export async function resolveIndexReferences(input: {
  indexedEntries: IndexedBuildSourceEntry[];
  sourceEntries: BuildSourceEntry[];
  packs: PackInfo[];
  rootPath: string;
}): Promise<ReferenceResolutionStageResult> {
  const referencedEntries = input.indexedEntries.map((entry) => cloneIndexedBuildSourceEntry(entry));
  const { aliasRows, legacyLinkRows } = await resolveBuildReferencesAndAliases({
    ...input,
    indexedEntries: referencedEntries,
  });

  return {
    referencedEntries,
    aliasRows,
    legacyLinkRows,
  };
}
