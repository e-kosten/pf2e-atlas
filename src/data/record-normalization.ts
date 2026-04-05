import {
  classifyRecordCategory,
  extractSpellTraditions,
} from "../domain/categories.js";
import {
  ACTOR_ABILITY_KEYS,
  ACTOR_SAVE_KEYS,
  type ActorMetricMap,
  normalizeActorMetricBooleanValue,
  normalizeActorMetricTextValue,
  normalizeRawSaveKey,
  slugifyActorMetricSegment,
} from "../domain/actor-metrics.js";
import {
  type ItemMetricMap,
  slugifyItemMetricSegment,
} from "../domain/item-metrics.js";
import type { SourceCategory } from "../types.js";
import {
  buildEmbeddedItemSearchChunks,
  getRecordDescriptionMarkup,
  getRecordDescriptionText,
  getRecordTraits,
} from "./nested-item-utils.js";
import {
  firstString,
  getNested,
  normalizeText,
  stripHtml,
  toStringArray,
  uniqueSorted,
} from "../utils.js";
import type {
  ActorIndexData,
  ItemIndexData,
  NormalizedIndexRecord,
  PackBuildInfo,
  SpellIndexData,
} from "./index-types.js";

const MAX_ACTOR_SEMANTIC_ITEM_CHUNKS = 40;
const HAZARD_DISABLE_PROFICIENCY_RANKS: Record<string, number> = {
  trained: 1,
  expert: 2,
  master: 3,
  legendary: 4,
};
const HAZARD_DISABLE_SKILLS = [
  "acrobatics",
  "arcana",
  "athletics",
  "crafting",
  "deception",
  "diplomacy",
  "intimidation",
  "medicine",
  "nature",
  "occultism",
  "performance",
  "religion",
  "society",
  "stealth",
  "survival",
  "thievery",
] as const;

function asNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getPublicationTitle(raw: Record<string, unknown>): string | null {
  return firstString(
    getNested(raw, ["system", "publication", "title"]),
    getNested(raw, ["system", "details", "publication", "title"]),
  );
}

export function getPublicationRemaster(raw: Record<string, unknown>): boolean {
  return getNested(raw, ["system", "publication", "remaster"]) === true ||
    getNested(raw, ["system", "details", "publication", "remaster"]) === true;
}

export function getDescriptionMarkup(raw: Record<string, unknown>): string | null {
  return getRecordDescriptionMarkup(raw);
}

function getLevel(raw: Record<string, unknown>): number | null {
  return asNumber(
    getNested(raw, ["system", "level", "value"]) ?? getNested(raw, ["system", "details", "level", "value"]),
  );
}

function getDescriptionText(raw: Record<string, unknown>): string | null {
  return getRecordDescriptionText(raw);
}

function getRarity(raw: Record<string, unknown>): string | null {
  const rarity = getNested(raw, ["system", "traits", "rarity"]);
  return typeof rarity === "string" && rarity.length > 0 ? rarity : null;
}

function parseSize(raw: Record<string, unknown>): string | null {
  return firstString(getNested(raw, ["system", "traits", "size", "value"]));
}

function parseLanguages(raw: Record<string, unknown>): string[] {
  return uniqueSorted(toStringArray(getNested(raw, ["system", "details", "languages", "value"])));
}

function parseSpeedTypes(raw: Record<string, unknown>): string[] {
  const values = ["land"];
  const otherSpeeds = getNested(raw, ["system", "attributes", "speed", "otherSpeeds"]);
  if (Array.isArray(otherSpeeds)) {
    for (const speed of otherSpeeds) {
      const speedType = firstString(getNested(speed, ["type"]));
      if (speedType) {
        values.push(speedType);
      }
    }
  }

  return uniqueSorted(values);
}

function parseSenses(raw: Record<string, unknown>): string[] {
  const senses = getNested(raw, ["system", "perception", "senses"]);
  if (!Array.isArray(senses)) {
    return [];
  }

  return uniqueSorted(
    senses
      .map((sense) => slugifyActorMetricSegment(firstString(getNested(sense, ["type"])) ?? "").replace(/_/g, " "))
      .filter(Boolean),
  );
}

function parseTypedCollection(raw: Record<string, unknown>, pathSegments: string[]): string[] {
  const collection = getNested(raw, pathSegments);
  if (!Array.isArray(collection)) {
    return [];
  }

  return uniqueSorted(
    collection
      .map((entry) => firstString(getNested(entry, ["type"])))
      .filter((value): value is string => Boolean(value)),
  );
}

function parseAbilityMetric(raw: Record<string, unknown>, abilityKey: string): number | null {
  const ability = getNested(raw, ["system", "abilities", abilityKey]);
  if (!ability || typeof ability !== "object") {
    return null;
  }

  return asNumber(getNested(ability, ["mod"]) ?? getNested(ability, ["modifier"]));
}

function parsePerceptionMetric(raw: Record<string, unknown>): number | null {
  const perception = getNested(raw, ["system", "perception"]);
  if (!perception || typeof perception !== "object") {
    return null;
  }

  return asNumber(
    getNested(perception, ["mod"]) ??
      getNested(perception, ["modifier"]) ??
      getNested(perception, ["value"]),
  );
}

function parseArmorClassMetric(raw: Record<string, unknown>): number | null {
  const ac = getNested(raw, ["system", "attributes", "ac"]);
  if (!ac || typeof ac !== "object") {
    return null;
  }

  return asNumber(getNested(ac, ["value"]));
}

function parseHardnessMetric(raw: Record<string, unknown>): number | null {
  return asNumber(getNested(raw, ["system", "attributes", "hardness"]));
}

function parseHitPointMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const hp = getNested(raw, ["system", "attributes", "hp"]);
  if (!hp || typeof hp !== "object") {
    return {};
  }

  const metrics: ActorMetricMap = {};
  const value = asNumber(getNested(hp, ["value"]));
  const max = asNumber(getNested(hp, ["max"]));
  const brokenThreshold = asNumber(
    getNested(hp, ["brokenThreshold"]) ??
      getNested(hp, ["broken"]) ??
      getNested(hp, ["bt"]),
  );

  if (value !== null) {
    metrics["hp.value"] = value;
  }

  if (max !== null) {
    metrics["hp.max"] = max;
  }

  if (brokenThreshold !== null) {
    metrics["hp.bt"] = brokenThreshold;
  }

  return metrics;
}

function parseStealthMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const stealth = getNested(raw, ["system", "attributes", "stealth"]);
  if (!stealth || typeof stealth !== "object") {
    return {};
  }

  const stealthMod = asNumber(
    getNested(stealth, ["value"]) ??
      getNested(stealth, ["mod"]) ??
      getNested(stealth, ["modifier"]),
  );
  const stealthDc = asNumber(getNested(stealth, ["dc"])) ?? (stealthMod !== null ? stealthMod + 10 : null);

  const metrics: ActorMetricMap = {};
  if (stealthMod !== null) {
    metrics["stealth.mod"] = stealthMod;
  }
  if (stealthDc !== null) {
    metrics["stealth.dc"] = stealthDc;
  }
  return metrics;
}

function parseSaveMetrics(raw: Record<string, unknown>): Partial<Record<(typeof ACTOR_SAVE_KEYS)[number], number>> {
  const saves = getNested(raw, ["system", "saves"]);
  if (!saves || typeof saves !== "object") {
    return {};
  }

  const result: Partial<Record<(typeof ACTOR_SAVE_KEYS)[number], number>> = {};
  for (const [rawSaveKey, saveValue] of Object.entries(saves as Record<string, unknown>)) {
    const saveKey = normalizeRawSaveKey(rawSaveKey);
    if (!saveKey || !saveValue || typeof saveValue !== "object") {
      continue;
    }

    const numericValue = asNumber(
      getNested(saveValue, ["mod"]) ??
        getNested(saveValue, ["modifier"]) ??
        getNested(saveValue, ["value"]) ??
        getNested(saveValue, ["totalModifier"]),
    );
    if (numericValue !== null) {
      result[saveKey] = numericValue;
    }
  }

  return result;
}

function parseSkillMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const skills = getNested(raw, ["system", "skills"]);
  if (!skills || typeof skills !== "object") {
    return {};
  }

  const metrics: ActorMetricMap = {};
  for (const [rawSkillKey, skillValue] of Object.entries(skills as Record<string, unknown>)) {
    if (!skillValue || typeof skillValue !== "object") {
      continue;
    }

    const skillKey = slugifyActorMetricSegment(rawSkillKey);
    if (!skillKey) {
      continue;
    }

    const skillMod = asNumber(
      getNested(skillValue, ["mod"]) ??
        getNested(skillValue, ["modifier"]) ??
        getNested(skillValue, ["value"]),
    );
    const skillRank = asNumber(getNested(skillValue, ["rank"]));

    if (skillMod !== null) {
      metrics[`skill.${skillKey}.mod`] = skillMod;
    }

    if (skillRank !== null) {
      metrics[`skill.${skillKey}.rank`] = skillRank;
      metrics[`skill.${skillKey}.proficient`] = normalizeActorMetricBooleanValue(skillRank >= 1);
    }
  }

  return metrics;
}

function parseSpeedMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const metrics: ActorMetricMap = {};
  const landSpeed = parseNumericLikeValue(getNested(raw, ["system", "attributes", "speed", "value"]));
  if (landSpeed !== null) {
    metrics["speed.land.value"] = landSpeed;
  }

  const otherSpeeds = getNested(raw, ["system", "attributes", "speed", "otherSpeeds"]);
  if (!Array.isArray(otherSpeeds)) {
    return metrics;
  }

  for (const speed of otherSpeeds) {
    const speedType = slugifyActorMetricSegment(firstString(getNested(speed, ["type"])) ?? "");
    const speedValue = parseNumericLikeValue(getNested(speed, ["value"]));
    if (!speedType || speedValue === null) {
      continue;
    }

    metrics[`speed.${speedType}.value`] = speedValue;
  }

  return metrics;
}

function parseSenseRangeMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const senses = getNested(raw, ["system", "perception", "senses"]);
  if (!Array.isArray(senses)) {
    return {};
  }

  const metrics: ActorMetricMap = {};
  for (const sense of senses) {
    const senseType = slugifyActorMetricSegment(firstString(getNested(sense, ["type"])) ?? "");
    const rangeValue = parseNumericLikeValue(getNested(sense, ["range"]));
    if (!senseType || rangeValue === null) {
      continue;
    }

    metrics[`sense.${senseType}.range`] = rangeValue;
  }

  return metrics;
}

type ParsedHazardDisableData = {
  disableText: string | null;
  disableSkills: string[];
  metrics: ActorMetricMap;
};

function parseHazardDisableData(raw: Record<string, unknown>): ParsedHazardDisableData {
  const disableMarkup = firstString(getNested(raw, ["system", "details", "disable"]));
  const disableText = stripHtml(disableMarkup);
  if (!disableMarkup) {
    return {
      disableText,
      disableSkills: [],
      metrics: {},
    };
  }

  const skills = new Set<string>();
  const rankBySkill = new Map<string, number>();
  const dcValuesBySkill = new Map<string, number[]>();
  const allDcs: number[] = [];
  const checkMatches = [...disableMarkup.matchAll(/@Check\[([^\]]+)\]([^@]*)/g)];

  for (const match of checkMatches) {
    const body = match[1] ?? "";
    const trailingText = stripHtml(match[2] ?? "") ?? "";
    const segments = body.split("|").map((segment) => segment.trim()).filter(Boolean);
    const dcSegment = segments.find((segment) => /^dc\s*:/i.test(segment));
    const dc = dcSegment ? parseNumericLikeValue(dcSegment.replace(/^dc\s*:/i, "")) : null;
    const primarySkillSegment = segments.find((segment) => !segment.includes(":"));
    const primarySkill = slugifyActorMetricSegment(primarySkillSegment ?? "");
    const leadingRankMatch = trailingText.match(/^\s*\((trained|expert|master|legendary)\)/i);

    if (dc !== null) {
      allDcs.push(dc);
    }

    if (primarySkill) {
      skills.add(primarySkill);
      if (dc !== null) {
        dcValuesBySkill.set(primarySkill, [...(dcValuesBySkill.get(primarySkill) ?? []), dc]);
      }
      if (leadingRankMatch?.[1]) {
        rankBySkill.set(primarySkill, Math.max(rankBySkill.get(primarySkill) ?? 0, HAZARD_DISABLE_PROFICIENCY_RANKS[leadingRankMatch[1].toLowerCase()] ?? 0));
      }
    }

    for (const skillName of HAZARD_DISABLE_SKILLS) {
      const skillPattern = new RegExp(`\\b${skillName}\\b(?:\\s*\\((trained|expert|master|legendary)\\))?`, "ig");
      for (const skillMatch of trailingText.matchAll(skillPattern)) {
        const skillKey = slugifyActorMetricSegment(skillName);
        skills.add(skillKey);
        if (dc !== null) {
          dcValuesBySkill.set(skillKey, [...(dcValuesBySkill.get(skillKey) ?? []), dc]);
        }
        if (skillMatch[1]) {
          rankBySkill.set(skillKey, Math.max(rankBySkill.get(skillKey) ?? 0, HAZARD_DISABLE_PROFICIENCY_RANKS[skillMatch[1].toLowerCase()] ?? 0));
        }
      }
    }
  }

  const metrics: ActorMetricMap = {};
  if (allDcs.length > 0) {
    metrics["disable.dc.min"] = Math.min(...allDcs);
    metrics["disable.dc.max"] = Math.max(...allDcs);
  }

  for (const [skill, values] of dcValuesBySkill.entries()) {
    if (values.length > 0) {
      metrics[`disable.${skill}.dc.min`] = Math.min(...values);
      metrics[`disable.${skill}.dc.max`] = Math.max(...values);
    }
  }

  for (const [skill, rank] of rankBySkill.entries()) {
    if (rank > 0) {
      metrics[`disable.${skill}.rank.min`] = rank;
    }
  }

  return {
    disableText,
    disableSkills: uniqueSorted([...skills]),
    metrics,
  };
}

function addBestAndWorstSaveMetrics(
  actorMetrics: ActorMetricMap,
  saveMetrics: Partial<Record<(typeof ACTOR_SAVE_KEYS)[number], number>>,
): void {
  let bestSave: (typeof ACTOR_SAVE_KEYS)[number] | null = null;
  let worstSave: (typeof ACTOR_SAVE_KEYS)[number] | null = null;
  let bestValue = Number.NEGATIVE_INFINITY;
  let worstValue = Number.POSITIVE_INFINITY;

  for (const saveKey of ACTOR_SAVE_KEYS) {
    const saveValue = saveMetrics[saveKey];
    if (saveValue === undefined) {
      continue;
    }

    if (saveValue > bestValue) {
      bestValue = saveValue;
      bestSave = saveKey;
    }

    if (saveValue < worstValue) {
      worstValue = saveValue;
      worstSave = saveKey;
    }
  }

  if (bestSave) {
    actorMetrics["save.best"] = normalizeActorMetricTextValue(bestSave);
  }

  if (worstSave) {
    actorMetrics["save.worst"] = normalizeActorMetricTextValue(worstSave);
  }
}

function parseActorMetrics(raw: Record<string, unknown>): ActorMetricMap {
  const metrics: ActorMetricMap = {};

  for (const abilityKey of ACTOR_ABILITY_KEYS) {
    const abilityMod = parseAbilityMetric(raw, abilityKey);
    if (abilityMod !== null) {
      metrics[`ability.${abilityKey}.mod`] = abilityMod;
    }
  }

  const perceptionMod = parsePerceptionMetric(raw);
  if (perceptionMod !== null) {
    metrics["perception.mod"] = perceptionMod;
  }

  const armorClass = parseArmorClassMetric(raw);
  if (armorClass !== null) {
    metrics["ac.value"] = armorClass;
  }

  const hardness = parseHardnessMetric(raw);
  if (hardness !== null) {
    metrics["hardness.value"] = hardness;
  }

  const saveMetrics = parseSaveMetrics(raw);
  for (const saveKey of ACTOR_SAVE_KEYS) {
    const saveMod = saveMetrics[saveKey];
    if (saveMod !== undefined) {
      metrics[`save.${saveKey}.mod`] = saveMod;
    }
  }

  addBestAndWorstSaveMetrics(metrics, saveMetrics);
  return {
    ...metrics,
    ...parseHitPointMetrics(raw),
    ...parseSkillMetrics(raw),
    ...parseSpeedMetrics(raw),
    ...parseSenseRangeMetrics(raw),
    ...parseStealthMetrics(raw),
    ...parseHazardDisableData(raw).metrics,
  };
}

export function parseActorIndexData(raw: Record<string, unknown>): ActorIndexData {
  const disableData = parseHazardDisableData(raw);
  return {
    size: parseSize(raw),
    languages: parseLanguages(raw),
    speedTypes: parseSpeedTypes(raw),
    senses: parseSenses(raw),
    immunities: parseTypedCollection(raw, ["system", "attributes", "immunities"]),
    resistances: parseTypedCollection(raw, ["system", "attributes", "resistances"]),
    weaknesses: parseTypedCollection(raw, ["system", "attributes", "weaknesses"]),
    disableText: disableData.disableText,
    disableSkills: disableData.disableSkills,
    isComplex: getNested(raw, ["system", "details", "isComplex"]) === true,
    actorMetrics: parseActorMetrics(raw),
  };
}

function normalizePriceToCopper(rawValue: unknown): number | null {
  if (!rawValue || typeof rawValue !== "object") {
    return null;
  }

  const price = rawValue as Record<string, unknown>;
  const gp = asNumber(price.gp) ?? 0;
  const sp = asNumber(price.sp) ?? 0;
  const cp = asNumber(price.cp) ?? 0;
  const pp = asNumber(price.pp) ?? 0;
  const total = (pp * 1000) + (gp * 100) + (sp * 10) + cp;
  return total > 0 ? total : null;
}

function parseBulkValue(rawValue: unknown): number | null {
  if (typeof rawValue === "number") {
    return rawValue;
  }

  if (typeof rawValue !== "string") {
    return null;
  }

  if (rawValue === "L") {
    return 0.1;
  }

  const parsed = Number(rawValue);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseHands(usage: string | null): number | null {
  if (!usage) {
    return null;
  }

  if (usage.includes("held-in-two-hands")) {
    return 2;
  }

  if (usage.includes("held-in-one-plus-hands")) {
    return 1;
  }

  if (usage.includes("held-in-one-hand")) {
    return 1;
  }

  return null;
}

function parseDamageTypes(raw: Record<string, unknown>): string[] {
  const damageTypes = new Set<string>();

  const directDamageType = firstString(getNested(raw, ["system", "damage", "damageType"]));
  if (directDamageType) {
    damageTypes.add(directDamageType);
  }

  const damageRolls = getNested(raw, ["system", "damageRolls"]);
  if (damageRolls && typeof damageRolls === "object") {
    for (const entry of Object.values(damageRolls as Record<string, unknown>)) {
      const damageType = firstString(getNested(entry, ["damageType"]));
      if (damageType) {
        damageTypes.add(damageType);
      }
    }
  }

  const spellDamage = getNested(raw, ["system", "damage"]);
  if (spellDamage && typeof spellDamage === "object") {
    for (const entry of Object.values(spellDamage as Record<string, unknown>)) {
      const damageType = firstString(getNested(entry, ["type"]));
      if (damageType) {
        damageTypes.add(damageType);
      }
    }
  }

  return [...damageTypes].sort((left, right) => left.localeCompare(right));
}

function parseItemCategory(raw: Record<string, unknown>): string | null {
  const recordType = firstString(raw.type);
  if (!recordType) {
    return null;
  }

  return recordType;
}

function parseActionCost(raw: Record<string, unknown>): number | null {
  return asNumber(getNested(raw, ["system", "actions", "value"]));
}

function parseNumericLikeValue(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }

  if (typeof value === "string" && value.trim().length > 0) {
    const trimmed = value.trim();
    const parsed = Number(trimmed);
    if (Number.isFinite(parsed)) {
      return parsed;
    }

    const leadingNumber = trimmed.match(/^-?\d+(?:\.\d+)?/);
    if (leadingNumber?.[0]) {
      const leadingParsed = Number(leadingNumber[0]);
      return Number.isFinite(leadingParsed) ? leadingParsed : null;
    }
  }

  return null;
}

function parseWeaponRangeIncrement(raw: Record<string, unknown>): number | null {
  return parseNumericLikeValue(
    getNested(raw, ["system", "range", "increment"]) ??
      getNested(raw, ["system", "range", "value"]) ??
      getNested(raw, ["system", "range"]),
  );
}

function parseWeaponReload(raw: Record<string, unknown>): number | null {
  return parseNumericLikeValue(
    getNested(raw, ["system", "reload", "value"]) ??
      getNested(raw, ["system", "reload"]),
  );
}

function parseDamageDieFaces(rawValue: unknown): number | null {
  if (typeof rawValue === "number" && Number.isFinite(rawValue)) {
    return rawValue;
  }

  if (typeof rawValue !== "string") {
    return null;
  }

  const match = rawValue.trim().match(/^d(\d+)$/i);
  if (!match?.[1]) {
    return null;
  }

  const parsed = Number(match[1]);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseItemMetrics(raw: Record<string, unknown>): ItemMetricMap {
  const metrics: ItemMetricMap = {};
  const recordType = slugifyItemMetricSegment(firstString(raw.type) ?? "");

  if (recordType === "weapon") {
    const rangeIncrement = parseWeaponRangeIncrement(raw);
    const reload = parseWeaponReload(raw);
    const damageDice = asNumber(getNested(raw, ["system", "damage", "dice"]));
    const damageDieFaces = parseDamageDieFaces(getNested(raw, ["system", "damage", "die"]));

    if (rangeIncrement !== null) {
      metrics["weapon.range_increment"] = rangeIncrement;
    }
    if (reload !== null) {
      metrics["weapon.reload"] = reload;
    }
    if (damageDice !== null) {
      metrics["weapon.damage_dice"] = damageDice;
    }
    if (damageDieFaces !== null) {
      metrics["weapon.damage_die_faces"] = damageDieFaces;
    }
  }

  if (recordType === "armor") {
    const acBonus = asNumber(getNested(raw, ["system", "acBonus"]));
    const dexCap = asNumber(getNested(raw, ["system", "dexCap"]));
    const strength = asNumber(getNested(raw, ["system", "strength"]));
    const checkPenalty = asNumber(getNested(raw, ["system", "checkPenalty"]));
    const speedPenalty = asNumber(getNested(raw, ["system", "speedPenalty"]));

    if (acBonus !== null) {
      metrics["armor.ac_bonus"] = acBonus;
    }
    if (dexCap !== null) {
      metrics["armor.dex_cap"] = dexCap;
    }
    if (strength !== null) {
      metrics["armor.strength"] = strength;
    }
    if (checkPenalty !== null) {
      metrics["armor.check_penalty"] = checkPenalty;
    }
    if (speedPenalty !== null) {
      metrics["armor.speed_penalty"] = speedPenalty;
    }
  }

  if (recordType === "shield") {
    const acBonus = asNumber(getNested(raw, ["system", "acBonus"]));
    const hardness = asNumber(getNested(raw, ["system", "hardness"]));
    const hp = asNumber(getNested(raw, ["system", "hp", "value"]) ?? getNested(raw, ["system", "hp", "max"]));
    const brokenThreshold = asNumber(
      getNested(raw, ["system", "hp", "brokenThreshold"]) ??
        getNested(raw, ["system", "hp", "broken"]) ??
        getNested(raw, ["system", "hp", "bt"]),
    );

    if (acBonus !== null) {
      metrics["shield.ac_bonus"] = acBonus;
    }
    if (hardness !== null) {
      metrics["shield.hardness"] = hardness;
    }
    if (hp !== null) {
      metrics["shield.hp"] = hp;
    }
    if (brokenThreshold !== null) {
      metrics["shield.bt"] = brokenThreshold;
    }
  }

  return metrics;
}

export function parseItemIndexData(raw: Record<string, unknown>): ItemIndexData {
  const usage = firstString(getNested(raw, ["system", "usage", "value"]));
  return {
    itemCategory: parseItemCategory(raw),
    priceCp: normalizePriceToCopper(getNested(raw, ["system", "price", "value"])),
    bulkValue: parseBulkValue(getNested(raw, ["system", "bulk", "value"])),
    usage,
    hands: parseHands(usage),
    damageTypes: parseDamageTypes(raw),
    weaponGroup: firstString(getNested(raw, ["system", "group"])),
    armorGroup: firstString(getNested(raw, ["system", "group"])),
    actionCost: parseActionCost(raw),
    itemMetrics: parseItemMetrics(raw),
  };
}

function parseRangeValue(raw: Record<string, unknown>): number | null {
  return parseNumericLikeValue(
    getNested(raw, ["system", "range", "value"]) ??
      getNested(raw, ["system", "range", "increment"]) ??
      getNested(raw, ["system", "area", "value"]),
  );
}

function extractSpellKinds(raw: Record<string, unknown>): string[] {
  const spellTraits = new Set(getRecordTraits(raw).map((trait) => normalizeText(trait)).filter(Boolean));
  return ["focus", "ritual", "cantrip"].filter((kind) => spellTraits.has(kind));
}

function parseSpellDurationUnit(durationText: string | null): string | null {
  const normalized = normalizeText(durationText ?? "");
  if (!normalized) {
    return null;
  }

  if (normalized.includes("unlimited")) {
    return "unlimited";
  }
  if (normalized.includes("permanent")) {
    return "permanent";
  }
  if (normalized.includes("varies")) {
    return "varies";
  }

  for (const unit of ["round", "minute", "hour", "day", "week", "month", "year"] as const) {
    if (normalized.includes(unit)) {
      return unit;
    }
  }

  return null;
}

export function parseSpellIndexData(raw: Record<string, unknown>): SpellIndexData {
  const durationText = firstString(getNested(raw, ["system", "duration", "value"]));
  return {
    actionCost: parseActionCost(raw),
    traditions: extractSpellTraditions(raw),
    spellKinds: extractSpellKinds(raw),
    rangeText: firstString(getNested(raw, ["system", "range", "value"])),
    rangeValue: parseRangeValue(raw),
    saveType: firstString(getNested(raw, ["system", "defense", "save", "statistic"])),
    areaType: firstString(getNested(raw, ["system", "area", "type"])),
    durationText,
    durationUnit: parseSpellDurationUnit(durationText),
    targetText: stripHtml(firstString(getNested(raw, ["system", "target", "value"]))),
    areaValue: parseNumericLikeValue(getNested(raw, ["system", "area", "value"])),
    sustained: getNested(raw, ["system", "duration", "sustained"]) === true,
    basicSave: getNested(raw, ["system", "defense", "save", "basic"]) === true,
    damageTypes: parseDamageTypes(raw),
  };
}

function appendUniqueTextChunk(chunks: string[], seen: Set<string>, value: string | null | undefined): boolean {
  if (!value) {
    return false;
  }

  const normalized = normalizeText(value);
  if (!normalized || seen.has(normalized)) {
    return false;
  }

  seen.add(normalized);
  chunks.push(value.trim());
  return true;
}

function buildActorSemanticItemChunks(raw: Record<string, unknown>): string[] {
  return buildEmbeddedItemSearchChunks(raw, MAX_ACTOR_SEMANTIC_ITEM_CHUNKS);
}

function buildSearchText(raw: Record<string, unknown>, base: { name: string; descriptionText: string | null; traits: string[] }): string {
  const chunks: string[] = [base.name, ...base.traits];
  if (base.descriptionText) {
    chunks.push(base.descriptionText);
  }

  chunks.push(...buildEmbeddedItemSearchChunks(raw));

  return chunks
    .filter((value): value is string => Boolean(value))
    .join("\n")
    .trim();
}

export function buildSemanticEmbeddingText(record: NormalizedIndexRecord, raw: Record<string, unknown>, aliases: string[]): string {
  const chunks: string[] = [];
  const seen = new Set<string>();

  appendUniqueTextChunk(chunks, seen, record.name);
  for (const trait of record.traits) {
    appendUniqueTextChunk(chunks, seen, trait);
  }
  for (const family of record.families) {
    appendUniqueTextChunk(chunks, seen, family);
  }
  for (const tag of record.derivedTags) {
    appendUniqueTextChunk(chunks, seen, tag);
  }
  appendUniqueTextChunk(chunks, seen, record.descriptionSnippet);

  if (record.documentType === "Actor") {
    for (const itemChunk of buildActorSemanticItemChunks(raw)) {
      appendUniqueTextChunk(chunks, seen, itemChunk);
    }
  }

  for (const alias of aliases) {
    appendUniqueTextChunk(chunks, seen, alias);
  }

  return chunks.join("\n").trim();
}

function hasDescriptionText(descriptionText: string | null): boolean {
  return Boolean(descriptionText && descriptionText.trim().length > 0);
}

function buildDescriptionSnippet(descriptionText: string | null): string | null {
  if (!hasDescriptionText(descriptionText)) {
    return null;
  }

  const normalized = (descriptionText ?? "").replace(/\s+/g, " ").trim();
  const sentenceMatch = normalized.match(/^(.{1,240}?[.!?])(?:\s|$)/);
  if (sentenceMatch) {
    return sentenceMatch[1]!.trim();
  }

  if (normalized.length <= 240) {
    return normalized;
  }

  return `${normalized.slice(0, 237).trimEnd()}...`;
}

function isCorePublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  return normalized === "pathfinder player core" ||
    normalized === "pathfinder player core 2" ||
    normalized === "pathfinder gm core" ||
    normalized === "pathfinder monster core" ||
    normalized === "pathfinder monster core 2" ||
    normalized === "pathfinder beginner box";
}

function isAdventurePublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  if (!normalized) {
    return false;
  }

  return normalized.includes("adventure path") ||
    normalized.includes("pathfinder society") ||
    normalized.includes("quest") ||
    normalized.includes("one shot") ||
    normalized.includes("special") ||
    normalized.startsWith("pathfinder adventure ") ||
    /^pathfinder \d+ /.test(normalized);
}

function isAdventurePack(packName: string): boolean {
  const normalizedPack = normalizeText(packName);
  return normalizedPack.startsWith("pfs ") ||
    normalizedPack.includes("one shot") ||
    normalizedPack.includes("quest");
}

export function isExcludedPackName(packName: string): boolean {
  const normalized = normalizeText(packName);
  return normalized.startsWith("pfs ") ||
    normalized === "pathfinder society boons" ||
    normalized === "macros" ||
    normalized === "action macros";
}

function isExcludedSocietyEffectPath(sourcePath: string): boolean {
  return sourcePath.replace(/\\/g, "/").includes("/campaign-effects/pathfinder-society/");
}

function isPfsBoonRecord(raw: Record<string, unknown>): boolean {
  return normalizeText(firstString(getNested(raw, ["system", "category"])) ?? "") === "pfsboon";
}

export function shouldExcludeRecordFromIndex(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): boolean {
  if (isExcludedPackName(pack.name) || isExcludedSocietyEffectPath(sourcePath) || isPfsBoonRecord(raw)) {
    return true;
  }

  const classification = classifyRecordCategory({
    documentType: pack.documentType,
    recordType: firstString(raw.type) ?? "unknown",
    packName: pack.name,
    sourcePath,
    traits: getRecordTraits(raw),
    traditions: extractSpellTraditions(raw),
    raw,
  });
  return classification === null;
}

function getSourceCategory(packName: string, publicationTitle: string | null): SourceCategory {
  if (isCorePublication(publicationTitle)) {
    return "core";
  }

  if (isAdventurePublication(publicationTitle) || isAdventurePack(packName)) {
    return "adventure";
  }

  if (publicationTitle) {
    return "rules";
  }

  return "unknown";
}

export function normalizeIndexRecord(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): NormalizedIndexRecord {
  const id = firstString(raw._id);
  const name = firstString(raw.name);
  const recordType = firstString(raw.type) ?? "unknown";

  if (!id || !name) {
    throw new Error(`Invalid record in ${sourcePath}: expected both _id and name.`);
  }

  const rarity = getRarity(raw);
  const traits = getRecordTraits(raw);
  const descriptionText = getDescriptionText(raw);
  const publicationTitle = getPublicationTitle(raw);
  const publicationRemaster = getPublicationRemaster(raw);
  const hasDescription = hasDescriptionText(descriptionText);
  const descriptionSnippet = buildDescriptionSnippet(descriptionText);
  const sourceCategory = getSourceCategory(pack.name, publicationTitle);
  const actorData = pack.documentType === "Actor" ? parseActorIndexData(raw) : null;
  const itemData = pack.documentType === "Item" ? parseItemIndexData(raw) : null;
  const spellData = recordType === "spell" ? parseSpellIndexData(raw) : null;
  const classification = classifyRecordCategory({
    documentType: pack.documentType,
    recordType,
    packName: pack.name,
    sourcePath,
    traits,
    traditions: spellData?.traditions ?? [],
    raw,
  });

  if (!classification) {
    throw new Error(`Record in ${sourcePath} did not map to a public search category.`);
  }

  return {
    recordKey: `${pack.name}:${id}`,
    id,
    name,
    normalizedName: normalizeText(name),
    type: recordType,
    category: classification.category,
    subcategory: classification.subcategory,
    packName: pack.name,
    packLabel: pack.label,
    documentType: pack.documentType,
    level: getLevel(raw),
    rarity,
    traits,
    derivedTags: [],
    publicationTitle,
    publicationRemaster,
    descriptionText,
    hasDescription,
    descriptionSnippet,
    sourceCategory,
    folderId: firstString(raw.folder),
    families: [],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath,
    isUnique: normalizeText(rarity ?? "") === "unique",
    size: actorData?.size ?? null,
    itemCategory: itemData?.itemCategory ?? null,
    priceCp: itemData?.priceCp ?? null,
    bulkValue: itemData?.bulkValue ?? null,
    actionCost: spellData?.actionCost ?? itemData?.actionCost ?? null,
    usage: itemData?.usage ?? null,
    hands: itemData?.hands ?? null,
    damageTypes: spellData?.damageTypes ?? itemData?.damageTypes ?? [],
    weaponGroup: itemData?.weaponGroup ?? null,
    armorGroup: itemData?.armorGroup ?? null,
    traditions: spellData?.traditions ?? [],
    spellKinds: spellData?.spellKinds ?? [],
    saveType: spellData?.saveType ?? null,
    areaType: spellData?.areaType ?? null,
    rangeText: spellData?.rangeText ?? null,
    durationText: spellData?.durationText ?? null,
    durationUnit: spellData?.durationUnit ?? null,
    targetText: spellData?.targetText ?? null,
    areaValue: spellData?.areaValue ?? null,
    sustained: spellData?.sustained ?? false,
    basicSave: spellData?.basicSave ?? false,
    languages: actorData?.languages ?? [],
    speedTypes: actorData?.speedTypes ?? [],
    senses: actorData?.senses ?? [],
    immunities: actorData?.immunities ?? [],
    resistances: actorData?.resistances ?? [],
    weaknesses: actorData?.weaknesses ?? [],
    disableText: actorData?.disableText ?? null,
    disableSkills: actorData?.disableSkills ?? [],
    isComplex: actorData?.isComplex ?? false,
    actorMetrics: actorData?.actorMetrics ?? {},
    itemMetrics: itemData?.itemMetrics ?? {},
    rangeValue: spellData?.rangeValue ?? null,
    searchText: buildSearchText(raw, { name, descriptionText, traits }),
  };
}
