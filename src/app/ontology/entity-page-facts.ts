import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";

export type EntityPageFact = {
  label: string;
  value: string;
};

export type EntityPageRecipeKind = "spell" | "creature" | "equipment" | "featAction" | "hazard" | "fallback";

export type ProjectedEntityPageFact = EntityPageFact & {
  key: string;
};

export type EntityPageFactInventory = {
  allFacts: ProjectedEntityPageFact[];
  spellSummary: ProjectedEntityPageFact[];
  creatureSummary: ProjectedEntityPageFact[];
  creatureDefense: ProjectedEntityPageFact[];
  creatureMovement: ProjectedEntityPageFact[];
  creatureOffense: ProjectedEntityPageFact[];
  equipmentSummary: ProjectedEntityPageFact[];
  featActionSummary: ProjectedEntityPageFact[];
  hazardSummary: ProjectedEntityPageFact[];
  hazardDefense: ProjectedEntityPageFact[];
  hazardRoutine: ProjectedEntityPageFact[];
  fallbackSummary: ProjectedEntityPageFact[];
};

export type ProjectedEntityPageFacts = {
  recipe: EntityPageRecipeKind;
  inventory: EntityPageFactInventory;
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

function asProjectedFact(key: string, label: string, value: string | null | undefined): ProjectedEntityPageFact | null {
  const fact = asFact(label, value);
  return fact ? { ...fact, key } : null;
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

function buildEquipmentItemMetricFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  const itemMetrics = record.itemMetrics ?? {};
  return [
    asProjectedFact("weapon.rangeIncrement", "Range Increment", formatFeet(getMetricNumber(itemMetrics, "weapon.range_increment"))),
    asProjectedFact("weapon.reload", "Reload", formatNumber(getMetricNumber(itemMetrics, "weapon.reload"))),
    asProjectedFact("weapon.damageDice", "Weapon Damage Dice", formatWeaponDamageDice(record)),
    asProjectedFact("armor.acBonus", "Armor AC Bonus", formatModifier(getMetricNumber(itemMetrics, "armor.ac_bonus"))),
    asProjectedFact("armor.dexCap", "Dex Cap", formatModifier(getMetricNumber(itemMetrics, "armor.dex_cap"))),
    asProjectedFact("armor.strength", "Strength", formatNumber(getMetricNumber(itemMetrics, "armor.strength"))),
    asProjectedFact("armor.checkPenalty", "Check Penalty", formatModifier(getMetricNumber(itemMetrics, "armor.check_penalty"))),
    asProjectedFact("armor.speedPenalty", "Speed Penalty", formatFeet(getMetricNumber(itemMetrics, "armor.speed_penalty"))),
    asProjectedFact("shield.acBonus", "Shield AC Bonus", formatModifier(getMetricNumber(itemMetrics, "shield.ac_bonus"))),
    asProjectedFact("shield.hardness", "Shield Hardness", formatNumber(getMetricNumber(itemMetrics, "shield.hardness"))),
    asProjectedFact("shield.hp", "Shield HP", formatNumber(getMetricNumber(itemMetrics, "shield.hp"))),
    asProjectedFact("shield.bt", "Shield BT", formatNumber(getMetricNumber(itemMetrics, "shield.bt"))),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
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

function buildSpellSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("traditions", "Traditions", formatList(record.traditions)),
    asProjectedFact("actionCost", "Cast", formatActionCost(record.actionCost)),
    asProjectedFact("range", "Range", record.rangeText),
    asProjectedFact("area", "Area", formatArea(record)),
    asProjectedFact("save", "Save", formatSave(record)),
    asProjectedFact("duration", "Duration", record.durationText),
    asProjectedFact("targets", "Targets", record.targetText),
    asProjectedFact("damage", "Damage", formatList(record.damageTypes)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildCreatureSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("size", "Size", humanize(record.size)),
    asProjectedFact("perception", "Perception", formatModifier(getMetricNumber(record.actorMetrics ?? {}, "perception.mod"))),
    asProjectedFact("languages", "Languages", formatList(record.languages)),
    asProjectedFact("senses", "Senses", formatList(record.senses)),
    asProjectedFact("skills", "Skills", formatCreatureSkills(record)),
    asProjectedFact("abilities", "Abilities", formatCreatureAbilities(record)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildCreatureDefenseFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("ac", "AC", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "ac.value"))),
    asProjectedFact("hp", "HP", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hp.value"))),
    asProjectedFact("hardness", "Hardness", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hardness.value"))),
    asProjectedFact("saves", "Saves", formatCreatureSaves(record)),
    asProjectedFact("immunities", "Immunities", formatList(record.immunities)),
    asProjectedFact("resistances", "Resistances", formatList(record.resistances)),
    asProjectedFact("weaknesses", "Weaknesses", formatList(record.weaknesses)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildCreatureMovementFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [asProjectedFact("speed", "Speed", formatSpeed(record))].filter(
    (fact): fact is ProjectedEntityPageFact => Boolean(fact),
  );
}

function buildCreatureOffenseFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("attacks", "Attacks", formatCreatureAttacks(record)),
    asProjectedFact("damage", "Damage", formatList(record.damageTypes)),
    asProjectedFact("spellKinds", "Spell Kinds", formatList(record.spellKinds)),
    asProjectedFact("save", "Save", formatSave(record)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildEquipmentSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("price", "Price", formatPriceCp(record.priceCp)),
    asProjectedFact("bulk", "Bulk", formatBulkValue(record.bulkValue ?? null)),
    asProjectedFact("activation", "Activation", formatActionCost(record.actionCost)),
    asProjectedFact("usage", "Usage", humanize(record.usage)),
    asProjectedFact("hands", "Hands", record.hands == null ? null : String(record.hands)),
    asProjectedFact("baseItem", "Base Item", humanize(record.baseItem)),
    asProjectedFact("itemCategory", "Category", humanize(record.itemCategory)),
    asProjectedFact("itemGroup", "Group", humanize(record.weaponGroup ?? record.armorGroup)),
    asProjectedFact("damage", "Damage", formatList(record.damageTypes)),
    ...buildEquipmentItemMetricFacts(record),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildFeatActionSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("actionCost", "Action Cost", formatActionCost(record.actionCost)),
    asProjectedFact(
      "trigger",
      "Trigger",
      formatRawField(record, [
        ["system", "trigger", "value"],
        ["system", "trigger"],
      ]),
    ),
    asProjectedFact(
      "requirements",
      "Requirements",
      formatRawField(record, [
        ["system", "requirements", "value"],
        ["system", "requirements"],
      ]),
    ),
    asProjectedFact(
      "frequency",
      "Frequency",
      formatRawField(record, [
        ["system", "frequency", "value"],
        ["system", "frequency"],
      ]),
    ),
    asProjectedFact(
      "prerequisites",
      "Prerequisites",
      formatRawField(record, [
        ["system", "prerequisites", "value"],
        ["system", "prerequisites"],
      ]),
    ),
    asProjectedFact("range", "Range", record.rangeText),
    asProjectedFact("area", "Area", formatArea(record)),
    asProjectedFact("save", "Save", formatSave(record)),
    asProjectedFact("duration", "Duration", record.durationText),
    asProjectedFact("targets", "Targets", record.targetText),
    asProjectedFact("damage", "Damage", formatList(record.damageTypes)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildHazardSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("complexity", "Complexity", formatBoolean(record.isComplex, "Complex")),
    asProjectedFact("stealth", "Stealth", formatHazardStealth(record)),
    asProjectedFact("disable", "Disable", record.disableText),
    asProjectedFact("disableSkills", "Disable Skills", formatList(record.disableSkills)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildHazardDefenseFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("ac", "AC", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "ac.value"))),
    asProjectedFact("hp", "HP", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hp.value"))),
    asProjectedFact("hardness", "Hardness", formatNumber(getMetricNumber(record.actorMetrics ?? {}, "hardness.value"))),
    asProjectedFact("saves", "Saves", formatCreatureSaves(record)),
    asProjectedFact("immunities", "Immunities", formatList(record.immunities)),
    asProjectedFact("resistances", "Resistances", formatList(record.resistances)),
    asProjectedFact("weaknesses", "Weaknesses", formatList(record.weaknesses)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildHazardRoutineFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("range", "Range", record.rangeText),
    asProjectedFact("area", "Area", formatArea(record)),
    asProjectedFact("save", "Save", formatSave(record)),
    asProjectedFact("damage", "Damage", formatList(record.damageTypes)),
    asProjectedFact("targets", "Targets", record.targetText),
    asProjectedFact("duration", "Duration", record.durationText),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildFallbackSummaryFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("actionCost", "Action Cost", formatActionCost(record.actionCost)),
    asProjectedFact("range", "Range", record.rangeText),
    asProjectedFact("area", "Area", formatArea(record)),
    asProjectedFact("save", "Save", formatSave(record)),
    asProjectedFact("targets", "Targets", record.targetText),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function buildGenericDetailFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFact[] {
  return [
    asProjectedFact("spellKinds", "Spell Kinds", formatList(record.spellKinds)),
    asProjectedFact("sourceCategory", "Source Category", humanize(record.sourceCategory)),
    asProjectedFact("documentType", "Document Type", record.documentType),
    asProjectedFact("sustained", "Sustained", formatBoolean(record.sustained)),
  ].filter((fact): fact is ProjectedEntityPageFact => Boolean(fact));
}

function collectUniqueFacts(facts: ProjectedEntityPageFact[]): ProjectedEntityPageFact[] {
  const seenKeys = new Set<string>();
  const uniqueFacts: ProjectedEntityPageFact[] = [];
  for (const fact of facts) {
    if (seenKeys.has(fact.key)) {
      continue;
    }
    seenKeys.add(fact.key);
    uniqueFacts.push(fact);
  }
  return uniqueFacts;
}

function isFeatActionRecord(record: OntologyExplorerEntityRecord): boolean {
  return record.category === "feat" || (record.category === "rule" && record.subcategory === "action");
}

function selectEntityPageRecipe(record: OntologyExplorerEntityRecord): EntityPageRecipeKind {
  switch (record.category) {
    case "equipment":
      return "equipment";
    case "feat":
      return "featAction";
    case "creature":
      return "creature";
    case "hazard":
      return "hazard";
    case "affliction":
      return "fallback";
    case "rule":
      return isFeatActionRecord(record) ? "featAction" : "fallback";
    case "spell":
      return "spell";
    case "characterCreation":
      return "fallback";
    case "lore":
      return "fallback";
  }
}

function buildEntityPageFactInventory(
  record: OntologyExplorerEntityRecord,
  recipe: EntityPageRecipeKind,
): EntityPageFactInventory {
  const empty: ProjectedEntityPageFact[] = [];
  const spellSummary = recipe === "spell" ? buildSpellSummaryFacts(record) : empty;
  const creatureSummary = recipe === "creature" ? buildCreatureSummaryFacts(record) : empty;
  const creatureDefense = recipe === "creature" ? buildCreatureDefenseFacts(record) : empty;
  const creatureMovement = recipe === "creature" ? buildCreatureMovementFacts(record) : empty;
  const creatureOffense = recipe === "creature" ? buildCreatureOffenseFacts(record) : empty;
  const equipmentSummary = recipe === "equipment" ? buildEquipmentSummaryFacts(record) : empty;
  const featActionSummary = recipe === "featAction" ? buildFeatActionSummaryFacts(record) : empty;
  const hazardSummary = recipe === "hazard" ? buildHazardSummaryFacts(record) : empty;
  const hazardDefense = recipe === "hazard" ? buildHazardDefenseFacts(record) : empty;
  const hazardRoutine = recipe === "hazard" ? buildHazardRoutineFacts(record) : empty;
  const fallbackSummary = recipe === "fallback" ? buildFallbackSummaryFacts(record) : empty;
  const genericDetails = buildGenericDetailFacts(record);

  return {
    allFacts: collectUniqueFacts([
      ...spellSummary,
      ...creatureSummary,
      ...creatureDefense,
      ...creatureMovement,
      ...creatureOffense,
      ...equipmentSummary,
      ...featActionSummary,
      ...hazardSummary,
      ...hazardDefense,
      ...hazardRoutine,
      ...fallbackSummary,
      ...genericDetails,
    ]),
    spellSummary,
    creatureSummary,
    creatureDefense,
    creatureMovement,
    creatureOffense,
    equipmentSummary,
    featActionSummary,
    hazardSummary,
    hazardDefense,
    hazardRoutine,
    fallbackSummary,
  };
}

export function projectEntityPageFacts(record: OntologyExplorerEntityRecord): ProjectedEntityPageFacts {
  const recipe = selectEntityPageRecipe(record);
  return {
    recipe,
    inventory: buildEntityPageFactInventory(record, recipe),
  };
}
