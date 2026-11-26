import { decodeReadingAddress, encodeReadingAddress } from "../../app/readingUrl";
import type { RecordNavigationView, SimilaritySeed } from "../../generated/atlas";
import type { SearchFormState } from "../filters/searchState";
import { recordNavigationIdentity } from "../records/recordNavigationIdentity";

export function documentSeed(navigation: RecordNavigationView): SimilaritySeed {
  const seed = { kind: "record" as const, record_key: navigation.record_key };
  if (!navigation.embedded_path.length) return seed;
  const local = navigation.embedded_path.some(
    (step) => "SnapshotLocal" in step.identity,
  );
  return {
    ...seed,
    embedded_path: navigation.embedded_path,
    ...(local && navigation.source_fingerprint
      ? { source_fingerprint: navigation.source_fingerprint }
      : {}),
  };
}

export function seedNavigation(seed: SimilaritySeed): RecordNavigationView {
  return {
    record_key: seed.record_key,
    embedded_path: seed.embedded_path ?? [],
    source_fingerprint: seed.source_fingerprint ?? null,
    field: null,
    passage: null,
  };
}

export function sameSimilaritySeed(
  left: SimilaritySeed | null,
  right: SimilaritySeed | null,
): boolean {
  if (!left || !right) return left === right;
  return (
    recordNavigationIdentity(left.record_key, seedNavigation(left)) ===
    recordNavigationIdentity(right.record_key, seedNavigation(right))
  );
}

export function searchSeed(
  state: Pick<SearchFormState, "similarSeed" | "similarLocation">,
): SimilaritySeed | null {
  return state.similarSeed
    ? { kind: "record", record_key: state.similarSeed, ...state.similarLocation }
    : null;
}

export function seedSearchFields(seed: SimilaritySeed) {
  return {
    similarSeed: seed.record_key,
    similarLocation: seed.embedded_path?.length
      ? {
          embedded_path: seed.embedded_path,
          ...(seed.source_fingerprint
            ? { source_fingerprint: seed.source_fingerprint }
            : {}),
        }
      : undefined,
  };
}

export function seedCriteria(seed: SimilaritySeed) {
  return {
    seed_record_key: seed.record_key,
    ...(seed.embedded_path?.length ? { embedded_path: seed.embedded_path } : {}),
    ...(seed.source_fingerprint ? { source_fingerprint: seed.source_fingerprint } : {}),
  };
}

export function encodeSimilarityLocation(seed: SimilaritySeed) {
  return encodeReadingAddress(seed.record_key, seedNavigation(seed), false);
}

export function decodeSimilarityLocation(encoded: string, recordKey: string) {
  const address = decodeReadingAddress(encoded, recordKey, false);
  const local = address.embedded_path.some((step) => "SnapshotLocal" in step.identity);
  if (
    !address.embedded_path.length ||
    address.field !== null ||
    address.passage !== null ||
    (local ? !address.source_fingerprint : address.source_fingerprint !== null)
  ) {
    throw new Error("Invalid similarity document address");
  }
  return seedSearchFields(documentSeed(address)).similarLocation;
}
