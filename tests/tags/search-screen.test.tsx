import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, NormalizedRecord, SearchCategory } from "../../src/types.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { SearchScreen } from "../../src/tui/search-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createTestConfig(): AppConfig {
  return {
    dataPath: "vendor/pf2e",
    rootPath: "vendor/pf2e",
    manifestPath: "vendor/pf2e/system.pf2e.json",
    indexPath: ".cache/pf2e-index.sqlite",
    embeddings: {
      provider: "hash",
      modelId: "test-model",
      modelRevision: null,
      cachePath: ".cache/models",
      localModelPath: null,
    },
    ranking: {
      configPath: "pf2e-ranking.json",
    },
  };
}

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "spell:test-alarm",
    id: "test-alarm",
    name: "Alarm Ward",
    normalizedName: "alarm ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spell",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: null,
    traits: [],
    derivedTags: ["alarm"],
    publicationTitle: null,
    publicationRemaster: false,
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
    targetText: "creature",
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

function createSearchServices(): Pf2eTerminalAppServices {
  const record = createRecord();
  const runQuery = vi.fn(async (request) => ({
    request,
    results: [record],
    total: 1,
    searchProfile: request.mode === "lookup" ? null : request.searchProfile,
  }));

  return {
    config: createTestConfig(),
    dataService: {
      getRecord: vi.fn(() => record),
      getSearchVocabulary: vi.fn(() => ({}) as never),
      listFilterValues: vi.fn(() => ({ field: "categories", values: [] }) as never),
      lookup: vi.fn(() => ({ match: record, alternatives: [] })),
      search: vi.fn(async () => ({
        searchProfile: "balanced",
        mode: "hybrid",
        total: 1,
        offset: 0,
        limit: 20,
        records: [record],
      })),
    },
    search: {
      getCategoryOptions: vi.fn(() => [
        {
          value: null,
          label: "Any Category",
          description: "Search across the full indexed PF2E corpus.",
        },
        {
          value: "spell" satisfies SearchCategory,
          label: "Spell",
          description: "1 indexed canonical record.",
        },
      ]),
      getProfileOptions: vi.fn(() => [
        {
          value: "balanced",
          label: "Balanced",
          description: "Default hybrid retrieval for concise themed searches.",
        },
        {
          value: "lexical",
          label: "Lexical",
          description: "Exact-wording heavy retrieval for names and precise PF2E terms.",
        },
      ]),
      runQuery,
    },
    tagWorkbench: {
      createSession: vi.fn(async () => {
        throw new Error("not implemented");
      }),
      getOntologyModel: vi.fn(() => ({ categories: [] })),
      getQueueItems: vi.fn(() => []),
      promptAndCreateSession: vi.fn(async () => undefined),
    },
    close: vi.fn(),
  };
}

describe("search screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("uses the selected search profile and category when running a search", async () => {
    const services = createSearchServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("p");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Profile");
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    app.stdin.write("f");
    await flushInk();
    expect(app.lastFrame()).toContain("Category Filter");
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    app.stdin.write("/");
    await flushInk();
    expect(app.lastFrame()).toContain("Search PF2E Records");

    app.stdin.write("g");
    app.stdin.write("h");
    app.stdin.write("o");
    app.stdin.write("s");
    app.stdin.write("t");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(services.search.runQuery).toHaveBeenCalledWith({
      category: "spell",
      limit: 20,
      mode: "search",
      queryText: "ghost",
      searchProfile: "lexical",
    });
    expect(app.lastFrame()).toContain("profile lexical");
    expect(app.lastFrame()).toContain("spell");
  });
});
