import { describe, expect, it } from "vitest";

import {
  buildEntityPageDocument,
  buildOntologyExplorerEntityDetailLines,
  renderEntityPageDocument,
} from "../../src/app/ontology/entity-page.js";
import type { OntologyExplorerEntityRecord } from "../../src/app/ontology/entity-record.js";
import type { PageRelationsResult } from "../../src/domain/page-relations-types.js";
import type { OntologyTextLine } from "../../src/domain/ontology-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";

function createRecord(overrides: Partial<OntologyExplorerEntityRecord> = {}): OntologyExplorerEntityRecord {
  return {
    recordKey: "spell:test-fireball",
    packName: "spells",
    name: "Fireball",
    type: "spell",
    category: "spell",
    subcategory: null,
    documentType: "Item",
    level: 3,
    rarity: "common",
    traits: ["concentrate", "fire", "manipulate"],
    derivedTags: ["explosive_magic"],
    families: ["damage_burst"],
    descriptionText: "A roaring blast of fire detonates at a spot you designate.",
    blurbText: "A compact burst spell.",
    sourceCategory: "core",
    publicationTitle: "Pathfinder Player Core",
    publicationRemaster: true,
    isUnique: false,
    size: null,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypes: ["fire"],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane", "primal"],
    spellKinds: ["spell"],
    saveType: "reflex",
    areaType: "burst",
    rangeText: "500 feet",
    durationText: null,
    targetText: null,
    areaValue: 20,
    sustained: false,
    basicSave: true,
    disableText: null,
    disableSkills: [],
    isComplex: false,
    actorMetrics: {},
    itemMetrics: {},
    raw: {},
    ...overrides,
  };
}

function createNormalizedRecord(
  overrides: Partial<NormalizedRecord> & Pick<NormalizedRecord, "recordKey" | "name" | "category">,
): NormalizedRecord {
  return {
    recordKey: overrides.recordKey,
    id: overrides.recordKey,
    name: overrides.name,
    normalizedName: overrides.name.toLowerCase(),
    type: overrides.category,
    category: overrides.category,
    subcategory: overrides.subcategory ?? null,
    packName: overrides.packName ?? "test-pack",
    packLabel: overrides.packLabel ?? "Test Pack",
    documentType: overrides.documentType ?? "Item",
    level: overrides.level ?? null,
    rarity: overrides.rarity ?? null,
    traits: overrides.traits ?? [],
    derivedTags: overrides.derivedTags ?? [],
    publicationTitle: overrides.publicationTitle ?? null,
    publicationRemaster: overrides.publicationRemaster ?? false,
    descriptionText: overrides.descriptionText ?? null,
    blurbText: overrides.blurbText ?? null,
    hasDescription: overrides.hasDescription ?? false,
    descriptionSnippet: overrides.descriptionSnippet ?? null,
    sourceCategory: overrides.sourceCategory ?? "core",
    folderId: overrides.folderId ?? null,
    families: overrides.families ?? [],
    variantFamilyKey: overrides.variantFamilyKey ?? null,
    variantBaseName: overrides.variantBaseName ?? null,
    variantLabel: overrides.variantLabel ?? null,
    variantAxes: overrides.variantAxes ?? [],
    variantConfidence: overrides.variantConfidence ?? null,
    variantSource: overrides.variantSource ?? "none",
    sourcePath: overrides.sourcePath ?? "test.json",
    isUnique: overrides.isUnique ?? false,
    size: overrides.size ?? null,
    itemCategory: overrides.itemCategory ?? null,
    baseItem: overrides.baseItem ?? null,
    priceCp: overrides.priceCp ?? null,
    bulkValue: overrides.bulkValue ?? null,
    actionCost: overrides.actionCost ?? null,
    usage: overrides.usage ?? null,
    hands: overrides.hands ?? null,
    damageTypes: overrides.damageTypes ?? [],
    weaponGroup: overrides.weaponGroup ?? null,
    armorGroup: overrides.armorGroup ?? null,
    traditions: overrides.traditions ?? [],
    spellKinds: overrides.spellKinds ?? [],
    saveType: overrides.saveType ?? null,
    areaType: overrides.areaType ?? null,
    rangeText: overrides.rangeText ?? null,
    durationText: overrides.durationText ?? null,
    durationUnit: overrides.durationUnit ?? null,
    targetText: overrides.targetText ?? null,
    areaValue: overrides.areaValue ?? null,
    sustained: overrides.sustained ?? false,
    basicSave: overrides.basicSave ?? false,
    languages: overrides.languages ?? [],
    speedTypes: overrides.speedTypes ?? [],
    senses: overrides.senses ?? [],
    immunities: overrides.immunities ?? [],
    resistances: overrides.resistances ?? [],
    weaknesses: overrides.weaknesses ?? [],
    disableText: overrides.disableText ?? null,
    disableSkills: overrides.disableSkills ?? [],
    isComplex: overrides.isComplex ?? false,
    actorMetrics: overrides.actorMetrics ?? {},
    itemMetrics: overrides.itemMetrics ?? {},
    rangeValue: overrides.rangeValue ?? null,
    aliases: overrides.aliases ?? [],
    legacyRecordLinks: overrides.legacyRecordLinks ?? [],
    raw: overrides.raw ?? {},
  };
}

function createRelations(): PageRelationsResult {
  const edge = {
    fromRecordKey: "spell:test-fireball",
    toRecordKey: "spells:fireball-effect",
    displayText: "Spell Effect: Fireball",
    referenceText: "Fireball",
    direction: "outgoing" as const,
    relationshipType: "references" as const,
    sourcePackName: "spells",
    sourceRecordType: "spell",
    sourceDocumentType: "Item",
    sourceCategory: "core" as const,
  };

  return {
    recordKey: "spell:test-fireball",
    outgoing: {
      records: [
        createNormalizedRecord({
          recordKey: "spells:fireball-effect",
          name: "Fireball Effect",
          category: "spell",
        }),
      ],
      edges: [edge],
    },
    incoming: {
      records: [
        createNormalizedRecord({
          recordKey: "feat:blazing-conduit",
          name: "Blazing Conduit",
          category: "feat",
        }),
      ],
      edges: [],
    },
    edges: [edge],
    incomingGroups: [
      {
        category: "feat",
        subcategory: null,
        count: 1,
        request: {
          mode: "browse",
          filter: {
            kind: "allOf",
            children: [
              {
                kind: "scope",
                category: "feat",
                subcategory: { kind: "any" },
              },
              {
                kind: "linksTo",
                target: "spell:test-fireball",
              },
            ],
          },
          sort: { kind: "alphabetical" },
          limit: 50,
        },
      },
    ],
  };
}

describe("entity page document", () => {
  it("builds a shared header with identity line, AoN link, and traits", () => {
    const document = buildEntityPageDocument(createRecord());
    const lines = renderEntityPageDocument(document);

    expect(document.title).toBe("Fireball");
    expect(document.identityLine).toBe("Spell | Rank 3 | Common | Pathfinder Player Core");
    expect(lines[0]).toMatchObject({ text: "Fireball", tone: "section" });
    expect(lines[1]).toMatchObject({ text: "Spell | Rank 3 | Common | Pathfinder Player Core", indent: 2 });
    expect(lines[2]).toMatchObject({ text: "AoN: Open in Archives of Nethys", indent: 2 });
    expect(lines[3]).toMatchObject({ text: "Traits: Concentrate, Fire, Manipulate", indent: 2 });
    expect(document.traitTargets?.map((target) => target.label)).toEqual([
      "Trait: Concentrate",
      "Trait: Fire",
      "Trait: Manipulate",
    ]);
  });

  it("uses the spell recipe ordering with summary, description, details, and classification", () => {
    const document = buildEntityPageDocument(createRecord());

    expect(document.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Description",
      "Details",
      "Classification",
    ]);
  });

  it("keeps spell summary facts in the first scan section", () => {
    const document = buildEntityPageDocument(createRecord());
    const summary = document.sections.find((section) => section.title === "Summary");

    expect(summary?.blocks).toEqual(
      expect.arrayContaining([
        { kind: "text", text: "A compact burst spell." },
        {
          kind: "factList",
          facts: expect.arrayContaining([
            { label: "Traditions", value: "Arcane, Primal" },
            { label: "Cast", value: "2 actions" },
            { label: "Range", value: "500 feet" },
            { label: "Area", value: "20 Burst" },
            { label: "Save", value: "basic Reflex" },
            { label: "Damage", value: "Fire" },
          ]),
        },
      ]),
    );
  });

  it("uses creature recipe sections when creature fields are available", () => {
    const document = buildEntityPageDocument(
      createRecord({
        recordKey: "creature:adult-red-dragon",
        name: "Adult Red Dragon",
        type: "creature",
        category: "creature",
        level: 14,
        size: "huge",
        languages: ["common", "draconic"],
        senses: ["darkvision"],
        speedTypes: ["fly", "climb"],
        immunities: ["fire"],
        resistances: ["physical"],
        weaknesses: ["cold"],
        damageTypes: ["fire"],
        actorMetrics: {
          "ability.str.mod": 8,
          "ability.dex.mod": 4,
          "ability.con.mod": 7,
          "ability.int.mod": 3,
          "ability.wis.mod": 5,
          "ability.cha.mod": 6,
          "ac.value": 36,
          "hp.value": 275,
          "perception.mod": 26,
          "save.fort.mod": 27,
          "save.ref.mod": 24,
          "save.will.mod": 25,
          "skill.acrobatics.mod": 24,
          "skill.arcana.mod": 25,
          "speed.climb.value": 30,
          "speed.fly.value": 180,
        },
      }),
    );

    expect(document.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Defense",
      "Movement",
      "Offense",
      "Description",
      "Details",
      "Classification",
    ]);
    expect(document.sections.find((section) => section.title === "Summary")?.blocks).toEqual(
      expect.arrayContaining([
        {
          kind: "factList",
          facts: expect.arrayContaining([
            { label: "Perception", value: "+26" },
            { label: "Skills", value: "Acrobatics +24, Arcana +25" },
            { label: "Abilities", value: "Str +8, Dex +4, Con +7, Int +3, Wis +5, Cha +6" },
          ]),
        },
      ]),
    );
    expect(document.sections.find((section) => section.title === "Defense")?.blocks).toEqual([
      {
        kind: "factList",
        facts: expect.arrayContaining([
          { label: "AC", value: "36" },
          { label: "HP", value: "275" },
          { label: "Saves", value: "Fort +27, Ref +24, Will +25" },
          { label: "Immunities", value: "Fire" },
          { label: "Resistances", value: "Physical" },
          { label: "Weaknesses", value: "Cold" },
        ]),
      },
    ]);
    expect(document.sections.find((section) => section.title === "Movement")?.blocks).toEqual([
      {
        kind: "factList",
        facts: [{ label: "Speed", value: "Climb 30 feet, Fly 180 feet" }],
      },
    ]);
  });

  it("uses equipment recipe sections for item-oriented facts", () => {
    const document = buildEntityPageDocument(
      createRecord({
        recordKey: "equipment:striking-longsword",
        name: "+1 Striking Longsword",
        type: "equipment",
        category: "equipment",
        level: 4,
        priceCp: 10_000,
        bulkValue: 1,
        usage: "held-in-one-hand",
        hands: 1,
        baseItem: "longsword",
        itemCategory: "weapon",
        weaponGroup: "sword",
        actionCost: 1,
      }),
    );

    expect(document.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Description",
      "Details",
      "Classification",
    ]);
    expect(document.sections.find((section) => section.title === "Summary")?.blocks).toEqual(
      expect.arrayContaining([
        {
          kind: "factList",
          facts: expect.arrayContaining([
            { label: "Price", value: "100 gp" },
            { label: "Bulk", value: "1" },
            { label: "Activation", value: "1 action" },
            { label: "Usage", value: "Held In One Hand" },
          ]),
        },
      ]),
    );
  });

  it("uses a dedicated feat/action recipe family instead of the generic fallback", () => {
    const featDocument = buildEntityPageDocument(
      createRecord({
        recordKey: "feat:blazing-conduit",
        name: "Blazing Conduit",
        type: "feat",
        category: "feat",
        level: 4,
        actionCost: 1,
        rangeText: "60 feet",
        targetText: "1 ally",
        durationText: "1 minute",
        damageTypes: ["fire"],
        raw: {
          system: {
            frequency: { value: "once per day" },
            prerequisites: { value: [{ value: "expert in Arcana" }] },
            requirements: { value: "You are wielding a torch" },
            trigger: { value: "An ally within range casts a fire spell" },
          },
        },
      }),
    );
    const actionDocument = buildEntityPageDocument(
      createRecord({
        recordKey: "rule:raise-a-shield",
        name: "Raise a Shield",
        type: "action",
        category: "rule",
        subcategory: "action",
        level: null,
        actionCost: 1,
        durationText: "until the start of your next turn",
      }),
    );

    expect(featDocument.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Description",
      "Details",
      "Classification",
    ]);
    expect(featDocument.sections.find((section) => section.title === "Summary")?.blocks).toEqual(
      expect.arrayContaining([
        {
          kind: "factList",
          facts: expect.arrayContaining([
            { label: "Action Cost", value: "1 action" },
            { label: "Trigger", value: "An ally within range casts a fire spell" },
            { label: "Requirements", value: "You are wielding a torch" },
            { label: "Frequency", value: "once per day" },
            { label: "Prerequisites", value: "expert in Arcana" },
            { label: "Range", value: "60 feet" },
            { label: "Targets", value: "1 ally" },
            { label: "Duration", value: "1 minute" },
            { label: "Damage", value: "Fire" },
          ]),
        },
      ]),
    );
    expect(actionDocument.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Description",
      "Details",
      "Classification",
    ]);
    expect(actionDocument.sections.find((section) => section.title === "Summary")?.blocks).toEqual(
      expect.arrayContaining([
        {
          kind: "factList",
          facts: expect.arrayContaining([
            { label: "Action Cost", value: "1 action" },
            { label: "Duration", value: "until the start of your next turn" },
          ]),
        },
      ]),
    );
  });

  it("uses hazard recipe sections when hazard-specific fields are available", () => {
    const document = buildEntityPageDocument(
      createRecord({
        recordKey: "hazard:flame-jet",
        name: "Flame Jet",
        type: "hazard",
        category: "hazard",
        level: 5,
        disableText: "Thievery DC 22 to jam the nozzle",
        disableSkills: ["thievery"],
        isComplex: false,
        rangeText: "30 feet",
        areaType: "line",
        areaValue: 30,
        damageTypes: ["fire"],
        actorMetrics: {
          "ac.value": 24,
          "hp.value": 64,
          "hardness.value": 12,
          "save.fort.mod": 15,
          "save.ref.mod": 12,
        },
      }),
    );

    expect(document.sections.map((section) => section.title)).toEqual([
      "Summary",
      "Defense",
      "Routine",
      "Description",
      "Details",
      "Classification",
    ]);
    expect(document.sections.find((section) => section.title === "Defense")?.blocks).toEqual([
      {
        kind: "factList",
        facts: expect.arrayContaining([
          { label: "AC", value: "24" },
          { label: "HP", value: "64" },
          { label: "Hardness", value: "12" },
          { label: "Saves", value: "Fort +15, Ref +12" },
        ]),
      },
    ]);
  });

  it("emits a fallback details section for leftover structured facts", () => {
    const document = buildEntityPageDocument(createRecord());
    const details = document.sections.find((section) => section.title === "Details");

    expect(details?.blocks).toEqual([
      {
        kind: "factList",
        facts: expect.arrayContaining([
          { label: "Spell Kinds", value: "Spell" },
          { label: "Source Category", value: "Core" },
          { label: "Document Type", value: "Item" },
        ]),
      },
    ]);
  });

  it("emits classification pivots as seeded search targets", () => {
    const document = buildEntityPageDocument(createRecord());
    const classification = document.sections.find((section) => section.title === "Classification");

    expect(classification?.blocks).toEqual([
      {
        kind: "targetList",
        targets: expect.arrayContaining([
          {
            kind: "searchPivot",
            label: "Pack: spells",
            request: {
              mode: "browse",
              filter: { kind: "pack", value: "spells" },
              sort: { kind: "alphabetical" },
              limit: 50,
            },
          },
          {
            kind: "searchPivot",
            label: "Category: Spell",
            request: {
              mode: "browse",
              filter: {
                kind: "scope",
                category: "spell",
                subcategory: { kind: "any" },
              },
              sort: { kind: "alphabetical" },
              limit: 50,
            },
          },
          {
            kind: "searchPivot",
            label: "Derived Tags: Explosive Magic",
            request: {
              mode: "browse",
              filter: {
                kind: "allOf",
                children: [
                  {
                    kind: "scope",
                    category: "spell",
                    subcategory: { kind: "any" },
                  },
                  {
                    kind: "metadataPredicate",
                    predicate: { field: "derivedTags", op: "includes", value: "explosive_magic" },
                  },
                ],
              },
              sort: { kind: "alphabetical" },
              limit: 50,
            },
          },
          {
            kind: "searchPivot",
            label: "Families: Damage Burst",
            request: {
              mode: "browse",
              filter: {
                kind: "allOf",
                children: [
                  {
                    kind: "scope",
                    category: "spell",
                    subcategory: { kind: "any" },
                  },
                  {
                    kind: "metadataPredicate",
                    predicate: { field: "families", op: "includes", value: "damage_burst" },
                  },
                ],
              },
              sort: { kind: "alphabetical" },
              limit: 50,
            },
          },
        ]),
      },
    ]);
  });

  it("uses relation edges for reference labels and grouped backlink pivots", () => {
    const document = buildEntityPageDocument(createRecord(), createRelations());
    const references = document.sections.find((section) => section.title === "References");
    const backlinks = document.sections.find((section) => section.title === "Referenced By");

    expect(references?.blocks).toEqual([
      {
        kind: "targetList",
        targets: [
          {
            kind: "record",
            label: "Spell Effect: Fireball",
            recordKey: "spells:fireball-effect",
            action: "open",
          },
        ],
      },
    ]);
    expect(backlinks?.blocks).toEqual([
      {
        kind: "targetList",
        targets: [
          {
            kind: "searchPivot",
            label: "Feat (1)",
            request: {
              mode: "browse",
              filter: {
                kind: "allOf",
                children: [
                  {
                    kind: "scope",
                    category: "feat",
                    subcategory: { kind: "any" },
                  },
                  {
                    kind: "linksTo",
                    target: "spell:test-fireball",
                  },
                ],
              },
              sort: { kind: "alphabetical" },
              limit: 50,
            },
          },
        ],
      },
    ]);
  });

  it("can emit preview-intent record references for embedded preview hosts", () => {
    const document = buildEntityPageDocument(createRecord(), createRelations(), {
      recordTargetAction: "preview",
    });
    const references = document.sections.find((section) => section.title === "References");

    expect(references?.targets).toEqual([
      {
        kind: "record",
        label: "Spell Effect: Fireball",
        recordKey: "spells:fireball-effect",
        action: "preview",
      },
    ]);
  });

  it("compiles resolved UUID prose links into readable inline record target spans", () => {
    const referenceText = "@UUID[Compendium.pf2e.spells.Item.fireball-effect]{Fireball Effect}";
    const relations = createRelations();
    relations.outgoing.edges[0] = {
      ...relations.outgoing.edges[0]!,
      referenceText,
    };
    relations.edges = [relations.outgoing.edges[0]!];
    const document = buildEntityPageDocument(
      createRecord({
        descriptionText: `Cast ${referenceText} before the flames spread.`,
      }),
      relations,
      { recordTargetAction: "preview" },
    );
    const description = document.sections.find((section) => section.id === "description");
    const block = description?.blocks[0];

    expect(block).toMatchObject({
      kind: "text",
      text: "Cast Spell Effect: Fireball before the flames spread.",
    });
    expect(block?.kind === "text" ? block.segments : undefined).toEqual([
      { text: "Cast " },
      {
        text: "Spell Effect: Fireball",
        target: {
          kind: "record",
          label: "Spell Effect: Fireball",
          recordKey: "spells:fireball-effect",
          action: "preview",
        },
      },
      { text: " before the flames spread." },
    ]);
  });

  it("renders unresolved UUID prose markup as a readable fallback without leaking raw markup", () => {
    const document = buildEntityPageDocument(
      createRecord({
        descriptionText: "Study @UUID[Compendium.pf2e.spells.Item.missing]{Missing Spell} carefully.",
      }),
    );
    const lines = renderEntityPageDocument(document);
    const renderedText = lines.map((line) => line.text).join("\n");

    expect(renderedText).toContain("Study Missing Spell carefully.");
    expect(renderedText).not.toContain("@UUID");
    expect(renderedText).not.toContain("Compendium.pf2e");
  });

  it("renders the AoN page action as a linked header line", () => {
    const lines = buildOntologyExplorerEntityDetailLines(createRecord({ name: "Alarm Ward", level: 1, rarity: "rare" }));
    const linkLine = lines.find(
      (line): line is OntologyTextLine & { href: string; plainTextFallback: string } =>
        typeof line.href === "string" && typeof line.plainTextFallback === "string",
    );

    expect(lines[0]?.text).toBe("Alarm Ward");
    expect(linkLine).toBeDefined();
    expect(linkLine?.text).toBe("AoN: Open in Archives of Nethys");
    expect(linkLine?.href).toContain("https://2e.aonprd.com/Search.aspx?display=short&type=eqs");
    expect(linkLine?.plainTextFallback).toContain("AoN: Open in Archives of Nethys: https://2e.aonprd.com");
    expect(linkLine?.href).toContain("include-traits=concentrate+fire+manipulate");
  });
});
