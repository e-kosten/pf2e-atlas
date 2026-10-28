import { documentSection } from "./urlState";
import { recordDestination } from "./destinations";
import { describe, expect, it } from "vitest";
import {
  parseSearchParams,
  stringifySearchParams,
  selectedNavigation,
} from "./urlState";

describe("Router URL transport", () => {
  it("retains repeated sort priority and opaque compact filter values", () => {
    const params =
      "?sort=level_desc&sort=name_asc&filter=3.Some_encoded-value&q=0&limit=50";
    const parsed = parseSearchParams(params);
    expect(parsed.sort).toEqual(["level_desc", "name_asc"]);
    expect(parsed.q).toBe("0");
    expect(parsed.filter).toBe("3.Some_encoded-value");
    expect(stringifySearchParams(parsed)).toBe(params);
  });
  it("does not interpret JSON-like text as numbers, arrays, booleans or null", () => {
    for (const text of ["null", "false", "123", '["one"]', '{"key":1}', "雪 + & #"]) {
      const original = { q: text };
      expect(parseSearchParams(stringifySearchParams(original))).toEqual(original);
    }
  });
  it("treats prototype names as ordinary URL keys", () => {
    const parsed = parseSearchParams(
      "?__proto__=one&constructor=two&toString=three&constructor=four",
    );
    expect(Object.getPrototypeOf(parsed)).toBe(Object.prototype);
    expect(parsed.__proto__).toBe("one");
    expect(parsed.constructor).toEqual(["two", "four"]);
    expect(parsed.toString).toBe("three");
  });
  it("preserves exact selection and separate landing including rank zero", () => {
    const selection = {
      record_key: "actors:with/space?雪",
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
      ],
      field: "system.description.value",
      passage: { kind: "identity" as const },
      source_fingerprint: "snapshot:abc",
      spell_rank: 0,
    };
    const landing = { ...selection, field: null, passage: null };
    const params = parseSearchParams(
      stringifySearchParams(
        recordDestination({
          recordKey: selection.record_key,
          selection,
          landing,
        }).search,
      ),
    );
    expect(selectedNavigation(params, selection.record_key)).toEqual(selection);
    expect(selectedNavigation(params, selection.record_key, "focus")).toEqual(landing);
  });
  it.each([
    "%",
    "null",
    "[]",
    '{"record_key":"x"}',
    '{"record_key":3,"embedded_path":[]}',
  ])("rejects malformed exact navigation %s", (selection) => {
    expect(() => selectedNavigation({ selection }, "records:example")).toThrow(
      "unsupported reading format",
    );
  });
});

it.each([
  ["/records/a#creature-a%3Ab-casting", "creature-a:b-casting"],
  ["/records/a#literal%2523-%E9%9B%AA", "literal%23-雪"],
  ["/records/a#%", "%"],
  ["/records/a", undefined],
])("restores document section from %s exactly once", (href, expected) => {
  expect(documentSection(href)).toBe(expected);
});
