import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import type { PackInfo } from "../../domain/record-types.js";
import type { RecordAliasRow, RecordLegacyLinkRow } from "../index-types.js";
import { INDEX_SCHEMA_VERSION } from "../schema.js";

function metricNamespacePrefixExpression(alias: string): string {
  return `CASE WHEN instr(${alias}.metric_key, '.') > 0 THEN substr(${alias}.metric_key, 1, instr(${alias}.metric_key, '.')) ELSE '' END`;
}

export function writeIndexMetadata(
  db: DatabaseSync,
  sourceSignature: string,
  embeddingProvider: EmbeddingProvider,
): void {
  const insertMetadata = db.prepare(`
    INSERT INTO metadata (key, value) VALUES (?, ?)
  `);

  insertMetadata.run("schema_version", String(INDEX_SCHEMA_VERSION));
  insertMetadata.run("source_signature", sourceSignature);
  insertMetadata.run("embedding_provider", embeddingProvider.identity.provider);
  insertMetadata.run("embedding_model", embeddingProvider.identity.model);
  insertMetadata.run("embedding_revision", embeddingProvider.identity.revision ?? "");
  insertMetadata.run("embedding_dimensions", String(embeddingProvider.identity.dimensions));
}

export function writeIndexPacks(db: DatabaseSync, packs: PackInfo[]): void {
  const insertPack = db.prepare(`
    INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
    VALUES (?, ?, ?, ?, ?, ?)
  `);

  for (const pack of packs) {
    insertPack.run(pack.name, pack.label, pack.documentType, pack.declaredPath, pack.resolvedPath, pack.recordCount);
  }
}

export function writeAliasCatalogRows(input: {
  db: DatabaseSync;
  aliasRows: RecordAliasRow[];
  legacyLinkRows: RecordLegacyLinkRow[];
}): void {
  const { db, aliasRows, legacyLinkRows } = input;
  const insertAlias = db.prepare(`
    INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
    VALUES (?, ?, ?, ?, ?)
  `);
  const insertLegacyLink = db.prepare(`
    INSERT INTO record_legacy_links (canonical_record_key, legacy_record_key, source_kind, source_ref)
    VALUES (?, ?, ?, ?)
  `);

  for (const alias of aliasRows) {
    insertAlias.run(
      alias.canonicalRecordKey,
      alias.aliasText,
      alias.normalizedAlias,
      alias.sourceKind,
      alias.sourceRef,
    );
  }

  for (const legacyLink of legacyLinkRows) {
    insertLegacyLink.run(
      legacyLink.canonicalRecordKey,
      legacyLink.legacyRecordKey,
      legacyLink.sourceKind,
      legacyLink.sourceRef,
    );
  }
}

export function populateMetricCatalog(db: DatabaseSync): void {
  for (const config of [
    { metricField: "actorMetrics", table: "actor_metrics", alias: "am" },
    { metricField: "itemMetrics", table: "item_metrics", alias: "im" },
  ] as const) {
    const namespacePrefix = metricNamespacePrefixExpression(config.alias);
    for (const subcategoryExpression of ["COALESCE(r.subcategory, '')", "'*'"]) {
      db.exec(`
        INSERT INTO metric_key_catalog (
          metric_field,
          category,
          subcategory,
          namespace_prefix,
          metric_key,
          value_type,
          catalog_count,
          numeric_min,
          numeric_max
        )
        SELECT
          '${config.metricField}',
          r.category,
          ${subcategoryExpression},
          ${namespacePrefix},
          ${config.alias}.metric_key,
          ${config.alias}.value_type,
          COUNT(*) AS catalog_count,
          CASE WHEN ${config.alias}.value_type = 'number' THEN MIN(${config.alias}.number_value) ELSE NULL END AS numeric_min,
          CASE WHEN ${config.alias}.value_type = 'number' THEN MAX(${config.alias}.number_value) ELSE NULL END AS numeric_max
        FROM ${config.table} ${config.alias}
        JOIN records r ON r.record_key = ${config.alias}.record_key
        WHERE r.is_search_canonical = 1
        GROUP BY
          r.category,
          ${subcategoryExpression},
          ${namespacePrefix},
          ${config.alias}.metric_key,
          ${config.alias}.value_type
      `);

      db.exec(`
        INSERT INTO metric_value_catalog (
          metric_field,
          category,
          subcategory,
          metric_key,
          value,
          catalog_count
        )
        SELECT
          '${config.metricField}',
          scoped.category,
          scoped.subcategory,
          scoped.metric_key,
          scoped.value,
          COUNT(*) AS catalog_count
        FROM (
          SELECT
            r.category AS category,
            ${subcategoryExpression} AS subcategory,
            ${config.alias}.metric_key AS metric_key,
            CASE
              WHEN ${config.alias}.value_type = 'text' THEN ${config.alias}.text_value
              WHEN ${config.alias}.value_type = 'boolean' AND ${config.alias}.bool_value = 1 THEN 'true'
              WHEN ${config.alias}.value_type = 'boolean' AND ${config.alias}.bool_value = 0 THEN 'false'
              ELSE NULL
            END AS value
          FROM ${config.table} ${config.alias}
          JOIN records r ON r.record_key = ${config.alias}.record_key
          WHERE r.is_search_canonical = 1
            AND ${config.alias}.value_type IN ('text', 'boolean')
        ) scoped
        WHERE scoped.value IS NOT NULL AND scoped.value <> ''
        GROUP BY scoped.category, scoped.subcategory, scoped.metric_key, scoped.value
      `);
    }
  }
}
