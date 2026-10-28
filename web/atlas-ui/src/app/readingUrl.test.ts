import LZString from "lz-string";
import type { RecordNavigationView } from "../generated/atlas";
import { recordDestination } from "./destinations";
import { decodeReadingAddress } from "./readingUrl";
import {
  selectedNavigation,
  stringifySearchParams,
  validateReadingSearch,
} from "./urlState";

const root: RecordNavigationView = {
  record_key: "spells-srd:sxQZ6yqTn0czJxVd",
  embedded_path: [],
  field: null,
  passage: null,
  source_fingerprint: null,
};

it("emits only the selected rank for a whole spell, including zero", () => {
  for (const spell_rank of [0, 8]) {
    const search = recordDestination({
      recordKey: root.record_key,
      selection: { ...root, spell_rank },
    }).search;
    expect(stringifySearchParams(search)).toBe(`?rank=${spell_rank}`);
    expect(selectedNavigation(search, root.record_key)).toEqual({
      ...root,
      spell_rank,
    });
  }
  expect(
    recordDestination({ recordKey: root.record_key, selection: root }).search,
  ).toEqual({});
});

it("binds exact selection and independent landing to the path identity", () => {
  const selection: RecordNavigationView = {
    ...root,
    embedded_path: [
      { collection: "/items", identity: { Stable: "spell/雪%" } },
      { collection: "/effects", identity: { SnapshotLocal: { index: 0 } } },
    ],
    field: "/system/description/value",
    source_fingerprint: "source:abc",
    spell_rank: 8,
    passage: {
      kind: "html_section",
      selection_version: "content/v10",
      section_ordinal: 2,
      label: "雪 & flame",
      chunk_bytes: { start: 10, end: 80 },
      prepared_html_sha256: "prepared-hash",
      canonical_text_sha256: "text-hash",
    },
  };
  const landing = { ...selection, passage: null, spell_rank: 0 };
  const search = recordDestination({
    recordKey: root.record_key,
    selection,
    landing,
  }).search;
  expect(search.at).toMatch(/^1\.[A-Za-z0-9_-]+$/);
  expect(search.focus).toMatch(/^1\.[A-Za-z0-9_-]+$/);
  expect(stringifySearchParams(search)).not.toContain(root.record_key);
  expect(selectedNavigation(search, root.record_key)).toEqual(selection);
  expect(selectedNavigation(search, root.record_key, "focus")).toEqual(landing);
  expect(selectedNavigation(search, "spells:path-authority")?.record_key).toBe(
    "spells:path-authority",
  );
});

it("rejects an internal destination with disagreeing record identities", () => {
  expect(() =>
    recordDestination({ recordKey: "spells:other", selection: root }),
  ).toThrow("conflicting record identities");
  expect(() => recordDestination({ recordKey: "spells:other", landing: root })).toThrow(
    "conflicting record identities",
  );
});

it("keeps Explore state and accepts only equal repeated singleton values", () => {
  const search = recordDestination({
    recordKey: root.record_key,
    view: "explore",
    tab: "similar",
  }).search;
  expect(search).toEqual({ record_view: "explore", record_tab: "similar" });
  expect(selectedNavigation({ rank: ["0", "0"] }, root.record_key)?.spell_rank).toBe(0);
  expect(() => validateReadingSearch({ rank: ["0", "8"] })).toThrow(
    "conflicting parameters",
  );
  expect(() => validateReadingSearch({ record_view: "other" })).toThrow(
    "invalid reading view",
  );
  expect(() => validateReadingSearch({ record_tab: "other" })).toThrow(
    "invalid Explore tab",
  );
});

it.each(["-1", "1.5", "256", "eight", ""])("rejects malformed rank %s", (rank) => {
  expect(() => validateReadingSearch({ rank })).toThrow("invalid spell rank");
});

function compressed(value: unknown) {
  return (
    "1." +
    LZString.compressToEncodedURIComponent(JSON.stringify(value)).replace(/\+/g, "_")
  );
}

it.each([
  null,
  {},
  ["a-second-record-key"],
  [[["/items", -1]]],
  [[["/items", 0.5]]],
  [[null]],
  [null, 4],
  [null, null, ["h", "v1", 0, null, 8, 2, "html", "text"]],
  [null, null, null, null, 8],
  [Array.from({ length: 33 }, () => ["/items", "stable-id"])],
  [null, "x".repeat(513)],
])("rejects malformed or out-of-bounds selected address %j", (value) => {
  expect(() => decodeReadingAddress(compressed(value), root.record_key, false)).toThrow(
    "couldn’t be restored",
  );
});

it("rejects unsupported, damaged, and oversized payloads", () => {
  expect(() => decodeReadingAddress("2.any", root.record_key, false)).toThrow(
    "unsupported address version",
  );
  expect(() => decodeReadingAddress("1.%", root.record_key, false)).toThrow(
    "couldn’t be restored",
  );
  expect(() =>
    decodeReadingAddress("1." + "a".repeat(16_384), root.record_key, false),
  ).toThrow("couldn’t be restored");
  expect(() =>
    decodeReadingAddress(
      compressed([null, "x".repeat(65_536)]),
      root.record_key,
      false,
    ),
  ).toThrow("couldn’t be restored");
});

it("decodes an independently authored compact address, including zero rank and byte offsets", () => {
  const encoded = compressed([
    [
      ["/items", "spell/雪%"],
      ["/effects", 0],
    ],
    "/system/description/value",
    ["p", "content/v10", 2, "雪 & flame", 10, 80, "text-hash"],
    "source:abc",
    0,
  ]);
  expect(decodeReadingAddress(encoded, root.record_key, true)).toEqual({
    record_key: root.record_key,
    embedded_path: [
      { collection: "/items", identity: { Stable: "spell/雪%" } },
      { collection: "/effects", identity: { SnapshotLocal: { index: 0 } } },
    ],
    field: "/system/description/value",
    passage: {
      kind: "plain_section",
      selection_version: "content/v10",
      section_ordinal: 2,
      label: "雪 & flame",
      chunk_bytes: { start: 10, end: 80 },
      source_text_sha256: "text-hash",
    },
    source_fingerprint: "source:abc",
    spell_rank: 0,
  });
});
