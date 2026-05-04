import { describe, expect, it } from "vitest";

import { buildAonSearchLink, type AonSearchRecordLike } from "../../src/app/external-links/aon-search.js";

function createAonSearchRecord(overrides: Partial<AonSearchRecordLike> = {}): AonSearchRecordLike {
  return {
    name: "Alarm Ward",
    category: "spell",
    subcategory: null,
    rarity: "rare",
    traits: ["Mental", "auditory", "fortune", "mental"],
    traditions: ["Occult", "arcane", "occult"],
    actionCost: 2,
    ...overrides,
  };
}

describe("AoN search links", () => {
  it("builds a spell search link with full trait carry-through and stable filter ordering", () => {
    const link = buildAonSearchLink(createAonSearchRecord());

    expect(link).not.toBeNull();
    expect(link?.appliedFilters).toEqual({
      query: "Alarm Ward",
      includeTypes: ["spell", "cantrip", "ritual"],
      includeTraits: ["auditory", "fortune", "mental"],
      includeRarities: ["rare"],
      includeTraditions: ["arcane", "occult"],
      includeActions: ["2"],
    });
    expect(link?.label).toBe("Open in Archives of Nethys");
    expect(link?.plainTextFallback).toBe(`Open in Archives of Nethys: ${link?.url ?? "(missing url)"}`);

    const url = new URL(link!.url);
    expect(`${url.origin}${url.pathname}`).toBe("https://2e.aonprd.com/Search.aspx");
    expect(url.searchParams.get("display")).toBe("short");
    expect(url.searchParams.get("type")).toBe("eqs");
    expect(url.searchParams.get("q")).toBe("Alarm Ward");
    expect(url.searchParams.get("include-types")).toBe("spell cantrip ritual");
    expect(url.searchParams.get("include-traits")).toBe("auditory fortune mental");
    expect(url.searchParams.get("include-rarities")).toBe("rare");
    expect(url.searchParams.get("include-traditions")).toBe("arcane occult");
    expect(url.searchParams.get("include-actions")).toBe("2");
  });

  it("omits empty structured filters but still produces a generic AoN search link", () => {
    const link = buildAonSearchLink(
      createAonSearchRecord({
        name: "Chronicle Note",
        category: "lore",
        rarity: "common",
        traits: [],
        traditions: [],
        actionCost: null,
      }),
    );

    expect(link).not.toBeNull();
    expect(link?.appliedFilters).toEqual({
      query: "Chronicle Note",
      includeTypes: [],
      includeTraits: [],
      includeRarities: [],
      includeTraditions: [],
      includeActions: [],
    });

    const url = new URL(link!.url);
    expect(url.searchParams.get("type")).toBe("eqs");
    expect(url.searchParams.get("q")).toBe("Chronicle Note");
    expect(url.searchParams.has("include-types")).toBe(false);
    expect(url.searchParams.has("include-traits")).toBe(false);
    expect(url.searchParams.has("include-rarities")).toBe(false);
    expect(url.searchParams.has("include-traditions")).toBe(false);
    expect(url.searchParams.has("include-actions")).toBe(false);
  });

  it("returns null when the record name is blank", () => {
    expect(
      buildAonSearchLink(
        createAonSearchRecord({
          name: "   ",
        }),
      ),
    ).toBeNull();
  });
});
