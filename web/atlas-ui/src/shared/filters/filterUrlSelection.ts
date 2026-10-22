import type { QueryRecordSelection, SourcePassageAddress } from "../../generated/atlas";
import {
  invalidFilterUrl,
  urlInteger,
  urlNullableString,
  urlString,
  urlTuple,
} from "./filterUrlValues";

/** Root selection is just a key. Exact reading context uses fixed optional slots. */
export function packSelection(selection: QueryRecordSelection): unknown {
  const slots: unknown[] = [
    selection.record_key,
    selection.embedded_path.length
      ? selection.embedded_path.map((step) => [
          step.collection,
          "Stable" in step.identity
            ? step.identity.Stable
            : step.identity.SnapshotLocal.index,
        ])
      : null,
    selection.field,
    selection.passage ? packPassage(selection.passage) : null,
    selection.source_fingerprint,
    selection.spell_rank ?? null,
  ];
  while (slots.length > 1 && slots[slots.length - 1] == null) slots.pop();
  return slots.length === 1 ? slots[0] : slots;
}

export function unpackSelection(value: unknown): QueryRecordSelection {
  const slots = typeof value === "string" ? [value] : urlTuple(value, 2, 6);
  const embedded_path =
    slots[1] == null
      ? []
      : urlTuple(slots[1], 0, Infinity).map((value) => {
          const step = urlTuple(value, 2);
          const collection = urlString(step[0]);
          return {
            collection,
            identity:
              typeof step[1] === "string"
                ? { Stable: step[1] }
                : { SnapshotLocal: { index: urlInteger(step[1]) } },
          };
        });
  return {
    record_key: urlString(slots[0]),
    embedded_path,
    field: urlNullableString(slots[2]),
    passage: slots[3] == null ? null : unpackPassage(slots[3]),
    source_fingerprint: urlNullableString(slots[4]),
    ...(slots[5] == null ? {} : { spell_rank: urlInteger(slots[5]) }),
  };
}

function packPassage(passage: SourcePassageAddress): unknown[] {
  if (passage.kind === "identity") return ["i"];
  const slots = [
    passage.kind === "html_section" ? "h" : "p",
    passage.selection_version,
    passage.section_ordinal,
    passage.label,
    passage.chunk_bytes.start,
    passage.chunk_bytes.end,
  ];
  return passage.kind === "html_section"
    ? [...slots, passage.prepared_html_sha256, passage.canonical_text_sha256]
    : [...slots, passage.source_text_sha256];
}

function unpackPassage(value: unknown): SourcePassageAddress {
  const slots = urlTuple(value, 1, 8);
  if (slots[0] === "i") {
    urlTuple(slots, 1);
    return { kind: "identity" };
  }
  if (slots[0] !== "h" && slots[0] !== "p") invalidFilterUrl();
  urlTuple(slots, slots[0] === "h" ? 8 : 7);
  const start = urlInteger(slots[4]),
    end = urlInteger(slots[5]);
  if (end < start) invalidFilterUrl();
  const section = {
    selection_version: urlString(slots[1]),
    section_ordinal: urlInteger(slots[2]),
    label: urlNullableString(slots[3]),
    chunk_bytes: { start, end },
  };
  return slots[0] === "h"
    ? {
        kind: "html_section",
        ...section,
        prepared_html_sha256: urlString(slots[6]),
        canonical_text_sha256: urlString(slots[7]),
      }
    : { kind: "plain_section", ...section, source_text_sha256: urlString(slots[6]) };
}
