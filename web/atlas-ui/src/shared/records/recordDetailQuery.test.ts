import { recordDetailQueryOptions } from "./recordDetailQuery";
import type { RecordNavigationView } from "../../generated/atlas";

describe("record detail query identity", () => {
  it("shares a root request and distinguishes every selected address component", () => {
    const root = recordDetailQueryOptions("actors:ghoul");
    expect(root.queryKey).toEqual(["record-detail", "actors:ghoul", null]);
    const selection: RecordNavigationView = {
      record_key: "actors:ghoul",
      owners: [],
      field: "system.description.value",
      passage: { kind: "identity" },
      source_fingerprint: "snapshot-a",
    };
    const key = recordDetailQueryOptions(selection.record_key, selection).queryKey;
    expect(key).toEqual([
      "record-detail",
      selection.record_key,
      { ...selection, fields: [selection.field] },
    ]);
    for (const change of [
      { field: "system.details.publicNotes" },
      { source_fingerprint: "snapshot-b" },
      { passage: null },
      { owners: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }] },
    ])
      expect(
        recordDetailQueryOptions(selection.record_key, { ...selection, ...change })
          .queryKey,
      ).not.toEqual(key);
  });
});
