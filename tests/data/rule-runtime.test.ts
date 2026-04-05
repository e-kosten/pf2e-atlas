import { describe, expect, it, vi } from "vitest";

import { collectRuleQuestionContext, getRuleGraph } from "../../src/data/rule-runtime.js";
import type {
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  RuleReferenceEdge,
} from "../../src/types.js";
import type { ReferenceEdgeRow } from "../../src/data/rows.js";

function createRecord(recordKey: string, name = recordKey): NormalizedRecord {
  return {
    recordKey,
    id: recordKey,
    name,
    normalizedName: name.toLowerCase(),
    type: "action",
    category: "rule",
    subcategory: "action",
    packName: "actions",
    packLabel: "Actions",
    documentType: "Item",
    level: null,
    rarity: "common",
    traits: [],
    derivedTags: [],
    publicationTitle: "Pathfinder Player Core",
    publicationRemaster: true,
    descriptionText: null,
    hasDescription: false,
    descriptionSnippet: null,
    sourceCategory: "core",
    folderId: null,
    families: [],
    sourcePath: `${recordKey}.json`,
    isUnique: false,
    size: null,
    itemCategory: null,
    priceCp: null,
    bulkValue: null,
    actionCost: null,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: [],
    spellKinds: [],
    saveType: null,
    areaType: null,
    rangeText: null,
    durationText: null,
    durationUnit: null,
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
    rangeValue: null,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
  };
}

describe("rule runtime", () => {
  it("sorts and limits backlinks per primary record", () => {
    const rows: ReferenceEdgeRow[] = [
      {
        fromRecordKey: "rule:feat",
        toRecordKey: "rule:target",
        displayText: "Feat Ref",
        referenceText: "feat ref",
        fromPackName: "feats",
        fromRecordType: "feat",
        fromDocumentType: "Item",
        fromSourceCategory: "core",
      },
      {
        fromRecordKey: "rule:action",
        toRecordKey: "rule:target",
        displayText: "Action Ref",
        referenceText: "action ref",
        fromPackName: "actions",
        fromRecordType: "action",
        fromDocumentType: "Item",
        fromSourceCategory: "core",
      },
      {
        fromRecordKey: "rule:rules",
        toRecordKey: "rule:target",
        displayText: "Rules Ref",
        referenceText: "rules ref",
        fromPackName: "journals",
        fromRecordType: "journal",
        fromDocumentType: "JournalEntry",
        fromSourceCategory: "rules",
      },
    ];

    const graph = getRuleGraph(["rule:target"], {
      includeBacklinks: true,
      maxBacklinksPerPrimary: 2,
    }, {
      fetchReferenceEdgeRows: (direction: RuleReferenceEdge["direction"]) => direction === "backlink" ? rows : [],
      getRecordsByKeys: (recordKeys) => recordKeys.map((recordKey) => createRecord(recordKey)),
      lookupMany: () => [],
    });

    expect(graph.backlinks.edges.map((edge) => edge.fromRecordKey)).toEqual([
      "rule:action",
      "rule:feat",
    ]);
    expect(graph.edges).toEqual(graph.backlinks.edges);
  });

  it("collects quoted rule names and reuses lookup plus graph dependencies", () => {
    const lookupMany = vi.fn((queries: LookupQuery[]): LookupResult[] => {
      return queries.map((query) => ({
        query,
        match: createRecord(`rule:${query.name.toLowerCase()}`, query.name),
        alternatives: [],
        matchType: "exact",
      }));
    });

    const context = collectRuleQuestionContext({
      question: 'How does "Grabbed" interact with "Hidden"?',
      includeBacklinks: false,
    }, {
      fetchReferenceEdgeRows: () => [
        {
          fromRecordKey: "rule:grabbed",
          toRecordKey: "rule:hidden",
          displayText: "Hidden",
          referenceText: "Hidden",
          fromPackName: "conditionitems",
          fromRecordType: "condition",
          fromDocumentType: "Item",
          fromSourceCategory: "core",
        },
      ],
      getRecordsByKeys: (recordKeys) => recordKeys.map((recordKey) => createRecord(recordKey)),
      lookupMany,
    });

    expect(lookupMany).toHaveBeenCalledWith([{ name: "Grabbed" }, { name: "Hidden" }], { coreOnly: undefined });
    expect(context.primary.map((result) => result.match?.name)).toEqual(["Grabbed", "Hidden"]);
    expect(context.outgoing.edges).toHaveLength(1);
    expect(context.backlinks.edges).toHaveLength(0);
  });
});
