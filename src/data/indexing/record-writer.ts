import type { DatabaseSync } from "node:sqlite";

import { normalizeText } from "../../shared/utils.js";
import { normalizeDerivedTag } from "../../tags/runtime.js";
import type { DerivedAfflictionBuild } from "../derived-afflictions.js";
import type { PendingCanonicalEmbeddingWithHash, WritableIndexEntry } from "../index-types.js";
import {
  createRecordWriteProgressCounter,
  reportRecordWriteProgress,
  reportRecordWritingStarted,
  type IndexingProgressReporter,
} from "./progress.js";
import { createSearchTextWriteArtifact } from "./search-text-writer.js";

export type RecordWritingStageResult = {
  pendingCanonicalEmbeddings: PendingCanonicalEmbeddingWithHash[];
  durationMs: number;
};

type MetricRowValue = {
  valueType: "number" | "text" | "boolean";
  numberValue: number | null;
  textValue: string | null;
  boolValue: number | null;
};

function normalizeMetricValue(metricValue: string | number | boolean): MetricRowValue {
  if (typeof metricValue === "number") {
    return {
      valueType: "number",
      numberValue: metricValue,
      textValue: null,
      boolValue: null,
    };
  }

  if (typeof metricValue === "boolean") {
    return {
      valueType: "boolean",
      numberValue: null,
      textValue: null,
      boolValue: metricValue ? 1 : 0,
    };
  }

  return {
    valueType: "text",
    numberValue: null,
    textValue: metricValue,
    boolValue: null,
  };
}

export function writeIndexRecords(
  db: DatabaseSync,
  writableEntries: WritableIndexEntry[],
  derivedAfflictions: DerivedAfflictionBuild,
  progress: IndexingProgressReporter = {},
): RecordWritingStageResult {
  const insertRecord = db.prepare(`
    INSERT INTO records (
      record_key, id, name, normalized_name, category, subcategory, pack_name, pack_label, document_type, record_type,
      level, rarity, traits_json, derived_tags_json, publication_title, publication_remaster, description_text, blurb_text, has_description, description_snippet,
      source_category, folder_id, families_json, variant_family_key, variant_base_name, variant_label, variant_axes_json, variant_confidence, variant_source,
      source_path, is_unique, is_search_canonical, search_text, raw_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertTrait = db.prepare(`
    INSERT INTO record_traits (record_key, trait) VALUES (?, ?)
  `);
  const insertDerivedTag = db.prepare(`
    INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)
  `);
  const insertActor = db.prepare(`
    INSERT INTO actor_records (
      record_key, size, languages_json, speed_types_json, senses_json, immunities_json, resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertActorMetric = db.prepare(`
    INSERT INTO actor_metrics (
      record_key, metric_key, value_type, number_value, text_value, bool_value
    ) VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertItem = db.prepare(`
    INSERT INTO item_records (
      record_key, item_category, base_item, price_cp, bulk_value, usage_text, hands, damage_types_json, weapon_group, armor_group, action_cost
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertItemMetric = db.prepare(`
    INSERT INTO item_metrics (
      record_key, metric_key, value_type, number_value, text_value, bool_value
    ) VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertSpell = db.prepare(`
    INSERT INTO spell_records (
      record_key, action_cost, traditions_json, spell_kinds_json, range_text, range_value, save_type, area_type, duration_text, duration_unit, target_text, area_value, sustained, basic_save, damage_types_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertReferenceEdge = db.prepare(`
    INSERT OR IGNORE INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertFts = db.prepare(`
    INSERT INTO records_fts (record_key, name, search_text) VALUES (?, ?, ?)
  `);

  const pendingCanonicalEmbeddings: PendingCanonicalEmbeddingWithHash[] = [];
  const recordWriteProgress = createRecordWriteProgressCounter(writableEntries.length);
  let writtenRecordCount = 0;
  const recordStorageStartTime = Date.now();

  reportRecordWritingStarted(progress);

  for (const entry of writableEntries) {
    const record = entry.record;
    const { searchText, pendingCanonicalEmbedding } = createSearchTextWriteArtifact(entry);

    insertRecord.run(
      record.recordKey,
      record.id,
      record.name,
      record.normalizedName,
      record.category,
      record.subcategory,
      record.packName,
      record.packLabel,
      record.documentType,
      record.type,
      record.level,
      record.rarity,
      JSON.stringify(record.traits),
      JSON.stringify(record.derivedTags),
      record.publicationTitle,
      record.publicationRemaster ? 1 : 0,
      record.descriptionText,
      record.blurbText,
      record.hasDescription ? 1 : 0,
      record.descriptionSnippet,
      record.sourceCategory,
      record.folderId,
      JSON.stringify(record.families),
      record.variantFamilyKey,
      record.variantBaseName,
      record.variantLabel,
      JSON.stringify(record.variantAxes),
      record.variantConfidence,
      record.variantSource,
      record.sourcePath,
      record.isUnique ? 1 : 0,
      entry.isSearchCanonical ? 1 : 0,
      searchText,
      JSON.stringify(entry.raw),
    );

    for (const trait of record.traits) {
      insertTrait.run(record.recordKey, normalizeText(trait));
    }

    for (const tag of record.derivedTags) {
      insertDerivedTag.run(record.recordKey, normalizeDerivedTag(tag));
    }

    if (entry.actorData) {
      insertActor.run(
        record.recordKey,
        entry.actorData.size,
        JSON.stringify(entry.actorData.languages),
        JSON.stringify(entry.actorData.speedTypes),
        JSON.stringify(entry.actorData.senses),
        JSON.stringify(entry.actorData.immunities),
        JSON.stringify(entry.actorData.resistances),
        JSON.stringify(entry.actorData.weaknesses),
        entry.actorData.disableText,
        JSON.stringify(entry.actorData.disableSkills),
        entry.actorData.isComplex ? 1 : 0,
      );

      for (const [metricKey, metricValue] of Object.entries(entry.actorData.actorMetrics)) {
        const normalizedValue = normalizeMetricValue(metricValue);
        insertActorMetric.run(
          record.recordKey,
          metricKey,
          normalizedValue.valueType,
          normalizedValue.numberValue,
          normalizedValue.textValue,
          normalizedValue.boolValue,
        );
      }
    }

    if (entry.itemData) {
      insertItem.run(
        record.recordKey,
        entry.itemData.itemCategory,
        entry.itemData.baseItem,
        entry.itemData.priceCp,
        entry.itemData.bulkValue,
        entry.itemData.usage,
        entry.itemData.hands,
        JSON.stringify(entry.itemData.damageTypes),
        entry.itemData.weaponGroup,
        entry.itemData.armorGroup,
        entry.itemData.actionCost,
      );

      for (const [metricKey, metricValue] of Object.entries(entry.itemData.itemMetrics)) {
        const normalizedValue = normalizeMetricValue(metricValue);
        insertItemMetric.run(
          record.recordKey,
          metricKey,
          normalizedValue.valueType,
          normalizedValue.numberValue,
          normalizedValue.textValue,
          normalizedValue.boolValue,
        );
      }
    }

    if (entry.spellData) {
      insertSpell.run(
        record.recordKey,
        entry.spellData.actionCost,
        JSON.stringify(entry.spellData.traditions),
        JSON.stringify(entry.spellData.spellKinds),
        entry.spellData.rangeText,
        entry.spellData.rangeValue,
        entry.spellData.saveType,
        entry.spellData.areaType,
        entry.spellData.durationText,
        entry.spellData.durationUnit,
        entry.spellData.targetText,
        entry.spellData.areaValue,
        entry.spellData.sustained ? 1 : 0,
        entry.spellData.basicSave ? 1 : 0,
        JSON.stringify(entry.spellData.damageTypes),
      );
    }

    for (const reference of entry.resolvedReferences) {
      insertReferenceEdge.run(
        record.recordKey,
        reference.targetRecordKey,
        reference.displayText,
        reference.referenceText,
        record.packName,
        record.type,
        record.documentType,
        record.sourceCategory,
      );
    }

    if (pendingCanonicalEmbedding) {
      insertFts.run(record.recordKey, record.name, searchText);
      pendingCanonicalEmbeddings.push(pendingCanonicalEmbedding);
    }

    writtenRecordCount += 1;
    if (recordWriteProgress.shouldReport(writtenRecordCount)) {
      reportRecordWriteProgress(progress, {
        writtenRecords: writtenRecordCount,
        totalRecords: writableEntries.length,
      });
    }
  }

  for (const edge of derivedAfflictions.edges) {
    insertReferenceEdge.run(
      edge.fromRecordKey,
      edge.toRecordKey,
      edge.displayText,
      edge.referenceText,
      edge.fromPackName,
      edge.fromRecordType,
      edge.fromDocumentType,
      edge.fromSourceCategory,
    );
  }

  return {
    pendingCanonicalEmbeddings,
    durationMs: Date.now() - recordStorageStartTime,
  };
}
