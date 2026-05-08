import type { OntologyTextLine } from "../../domain/ontology-types.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { MetadataSetField } from "../../domain/metadata-field-types.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
import type { RecordKey } from "../../domain/record-types.js";
import { SEARCH_REQUEST_VOCABULARY } from "../../domain/search-request-types.js";
import { buildAllOfFilter, buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import { buildAonSearchLink } from "../external-links/aon-search.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";
import {
  projectEntityPageFacts,
  type EntityPageFact,
  type EntityPageFactInventory,
  type EntityPageRecipeKind,
  type ProjectedEntityPageFact,
} from "./entity-page-facts.js";

export type { EntityPageFact } from "./entity-page-facts.js";

export type EntityPageTarget =
  | { kind: "record"; label: string; recordKey: RecordKey; action: "preview" | "open" }
  | { kind: "searchPivot"; label: string; request: SearchRequest }
  | { kind: "external"; label: string; href: string; plainTextFallback?: string };

export type EntityPageRecordTargetAction = Extract<EntityPageTarget, { kind: "record" }>["action"];

export type EntityPageDocumentBuildOptions = {
  recordTargetAction?: EntityPageRecordTargetAction;
};

export type EntityPageTextSegment = {
  text: string;
  target?: EntityPageTarget;
  tone?: OntologyTextLine["tone"];
};

export type EntityPageBlock =
  | { kind: "factList"; facts: EntityPageFact[] }
  | { kind: "text"; text: string; segments?: EntityPageTextSegment[] }
  | { kind: "targetList"; targets: EntityPageTarget[] };

export type EntityPageSection = {
  id: string;
  kind:
    | "identity"
    | "summary"
    | "description"
    | "defense"
    | "movement"
    | "offense"
    | "routine"
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
  traitTargets?: EntityPageTarget[];
  sections: EntityPageSection[];
};

type PreparedEntityPageInput = {
  recipe: EntityPageRecipeKind;
  identityLine: string;
  traits: string[];
  aonLink?: Extract<EntityPageTarget, { kind: "external" }>;
  blurb?: EntityPageTextContent;
  description?: EntityPageTextContent;
  facts: EntityPageFactInventory;
  traitTargets: EntityPageTarget[];
  classificationTargets: EntityPageTarget[];
  references: EntityPageTarget[];
  referencedBy: EntityPageTarget[];
};

type EntityPageTextContent = {
  text: string;
  segments?: EntityPageTextSegment[];
};

type EntityPageRecipeBuildContext = {
  input: PreparedEntityPageInput;
  consumedFactKeys: Set<string>;
  seenFacts: Set<string>;
  push: (section: EntityPageSection | null) => void;
};

const UUID_REFERENCE_MARKUP_PATTERN = /@UUID\[[^\]]+\](?:\{([^}]+)\})?/g;

function humanize(value: string | null | undefined): string {
  if (!value) {
    return "";
  }
  return formatOntologySearchVocabularyLabel(value);
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

function buildBrowseRequest(filter: SearchRequest["filter"]): SearchRequest {
  return {
    mode: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
    filter,
    sort: { kind: SEARCH_REQUEST_VOCABULARY.SORT_KIND.ALPHABETICAL },
    limit: 50,
  };
}

function buildPackTarget(record: OntologyExplorerEntityRecord): EntityPageTarget | null {
  const packName = record.packName.trim();
  if (!packName) {
    return null;
  }
  return {
    kind: "searchPivot",
    label: `Pack: ${packName}`,
    request: buildBrowseRequest({
      kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK,
      value: packName,
    }),
  };
}

function buildCategoryTarget(record: OntologyExplorerEntityRecord): EntityPageTarget {
  return {
    kind: "searchPivot",
    label: `Category: ${humanize(record.category)}`,
    request: buildBrowseRequest(buildScopeFilter(record.category)),
  };
}

function buildSubcategoryTarget(record: OntologyExplorerEntityRecord): EntityPageTarget | null {
  if (!record.subcategory) {
    return null;
  }
  return {
    kind: "searchPivot",
    label: `Subcategory: ${humanize(record.subcategory)}`,
    request: buildBrowseRequest(buildScopeFilter(record.category, record.subcategory)),
  };
}

type EntityPageMetadataPivotSpec = {
  field: MetadataSetField;
  label: string;
  placement: "header" | "classification";
  getValues: (record: OntologyExplorerEntityRecord) => string[];
};

const ENTITY_PAGE_METADATA_PIVOTS: readonly EntityPageMetadataPivotSpec[] = [
  {
    field: "traits",
    label: "Trait",
    placement: "header",
    getValues: (record) => record.traits,
  },
  {
    field: "derivedTags",
    label: "Derived Tags",
    placement: "classification",
    getValues: (record) => record.derivedTags,
  },
  {
    field: "families",
    label: "Families",
    placement: "classification",
    getValues: (record) => record.families,
  },
];

function buildMetadataPivotTarget(
  record: OntologyExplorerEntityRecord,
  spec: EntityPageMetadataPivotSpec,
  value: string,
): EntityPageTarget {
  return {
    kind: "searchPivot" as const,
    label: `${spec.label}: ${humanize(value)}`,
    request: buildBrowseRequest(
      buildAllOfFilter([
        buildScopeFilter(record.category),
        {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
          predicate: { field: spec.field, op: "includes", value },
        },
      ]),
    ),
  };
}

function buildMetadataPivotTargets(
  record: OntologyExplorerEntityRecord,
  placement: EntityPageMetadataPivotSpec["placement"],
): EntityPageTarget[] {
  return ENTITY_PAGE_METADATA_PIVOTS.filter((spec) => spec.placement === placement).flatMap((spec) =>
    spec.getValues(record).map((value) => buildMetadataPivotTarget(record, spec, value)),
  );
}

function buildClassificationTargets(record: OntologyExplorerEntityRecord): EntityPageTarget[] {
  return [
    buildPackTarget(record),
    buildCategoryTarget(record),
    buildSubcategoryTarget(record),
    ...buildMetadataPivotTargets(record, "classification"),
  ].filter((target): target is EntityPageTarget => Boolean(target));
}

function buildReferenceTargetData(
  relations: PageRelationsResult | undefined,
  recordTargetAction: EntityPageRecordTargetAction,
): {
  targets: EntityPageTarget[];
  targetsByReferenceText: Map<string, EntityPageTarget>;
} {
  if (!relations) {
    return {
      targets: [],
      targetsByReferenceText: new Map(),
    };
  }

  const recordsByKey = new Map(relations.outgoing.records.map((record) => [record.recordKey, record]));
  const seen = new Set<string>();
  const targets: EntityPageTarget[] = [];
  const targetsByReferenceText = new Map<string, EntityPageTarget>();

  for (const edge of relations.outgoing.edges) {
    const record = recordsByKey.get(edge.toRecordKey);
    if (!record) {
      continue;
    }

    const label = edge.displayText?.trim() || record.name || edge.referenceText;
    const target: EntityPageTarget = {
      kind: "record",
      label,
      recordKey: record.recordKey,
      action: recordTargetAction,
    };
    if (edge.referenceText.trim()) {
      targetsByReferenceText.set(edge.referenceText, target);
    }

    const dedupeKey = `${record.recordKey}|${label}`;
    if (seen.has(dedupeKey)) {
      continue;
    }
    seen.add(dedupeKey);
    targets.push(target);
  }

  return {
    targets,
    targetsByReferenceText,
  };
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

function compileProseReferenceSegments(
  text: string | null | undefined,
  targetsByReferenceText: ReadonlyMap<string, EntityPageTarget>,
): EntityPageTextContent | undefined {
  if (!text) {
    return undefined;
  }

  const segments: EntityPageTextSegment[] = [];
  let lastIndex = 0;
  let replacedMarkup = false;

  for (const match of text.matchAll(UUID_REFERENCE_MARKUP_PATTERN)) {
    const referenceText = match[0];
    const startIndex = match.index;
    if (startIndex > lastIndex) {
      segments.push({ text: text.slice(lastIndex, startIndex) });
    }

    const target = targetsByReferenceText.get(referenceText);
    if (target) {
      segments.push({ text: target.label, target });
    } else {
      const fallbackLabel = match[1]?.trim() || "unresolved reference";
      segments.push({ text: fallbackLabel, tone: "dim" });
    }

    replacedMarkup = true;
    lastIndex = startIndex + referenceText.length;
  }

  if (!replacedMarkup) {
    return { text };
  }

  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex) });
  }

  return {
    text: segments.map((segment) => segment.text).join(""),
    segments,
  };
}

function buildEntityPageInput(
  record: OntologyExplorerEntityRecord,
  relations?: PageRelationsResult,
  options: EntityPageDocumentBuildOptions = {},
): PreparedEntityPageInput {
  const aonLink = buildAonSearchLink(record);
  const recordTargetAction = options.recordTargetAction ?? "open";
  const referenceTargetData = buildReferenceTargetData(relations, recordTargetAction);
  const projectedFacts = projectEntityPageFacts(record);

  return {
    recipe: projectedFacts.recipe,
    identityLine: buildIdentityLine(record),
    traits: record.traits,
    aonLink: aonLink
      ? {
          kind: "external",
          label: `AoN: ${aonLink.label}`,
          href: aonLink.url,
          plainTextFallback: `AoN: ${aonLink.plainTextFallback}`,
        }
      : undefined,
    blurb: compileProseReferenceSegments(record.blurbText, referenceTargetData.targetsByReferenceText),
    description: compileProseReferenceSegments(record.descriptionText, referenceTargetData.targetsByReferenceText),
    facts: projectedFacts.inventory,
    traitTargets: buildMetadataPivotTargets(record, "header"),
    classificationTargets: buildClassificationTargets(record),
    references: referenceTargetData.targets,
    referencedBy: buildBacklinkTargets(relations),
  };
}

function dedupeFacts<T extends EntityPageFact>(facts: T[], seenValues: Set<string>): T[] {
  const deduped: T[] = [];
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

function stripFactKeys(facts: readonly ProjectedEntityPageFact[]): EntityPageFact[] {
  return facts.map(({ label, value }) => ({ label, value }));
}

function createFactSection(
  id: string,
  kind: EntityPageSection["kind"],
  title: string,
  facts: ProjectedEntityPageFact[],
  consumedFactKeys: Set<string>,
  seenFacts: Set<string>,
): EntityPageSection | null {
  for (const fact of facts) {
    consumedFactKeys.add(fact.key);
  }
  const dedupedFacts = dedupeFacts(facts, seenFacts);
  if (dedupedFacts.length === 0) {
    return null;
  }

  return {
    id,
    kind,
    title,
    blocks: [{ kind: "factList", facts: stripFactKeys(dedupedFacts) }],
    targets: [],
  };
}

function createSummarySection(
  blurb: EntityPageTextContent | undefined,
  facts: ProjectedEntityPageFact[],
  consumedFactKeys: Set<string>,
  seenFacts: Set<string>,
): EntityPageSection | null {
  for (const fact of facts) {
    consumedFactKeys.add(fact.key);
  }
  const dedupedFacts = dedupeFacts(facts, seenFacts);
  if (!blurb && dedupedFacts.length === 0) {
    return null;
  }

  return {
    id: "summary",
    kind: "summary",
    title: "Summary",
    blocks: [
      ...(blurb ? [{ kind: "text" as const, text: blurb.text, segments: blurb.segments }] : []),
      ...(dedupedFacts.length > 0 ? [{ kind: "factList" as const, facts: stripFactKeys(dedupedFacts) }] : []),
    ],
    targets: [],
  };
}

function createTextSection(
  id: string,
  kind: EntityPageSection["kind"],
  title: string,
  text: EntityPageTextContent | undefined,
): EntityPageSection | null {
  if (!text) {
    return null;
  }

  return {
    id,
    kind,
    title,
    blocks: [{ kind: "text", text: text.text, segments: text.segments }],
    targets: [],
  };
}

function createTargetSection(
  id: string,
  kind: EntityPageSection["kind"],
  title: string,
  targets: EntityPageTarget[],
): EntityPageSection | null {
  if (targets.length === 0) {
    return null;
  }

  return {
    id,
    kind,
    title,
    blocks: [{ kind: "targetList", targets }],
    targets,
  };
}

function createDetailsSection({
  input,
  consumedFactKeys,
  seenFacts,
}: Pick<EntityPageRecipeBuildContext, "input" | "consumedFactKeys" | "seenFacts">): EntityPageSection | null {
  const remainingFacts = input.facts.allFacts.filter((fact) => !consumedFactKeys.has(fact.key));
  return createFactSection("details", "details", "Details", remainingFacts, consumedFactKeys, seenFacts);
}

function buildSpellRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.spellSummary, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

function buildCreatureRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.creatureSummary, consumedFactKeys, seenFacts));
  push(createFactSection("defense", "defense", "Defense", input.facts.creatureDefense, consumedFactKeys, seenFacts));
  push(createFactSection("movement", "movement", "Movement", input.facts.creatureMovement, consumedFactKeys, seenFacts));
  push(createFactSection("offense", "offense", "Offense", input.facts.creatureOffense, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

function buildEquipmentRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.equipmentSummary, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

function buildFeatActionRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.featActionSummary, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

function buildHazardRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.hazardSummary, consumedFactKeys, seenFacts));
  push(createFactSection("defense", "defense", "Defense", input.facts.hazardDefense, consumedFactKeys, seenFacts));
  push(createFactSection("routine", "routine", "Routine", input.facts.hazardRoutine, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

function buildFallbackRecipeSections(context: EntityPageRecipeBuildContext): void {
  const { input, consumedFactKeys, seenFacts, push } = context;
  push(createSummarySection(input.blurb, input.facts.fallbackSummary, consumedFactKeys, seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createDetailsSection(context));
}

const ENTITY_PAGE_RECIPE_SECTION_BUILDERS: Record<
  EntityPageRecipeKind,
  (context: EntityPageRecipeBuildContext) => void
> = {
  spell: buildSpellRecipeSections,
  creature: buildCreatureRecipeSections,
  equipment: buildEquipmentRecipeSections,
  featAction: buildFeatActionRecipeSections,
  hazard: buildHazardRecipeSections,
  fallback: buildFallbackRecipeSections,
};

function buildRecipeSections(input: PreparedEntityPageInput, seenFacts: Set<string>): EntityPageSection[] {
  const sections: EntityPageSection[] = [];
  const consumedFactKeys = new Set<string>();
  const push = (section: EntityPageSection | null) => {
    if (section) {
      sections.push(section);
    }
  };
  const context: EntityPageRecipeBuildContext = {
    input,
    consumedFactKeys,
    seenFacts,
    push,
  };
  const buildRecipeSectionsForRecord = ENTITY_PAGE_RECIPE_SECTION_BUILDERS[input.recipe];
  buildRecipeSectionsForRecord(context);

  push(createTargetSection("references", "references", "References", input.references));
  push(createTargetSection("backlinks", "backlinks", "Referenced By", input.referencedBy));
  push(createTargetSection("classification", "classification", "Classification", input.classificationTargets));

  return sections;
}

export function buildEntityPageDocument(
  record: OntologyExplorerEntityRecord,
  relations?: PageRelationsResult,
  options: EntityPageDocumentBuildOptions = {},
): EntityPageDocument {
  const input = buildEntityPageInput(record, relations, options);
  const seenFacts = new Set<string>();

  return {
    recordKey: record.recordKey,
    title: record.name,
    identityLine: input.identityLine,
    aonLink: input.aonLink,
    traits: input.traits,
    traitTargets: input.traitTargets,
    sections: buildRecipeSections(input, seenFacts),
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
