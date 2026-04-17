import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, NormalizedRecord } from "../../src/types.js";
import {
  Pf2eTerminalApp,
  Pf2eTerminalBootstrap,
} from "../../src/tui/pf2e-app.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import type { SearchCategory } from "../../src/types.js";
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

function createFakeServices(overrides: Partial<Pf2eTerminalAppServices> = {}): Pf2eTerminalAppServices {
  const record = createRecord();
  return {
    config: createTestConfig(),
    catalog: {
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
    user: {
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
        ]),
        runQuery: vi.fn(async (request) => ({
          request,
          results: [record],
          total: 1,
          searchProfile: request.mode === "lookup" ? null : request.searchProfile,
        })),
      },
      ontology: {
        loadModel: vi.fn(() => ({
          categories: [
            {
              kind: "category",
              key: "spell",
              category: "spell",
              familyCount: 1,
              tagCount: 1,
              taggedRecordCount: 1,
              filterText: "spell",
              families: [
                {
                  kind: "family",
                  key: "spell:security",
                  category: "spell",
                  family: "security",
                  axis: "utility",
                  description: "Security spells",
                  subcategories: undefined,
                  variantInheritance: false,
                  tagCount: 1,
                  liveRecordCount: 1,
                  filterText: "security",
                  tags: [
                    {
                      kind: "tag",
                      key: "spell:alarm",
                      category: "spell",
                      family: "security",
                      subcategories: undefined,
                      tag: "alarm",
                      description: "Alarm effects",
                      assignmentMode: "editorial",
                      liveRecordCount: 1,
                      authoredRuleCount: 0,
                      exemplarPositiveCount: 0,
                      exemplarNegativeCount: 0,
                      legacyMigrationDefinitionCount: 0,
                      legacyMigrationRecordCount: 0,
                      filterText: "alarm",
                      records: [
                        {
                          kind: "record",
                          key: record.recordKey,
                          category: "spell",
                          tag: "alarm",
                          filterText: "alarm ward",
                          record: {
                            recordKey: record.recordKey,
                            packName: record.packName,
                            name: record.name,
                            type: record.type,
                            category: record.category,
                            subcategory: record.subcategory,
                            documentType: record.documentType,
                            level: record.level,
                            rarity: record.rarity,
                            traits: record.traits,
                            derivedTags: record.derivedTags,
                            families: record.families,
                            descriptionText: record.descriptionText,
                            blurbText: record.blurbText,
                            sourceCategory: record.sourceCategory,
                            publicationTitle: record.publicationTitle,
                            publicationRemaster: record.publicationRemaster,
                            isUnique: record.isUnique,
                            size: record.size,
                            languages: record.languages,
                            speedTypes: record.speedTypes,
                            senses: record.senses,
                            immunities: record.immunities,
                            resistances: record.resistances,
                            weaknesses: record.weaknesses,
                            itemCategory: record.itemCategory,
                            baseItem: record.baseItem,
                            priceCp: record.priceCp,
                            usage: record.usage,
                            hands: record.hands,
                            damageTypes: record.damageTypes,
                            weaponGroup: record.weaponGroup,
                            armorGroup: record.armorGroup,
                            traditions: record.traditions,
                            spellKinds: record.spellKinds,
                            saveType: record.saveType,
                            areaType: record.areaType,
                            rangeText: record.rangeText,
                            durationText: record.durationText,
                            targetText: record.targetText,
                            areaValue: record.areaValue,
                            sustained: record.sustained,
                            basicSave: record.basicSave,
                            disableText: record.disableText,
                            disableSkills: record.disableSkills,
                            isComplex: record.isComplex,
                          },
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        })),
      },
    },
    dev: {
      tagRefinement: {
        createSession: vi.fn(async () => {
          throw new Error("not implemented");
        }),
        getQueueItems: vi.fn(() => []),
        promptAndCreateSession: vi.fn(async () => undefined),
      },
    },
    close: vi.fn(),
    ...overrides,
  };
}

describe("pf2e terminal app", () => {
  afterEach(() => {
    cleanup();
  });

  it("routes from the top level into search and back out", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Choose a first-class TUI area");

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Search Results");

    app.stdin.write("q");
    await flushInk();

    expect(app.lastFrame()).toContain("Choose a first-class TUI area");
  });

  it("opens ontology via the app service layer", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(services.user.ontology.loadModel).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).toContain("Ontology Search");
  });

  it("closes loaded services when the bootstrap unmounts", async () => {
    const services = createFakeServices();
    const loadServices = vi.fn(async () => services);
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalBootstrap
          rootPath={process.cwd()}
          argv={[]}
          onExit={vi.fn()}
          loadServices={loadServices}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    app.unmount();

    expect(loadServices).toHaveBeenCalledTimes(1);
    expect(services.close).toHaveBeenCalledTimes(1);
  });
});
