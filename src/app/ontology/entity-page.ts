import type { OntologyTextLine } from "../../domain/ontology-types.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
import type { RecordKey } from "../../domain/record-types.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { buildAonSearchLink } from "../external-links/aon-search.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";

export type EntityPageTarget =
  | { kind: "record"; label: string; recordKey: RecordKey; action: "preview" | "open" }
  | { kind: "searchPivot"; label: string; request: SearchRequest }
  | { kind: "external"; label: string; href: string; plainTextFallback?: string };

export type EntityPageFact = {
  label: string;
  value: string;
};

export type EntityPageBlock =
  | { kind: "factList"; facts: EntityPageFact[] }
  | { kind: "text"; text: string }
  | { kind: "targetList"; targets: EntityPageTarget[] };

export type EntityPageSection = {
  id: string;
  kind:
    | "identity"
    | "summary"
    | "description"
    | "details"
    | "references"
    | "backlinks"
    | "classification";
  title?: string;
  blocks: EntityPageBlock[];
  targets: EntityPageTarget[];
};

export type EntityPageDocument = {
  recordKey: RecordKey;
  title: string;
  identityLine?: string;
  aonLink?: Extract<EntityPageTarget, { kind: "external" }>;
  traits: string[];
  sections: EntityPageSection[];
};

type PreparedEntityPageInput = {
  record: OntologyExplorerEntityRecord;
  identityLine: string;
  traits: string[];
  aonLink?: Extract<EntityPageTarget, { kind: "external" }>;
  blurb?: string;
  description?: string;
  summaryFacts: EntityPageFact[];
  detailFacts: EntityPageFact[];
  classificationFacts: EntityPageFact[];
  references: EntityPageTarget[];
  referencedBy: EntityPageTarget[];
};

function humanize(value: string | null | undefined): string {
  if (!value) {
    return "";
  }
  return formatOntologySearchVocabularyLabel(value);
}

function asFact(label: string, value: string | null | undefined): EntityPageFact | null {
  const normalized = value?.trim();
  return normalized ? { label, value: normalized } : null;
}

function formatActionCost(value: number | null): string | null {
  if (value == null || value < 1 || value > 3) {
    return null;
  }
  return value === 1 ? "1 action" : `${value} actions`;
}

function formatPriceCp(priceCp: number | null): string | null {
  if (priceCp == null) {
    return null;
  }
  const gp = Math.floor(priceCp / 100);
  const sp = Math.floor((priceCp % 100) / 10);
  const cp = priceCp % 10;
  const parts = [
    gp > 0 ? `${gp} gp` : null,
    sp > 0 ? `${sp} sp` : null,
    cp > 0 || priceCp === 0 ? `${cp} cp` : null,
  ].filter((part): part is string => Boolean(part));
  return parts.join(", ");
}

function formatBoolean(value: boolean, trueLabel = "Yes"): string | null {
  return value ? trueLabel : null;
}

function formatList(values: string[]): string | null {
  return values.length > 0 ? values.map(humanize).join(", ") : null;
}

function formatArea(record: OntologyExplorerEntityRecord): string | null {
  if (record.areaType && record.areaValue != null) {
    return `${record.areaValue} ${humanize(record.areaType)}`;
  }
  return humanize(record.areaType) || (record.areaValue != null ? String(record.areaValue) : null);
}

function formatSave(record: OntologyExplorerEntityRecord): string | null {
  const save = humanize(record.saveType);
  if (!save) {
    return null;
  }
  return record.basicSave ? `basic ${save}` : save;
}

function buildIdentityLine(record: OntologyExplorerEntityRecord): string {
  const typeLabel =
    record.category === "spell"
      ? "Spell"
      : record.category === "equipment"
        ? "Item"
        : humanize(record.type) || humanize(record.category);
  const levelLabel =
    record.level == null ? null : record.category === "spell" ? `Rank ${record.level}` : `Level ${record.level}`;
  return [typeLabel, levelLabel, humanize(record.rarity), record.publicationTitle].filter(Boolean).join(" | ");
}

function buildSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  if (record.category === "spell") {
    return [
      asFact("Traditions", formatList(record.traditions)),
      asFact("Cast", formatActionCost(record.actionCost)),
      asFact("Range", record.rangeText),
      asFact("Area", formatArea(record)),
      asFact("Save", formatSave(record)),
      asFact("Duration", record.durationText),
      asFact("Targets", record.targetText),
    ].filter((fact): fact is EntityPageFact => Boolean(fact));
  }

  if (record.category === "creature") {
    return [
      asFact("Size", humanize(record.size)),
      asFact("Languages", formatList(record.languages)),
      asFact("Senses", formatList(record.senses)),
      asFact("Immunities", formatList(record.immunities)),
      asFact("Resistances", formatList(record.resistances)),
      asFact("Weaknesses", formatList(record.weaknesses)),
    ].filter((fact): fact is EntityPageFact => Boolean(fact));
  }

  if (record.category === "equipment") {
    return [
      asFact("Price", formatPriceCp(record.priceCp)),
      asFact("Usage", humanize(record.usage)),
      asFact("Hands", record.hands == null ? null : String(record.hands)),
      asFact("Base Item", humanize(record.baseItem)),
      asFact("Category", humanize(record.itemCategory)),
      asFact("Group", humanize(record.weaponGroup ?? record.armorGroup)),
      asFact("Damage", formatList(record.damageTypes)),
    ].filter((fact): fact is EntityPageFact => Boolean(fact));
  }

  return [
    asFact("Action Cost", formatActionCost(record.actionCost)),
    asFact("Disable", record.disableText),
    asFact("Disable Skills", formatList(record.disableSkills)),
    asFact("Complexity", formatBoolean(record.isComplex, "Complex")),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildDetailFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Spell Kinds", formatList(record.spellKinds)),
    asFact("Source Category", humanize(record.sourceCategory)),
    asFact("Document Type", record.documentType),
    asFact("Publication", record.publicationTitle),
    asFact("Pack", record.packName),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildClassificationFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Pack", record.packName),
    asFact("Derived Tags", formatList(record.derivedTags)),
    asFact("Families", formatList(record.families)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildReferenceTargets(relations?: PageRelationsResult): EntityPageTarget[] {
  if (!relations) {
    return [];
  }

  return relations.outgoing.records.map((record) => ({
    kind: "record" as const,
    label: record.name,
    recordKey: record.recordKey,
    action: "open" as const,
  }));
}

function buildBacklinkTargets(relations?: PageRelationsResult): EntityPageTarget[] {
  if (!relations) {
    return [];
  }

  return relations.incomingGroups.map((group) => ({
    kind: "searchPivot" as const,
    label: `${humanize(group.subcategory ?? group.category)} (${group.count})`,
    request: group.request,
  }));
}

function buildEntityPageInput(
  record: OntologyExplorerEntityRecord,
  relations?: PageRelationsResult,
): PreparedEntityPageInput {
  const aonLink = buildAonSearchLink(record);

  return {
    record,
    identityLine: buildIdentityLine(record),
    traits: record.traits,
    aonLink: aonLink
      ? {
          kind: "external",
          label: aonLink.label,
          href: aonLink.url,
          plainTextFallback: aonLink.plainTextFallback,
        }
      : undefined,
    blurb: record.blurbText ?? undefined,
    description: record.descriptionText ?? undefined,
    summaryFacts: buildSummaryFacts(record),
    detailFacts: buildDetailFacts(record),
    classificationFacts: buildClassificationFacts(record),
    references: buildReferenceTargets(relations),
    referencedBy: buildBacklinkTargets(relations),
  };
}

function dedupeFacts(
  facts: EntityPageFact[],
  seenValues: Set<string>,
): EntityPageFact[] {
  const deduped: EntityPageFact[] = [];
  for (const fact of facts) {
    const key = `${fact.label}:${fact.value}`.toLowerCase();
    if (seenValues.has(key)) {
      continue;
    }
    seenValues.add(key);
    deduped.push(fact);
  }
  return deduped;
}

export function buildEntityPageDocument(
  record: OntologyExplorerEntityRecord,
  relations?: PageRelationsResult,
): EntityPageDocument {
  const input = buildEntityPageInput(record, relations);
  const seenFacts = new Set<string>();
  const summaryFacts = dedupeFacts(input.summaryFacts, seenFacts);
  const detailFacts = dedupeFacts(input.detailFacts, seenFacts);
  const classificationFacts = dedupeFacts(input.classificationFacts, seenFacts);
  const sections: EntityPageSection[] = [];

  if (input.blurb || summaryFacts.length > 0) {
    sections.push({
      id: "summary",
      kind: "summary",
      title: "Summary",
      blocks: [
        ...(input.blurb ? [{ kind: "text" as const, text: input.blurb }] : []),
        ...(summaryFacts.length > 0 ? [{ kind: "factList" as const, facts: summaryFacts }] : []),
      ],
      targets: [],
    });
  }

  if (input.description) {
    sections.push({
      id: "description",
      kind: "description",
      title: "Description",
      blocks: [{ kind: "text", text: input.description }],
      targets: [],
    });
  }

  if (detailFacts.length > 0) {
    sections.push({
      id: "details",
      kind: "details",
      title: "Details",
      blocks: [{ kind: "factList", facts: detailFacts }],
      targets: [],
    });
  }

  if (input.references.length > 0) {
    sections.push({
      id: "references",
      kind: "references",
      title: "References",
      blocks: [{ kind: "targetList", targets: input.references }],
      targets: input.references,
    });
  }

  if (input.referencedBy.length > 0) {
    sections.push({
      id: "backlinks",
      kind: "backlinks",
      title: "Referenced By",
      blocks: [{ kind: "targetList", targets: input.referencedBy }],
      targets: input.referencedBy,
    });
  }

  if (classificationFacts.length > 0) {
    sections.push({
      id: "classification",
      kind: "classification",
      title: "Classification",
      blocks: [{ kind: "factList", facts: classificationFacts }],
      targets: [],
    });
  }

  return {
    recordKey: record.recordKey,
    title: record.name,
    identityLine: input.identityLine,
    aonLink: input.aonLink,
    traits: input.traits,
    sections,
  };
}

export function renderEntityPageDocument(
  document: EntityPageDocument,
  options: { includeHeader?: boolean } = {},
): OntologyTextLine[] {
  const lines: OntologyTextLine[] = [];
  const includeHeader = options.includeHeader ?? true;

  if (includeHeader) {
    lines.push({ text: document.title, tone: "section" });
    if (document.identityLine) {
      lines.push({ text: document.identityLine, indent: 2 });
    }
  }

  if (document.aonLink) {
    lines.push({
      text: document.aonLink.label,
      indent: 2,
      href: document.aonLink.href,
      plainTextFallback: document.aonLink.plainTextFallback,
    });
  }
  if (document.traits.length > 0) {
    lines.push({ text: `Traits: ${document.traits.map(humanize).join(", ")}`, indent: 2 });
  }

  for (const section of document.sections) {
    if (section.title) {
      lines.push({ text: section.title, tone: "section" });
    }
    for (const block of section.blocks) {
      if (block.kind === "text") {
        lines.push({ text: block.text, indent: 2 });
        continue;
      }
      if (block.kind === "factList") {
        for (const fact of block.facts) {
          lines.push({ text: `${fact.label}: ${fact.value}`, indent: 2 });
        }
        continue;
      }
      for (const target of block.targets) {
        if (target.kind === "external") {
          lines.push({
            text: target.label,
            indent: 2,
            href: target.href,
            plainTextFallback: target.plainTextFallback,
          });
          continue;
        }
        lines.push({ text: target.label, indent: 2 });
      }
    }
  }

  return lines;
}

export function buildOntologyExplorerEntityDetailLines(
  record: OntologyExplorerEntityRecord,
  options: { includeHeader?: boolean } = {},
): OntologyTextLine[] {
  return renderEntityPageDocument(buildEntityPageDocument(record), options);
}

export function buildOntologyExplorerEntitySummary(record: OntologyExplorerEntityRecord): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level}`;
}
