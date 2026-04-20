import type { OntologyTextLine } from "../../types.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";

function renderNullable(value: string | number | null | undefined): string {
  if (value === null || value === undefined || value === "") {
    return "(none)";
  }
  return String(value);
}

function renderBoolean(value: boolean): string {
  return value ? "yes" : "no";
}

function renderList(values: string[]): string {
  return values.length > 0 ? values.join(", ") : "(none)";
}

function formatPriceCp(priceCp: number | null): string {
  if (priceCp === null) {
    return "(none)";
  }

  const gp = Math.floor(priceCp / 100);
  const sp = Math.floor((priceCp % 100) / 10);
  const cp = priceCp % 10;
  const parts = [
    gp > 0 ? `${gp} gp` : null,
    sp > 0 ? `${sp} sp` : null,
    cp > 0 || priceCp === 0 ? `${cp} cp` : null,
  ].filter((part): part is string => part !== null);
  return parts.join(", ");
}

function appendSection(lines: OntologyTextLine[], heading: string, values: Array<[string, string]>): void {
  const visibleValues = values.filter(([, value]) => value !== "(none)");
  if (visibleValues.length === 0) {
    return;
  }

  lines.push({ text: heading, tone: "section" });
  for (const [label, value] of visibleValues) {
    lines.push({ text: `${label}: ${value}`, indent: 2 });
  }
}

export function buildOntologyExplorerEntityDetailLines(
  record: OntologyExplorerEntityRecord,
  options: { includeHeader?: boolean } = {},
): OntologyTextLine[] {
  const lines: OntologyTextLine[] = [];

  if (options.includeHeader ?? true) {
    lines.push({ text: record.name, tone: "section" }, { text: record.recordKey, tone: "dim" });
  }

  appendSection(lines, "Identity", [
    ["Pack", record.packName],
    ["Type", record.type],
    ["Document", record.documentType],
    ["Category", record.category],
    ["Subcategory", renderNullable(record.subcategory)],
    ["Level", renderNullable(record.level)],
    ["Rarity", renderNullable(record.rarity)],
    ["Unique", renderBoolean(record.isUnique)],
    ["Source category", record.sourceCategory],
    ["Publication", renderNullable(record.publicationTitle)],
    ["Remaster", renderBoolean(record.publicationRemaster)],
  ]);

  appendSection(lines, "Retrieval", [
    ["Traits", renderList(record.traits)],
    ["Derived tags", renderList(record.derivedTags)],
    ["Families", renderList(record.families)],
  ]);

  appendSection(lines, "Creature/Actor", [
    ["Size", renderNullable(record.size)],
    ["Languages", renderList(record.languages)],
    ["Speeds", renderList(record.speedTypes)],
    ["Senses", renderList(record.senses)],
    ["Immunities", renderList(record.immunities)],
    ["Resistances", renderList(record.resistances)],
    ["Weaknesses", renderList(record.weaknesses)],
  ]);

  appendSection(lines, "Item", [
    ["Item category", renderNullable(record.itemCategory)],
    ["Base item", renderNullable(record.baseItem)],
    ["Price", formatPriceCp(record.priceCp)],
    ["Usage", renderNullable(record.usage)],
    ["Hands", renderNullable(record.hands)],
    ["Damage types", renderList(record.damageTypes)],
    ["Weapon group", renderNullable(record.weaponGroup)],
    ["Armor group", renderNullable(record.armorGroup)],
  ]);

  appendSection(lines, "Spell", [
    ["Traditions", renderList(record.traditions)],
    ["Spell kinds", renderList(record.spellKinds)],
    ["Range", renderNullable(record.rangeText)],
    ["Area type", renderNullable(record.areaType)],
    ["Area value", renderNullable(record.areaValue)],
    ["Save", renderNullable(record.saveType)],
    ["Duration", renderNullable(record.durationText)],
    ["Target", renderNullable(record.targetText)],
    ["Sustained", renderBoolean(record.sustained)],
    ["Basic save", renderBoolean(record.basicSave)],
  ]);

  appendSection(lines, "Hazard", [
    ["Complex", renderBoolean(record.isComplex)],
    ["Disable", renderNullable(record.disableText)],
    ["Disable skills", renderList(record.disableSkills)],
  ]);

  if (record.blurbText) {
    lines.push({ text: "Blurb", tone: "section" });
    lines.push({ text: record.blurbText, indent: 2 });
  }

  if (record.descriptionText) {
    lines.push({ text: "Description", tone: "section" });
    lines.push({ text: record.descriptionText, indent: 2 });
  }

  return lines;
}

export function buildOntologyExplorerEntitySummary(record: OntologyExplorerEntityRecord): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level}`;
}
