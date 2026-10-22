import { compressToEncodedURIComponent } from "lz-string";
import type { QueryPredicate, QueryRecordSelection } from "../../generated/atlas";
import { decodeFilterUrl, encodeFilterUrl } from "./filterUrl";

describe("self-contained filter URLs", () => {
  const predicate: QueryPredicate = {
    kind: "all_of",
    clause_id: "original-root",
    children: [
      {
        kind: "in",
        clause_id: "original-kind",
        field: "kind",
        values: ["creature", "hazard"],
      },
      {
        kind: "any_of",
        clause_id: "original-or",
        children: [
          { kind: "compare", field: "level_or_rank", op: "eq", value: 0 },
          {
            kind: "not",
            predicate: {
              kind: "set_match",
              field: "record.traits",
              op: "includes_all",
              values: ["fire", "magical"],
            },
          },
        ],
      },
      { kind: "state_match", field: "defenses.hp.maximum", state: "missing" },
      {
        kind: "exists",
        clause_id: "first-exists",
        scope_id: "first-scope",
        collection: "embedded.equipment",
        predicate: {
          kind: "compare",
          field: "name",
          op: "eq",
          value: 'Épée + 盾 & "staff"',
        },
      },
      {
        kind: "exists",
        clause_id: "second-exists",
        scope_id: "second-scope",
        collection: "embedded.equipment",
        predicate: {
          kind: "boolean_constant",
          value: false,
        },
      },
    ],
  };
  it("preserves all predicate forms, literals and grouping without retaining editor identities", () => {
    const encoded = encodeFilterUrl(predicate);
    const restored = decodeFilterUrl(
      new URLSearchParams({ filter: encoded }).get("filter")!,
    );
    expect(encodeFilterUrl(restored)).toBe(encoded);
    expect(restored).toMatchObject({
      kind: "all_of",
      children: [
        { kind: "in", values: ["creature", "hazard"] },
        {
          kind: "any_of",
          children: [
            { kind: "compare", value: 0 },
            { kind: "not", predicate: { kind: "set_match", op: "includes_all" } },
          ],
        },
        { kind: "state_match", state: "missing" },
        { kind: "exists", predicate: { value: 'Épée + 盾 & "staff"' } },
        { kind: "exists", predicate: { value: false } },
      ],
    });
    expect(JSON.stringify(restored)).not.toContain("original-");
    if (restored.kind !== "all_of") throw new Error("Expected conjunction");
    const first = restored.children[3],
      second = restored.children[4];
    if (first.kind !== "exists" || second.kind !== "exists")
      throw new Error("Expected scopes");
    expect(first.scope_id).not.toBe(second.scope_id);
    expect(first.scope_id).not.toBe(first.clause_id);
    expect(decodeFilterUrl(encoded)).toEqual(restored);
  });
  it("produces substantially shorter query parameters than percent-escaped predicate JSON", () => {
    const packed = new URLSearchParams({
      filter: encodeFilterUrl(predicate),
    }).toString();
    const original = new URLSearchParams({
      filter: JSON.stringify(predicate),
    }).toString();
    expect(packed.length).toBeLessThan(original.length / 2);
  });
  it.each(["bad", "4.unknown", "3.", JSON.stringify(predicate)])(
    "rejects unsupported or malformed encodings: %s",
    (input) => {
      expect(() => decodeFilterUrl(input)).toThrow();
    },
  );
  it.each(
    [null, ["a", null], ["e", 42, ["b", true]], ["i", "kind", {}]].map((value) => ({
      value,
    })),
  )(
    "rejects malformed decoded structure instead of dropping conditions",
    ({ value }) => {
      expect(() => decodeFilterUrl(wire(value))).toThrow("Invalid filter URL");
    },
  );
});

it("rejects an otherwise valid old vocabulary link explicitly", () => {
  const old =
    "1." +
    compressToEncodedURIComponent(
      JSON.stringify({
        kind: "compare",
        field: "record.kind",
        op: "eq",
        value: "spell",
      }),
    );
  expect(() => decodeFilterUrl(old)).toThrow("unsupported filter version");
});

function wire(value: unknown): string {
  return (
    "3." + compressToEncodedURIComponent(JSON.stringify(value)).replace(/\+/g, "_")
  );
}

it.each(
  [
    ["secret:internal", [["items"]]],
    ["secret:internal", [["items", -1]]],
    ["secret:internal", null, null, ["h"]],
    ["secret:internal", null, null, null, null, "2"],
  ].map((selection) => ({ selection })),
)(
  "rejects malformed exact selection without exposing its address: %j",
  ({ selection }) => {
    expect(() => decodeFilterUrl(wire(["t", selection]))).toThrow("Invalid filter URL");
  },
);

const rootSelection: QueryRecordSelection = {
  record_key: "equipment-srd:4tIVTg9wj56RrveA",
  embedded_path: [],
  field: null,
  passage: null,
  source_fingerprint: null,
};

const exactSelections: QueryRecordSelection[] = [
  rootSelection,
  { ...rootSelection, spell_rank: 0 },
  {
    ...rootSelection,
    embedded_path: [
      { collection: "items", identity: { Stable: "123" } },
      { collection: "pages", identity: { SnapshotLocal: { index: 0 } } },
    ],
    field: "description",
    source_fingerprint: "source-fingerprint",
    spell_rank: 2,
    passage: { kind: "identity" },
  },
  {
    ...rootSelection,
    passage: {
      kind: "html_section",
      selection_version: "v1",
      section_ordinal: 0,
      label: 'Épée + 盾 & "staff"',
      chunk_bytes: { start: 0, end: 55 },
      prepared_html_sha256: "html-fingerprint",
      canonical_text_sha256: "text-fingerprint",
    },
  },
  {
    ...rootSelection,
    field: "",
    source_fingerprint: "",
    passage: {
      kind: "plain_section",
      selection_version: "v1",
      section_ordinal: 3,
      label: null,
      chunk_bytes: { start: 55, end: 55 },
      source_text_sha256: "source-text",
    },
  },
];

it.each(exactSelections)(
  "preserves every exact selection slot in either direction and scope: %j",
  (selection) => {
    for (const direction of ["incoming", "outgoing"] as const) {
      for (const include_contents of [true, false]) {
        const relationship = { direction, include_contents, selection };
        const encoded = encodeFilterUrl({ kind: "relationship", relationship });
        expect(decodeFilterUrl(encoded)).toEqual({
          kind: "relationship",
          clause_id: "url-clause-1",
          relationship,
        });
        expect(encodeFilterUrl(decodeFilterUrl(encoded))).toBe(encoded);
      }
    }
  },
);

it("shortens the user's two-link URL substantially without dropping either constraint", () => {
  const predicate: QueryPredicate = {
    kind: "all_of",
    children: [
      {
        kind: "relationship",
        relationship: {
          direction: "incoming",
          include_contents: true,
          selection: rootSelection,
        },
      },
      {
        kind: "relationship",
        relationship: {
          direction: "incoming",
          include_contents: true,
          selection: {
            ...rootSelection,
            record_key: "conditionitems:dfCMdR4wnpbYNTix",
          },
        },
      },
    ],
  };
  const encoded = encodeFilterUrl(predicate);
  expect(encodeURIComponent(encoded).length).toBeLessThan(160);
  expect(encoded).toMatch(/^3\.[A-Za-z0-9_-]+$/);
  const restored = decodeFilterUrl(encoded);
  const stripIds = (_key: string, value: unknown) =>
    _key === "clause_id" ? undefined : value;
  expect(JSON.stringify(restored, stripIds)).toBe(JSON.stringify(predicate));
});

it.each(
  [
    [],
    ["unknown"],
    ["n"],
    ["n", ["b", true], ["b", false]],
    ["b", 1],
    ["b", true, false],
    ["c", "level_or_rank", "eq", null],
    ["i", "kind", [null]],
    ["s", "traits", "includes_any", [1]],
    ["m", "kind", 1],
    ["t", "secret:internal", true],
    ["f", "secret:internal", 0, 0],
    ["t", ["secret:internal", null, null, ["i", "extra"]]],
    ["t", ["secret:internal", null, null, ["p", "v1", 0, null, 5, 4, "hash"]]],
    ["t", ["secret:internal", [["items", 0.5]]]],
    ["t", ["secret:internal", null, null, null, 5]],
    ["t", ["secret:internal", null, null, null, null, 0, "extra"]],
  ].map((value) => ({ value })),
)("rejects ambiguous, extra or malformed wire fields: %j", ({ value }) => {
  expect(() => decodeFilterUrl(wire(value))).toThrow("Invalid filter URL");
});

it("retains transport depth, node and encoded-length bounds", () => {
  let deep: unknown = ["b", true];
  for (let i = 0; i < 65; i++) deep = ["n", deep];
  expect(() => decodeFilterUrl(wire(deep))).toThrow("Invalid filter URL");
  expect(() =>
    decodeFilterUrl(wire(["a", ...Array.from({ length: 1024 }, () => ["b", true])])),
  ).toThrow("Invalid filter URL");
  expect(() => decodeFilterUrl("3." + "A".repeat(16_383))).toThrow(
    "Invalid filter URL",
  );
});

it("does not reinterpret a previous full-JSON URL as the new compact vocabulary", () => {
  const old =
    "2." +
    compressToEncodedURIComponent(
      JSON.stringify({ kind: "boolean_constant", value: true }),
    );
  expect(() => decodeFilterUrl(old)).toThrow("unsupported filter version");
});

it("keeps the version-three wire contract stable for an existing compact link", () => {
  const saved =
    "3.NoIghiA0oC5SBTAjgVwJYAcC2CB2MBaAZwCcATALgBYYBJANQBUBzATgHcArAVgDYAlEgDcEAQRABdaCDiQQAYwD2uMmhhplahFiIUyAMwDCAWTL8q7XBgBGATQByjNAA9JEoA";
  expect(decodeFilterUrl(saved)).toMatchObject({
    kind: "all_of",
    children: [
      {
        kind: "relationship",
        relationship: {
          direction: "incoming",
          include_contents: true,
          selection: { record_key: "equipment-srd:4tIVTg9wj56RrveA" },
        },
      },
      {
        kind: "relationship",
        relationship: {
          direction: "incoming",
          include_contents: true,
          selection: { record_key: "conditionitems:dfCMdR4wnpbYNTix" },
        },
      },
    ],
  });
  expect(encodeFilterUrl(decodeFilterUrl(saved))).toBe(saved);
});
