import LZString from "lz-string";
import type { RecordNavigationView } from "../generated/atlas";
import { packSelection, unpackSelection } from "../shared/filters/filterUrlSelection";

const VERSION = "1.";
const MAX_ENCODED_LENGTH = 16_384;
const MAX_DECODED_LENGTH = 65_536;

export class ReadingUrlError extends Error {
  constructor(message = "This record link couldn’t be restored.") {
    super(message);
    this.name = "ReadingUrlError";
  }
}

function invalidAddress(): never {
  throw new ReadingUrlError();
}

function boundedString(value: string | null, limit = 512) {
  if (value !== null && value.length > limit) invalidAddress();
}

function safeInteger(value: number) {
  if (!Number.isSafeInteger(value) || value < 0) invalidAddress();
}

function checkAddress(address: RecordNavigationView): RecordNavigationView {
  if (address.embedded_path.length > 32) invalidAddress();
  for (const step of address.embedded_path) {
    boundedString(step.collection);
    if ("Stable" in step.identity) boundedString(step.identity.Stable);
    else safeInteger(step.identity.SnapshotLocal.index);
  }
  boundedString(address.field);
  boundedString(address.source_fingerprint);
  if (address.spell_rank !== undefined) {
    safeInteger(address.spell_rank);
    if (address.spell_rank > 255) invalidAddress();
  }
  const passage = address.passage;
  if (passage && passage.kind !== "identity") {
    boundedString(passage.selection_version);
    boundedString(passage.label, 4096);
    safeInteger(passage.section_ordinal);
    safeInteger(passage.chunk_bytes.start);
    safeInteger(passage.chunk_bytes.end);
    if (passage.chunk_bytes.end < passage.chunk_bytes.start) invalidAddress();
    if (passage.kind === "html_section") {
      boundedString(passage.prepared_html_sha256);
      boundedString(passage.canonical_text_sha256);
    } else boundedString(passage.source_text_sha256);
  }
  return address;
}

export function encodeReadingAddress(
  recordKey: string,
  address: RecordNavigationView,
  includeRank: boolean,
): string | undefined {
  if (address.record_key !== recordKey)
    throw new ReadingUrlError(
      "The record link contains conflicting record identities.",
    );
  checkAddress(address);
  const packed = packSelection({
    ...address,
    spell_rank: includeRank ? address.spell_rank : undefined,
  });
  if (typeof packed === "string") return undefined;
  const slots = (packed as unknown[]).slice(1);
  const json = JSON.stringify(slots);
  if (json.length > MAX_DECODED_LENGTH) invalidAddress();
  const encoded =
    VERSION + LZString.compressToEncodedURIComponent(json).replace(/\+/g, "_");
  if (encoded.length > MAX_ENCODED_LENGTH) invalidAddress();
  return encoded;
}

export function decodeReadingAddress(
  encoded: string,
  recordKey: string,
  includeRank: boolean,
): RecordNavigationView {
  if (!encoded.startsWith(VERSION))
    throw new ReadingUrlError("This record link uses an unsupported address version.");
  if (encoded.length > MAX_ENCODED_LENGTH) invalidAddress();
  const payload = encoded.slice(VERSION.length);
  if (!/^[A-Za-z0-9_-]+$/.test(payload)) invalidAddress();
  try {
    const json = LZString.decompressFromEncodedURIComponent(payload.replace(/_/g, "+"));
    if (!json || json.length > MAX_DECODED_LENGTH) invalidAddress();
    const slots: unknown = JSON.parse(json);
    if (!Array.isArray(slots) || slots.length > (includeRank ? 5 : 4)) invalidAddress();
    return checkAddress(
      unpackSelection(slots.length ? [recordKey, ...slots] : recordKey),
    );
  } catch (error) {
    if (error instanceof ReadingUrlError) throw error;
    return invalidAddress();
  }
}
