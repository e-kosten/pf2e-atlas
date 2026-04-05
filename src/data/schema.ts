import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import * as sqliteVec from "sqlite-vec";

import {
  DEFAULT_EMBEDDING_MODEL_ID,
  DEFAULT_EMBEDDING_REVISION,
  EmbeddingProvider,
} from "../embeddings.js";
import {
  EmbeddingConfig,
  LinkedRecordSummary,
  PackInfo,
} from "../types.js";
import { uniqueSorted } from "../utils.js";

export const INDEX_SCHEMA_VERSION = 21;

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

export function createSchema(db: DatabaseSync, embeddingDimensions: number): void {
  db.exec(`
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA temp_store = MEMORY;
    PRAGMA foreign_keys = ON;

    CREATE TABLE metadata (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    CREATE TABLE packs (
      name TEXT PRIMARY KEY,
      label TEXT NOT NULL,
      document_type TEXT NOT NULL,
      declared_path TEXT NOT NULL,
      resolved_path TEXT NOT NULL,
      record_count INTEGER NOT NULL
    );

    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      id TEXT NOT NULL,
      name TEXT NOT NULL,
      normalized_name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      pack_name TEXT NOT NULL,
      pack_label TEXT NOT NULL,
      document_type TEXT NOT NULL,
      record_type TEXT NOT NULL,
      level INTEGER,
      rarity TEXT,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT NOT NULL,
      publication_title TEXT,
      publication_remaster INTEGER NOT NULL,
      description_text TEXT,
      has_description INTEGER NOT NULL,
      description_snippet TEXT,
      source_category TEXT NOT NULL,
      folder_id TEXT,
      families_json TEXT NOT NULL,
      source_path TEXT NOT NULL,
      is_unique INTEGER NOT NULL,
      is_search_canonical INTEGER NOT NULL,
      search_text TEXT NOT NULL,
      raw_json TEXT NOT NULL
    );

    CREATE TABLE record_aliases (
      canonical_record_key TEXT NOT NULL,
      alias_text TEXT NOT NULL,
      normalized_alias TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL,
      PRIMARY KEY (canonical_record_key, normalized_alias, source_kind, source_ref),
      FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_legacy_links (
      canonical_record_key TEXT NOT NULL,
      legacy_record_key TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL,
      PRIMARY KEY (canonical_record_key, legacy_record_key, source_kind, source_ref),
      FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
      FOREIGN KEY (legacy_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_traits (
      record_key TEXT NOT NULL,
      trait TEXT NOT NULL,
      PRIMARY KEY (record_key, trait),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL,
      PRIMARY KEY (record_key, tag),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE actor_records (
      record_key TEXT PRIMARY KEY,
      size TEXT,
      languages_json TEXT NOT NULL,
      speed_types_json TEXT NOT NULL,
      senses_json TEXT NOT NULL,
      immunities_json TEXT NOT NULL,
      resistances_json TEXT NOT NULL,
      weaknesses_json TEXT NOT NULL,
      disable_text TEXT,
      disable_skills_json TEXT NOT NULL,
      is_complex INTEGER NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE actor_metrics (
      record_key TEXT NOT NULL,
      metric_key TEXT NOT NULL,
      value_type TEXT NOT NULL,
      number_value REAL,
      text_value TEXT,
      bool_value INTEGER,
      PRIMARY KEY (record_key, metric_key),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE item_records (
      record_key TEXT PRIMARY KEY,
      item_category TEXT,
      price_cp INTEGER,
      bulk_value REAL,
      usage_text TEXT,
      hands INTEGER,
      damage_types_json TEXT NOT NULL,
      weapon_group TEXT,
      armor_group TEXT,
      action_cost INTEGER,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE item_metrics (
      record_key TEXT NOT NULL,
      metric_key TEXT NOT NULL,
      value_type TEXT NOT NULL,
      number_value REAL,
      text_value TEXT,
      bool_value INTEGER,
      PRIMARY KEY (record_key, metric_key),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE spell_records (
      record_key TEXT PRIMARY KEY,
      action_cost INTEGER,
      traditions_json TEXT NOT NULL,
      spell_kinds_json TEXT NOT NULL,
      range_text TEXT,
      range_value REAL,
      save_type TEXT,
      area_type TEXT,
      duration_text TEXT,
      duration_unit TEXT,
      target_text TEXT,
      area_value REAL,
      sustained INTEGER NOT NULL,
      basic_save INTEGER NOT NULL,
      damage_types_json TEXT NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE embeddings (
      record_key TEXT PRIMARY KEY,
      dimensions INTEGER NOT NULL,
      semantic_input_hash TEXT NOT NULL,
      vector_blob BLOB NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE VIRTUAL TABLE record_embeddings USING vec0(
      record_key TEXT PRIMARY KEY,
      embedding FLOAT[${embeddingDimensions}],
      category TEXT partition key,
      subcategory TEXT,
      pack_name TEXT,
      pack_label TEXT,
      document_type TEXT,
      record_type TEXT,
      level INTEGER,
      rarity TEXT,
      source_category TEXT,
      publication_title TEXT,
      publication_remaster INTEGER,
      has_description INTEGER,
      is_unique INTEGER,
      size TEXT,
      item_category TEXT,
      price_cp INTEGER,
      action_cost INTEGER
    );

    CREATE TABLE reference_edges (
      from_record_key TEXT NOT NULL,
      to_record_key TEXT NOT NULL,
      display_text TEXT,
      reference_text TEXT NOT NULL,
      from_pack_name TEXT NOT NULL,
      from_record_type TEXT NOT NULL,
      from_document_type TEXT NOT NULL,
      from_source_category TEXT NOT NULL,
      PRIMARY KEY (from_record_key, to_record_key, reference_text),
      FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE VIRTUAL TABLE records_fts USING fts5(
      record_key UNINDEXED,
      name,
      search_text
    );

    CREATE INDEX records_pack_idx ON records(pack_name);
    CREATE INDEX records_category_idx ON records(category);
    CREATE INDEX records_subcategory_idx ON records(subcategory);
    CREATE INDEX records_doc_type_idx ON records(document_type);
    CREATE INDEX records_record_type_idx ON records(record_type);
    CREATE INDEX records_level_idx ON records(level);
    CREATE INDEX records_rarity_idx ON records(rarity);
    CREATE INDEX records_unique_idx ON records(is_unique);
    CREATE INDEX records_publication_remaster_idx ON records(publication_remaster);
    CREATE INDEX records_search_canonical_idx ON records(is_search_canonical);
    CREATE INDEX records_has_description_idx ON records(has_description);
    CREATE INDEX records_source_category_idx ON records(source_category);
    CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias);
    CREATE INDEX record_legacy_links_canonical_idx ON record_legacy_links(canonical_record_key);
    CREATE INDEX record_traits_trait_idx ON record_traits(trait);
    CREATE INDEX record_derived_tags_tag_idx ON record_derived_tags(tag);
    CREATE INDEX actor_records_size_idx ON actor_records(size);
    CREATE INDEX actor_metrics_key_idx ON actor_metrics(metric_key);
    CREATE INDEX actor_metrics_number_idx ON actor_metrics(metric_key, number_value);
    CREATE INDEX actor_metrics_text_idx ON actor_metrics(metric_key, text_value);
    CREATE INDEX actor_metrics_bool_idx ON actor_metrics(metric_key, bool_value);
    CREATE INDEX item_records_category_idx ON item_records(item_category);
    CREATE INDEX item_records_price_idx ON item_records(price_cp);
    CREATE INDEX item_records_action_cost_idx ON item_records(action_cost);
    CREATE INDEX item_metrics_key_idx ON item_metrics(metric_key);
    CREATE INDEX item_metrics_number_idx ON item_metrics(metric_key, number_value);
    CREATE INDEX item_metrics_text_idx ON item_metrics(metric_key, text_value);
    CREATE INDEX item_metrics_bool_idx ON item_metrics(metric_key, bool_value);
    CREATE INDEX spell_records_action_cost_idx ON spell_records(action_cost);
    CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key);
    CREATE INDEX reference_edges_from_type_idx ON reference_edges(from_record_type);
    CREATE INDEX reference_edges_from_pack_idx ON reference_edges(from_pack_name);
    CREATE INDEX reference_edges_from_source_category_idx ON reference_edges(from_source_category);
  `);
}

export function defaultIndexPath(manifestPath: string): string {
  return path.join(os.tmpdir(), `pf2e-mcp-${hashText(manifestPath).toString(16)}.sqlite`);
}

export function loadPacksFromIndex(db: DatabaseSync): PackInfo[] {
  const rows = db
    .prepare(`
      SELECT
        name,
        label,
        document_type AS documentType,
        declared_path AS declaredPath,
        resolved_path AS resolvedPath,
        record_count AS recordCount
      FROM packs
      ORDER BY label ASC
    `)
    .all() as Array<Record<string, unknown>>;

  return rows.map((row) => ({
    name: String(row.name),
    label: String(row.label),
    documentType: String(row.documentType),
    declaredPath: String(row.declaredPath),
    resolvedPath: String(row.resolvedPath),
    recordCount: Number(row.recordCount ?? 0),
  }));
}

export function loadAliasesByRecordKey(db: DatabaseSync): Map<string, string[]> {
  const rows = db.prepare(`
    SELECT canonical_record_key AS canonicalRecordKey, alias_text AS aliasText
    FROM record_aliases
    ORDER BY canonical_record_key ASC, alias_text ASC
  `).all() as Array<{ canonicalRecordKey: string; aliasText: string }>;

  const aliasesByRecordKey = new Map<string, string[]>();
  for (const row of rows) {
    const bucket = aliasesByRecordKey.get(row.canonicalRecordKey) ?? [];
    bucket.push(row.aliasText);
    aliasesByRecordKey.set(row.canonicalRecordKey, uniqueSorted(bucket));
  }

  return aliasesByRecordKey;
}

export function loadLegacyLinksByRecordKey(db: DatabaseSync): Map<string, LinkedRecordSummary[]> {
  const rows = db.prepare(`
    SELECT
      rll.canonical_record_key AS canonicalRecordKey,
      rll.legacy_record_key AS legacyRecordKey,
      records.name AS legacyName
    FROM record_legacy_links rll
    JOIN records ON records.record_key = rll.legacy_record_key
    ORDER BY rll.canonical_record_key ASC, records.name ASC, rll.legacy_record_key ASC
  `).all() as Array<{ canonicalRecordKey: string; legacyRecordKey: string; legacyName: string }>;

  const linksByRecordKey = new Map<string, LinkedRecordSummary[]>();
  for (const row of rows) {
    const bucket = linksByRecordKey.get(row.canonicalRecordKey) ?? [];
    bucket.push({
      recordKey: row.legacyRecordKey,
      name: row.legacyName,
    });
    linksByRecordKey.set(row.canonicalRecordKey, bucket);
  }

  return linksByRecordKey;
}

export function readMetadata(db: DatabaseSync): Map<string, string> {
  const rows = db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>;
  return new Map(rows.map((row) => [row.key, row.value]));
}

export function canReuseIndex(db: DatabaseSync, sourceSignature: string, embeddingProvider: EmbeddingProvider): boolean {
  return getIndexInvalidReason(db, sourceSignature, embeddingProvider) === null;
}

export function getEmbeddingReuseInvalidReason(
  db: DatabaseSync,
  embeddingProvider: EmbeddingProvider,
): string | null {
  try {
    const metadata = readMetadata(db);
    if (metadata.get("schema_version") !== String(INDEX_SCHEMA_VERSION)) {
      return "index schema version does not match the current code";
    }
    if (metadata.get("embedding_provider") !== embeddingProvider.identity.provider) {
      return "embedding provider changed since the index was built";
    }
    if (metadata.get("embedding_model") !== embeddingProvider.identity.model) {
      return "embedding model changed since the index was built";
    }
    if (metadata.get("embedding_revision") !== (embeddingProvider.identity.revision ?? "")) {
      return "embedding model revision changed since the index was built";
    }
    if (metadata.get("embedding_dimensions") !== String(embeddingProvider.identity.dimensions)) {
      return "embedding dimensions changed since the index was built";
    }
    db.prepare("SELECT semantic_input_hash FROM embeddings LIMIT 1").get();
    return null;
  } catch {
    return "existing embedding cache metadata could not be read";
  }
}

export function getIndexInvalidReason(
  db: DatabaseSync,
  sourceSignature: string,
  embeddingProvider: EmbeddingProvider,
): string | null {
  try {
    const metadata = readMetadata(db);
    if (metadata.get("schema_version") !== String(INDEX_SCHEMA_VERSION)) {
      return "index schema version does not match the current code";
    }
    if (metadata.get("source_signature") !== sourceSignature) {
      return "PF2E source data changed since the index was built";
    }
    if (metadata.get("embedding_provider") !== embeddingProvider.identity.provider) {
      return "embedding provider changed since the index was built";
    }
    if (metadata.get("embedding_model") !== embeddingProvider.identity.model) {
      return "embedding model changed since the index was built";
    }
    if (metadata.get("embedding_revision") !== (embeddingProvider.identity.revision ?? "")) {
      return "embedding model revision changed since the index was built";
    }
    if (metadata.get("embedding_dimensions") !== String(embeddingProvider.identity.dimensions)) {
      return "embedding dimensions changed since the index was built";
    }
    return null;
  } catch {
    return "index metadata could not be read";
  }
}

export function loadRequiredVectorExtension(
  db: DatabaseSync,
  loader: (db: DatabaseSync) => void = sqliteVec.load,
): void {
  try {
    loader(db);
    db.enableLoadExtension(false);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to load required sqlite-vec extension. Fix the installation and retry startup. Underlying error: ${reason}`);
  }
}

export function openDatabase(indexPath: string, options: { vectorExtensionLoader?: (db: DatabaseSync) => void } = {}): DatabaseSync {
  const db = new DatabaseSync(indexPath, { allowExtension: true });
  loadRequiredVectorExtension(db, options.vectorExtensionLoader);
  return db;
}

export function defaultEmbeddingConfig(indexPath: string): EmbeddingConfig {
  return {
    provider: "hf-local",
    modelId: DEFAULT_EMBEDDING_MODEL_ID,
    modelRevision: DEFAULT_EMBEDDING_REVISION,
    cachePath: path.join(path.dirname(indexPath), "hf-models"),
    localModelPath: null,
  };
}

export function buildMissingIndexError(indexPath: string): Error {
  return new Error(
    `PF2E index not found at ${indexPath}. Run 'npm run refresh-index' or 'npm run refresh-external' before starting the MCP server.`,
  );
}

export function buildStaleIndexError(indexPath: string, reason: string): Error {
  return new Error(
    `PF2E index at ${indexPath} is stale: ${reason}. Run 'npm run refresh-index' or 'npm run refresh-external' before starting the MCP server.`,
  );
}
