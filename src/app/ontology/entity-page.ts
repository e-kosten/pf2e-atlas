import type { OntologyTextLine } from "../../domain/ontology-types.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { MetadataSetField } from "../../domain/metadata-field-types.js";
import type { PageRelationsResult } from "../../domain/page-relations-types.js";
import type { RecordKey } from "../../domain/record-types.js";
import { buildAllOfFilter, buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import { buildAonSearchLink } from "../external-links/aon-search.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";

export type EntityPageTarget =
  | { kind: "record"; label: string; recordKey: RecordKey; action: "preview" | "open" }
  | { kind: "searchPivot"; label: string; request: SearchRequest }
  | { kind: "external"; label: string; href: string; plainTextFallback?: string };

export type EntityPageRecordTargetAction = Extract<EntityPageTarget, { kind: "record" }>["action"];

export type EntityPageDocumentBuildOptions = {
  recordTargetAction?: EntityPageRecordTargetAction;
};

export type EntityPageFact = {
  label: string;
  value: string;
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
  record: OntologyExplorerEntityRecord;
  identityLine: string;
  traits: string[];
  aonLink?: Extract<EntityPageTarget, { kind: "external" }>;
  blurb?: EntityPageTextContent;
  description?: EntityPageTextContent;
  traitTargets: EntityPageTarget[];
  classificationTargets: EntityPageTarget[];
  references: EntityPageTarget[];
  referencedBy: EntityPageTarget[];
};

type EntityPageTextContent = {
  text: string;
  segments?: EntityPageTextSegment[];
};

type EntityPageRecipeKind = "spell" | "creature" | "equipment" | "featAction" | "hazard" | "fallback";

type EntityPageRecipeBuildContext = {
  input: PreparedEntityPageInput;
  detailFacts: EntityPageFact[];
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

function formatModifier(value: number | null | undefined): string | null {
  if (value == null) {
    return null;
  }
  return value >= 0 ? `+${value}` : String(value);
}

function formatNumber(value: number | null | undefined): string | null {
  return value == null ? null : String(value);
}

function formatFeet(value: number | null | undefined): string | null {
  return value == null ? null : `${value} feet`;
}

function formatBulkValue(value: number | null): string | null {
  if (value == null) {
    return null;
  }
  if (value === 0) {
    return "-";
  }
  if (value === 0.1) {
    return "L";
  }
  return Number.isInteger(value) ? String(value) : String(value);
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

function formatSpeedTypes(speedTypes: string[]): string | null {
  return speedTypes.length > 0 ? speedTypes.map(humanize).join(", ") : null;
}

function getMetricNumber(metrics: Record<string, unknown>, key: string): number | null {
  const value = metrics[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getRawObject(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : null;
}

function getRawPath(value: unknown, path: readonly string[]): unknown {
  let current = value;
  for (const segment of path) {
    if (!current || typeof current !== "object" || Array.isArray(current)) {
      return undefined;
    }
    current = (current as Record<string, unknown>)[segment];
  }
  return current;
}

function getRawNumber(value: unknown, path: readonly string[]): number | null {
  const rawValue = getRawPath(value, path);
  return typeof rawValue === "number" && Number.isFinite(rawValue) ? rawValue : null;
}

function formatRawValue(value: unknown): string | null {
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  if (Array.isArray(value)) {
    const parts = value
      .map((entry) =>
        typeof entry === "object" && entry !== null && !Array.isArray(entry)
          ? formatRawValue(
              (entry as Record<string, unknown>).value ??
                (entry as Record<string, unknown>).label ??
                (entry as Record<string, unknown>).name,
            )
          : formatRawValue(entry),
      )
      .filter((entry): entry is string => Boolean(entry));
    return parts.length > 0 ? parts.join(", ") : null;
  }
  if (value && typeof value === "object") {
    const objectValue = value as Record<string, unknown>;
    return formatRawValue(objectValue.value ?? objectValue.label ?? objectValue.name);
  }
  return null;
}

function formatRawField(record: OntologyExplorerEntityRecord, paths: readonly (readonly string[])[]): string | null {
  for (const path of paths) {
    const formatted = formatRawValue(getRawPath(record.raw ?? {}, path));
    if (formatted) {
      return formatted;
    }
  }
  return null;
}

function formatCreatureAbilities(record: OntologyExplorerEntityRecord): string | null {
  const abilities = [
    ["str", "Str"],
    ["dex", "Dex"],
    ["con", "Con"],
    ["int", "Int"],
    ["wis", "Wis"],
    ["cha", "Cha"],
  ] as const;
  const parts = abilities
    .map(([key, label]) => {
      const modifier = formatModifier(getMetricNumber(record.actorMetrics ?? {}, `ability.${key}.mod`));
      return modifier ? `${label} ${modifier}` : null;
    })
    .filter((part): part is string => Boolean(part));
  return parts.length > 0 ? parts.join(", ") : null;
}

function formatCreatureSaves(record: OntologyExplorerEntityRecord): string | null {
  const saves = [
    ["fort", "Fort"],
    ["ref", "Ref"],
    ["will", "Will"],
  ] as const;
  const parts = saves
    .map(([key, label]) => {
      const modifier = formatModifier(getMetricNumber(record.actorMetrics ?? {}, `save.${key}.mod`));
      return modifier ? `${label} ${modifier}` : null;
    })
    .filter((part): part is string => Boolean(part));
  return parts.length > 0 ? parts.join(", ") : null;
}

function formatCreatureSkills(record: OntologyExplorerEntityRecord): string | null {
  const parts = Object.entries(record.actorMetrics ?? {})
    .filter(([key, value]) => key.startsWith("skill.") && key.endsWith(".mod") && typeof value === "number")
    .map(([key, value]) => {
      const skill = key.slice("skill.".length, -".mod".length);
      return `${humanize(skill)} ${formatModifier(value as number)}`;
    })
    .sort((left, right) => left.localeCompare(right));
  return parts.length > 0 ? parts.join(", ") : null;
}

function formatSpeed(record: OntologyExplorerEntityRecord): string | null {
  const metricSpeeds = Object.entries(record.actorMetrics ?? {})
    .filter(([key, value]) => key.startsWith("speed.") && key.endsWith(".value") && typeof value === "number")
    .map(([key, value]) => {
      const speedType = key.slice("speed.".length, -".value".length);
      const speed = formatFeet(value as number);
      return speed ? `${humanize(speedType)} ${speed}` : null;
    })
    .filter((part): part is string => Boolean(part))
    .sort((left, right) => left.localeCompare(right));
  return metricSpeeds.length > 0 ? metricSpeeds.join(", ") : formatSpeedTypes(record.speedTypes);
}

function formatHazardStealth(record: OntologyExplorerEntityRecord): string | null {
  const stealthMod = formatModifier(getMetricNumber(record.actorMetrics ?? {}, "stealth.mod"));
  const stealthDc = formatNumber(getMetricNumber(record.actorMetrics ?? {}, "stealth.dc"));
  if (stealthMod && stealthDc) {
    return `${stealthMod} (DC ${stealthDc})`;
  }
  if (stealthDc) {
    return `DC ${stealthDc}`;
  }
  return stealthMod;
}

function formatWeaponDamageDice(record: OntologyExplorerEntityRecord): string | null {
  const dice = getMetricNumber(record.itemMetrics ?? {}, "weapon.damage_dice");
  const dieFaces = getMetricNumber(record.itemMetrics ?? {}, "weapon.damage_die_faces");
  if (dice == null && dieFaces == null) {
    return null;
  }
  if (dice != null && dieFaces != null) {
    return `${dice}d${dieFaces}`;
  }
  if (dieFaces != null) {
    return `d${dieFaces}`;
  }
  return String(dice);
}

function buildEquipmentItemMetricFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  const itemMetrics = record.itemMetrics ?? {};
  return [
    asFact("Range Increment", formatFeet(getMetricNumber(itemMetrics, "weapon.range_increment"))),
    asFact("Reload", formatNumber(getMetricNumber(itemMetrics, "weapon.reload"))),
    asFact("Weapon Damage Dice", formatWeaponDamageDice(record)),
    asFact("Armor AC Bonus", formatModifier(getMetricNumber(itemMetrics, "armor.ac_bonus"))),
    asFact("Dex Cap", formatModifier(getMetricNumber(itemMetrics, "armor.dex_cap"))),
    asFact("Strength", formatNumber(getMetricNumber(itemMetrics, "armor.strength"))),
    asFact("Check Penalty", formatModifier(getMetricNumber(itemMetrics, "armor.check_penalty"))),
    asFact("Speed Penalty", formatFeet(getMetricNumber(itemMetrics, "armor.speed_penalty"))),
    asFact("Shield AC Bonus", formatModifier(getMetricNumber(itemMetrics, "shield.ac_bonus"))),
    asFact("Shield Hardness", formatNumber(getMetricNumber(itemMetrics, "shield.hardness"))),
    asFact("Shield HP", formatNumber(getMetricNumber(itemMetrics, "shield.hp"))),
    asFact("Shield BT", formatNumber(getMetricNumber(itemMetrics, "shield.bt"))),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function formatCreatureAttackDamage(attack: Record<string, unknown>): string | null {
  const damageRolls = getRawPath(attack, ["system", "damageRolls"]);
  const entries = Array.isArray(damageRolls) ? damageRolls : Object.values(getRawObject(damageRolls) ?? {});
  const parts = entries
    .map((entry) => {
      const damage = getRawObject(entry);
      if (!damage) {
        return null;
      }

      const formula = formatRawValue(damage.damage ?? damage.formula);
      const damageType = humanize(formatRawValue(damage.damageType) ?? "");
      return [formula, damageType].filter(Boolean).join(" ");
    })
    .filter((part): part is string => Boolean(part));
  return parts.length > 0 ? parts.join(", ") : null;
}

function formatCreatureAttackItem(attack: Record<string, unknown>): string | null {
  const name = formatRawValue(attack.name);
  if (!name) {
    return null;
  }

  const bonus = formatModifier(
    getRawNumber(attack, ["system", "bonus", "value"]) ?? getRawNumber(attack, ["system", "bonus"]),
  );
  const damage = formatCreatureAttackDamage(attack);
  return [name, bonus, damage ? `(${damage})` : null].filter(Boolean).join(" ");
}

function formatCreatureAttacks(record: OntologyExplorerEntityRecord): string | null {
  const items = getRawPath(record.raw ?? {}, ["items"]);
  if (!Array.isArray(items)) {
    return null;
  }

  const attacks = items
    .map((item) => getRawObject(item))
    .filter((item): item is Record<string, unknown> => Boolean(item))
    .filter((item): item is Record<string, unknown> => {
      const type = formatRawValue(item.type)?.toLowerCase();
      return type === "melee" || type === "weapon";
    })
    .map(formatCreatureAttackItem)
    .filter((attack): attack is string => Boolean(attack));
  return attacks.length > 0 ? attacks.join(", ") : null;
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

function buildSpellSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Traditions", formatList(record.traditions)),
    asFact("Cast", formatActionCost(record.actionCost)),
    asFact("Range", record.rangeText),
    asFact("Area", formatArea(record)),
    asFact("Save", formatSave(record)),
    asFact("Duration", record.durationText),
    asFact("Targets", record.targetText),
    asFact("Damage", formatList(record.damageTypes)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildCreatureSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Size", humanize(record.size)),
    asFact("Perception", formatModifier(getMetricNumber(record.actorMetrics ?? {}, "perception.mod"))),
    asFact("Languages", formatList(record.languages)),
    asFact("Senses", formatList(record.senses)),
    asFact("Skills", formatCreatureSkills(record)),
    asFact("Abilities", formatCreatureAbilities(record)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildCreatureDefenseFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("AC", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "ac.value"))),
    asFact("HP", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hp.value"))),
    asFact("Hardness", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hardness.value"))),
    asFact("Saves", formatCreatureSaves(record)),
    asFact("Immunities", formatList(record.immunities)),
    asFact("Resistances", formatList(record.resistances)),
    asFact("Weaknesses", formatList(record.weaknesses)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildCreatureMovementFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [asFact("Speed", formatSpeed(record))].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildCreatureOffenseFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Attacks", formatCreatureAttacks(record)),
    asFact("Damage", formatList(record.damageTypes)),
    asFact("Spell Kinds", formatList(record.spellKinds)),
    asFact("Save", formatSave(record)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildEquipmentSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Price", formatPriceCp(record.priceCp)),
    asFact("Bulk", formatBulkValue(record.bulkValue ?? null)),
    asFact("Activation", formatActionCost(record.actionCost)),
    asFact("Usage", humanize(record.usage)),
    asFact("Hands", record.hands == null ? null : String(record.hands)),
    asFact("Base Item", humanize(record.baseItem)),
    asFact("Category", humanize(record.itemCategory)),
    asFact("Group", humanize(record.weaponGroup ?? record.armorGroup)),
    asFact("Damage", formatList(record.damageTypes)),
    ...buildEquipmentItemMetricFacts(record),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildFeatActionSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Action Cost", formatActionCost(record.actionCost)),
    asFact(
      "Trigger",
      formatRawField(record, [
        ["system", "trigger", "value"],
        ["system", "trigger"],
      ]),
    ),
    asFact(
      "Requirements",
      formatRawField(record, [
        ["system", "requirements", "value"],
        ["system", "requirements"],
      ]),
    ),
    asFact(
      "Frequency",
      formatRawField(record, [
        ["system", "frequency", "value"],
        ["system", "frequency"],
      ]),
    ),
    asFact(
      "Prerequisites",
      formatRawField(record, [
        ["system", "prerequisites", "value"],
        ["system", "prerequisites"],
      ]),
    ),
    asFact("Range", record.rangeText),
    asFact("Area", formatArea(record)),
    asFact("Save", formatSave(record)),
    asFact("Duration", record.durationText),
    asFact("Targets", record.targetText),
    asFact("Damage", formatList(record.damageTypes)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildHazardSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Complexity", formatBoolean(record.isComplex, "Complex")),
    asFact("Stealth", formatHazardStealth(record)),
    asFact("Disable", record.disableText),
    asFact("Disable Skills", formatList(record.disableSkills)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildHazardDefenseFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("AC", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "ac.value"))),
    asFact("HP", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hp.value"))),
    asFact("Hardness", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hardness.value"))),
    asFact("Saves", formatCreatureSaves(record)),
    asFact("Immunities", formatList(record.immunities)),
    asFact("Resistances", formatList(record.resistances)),
    asFact("Weaknesses", formatList(record.weaknesses)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildHazardRoutineFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Range", record.rangeText),
    asFact("Area", formatArea(record)),
    asFact("Save", formatSave(record)),
    asFact("Damage", formatList(record.damageTypes)),
    asFact("Targets", record.targetText),
    asFact("Duration", record.durationText),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildFallbackSummaryFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Action Cost", formatActionCost(record.actionCost)),
    asFact("Range", record.rangeText),
    asFact("Area", formatArea(record)),
    asFact("Save", formatSave(record)),
    asFact("Targets", record.targetText),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildDetailFacts(record: OntologyExplorerEntityRecord): EntityPageFact[] {
  return [
    asFact("Spell Kinds", formatList(record.spellKinds)),
    asFact("Source Category", humanize(record.sourceCategory)),
    asFact("Document Type", record.documentType),
    asFact("Sustained", formatBoolean(record.sustained)),
  ].filter((fact): fact is EntityPageFact => Boolean(fact));
}

function buildBrowseRequest(filter: SearchRequest["filter"]): SearchRequest {
  return {
    mode: "browse",
    filter,
    sort: { kind: "alphabetical" },
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
    request: buildBrowseRequest({ kind: "pack", value: packName }),
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
          kind: "metadataPredicate",
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
    const startIndex = match.index ?? 0;
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

  return {
    record,
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
    traitTargets: buildMetadataPivotTargets(record, "header"),
    classificationTargets: buildClassificationTargets(record),
    references: referenceTargetData.targets,
    referencedBy: buildBacklinkTargets(relations),
  };
}

function dedupeFacts(facts: EntityPageFact[], seenValues: Set<string>): EntityPageFact[] {
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

function createFactSection(
  id: string,
  kind: EntityPageSection["kind"],
  title: string,
  facts: EntityPageFact[],
  seenFacts: Set<string>,
): EntityPageSection | null {
  const dedupedFacts = dedupeFacts(facts, seenFacts);
  if (dedupedFacts.length === 0) {
    return null;
  }

  return {
    id,
    kind,
    title,
    blocks: [{ kind: "factList", facts: dedupedFacts }],
    targets: [],
  };
}

function createSummarySection(
  blurb: EntityPageTextContent | undefined,
  facts: EntityPageFact[],
  seenFacts: Set<string>,
): EntityPageSection | null {
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
      ...(dedupedFacts.length > 0 ? [{ kind: "factList" as const, facts: dedupedFacts }] : []),
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

function isFeatActionRecord(record: OntologyExplorerEntityRecord): boolean {
  return record.category === "feat" || (record.category === "rule" && record.subcategory === "action");
}

function selectEntityPageRecipe(record: OntologyExplorerEntityRecord): EntityPageRecipeKind {
  switch (record.category) {
    case "spell":
      return "spell";
    case "creature":
      return "creature";
    case "equipment":
      return "equipment";
    case "hazard":
      return "hazard";
    default:
      return isFeatActionRecord(record) ? "featAction" : "fallback";
  }
}

function buildSpellRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildSpellSummaryFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildCreatureRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildCreatureSummaryFacts(input.record), seenFacts));
  push(createFactSection("defense", "defense", "Defense", buildCreatureDefenseFacts(input.record), seenFacts));
  push(createFactSection("movement", "movement", "Movement", buildCreatureMovementFacts(input.record), seenFacts));
  push(createFactSection("offense", "offense", "Offense", buildCreatureOffenseFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildEquipmentRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildEquipmentSummaryFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildFeatActionRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildFeatActionSummaryFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildHazardRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildHazardSummaryFacts(input.record), seenFacts));
  push(createFactSection("defense", "defense", "Defense", buildHazardDefenseFacts(input.record), seenFacts));
  push(createFactSection("routine", "routine", "Routine", buildHazardRoutineFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildFallbackRecipeSections({ input, detailFacts, seenFacts, push }: EntityPageRecipeBuildContext): void {
  push(createSummarySection(input.blurb, buildFallbackSummaryFacts(input.record), seenFacts));
  push(createTextSection("description", "description", "Description", input.description));
  push(createFactSection("details", "details", "Details", detailFacts, seenFacts));
}

function buildRecipeSections(input: PreparedEntityPageInput, seenFacts: Set<string>): EntityPageSection[] {
  const sections: EntityPageSection[] = [];
  const detailFacts = buildDetailFacts(input.record);
  const push = (section: EntityPageSection | null) => {
    if (section) {
      sections.push(section);
    }
  };
  const context: EntityPageRecipeBuildContext = {
    input,
    detailFacts,
    seenFacts,
    push,
  };

  switch (selectEntityPageRecipe(input.record)) {
    case "spell":
      buildSpellRecipeSections(context);
      break;
    case "creature":
      buildCreatureRecipeSections(context);
      break;
    case "equipment":
      buildEquipmentRecipeSections(context);
      break;
    case "featAction":
      buildFeatActionRecipeSections(context);
      break;
    case "hazard":
      buildHazardRecipeSections(context);
      break;
    case "fallback":
      buildFallbackRecipeSections(context);
      break;
  }

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
