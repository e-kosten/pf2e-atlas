import { describe, expect, it } from "vitest";

import type { OntologyTextLine } from "../../src/domain/ontology-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { Pf2eTerminalSearchQuery, Pf2eTerminalSearchSession } from "../../src/tui/search/service-types.js";
import { buildSearchFooterText, buildSearchHelpLines } from "../../src/tui/search-screen/interactions.js";
import {
  buildResultActionEntries,
  buildResultDetailLines,
  buildResultLines,
  canChangeResultSort,
} from "../../src/tui/search-screen/results.js";
import { createInitialSearchScreenState } from "../../src/tui/search-screen/state.js";

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "spell:test-alarm-ward",
    id: "test-alarm-ward",
    name: "Alarm Ward",
    normalizedName: "alarm ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spell",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: "rare",
    traits: ["fortune", "mental"],
    derivedTags: ["alarm"],
    publicationTitle: "Player Core",
    publicationRemaster: true,
    descriptionText: "Warns against intruders.",
    blurbText: null,
    hasDescription: true,
    descriptionSnippet: "Warns against intruders.",
    sourceCategory: "core",
    folderId: null,
    families: ["security"],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/spells/alarm-ward.json",
    isUnique: false,
    size: null,
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane"],
    spellKinds: ["spell"],
    saveType: null,
    areaType: null,
    rangeText: "30 feet",
    durationText: "1 minute",
    durationUnit: "minute",
    targetText: null,
    areaValue: null,
    sustained: false,
    basicSave: false,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    disableText: null,
    disableSkills: [],
    isComplex: false,
    actorMetrics: {},
    itemMetrics: {},
    rangeValue: 30,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

function createQuery(): Pf2eTerminalSearchQuery {
  return {
    mode: "search",
    limit: 20,
    search: {
      query: "alarm ward",
      profile: "balanced",
    },
  };
}

function createSession(
  record: NormalizedRecord,
  overrides: Partial<Pf2eTerminalSearchSession> = {},
): Pf2eTerminalSearchSession {
  return {
    windowId: "window-1",
    query: createQuery(),
    results: [record],
    windowOffset: 0,
    resultMode: "hybrid",
    total: 1,
    loadedCount: 1,
    hasMore: false,
    nextOffset: null,
    searchProfile: "balanced",
    sort: "ranked",
    sortSeed: null,
    ...overrides,
  };
}

function createResultState(session: Pf2eTerminalSearchSession) {
  return createInitialSearchScreenState(session.query, {
    layout: "results",
    session,
  });
}

describe("search result detail lines", () => {
  it("reuses the shared ontology detail presenter so result previews include the AoN link metadata", () => {
    const lines = buildResultDetailLines(createRecord(), createSession(createRecord()), 0);
    const linkLine = lines.find(
      (line): line is OntologyTextLine & { href: string; plainTextFallback: string } =>
        typeof line.href === "string" && typeof line.plainTextFallback === "string",
    );

    expect(lines[0]?.text).toBe("Result Preview");
    expect(lines[1]?.text).toBe("Showing: result 1/1");
    expect(lines[2]?.text).toBe("Sort: Ranked");
    expect(lines[3]?.text).toBe("Source: Spells");
    expect(lines.some((line) => line.text === "Archives of Nethys" && line.tone === "section")).toBe(true);
    expect(linkLine).toBeDefined();
    expect(linkLine?.text).toBe("Open in Archives of Nethys");
    expect(linkLine?.href).toContain("https://2e.aonprd.com/Search.aspx?display=short&type=eqs");
    expect(linkLine?.plainTextFallback).toContain("Open in Archives of Nethys: https://2e.aonprd.com");
    expect(linkLine?.href).toContain("include-traits=fortune+mental");
  });

  it("adds a light match-type metadata line for lookup previews", () => {
    const lines = buildResultDetailLines(
      { ...createRecord({ name: "Fire Ball" }), matchType: "exact" as const },
      createSession(createRecord({ name: "Fire Ball" }), {
        query: {
          mode: "lookup",
          limit: 20,
          search: {
            query: "Fire Ball",
          },
        },
        searchProfile: null,
        resultMode: "lexical",
        sort: "alphabeticalTiered",
      }),
      0,
    );

    expect(lines[1]?.text).toBe("Showing: result 1/1");
    expect(lines[2]?.text).toBe("Sort: Alphabetical (tiered)");
    expect(lines[3]?.text).toBe("Match: Exact");
    expect(lines[4]?.text).toBe("Source: Spells");
  });

  it("renders lookup tiered sections without per-row badges and global lookup badges without sections", () => {
    const exact = { ...createRecord({ name: "Fire Ball", recordKey: "spell:exact" }), matchType: "exact" as const };
    const normalized = {
      ...createRecord({ name: "fire ball", recordKey: "spell:normalized" }),
      matchType: "normalized_exact" as const,
    };
    const fuzzy = { ...createRecord({ name: "Firewall", recordKey: "spell:fuzzy" }), matchType: "fuzzy" as const };
    const query: Pf2eTerminalSearchQuery = {
      mode: "lookup",
      limit: 20,
      search: {
        query: "Fire Ball",
      },
    };

    const tieredLines = buildResultLines(
      createSession(exact, {
        query,
        results: [exact, normalized, fuzzy],
        total: 3,
        loadedCount: 3,
        sort: "alphabeticalTiered",
      }),
      0,
      12,
      false,
    );
    const globalLines = buildResultLines(
      createSession(exact, {
        query,
        results: [normalized, exact, fuzzy],
        total: 3,
        loadedCount: 3,
        sort: "alphabeticalGlobal",
      }),
      0,
      12,
      false,
    );

    expect(tieredLines.map((line) => line.text)).toContain("Exact");
    expect(tieredLines.map((line) => line.text)).toContain("Normalized Exact");
    expect(tieredLines.map((line) => line.text)).toContain("Fuzzy");
    expect(tieredLines.some((line) => line.text.includes("[exact]"))).toBe(false);
    expect(globalLines.some((line) => line.text === "Exact")).toBe(false);
    expect(globalLines.some((line) => line.text.includes("[normalized]"))).toBe(true);
    expect(globalLines.some((line) => line.text.includes("[exact]"))).toBe(true);
    expect(globalLines.some((line) => line.text.includes("[fuzzy]"))).toBe(true);
  });

  it("only exposes sort actions for browse and lookup result readers", () => {
    const record = createRecord();
    const browseSession = createSession(record, {
      query: { mode: "browse", limit: 20 },
      resultMode: "browse",
      searchProfile: null,
      sort: "alphabetical",
    });
    const searchSession = createSession(record);
    const lookupSession = createSession(record, {
      query: {
        mode: "lookup",
        limit: 20,
        search: {
          query: "Alarm Ward",
        },
      },
      resultMode: "lexical",
      searchProfile: null,
      sort: "alphabeticalTiered",
    });

    expect(canChangeResultSort(browseSession)).toBe(true);
    expect(canChangeResultSort(searchSession)).toBe(false);
    expect(canChangeResultSort(lookupSession)).toBe(true);

    expect(buildResultActionEntries(createResultState(browseSession), "app").map((entry) => entry.id)).toContain(
      "sortResults",
    );
    expect(buildResultActionEntries(createResultState(searchSession), "app").map((entry) => entry.id)).not.toContain(
      "sortResults",
    );
    expect(buildResultActionEntries(createResultState(lookupSession), "app").map((entry) => entry.id)).toContain(
      "sortResults",
    );
  });

  it("uses action-oriented help and footer copy in the result reader", () => {
    const session = createSession(createRecord(), {
      query: { mode: "browse", limit: 20 },
      resultMode: "browse",
      searchProfile: null,
      sort: "alphabetical",
    });
    const state = createResultState(session);
    const footer = buildSearchFooterText(state, false, "app");
    const helpLines = buildSearchHelpLines(state, [], "app", buildResultActionEntries(state, "app")).map((line) => line.text);

    expect(footer).toContain("actions");
    expect(footer).not.toContain("commands");
    expect(helpLines.some((line) => line.toLowerCase().includes("focus the result action rail"))).toBe(true);
    expect(helpLines.some((line) => line.toLowerCase().includes("jump to result"))).toBe(true);
  });
});
