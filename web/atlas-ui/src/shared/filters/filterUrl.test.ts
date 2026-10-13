import { compressToEncodedURIComponent } from "lz-string";
import type { QueryPredicate } from "../../generated/atlas";
import { decodeFilterUrl, encodeFilterUrl } from "./filterUrl";

describe("self-contained filter URLs", () => {
  const predicate: QueryPredicate = {
    kind: "all_of",
    clause_id: "original-root",
    children: [
      {
        kind: "in",
        clause_id: "original-kind",
        field: "record.kind",
        values: ["creature", "hazard"],
      },
      {
        kind: "any_of",
        clause_id: "original-or",
        children: [
          { kind: "compare", field: "record.level_or_rank", op: "eq", value: 0 },
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
      { kind: "state_match", field: "actor.hp.maximum", state: "missing" },
      {
        kind: "exists",
        clause_id: "first-exists",
        scope_id: "first-scope",
        collection: "actor.items",
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
        collection: "actor.items",
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
  it.each(["bad", "2.unknown", "1.", JSON.stringify(predicate)])(
    "rejects unsupported or malformed encodings: %s",
    (input) => {
      expect(() => decodeFilterUrl(input)).toThrow();
    },
  );
  it.each([
    null,
    { kind: "all_of", children: [null] },
    {
      kind: "exists",
      collection: 42,
      predicate: { kind: "boolean_constant", value: true },
    },
    { kind: "in", field: "record.kind", values: {} },
  ])("rejects malformed decoded structure instead of dropping conditions", (value) => {
    expect(() =>
      decodeFilterUrl("1." + compressToEncodedURIComponent(JSON.stringify(value))),
    ).toThrow();
  });
});
