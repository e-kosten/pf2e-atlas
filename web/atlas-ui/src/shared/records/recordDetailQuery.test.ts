import { recordDetailRequest } from "./recordDetailQuery";
import { recordDetailQueryOptions } from "./recordDetailQuery";
import type { RecordNavigationView } from "../../generated/atlas";

describe("record detail query identity", () => {
  it("retains rank zero and takes record identity from the requested path", () => {
    const selected: RecordNavigationView = {
      record_key: "spells:other",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
      spell_rank: 0,
    };
    const query = recordDetailQueryOptions("spells:cantrip", selected);
    expect(query.queryKey).toEqual([
      "record-detail",
      "spells:cantrip",
      {
        record_key: "spells:cantrip",
        embedded_path: [],
        fields: [],
        passage: null,
        source_fingerprint: null,
        spell_rank: 0,
      },
    ]);
  });

  it("shares a root request and distinguishes every selected address component", () => {
    const root = recordDetailQueryOptions("actors:ghoul");
    expect(root.queryKey).toEqual(["record-detail", "actors:ghoul", null]);
    const selection: RecordNavigationView = {
      record_key: "actors:ghoul",
      embedded_path: [],
      field: "system.description.value",
      passage: { kind: "identity" },
      source_fingerprint: "snapshot-a",
    };
    const key = recordDetailQueryOptions(selection.record_key, selection).queryKey;
    expect(key).toEqual([
      "record-detail",
      selection.record_key,
      recordDetailRequest(selection),
    ]);
    for (const change of [
      { field: "system.details.publicNotes" },
      { source_fingerprint: "snapshot-b" },
      { passage: null },
      {
        embedded_path: [
          { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
        ],
      },
    ])
      expect(
        recordDetailQueryOptions(selection.record_key, { ...selection, ...change })
          .queryKey,
      ).not.toEqual(key);
  });
});

it("converts a navigation field into the request field list without leaking singular field", () => {
  const navigation = {
    record_key: "equipment:wand",
    embedded_path: [],
    field: "/system/description/value",
    passage: null,
    source_fingerprint: null,
  };
  expect(recordDetailRequest(navigation)).toEqual({
    record_key: "equipment:wand",
    embedded_path: [],
    fields: ["/system/description/value"],
    passage: null,
    source_fingerprint: null,
  });
});
