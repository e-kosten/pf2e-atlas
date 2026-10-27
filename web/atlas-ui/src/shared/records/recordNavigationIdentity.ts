import type { RecordNavigationView } from "../../generated/atlas";
import { packSelection } from "../filters/filterUrlSelection";

export function recordNavigationIdentity(
  recordKey: string | null,
  selection?: RecordNavigationView,
): string {
  const packed = packSelection({
    record_key: recordKey ?? "",
    embedded_path: selection?.embedded_path ?? [],
    field: selection?.field ?? null,
    passage: selection?.passage ?? null,
    source_fingerprint: selection?.source_fingerprint ?? null,
    spell_rank: selection?.spell_rank,
  });
  return JSON.stringify([recordKey, ...(Array.isArray(packed) ? packed.slice(1) : [])]);
}
