import { describe, expect, it } from "vitest";

import {
  buildEntityPageDocument,
  buildOntologyExplorerEntityDetailLines,
  renderEntityPageDocument,
} from "../../src/app/ontology/entity-page.js";
import type { OntologyExplorerEntityRecord } from "../../src/app/ontology/entity-record.js";
import type { OntologyTextLine } from "../../src/domain/ontology-types.js";

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
    ...overrides,
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
    expect(lines[2]).toMatchObject({ text: "Open in Archives of Nethys", indent: 2 });
    expect(lines[3]).toMatchObject({ text: "Traits: Concentrate, Fire, Manipulate", indent: 2 });
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
          ]),
        },
      ]),
    );
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
          { label: "Publication", value: "Pathfinder Player Core" },
          { label: "Pack", value: "spells" },
        ]),
      },
    ]);
  });

  it("renders the AoN page action as a linked header line", () => {
    const lines = buildOntologyExplorerEntityDetailLines(createRecord({ name: "Alarm Ward", level: 1, rarity: "rare" }));
    const linkLine = lines.find(
      (line): line is OntologyTextLine & { href: string; plainTextFallback: string } =>
        typeof line.href === "string" && typeof line.plainTextFallback === "string",
    );

    expect(lines[0]?.text).toBe("Alarm Ward");
    expect(linkLine).toBeDefined();
    expect(linkLine?.text).toBe("Open in Archives of Nethys");
    expect(linkLine?.href).toContain("https://2e.aonprd.com/Search.aspx?display=short&type=eqs");
    expect(linkLine?.plainTextFallback).toContain("Open in Archives of Nethys: https://2e.aonprd.com");
    expect(linkLine?.href).toContain("include-traits=concentrate+fire+manipulate");
  });
});
