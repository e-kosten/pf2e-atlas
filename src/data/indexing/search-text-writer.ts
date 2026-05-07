import { createHash } from "node:crypto";

import { uniqueSorted } from "../../shared/utils.js";
import type { PendingCanonicalEmbeddingWithHash, WritableIndexEntry } from "../index-types.js";
import { buildSemanticEmbeddingText } from "../record-normalization.js";

function hashSemanticInput(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export type SearchTextWriteArtifact = {
  searchText: string;
  pendingCanonicalEmbedding: PendingCanonicalEmbeddingWithHash | null;
};

export function createSearchTextWriteArtifact(entry: WritableIndexEntry): SearchTextWriteArtifact {
  const searchText = uniqueSorted([entry.record.searchText, ...entry.aliasTexts].filter(Boolean)).join("\n");
  if (!entry.isSearchCanonical) {
    return {
      searchText,
      pendingCanonicalEmbedding: null,
    };
  }

  const encodedEmbeddingInput = buildSemanticEmbeddingText(entry.record, entry.raw, entry.aliasTexts);
  return {
    searchText,
    pendingCanonicalEmbedding: {
      record: entry.record,
      encodedEmbeddingInput,
      semanticInputHash: hashSemanticInput(encodedEmbeddingInput),
    },
  };
}
