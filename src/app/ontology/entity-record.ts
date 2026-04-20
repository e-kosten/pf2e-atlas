import type { NormalizedRecord, SearchCategory, SearchSubcategory, SourceCategory } from "../../domain/index.js";
import {
  parseSearchCategoryValue,
  parseSearchSubcategoryForCategory,
  parseSourceCategoryValue,
  parseStringArrayJson,
  toSqliteNumber,
} from "../../data/sql-row-decoding.js";

export type OntologyExplorerEntityRecord = {
  recordKey: string;
  packName: string;
  name: string;
  type: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traits: string[];
  derivedTags: string[];
  families: string[];
  descriptionText: string | null;
  blurbText: string | null;
  sourceCategory: SourceCategory;
  publicationTitle: string | null;
  publicationRemaster: boolean;
  isUnique: boolean;
  size: string | null;
  languages: string[];
  speedTypes: string[];
  senses: string[];
  immunities: string[];
  resistances: string[];
  weaknesses: string[];
  itemCategory: string | null;
  baseItem: string | null;
  priceCp: number | null;
  usage: string | null;
  hands: number | null;
  damageTypes: string[];
  weaponGroup: string | null;
  armorGroup: string | null;
  traditions: string[];
  spellKinds: string[];
  saveType: string | null;
  areaType: string | null;
  rangeText: string | null;
  durationText: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: boolean;
  basicSave: boolean;
  disableText: string | null;
  disableSkills: string[];
  isComplex: boolean;
};

export type OntologyExplorerEntityRecordRow = {
  recordKey: string;
  packName: string | null;
  name: string;
  type: string;
  category: string;
  subcategory: string | null;
  documentType: string;
  level: number | bigint | null;
  rarity: string | null;
  traitsJson: string;
  derivedTagsJson: string;
  familiesJson: string | null;
  descriptionText: string | null;
  blurbText: string | null;
  sourceCategory: string;
  publicationTitle: string | null;
  publicationRemaster: number;
  isUnique: number;
  size: string | null;
  languagesJson: string | null;
  speedTypesJson: string | null;
  sensesJson: string | null;
  immunitiesJson: string | null;
  resistancesJson: string | null;
  weaknessesJson: string | null;
  itemCategory: string | null;
  baseItem: string | null;
  priceCp: number | bigint | null;
  usage: string | null;
  hands: number | bigint | null;
  damageTypesJson: string | null;
  weaponGroup: string | null;
  armorGroup: string | null;
  traditionsJson: string | null;
  spellKindsJson: string | null;
  saveType: string | null;
  areaType: string | null;
  rangeText: string | null;
  durationText: string | null;
  targetText: string | null;
  areaValue: number | bigint | null;
  sustained: number | null;
  basicSave: number | null;
  disableText: string | null;
  disableSkillsJson: string | null;
  isComplex: number | null;
};

function toNullableNumber(value: number | bigint | null, context: string): number | null {
  return value == null ? null : toSqliteNumber(value, context);
}

export function buildOntologyExplorerEntityRecordSelectColumns(
  aliases: {
    record?: string;
    actor?: string;
    item?: string;
    spell?: string;
    includeActor?: boolean;
    includeItem?: boolean;
    includeSpell?: boolean;
  } = {},
): string[] {
  const recordAlias = aliases.record ?? "r";
  const actorAlias = aliases.actor ?? "a";
  const itemAlias = aliases.item ?? "i";
  const spellAlias = aliases.spell ?? "s";
  const includeActor = aliases.includeActor ?? true;
  const includeItem = aliases.includeItem ?? true;
  const includeSpell = aliases.includeSpell ?? true;
  const optionalColumn = (enabled: boolean, expression: string, alias: string): string =>
    enabled ? `${expression} AS ${alias}` : `NULL AS ${alias}`;

  return [
    `${recordAlias}.record_key AS recordKey`,
    `${recordAlias}.pack_name AS packName`,
    `${recordAlias}.name AS name`,
    `${recordAlias}.record_type AS type`,
    `${recordAlias}.category AS category`,
    `${recordAlias}.subcategory AS subcategory`,
    `${recordAlias}.document_type AS documentType`,
    `${recordAlias}.level AS level`,
    `${recordAlias}.rarity AS rarity`,
    `${recordAlias}.traits_json AS traitsJson`,
    `${recordAlias}.derived_tags_json AS derivedTagsJson`,
    `${recordAlias}.families_json AS familiesJson`,
    `${recordAlias}.description_text AS descriptionText`,
    `${recordAlias}.blurb_text AS blurbText`,
    `${recordAlias}.source_category AS sourceCategory`,
    `${recordAlias}.publication_title AS publicationTitle`,
    `${recordAlias}.publication_remaster AS publicationRemaster`,
    `${recordAlias}.is_unique AS isUnique`,
    optionalColumn(includeActor, `${actorAlias}.size`, "size"),
    optionalColumn(includeActor, `${actorAlias}.languages_json`, "languagesJson"),
    optionalColumn(includeActor, `${actorAlias}.speed_types_json`, "speedTypesJson"),
    optionalColumn(includeActor, `${actorAlias}.senses_json`, "sensesJson"),
    optionalColumn(includeActor, `${actorAlias}.immunities_json`, "immunitiesJson"),
    optionalColumn(includeActor, `${actorAlias}.resistances_json`, "resistancesJson"),
    optionalColumn(includeActor, `${actorAlias}.weaknesses_json`, "weaknessesJson"),
    optionalColumn(includeActor, `${actorAlias}.disable_text`, "disableText"),
    optionalColumn(includeActor, `${actorAlias}.disable_skills_json`, "disableSkillsJson"),
    optionalColumn(includeActor, `${actorAlias}.is_complex`, "isComplex"),
    optionalColumn(includeItem, `${itemAlias}.item_category`, "itemCategory"),
    optionalColumn(includeItem, `${itemAlias}.base_item`, "baseItem"),
    optionalColumn(includeItem, `${itemAlias}.price_cp`, "priceCp"),
    optionalColumn(includeItem, `${itemAlias}.usage_text`, "usage"),
    optionalColumn(includeItem, `${itemAlias}.hands`, "hands"),
    optionalColumn(includeItem, `${itemAlias}.damage_types_json`, "damageTypesJson"),
    optionalColumn(includeItem, `${itemAlias}.weapon_group`, "weaponGroup"),
    optionalColumn(includeItem, `${itemAlias}.armor_group`, "armorGroup"),
    optionalColumn(includeSpell, `${spellAlias}.traditions_json`, "traditionsJson"),
    optionalColumn(includeSpell, `${spellAlias}.spell_kinds_json`, "spellKindsJson"),
    optionalColumn(includeSpell, `${spellAlias}.save_type`, "saveType"),
    optionalColumn(includeSpell, `${spellAlias}.area_type`, "areaType"),
    optionalColumn(includeSpell, `${spellAlias}.range_text`, "rangeText"),
    optionalColumn(includeSpell, `${spellAlias}.duration_text`, "durationText"),
    optionalColumn(includeSpell, `${spellAlias}.target_text`, "targetText"),
    optionalColumn(includeSpell, `${spellAlias}.area_value`, "areaValue"),
    optionalColumn(includeSpell, `${spellAlias}.sustained`, "sustained"),
    optionalColumn(includeSpell, `${spellAlias}.basic_save`, "basicSave"),
  ];
}

export function mapOntologyExplorerEntityRecordRow(row: OntologyExplorerEntityRecordRow): OntologyExplorerEntityRecord {
  const category = parseSearchCategoryValue(row.category, `ontology explorer record "${row.recordKey}"`);
  return {
    recordKey: row.recordKey,
    packName: row.packName ?? row.recordKey.split(":")[0] ?? "",
    name: row.name,
    type: row.type,
    category,
    subcategory: parseSearchSubcategoryForCategory(
      category,
      row.subcategory,
      `ontology explorer record "${row.recordKey}"`,
    ),
    documentType: row.documentType,
    level: toNullableNumber(row.level, `ontology explorer level for "${row.recordKey}"`),
    rarity: row.rarity,
    traits: parseStringArrayJson(row.traitsJson, "traitsJson", `ontology explorer record "${row.recordKey}"`),
    derivedTags: parseStringArrayJson(
      row.derivedTagsJson,
      "derivedTagsJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    families: parseStringArrayJson(row.familiesJson, "familiesJson", `ontology explorer record "${row.recordKey}"`),
    descriptionText: row.descriptionText,
    blurbText: row.blurbText,
    sourceCategory: parseSourceCategoryValue(row.sourceCategory, `ontology explorer record "${row.recordKey}"`),
    publicationTitle: row.publicationTitle,
    publicationRemaster: Boolean(row.publicationRemaster),
    isUnique: Boolean(row.isUnique),
    size: row.size,
    languages: parseStringArrayJson(row.languagesJson, "languagesJson", `ontology explorer record "${row.recordKey}"`),
    speedTypes: parseStringArrayJson(
      row.speedTypesJson,
      "speedTypesJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    senses: parseStringArrayJson(row.sensesJson, "sensesJson", `ontology explorer record "${row.recordKey}"`),
    immunities: parseStringArrayJson(
      row.immunitiesJson,
      "immunitiesJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    resistances: parseStringArrayJson(
      row.resistancesJson,
      "resistancesJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    weaknesses: parseStringArrayJson(
      row.weaknessesJson,
      "weaknessesJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    itemCategory: row.itemCategory,
    baseItem: row.baseItem,
    priceCp: toNullableNumber(row.priceCp, `ontology explorer price for "${row.recordKey}"`),
    usage: row.usage,
    hands: toNullableNumber(row.hands, `ontology explorer hands for "${row.recordKey}"`),
    damageTypes: parseStringArrayJson(
      row.damageTypesJson,
      "damageTypesJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    weaponGroup: row.weaponGroup,
    armorGroup: row.armorGroup,
    traditions: parseStringArrayJson(
      row.traditionsJson,
      "traditionsJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    spellKinds: parseStringArrayJson(
      row.spellKindsJson,
      "spellKindsJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    saveType: row.saveType,
    areaType: row.areaType,
    rangeText: row.rangeText,
    durationText: row.durationText,
    targetText: row.targetText,
    areaValue: toNullableNumber(row.areaValue, `ontology explorer area value for "${row.recordKey}"`),
    sustained: Boolean(row.sustained),
    basicSave: Boolean(row.basicSave),
    disableText: row.disableText,
    disableSkills: parseStringArrayJson(
      row.disableSkillsJson,
      "disableSkillsJson",
      `ontology explorer record "${row.recordKey}"`,
    ),
    isComplex: Boolean(row.isComplex),
  };
}

export function mapNormalizedRecordToOntologyExplorerEntityRecord(
  record: NormalizedRecord,
): OntologyExplorerEntityRecord {
  return {
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
  };
}
