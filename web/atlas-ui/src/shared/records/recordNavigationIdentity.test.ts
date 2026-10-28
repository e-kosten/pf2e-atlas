import type { RecordNavigationView } from "../../generated/atlas";
import { decodeReadingAddress, encodeReadingAddress } from "../../app/readingUrl";
import { recordNavigationIdentity } from "./recordNavigationIdentity";
import { recordReadingIdentity } from "./useRetainedRecord";

const serverSelection: RecordNavigationView = {
  record_key: "actors:example",
  embedded_path: [{ collection: "/items", identity: { Stable: "ability" } }],
  field: "/system/description/value",
  passage: {
    kind: "html_section",
    prepared_html_sha256: "prepared-a",
    canonical_text_sha256: "text-a",
    selection_version: "selection-v1",
    section_ordinal: 2,
    label: "Brood Variance",
    chunk_bytes: { start: 10, end: 50 },
  },
  source_fingerprint: "source-a",
  spell_rank: 0,
};

it("keeps server DTO and decoded URL addresses identical despite object key order", () => {
  const decoded = decodeReadingAddress(
    encodeReadingAddress(serverSelection.record_key, serverSelection, true)!,
    serverSelection.record_key,
    true,
  );
  expect(decoded).toEqual(serverSelection);
  expect(JSON.stringify(decoded.passage)).not.toBe(
    JSON.stringify(serverSelection.passage),
  );
  expect(recordNavigationIdentity(decoded.record_key, decoded)).toBe(
    recordNavigationIdentity(serverSelection.record_key, serverSelection),
  );
  expect(recordReadingIdentity(decoded.record_key, decoded)).toBe(
    recordReadingIdentity(serverSelection.record_key, serverSelection),
  );
});

it("distinguishes each exact address dimension and excludes only rank from retained reading", () => {
  const passage = serverSelection.passage!;
  if (passage.kind !== "html_section") throw new Error("Expected an HTML passage");
  for (const change of [
    { record_key: "actors:other" },
    { embedded_path: [{ collection: "/spells", identity: { Stable: "ability" } }] },
    {
      embedded_path: [
        { collection: "/items", identity: { SnapshotLocal: { index: 0 } } },
      ],
    },
    { embedded_path: [{ collection: "/items", identity: { Stable: "other" } }] },
    { field: "/system/details/publicNotes" },
    { source_fingerprint: "source-b" },
    { passage: { ...passage, prepared_html_sha256: "prepared-b" } },
    { passage: { ...passage, canonical_text_sha256: "text-b" } },
    { passage: { ...passage, selection_version: "selection-v2" } },
    { passage: { ...passage, section_ordinal: 3 } },
    { passage: { ...passage, label: "Other section" } },
    { passage: { ...passage, chunk_bytes: { start: 11, end: 50 } } },
    { passage: { ...passage, chunk_bytes: { start: 10, end: 51 } } },
  ]) {
    const changed = { ...serverSelection, ...change };
    expect(recordNavigationIdentity(changed.record_key, changed)).not.toBe(
      recordNavigationIdentity(serverSelection.record_key, serverSelection),
    );
    expect(recordReadingIdentity(changed.record_key, changed)).not.toBe(
      recordReadingIdentity(serverSelection.record_key, serverSelection),
    );
  }
  const ranked = { ...serverSelection, spell_rank: 8 };
  expect(recordNavigationIdentity(ranked.record_key, ranked)).not.toBe(
    recordNavigationIdentity(serverSelection.record_key, serverSelection),
  );
  expect(recordReadingIdentity(ranked.record_key, ranked)).toBe(
    recordReadingIdentity(serverSelection.record_key, serverSelection),
  );
});
