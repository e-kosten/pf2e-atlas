import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { readdir, readFile, rm, stat } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { promisify } from "node:util";

import { EmbeddingProvider } from "../embeddings.js";
import { pathExists } from "../shared/fs.js";
import {
  deriveRecordTags,
  getVariantInheritableTags,
  normalizeDerivedTag,
  validateConfiguredDerivedTagAssignments,
} from "../tags/runtime.js";
import type { PackInfo, PackManifestEntry } from "../domain/record-types.js";
import { normalizeText, uniqueSorted } from "../shared/utils.js";
import { formatInteger } from "./format.js";
import {
  ActorIndexData,
  BuildIndexResult,
  BuildSourceEntry,
  ItemIndexData,
  NormalizedIndexRecord,
  PackBuildInfo,
  PendingCanonicalEmbeddingWithHash,
  ResolvedBuildReference,
  SpellIndexData,
} from "./index-types.js";
import { INDEX_SCHEMA_VERSION } from "./schema.js";
import {
  buildSemanticEmbeddingText,
  isExcludedPackName,
  normalizeIndexRecord,
  parseActorIndexData,
  parseItemIndexData,
  parseSpellIndexData,
  shouldExcludeRecordFromIndex,
} from "./record-normalization.js";
import { buildDerivedAfflictionArtifacts } from "./derived-afflictions.js";
import { applyVariantBaseTagInheritance } from "./variant-tag-inheritance.js";
import { assignVariantFamilies } from "./variant-families.js";
import { extractRulesReferences, resolveBuildReferencesAndAliases } from "./references.js";

const execFileAsync = promisify(execFile);
const VEC_TEXT_NONE = "";
const VEC_INT_NONE = -1n;
const EMBEDDING_BATCH_SIZE = 64;
const PACK_PROGRESS_BAR_WIDTH = 24;
const PACK_PROGRESS_LOG_INTERVAL_MS = 5_000;
const RESOLUTION_PROGRESS_BAR_WIDTH = 24;

type WritableIndexEntry = {
  record: NormalizedIndexRecord;
  raw: Record<string, unknown>;
  actorData: ActorIndexData | null;
  itemData: ItemIndexData | null;
  spellData: SpellIndexData | null;
  resolvedReferences: ResolvedBuildReference[];
  aliasTexts: string[];
  isSearchCanonical: boolean;
};

type ReusableEmbeddingRow = {
  semanticInputHash: string;
  dimensions: number;
  vectorBlob: Uint8Array;
};

type ReusableEmbeddingLookup = {
  get(recordKey: string): ReusableEmbeddingRow | null;
};

function metricNamespacePrefixExpression(alias: string): string {
  return `CASE WHEN instr(${alias}.metric_key, '.') > 0 THEN substr(${alias}.metric_key, 1, instr(${alias}.metric_key, '.')) ELSE '' END`;
}

function populateMetricCatalog(db: DatabaseSync): void {
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

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function normalizeVecText(value: string | null | undefined): string {
  return normalizeText(value ?? "") || VEC_TEXT_NONE;
}

function normalizeVecInteger(value: number | null | undefined): bigint {
  return value === null || value === undefined ? VEC_INT_NONE : BigInt(value);
}

function hashSemanticInput(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

function isJsonRecord(filename: string): boolean {
  return filename.endsWith(".json") && filename !== "_folders.json";
}

async function walkJsonFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(root, entry.name);
      if (entry.isDirectory()) {
        return walkJsonFiles(entryPath);
      }

      if (entry.isFile() && isJsonRecord(entry.name)) {
        return [entryPath];
      }

      return [];
    }),
  );

  return files.flat();
}

async function directoryExists(targetPath: string): Promise<boolean> {
  try {
    const details = await stat(targetPath);
    return details.isDirectory();
  } catch {
    return false;
  }
}

function renderProgressBar(completed: number, total: number, width = PACK_PROGRESS_BAR_WIDTH): string {
  if (total <= 0) {
    return `[${"-".repeat(width)}]`;
  }

  const boundedCompleted = Math.max(0, Math.min(completed, total));
  const filled = Math.max(0, Math.min(width, Math.round((boundedCompleted / total) * width)));
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

function formatPercentage(completed: number, total: number): string {
  if (total <= 0) {
    return "  0%";
  }

  return `${Math.round((Math.max(0, Math.min(completed, total)) / total) * 100)}`.padStart(3, " ") + "%";
}

function shouldLogProgressUpdate(
  processed: number,
  total: number,
  lastLoggedProcessed: number,
  lastLoggedTime: number,
  interval: number,
  now: number,
): boolean {
  return (
    processed === total ||
    processed - lastLoggedProcessed >= interval ||
    now - lastLoggedTime >= PACK_PROGRESS_LOG_INTERVAL_MS
  );
}

async function isGitCheckout(rootPath: string): Promise<boolean> {
  return pathExists(path.join(rootPath, ".git"));
}

async function computeFileSignature(rootPath: string, filePaths: string[]): Promise<string> {
  const files = [...new Set(filePaths)].sort((left, right) => left.localeCompare(right));
  let hash = 2166136261;
  for (const filePath of files) {
    const details = await stat(filePath);
    const value = `${path.relative(rootPath, filePath)}:${details.size}:${Math.trunc(details.mtimeMs)}`;
    hash ^= hashText(value);
    hash = Math.imul(hash, 16777619);
  }

  return String(hash >>> 0);
}

export async function computeSourceSignature(rootPath: string, manifestPath: string): Promise<string> {
  if (await isGitCheckout(rootPath)) {
    try {
      const [{ stdout: headStdout }, { stdout: statusStdout }, { stdout: untrackedStdout }] = await Promise.all([
        execFileAsync("git", ["-C", rootPath, "rev-parse", "HEAD"], { timeout: 10_000 }),
        execFileAsync("git", ["-C", rootPath, "status", "--porcelain", "--untracked-files=no"], { timeout: 10_000 }),
        execFileAsync(
          "git",
          [
            "-C",
            rootPath,
            "ls-files",
            "--others",
            "--exclude-standard",
            "--full-name",
            "--",
            "*.json",
            ":(glob)**/*.json",
          ],
          { timeout: 10_000 },
        ),
      ]);
      const head = headStdout.trim();
      const dirty = statusStdout.trim();
      const untrackedJsonFiles = untrackedStdout
        .split(/\r?\n/)
        .map((filePath) => filePath.trim())
        .filter(Boolean)
        .map((filePath) => path.join(rootPath, filePath));
      const untrackedJsonSignature = await computeFileSignature(rootPath, untrackedJsonFiles);
      return `git:${head}:${dirty}:${untrackedJsonSignature}`;
    } catch {
      // Fall through to filesystem signature.
    }
  }

  const files = [manifestPath, ...(await walkJsonFiles(rootPath))];
  return `fs:${await computeFileSignature(rootPath, files)}`;
}

export async function removeIndexFiles(indexPath: string): Promise<void> {
  await rm(indexPath, { force: true });
  await rm(`${indexPath}-wal`, { force: true });
  await rm(`${indexPath}-shm`, { force: true });
}

async function resolvePackPath(rootPath: string, pack: PackManifestEntry): Promise<string | null> {
  const candidates = [
    path.join(rootPath, pack.path),
    path.join(rootPath, "packs", "pf2e", pack.name),
    path.join(rootPath, pack.path.replace(/^packs\//, "packs/pf2e/")),
  ];

  for (const candidate of candidates) {
    const normalized = path.normalize(candidate);
    if (await directoryExists(normalized)) {
      return normalized;
    }
  }

  return null;
}

export function buildReusableEmbeddingLookup(db: DatabaseSync): ReusableEmbeddingLookup {
  const selectReusableEmbedding = db.prepare(`
    SELECT
      semantic_input_hash AS semanticInputHash,
      dimensions,
      vector_blob AS vectorBlob
    FROM embeddings
    WHERE record_key = ?
  `);

  return {
    get(recordKey: string): ReusableEmbeddingRow | null {
      return selectReusableEmbedding.get(recordKey) as ReusableEmbeddingRow | null;
    },
  };
}

export async function buildIndex(
  db: DatabaseSync,
  rootPath: string,
  manifestPath: string,
  embeddingProvider: EmbeddingProvider,
  sourceSignature: string,
  progressLogger?: (message: string) => void,
  progressStatusLogger?: (message: string) => void,
  reusableEmbeddingLookup?: ReusableEmbeddingLookup | null,
): Promise<BuildIndexResult> {
  const manifestRaw = JSON.parse(await readFile(manifestPath, "utf8")) as { packs?: PackManifestEntry[] };
  const manifestPacks = Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];
  const includedManifestPacks = manifestPacks.filter((manifestPack) => !isExcludedPackName(manifestPack.name));

  const warnings: string[] = [];
  const packs: PackInfo[] = [];
  let recordCount = 0;
  let processedPackCount = 0;
  const sourceEntries: BuildSourceEntry[] = [];
  const scanStartTime = Date.now();
  let scanNormalizationDurationMs: number;
  let resolutionDurationMs: number;
  let recordStorageDurationMs: number;
  let embeddingGenerationDurationMs = 0;
  let vecInsertDurationMs = 0;
  let reusedCanonicalEmbeddingCount = 0;
  let regeneratedCanonicalEmbeddingCount = 0;

  const insertPack = db.prepare(`
    INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
    VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertRecord = db.prepare(`
    INSERT INTO records (
      record_key, id, name, normalized_name, category, subcategory, pack_name, pack_label, document_type, record_type,
      level, rarity, traits_json, derived_tags_json, publication_title, publication_remaster, description_text, blurb_text, has_description, description_snippet,
      source_category, folder_id, families_json, variant_family_key, variant_base_name, variant_label, variant_axes_json, variant_confidence, variant_source,
      source_path, is_unique, is_search_canonical, search_text, raw_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertAlias = db.prepare(`
    INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
    VALUES (?, ?, ?, ?, ?)
  `);
  const insertLegacyLink = db.prepare(`
    INSERT INTO record_legacy_links (canonical_record_key, legacy_record_key, source_kind, source_ref)
    VALUES (?, ?, ?, ?)
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
  const insertEmbedding = db.prepare(`
    INSERT INTO embeddings (record_key, dimensions, semantic_input_hash, vector_blob) VALUES (?, ?, ?, ?)
  `);
  const insertVecEmbedding = db.prepare(`
    INSERT INTO record_embeddings (
      record_key,
      embedding,
      category,
      subcategory,
      pack_name,
      pack_label,
      document_type,
      record_type,
      level,
      rarity,
      source_category,
      publication_title,
      publication_remaster,
      has_description,
      is_unique,
      size,
      item_category,
      price_cp,
      action_cost
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertReferenceEdge = db.prepare(`
    INSERT OR IGNORE INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertFts = db.prepare(`
    INSERT INTO records_fts (record_key, name, search_text) VALUES (?, ?, ?)
  `);
  const insertMetadata = db.prepare(`
    INSERT INTO metadata (key, value) VALUES (?, ?)
  `);

  db.exec("BEGIN");
  try {
    progressLogger?.(`Building SQLite index from ${formatInteger(includedManifestPacks.length)} PF2E packs.`);
    insertMetadata.run("schema_version", String(INDEX_SCHEMA_VERSION));
    insertMetadata.run("source_signature", sourceSignature);
    insertMetadata.run("embedding_provider", embeddingProvider.identity.provider);
    insertMetadata.run("embedding_model", embeddingProvider.identity.model);
    insertMetadata.run("embedding_revision", embeddingProvider.identity.revision ?? "");
    insertMetadata.run("embedding_dimensions", String(embeddingProvider.identity.dimensions));

    for (const manifestPack of manifestPacks) {
      const resolvedPath = await resolvePackPath(rootPath, manifestPack);
      if (!resolvedPath) {
        warnings.push(`Skipping pack ${manifestPack.name}: could not resolve a readable directory.`);
        continue;
      }

      const pack: PackBuildInfo = {
        name: manifestPack.name,
        label: manifestPack.label,
        documentType: manifestPack.type,
        declaredPath: manifestPack.path,
        resolvedPath,
      };

      if (isExcludedPackName(pack.name)) {
        continue;
      }

      processedPackCount += 1;

      let filePaths: string[];
      try {
        filePaths = await walkJsonFiles(pack.resolvedPath);
      } catch (error) {
        warnings.push(`Skipping pack ${pack.name}: ${(error as Error).message}`);
        continue;
      }

      let packRecordCount = 0;
      const progressInterval = Math.max(100, Math.ceil(filePaths.length / 10));
      let lastProgressLogTime = 0;
      let lastLoggedFileCount = 0;

      for (const [fileIndex, filePath] of filePaths.entries()) {
        const raw = JSON.parse(await readFile(filePath, "utf8")) as Record<string, unknown>;
        const shouldIndexRecord = !shouldExcludeRecordFromIndex(pack, filePath, raw);
        if (!shouldIndexRecord) {
          sourceEntries.push({
            pack,
            filePath,
            raw,
            record: null,
            actorData: null,
            itemData: null,
            spellData: null,
            references: [],
            resolvedReferences: [],
          });
          const processedFiles = fileIndex + 1;
          const now = Date.now();
          const shouldLogProgress =
            processedFiles === filePaths.length ||
            processedFiles - lastLoggedFileCount >= progressInterval ||
            now - lastProgressLogTime >= PACK_PROGRESS_LOG_INTERVAL_MS;

          if (shouldLogProgress) {
            progressStatusLogger?.(
              `[scan ${processedPackCount}/${includedManifestPacks.length}] ${pack.label} ${renderProgressBar(processedFiles, filePaths.length)} ${formatPercentage(processedFiles, filePaths.length)} (${formatInteger(processedFiles)}/${formatInteger(filePaths.length)} files, ${formatInteger(recordCount)} records discovered total).`,
            );
            lastProgressLogTime = now;
            lastLoggedFileCount = processedFiles;
          }
          continue;
        }

        const record = normalizeIndexRecord(pack, filePath, raw);
        sourceEntries.push({
          pack,
          filePath,
          raw,
          record,
          actorData: pack.documentType === "Actor" ? parseActorIndexData(raw) : null,
          itemData: pack.documentType === "Item" ? parseItemIndexData(raw) : null,
          spellData: record.type === "spell" ? parseSpellIndexData(raw) : null,
          references: extractRulesReferences(raw),
          resolvedReferences: [],
        });
        packRecordCount += 1;
        recordCount += 1;

        const processedFiles = fileIndex + 1;
        const now = Date.now();
        const shouldLogProgress =
          processedFiles === 1 ||
          processedFiles === filePaths.length ||
          processedFiles - lastLoggedFileCount >= progressInterval ||
          now - lastProgressLogTime >= PACK_PROGRESS_LOG_INTERVAL_MS;

        if (shouldLogProgress) {
          progressStatusLogger?.(
            `[scan ${processedPackCount}/${includedManifestPacks.length}] ${pack.label} ${renderProgressBar(processedFiles, filePaths.length)} ${formatPercentage(processedFiles, filePaths.length)} (${formatInteger(processedFiles)}/${formatInteger(filePaths.length)} files, ${formatInteger(recordCount)} records discovered total).`,
          );
          lastProgressLogTime = now;
          lastLoggedFileCount = processedFiles;
        }
      }

      if (packRecordCount === 0) {
        continue;
      }

      packs.push({ ...pack, recordCount: packRecordCount });
      insertPack.run(pack.name, pack.label, pack.documentType, pack.declaredPath, pack.resolvedPath, packRecordCount);
    }

    scanNormalizationDurationMs = Date.now() - scanStartTime;

    progressLogger?.("Finished scanning pack files. Resolving verified remaster aliases.");
    const resolutionStartTime = Date.now();

    const indexedEntries = sourceEntries.filter(
      (entry): entry is BuildSourceEntry & { record: NonNullable<BuildSourceEntry["record"]> } => entry.record !== null,
    );
    assignVariantFamilies(indexedEntries);
    validateConfiguredDerivedTagAssignments(
      indexedEntries.map((entry) => ({
        recordKey: entry.record.recordKey,
        name: entry.record.name,
        category: entry.record.category,
      })),
    );
    const { aliasRows, legacyLinkRows } = await resolveBuildReferencesAndAliases({
      indexedEntries,
      sourceEntries,
      packs,
      rootPath,
    });

    const derivedAfflictions = buildDerivedAfflictionArtifacts(indexedEntries);
    const canonicalDerivedAfflictions = derivedAfflictions.records.filter((entry) => entry.isSearchCanonical);
    const totalResolutionRecords = indexedEntries.length + canonicalDerivedAfflictions.length;
    const resolutionProgressInterval = Math.max(100, Math.ceil(Math.max(totalResolutionRecords, 1) / 20));
    let resolvedRecordCount = 0;
    let lastLoggedResolvedRecordCount = 0;
    let lastResolutionProgressLogTime = Date.now();

    const logResolutionProgress = (): void => {
      if (totalResolutionRecords <= 0) {
        return;
      }

      progressStatusLogger?.(
        `[resolve] Derived tags ${renderProgressBar(resolvedRecordCount, totalResolutionRecords, RESOLUTION_PROGRESS_BAR_WIDTH)} ${formatPercentage(resolvedRecordCount, totalResolutionRecords)} (${formatInteger(resolvedRecordCount)}/${formatInteger(totalResolutionRecords)} records).`,
      );
    };

    progressLogger?.(
      `Finished resolving aliases and references. Deriving tags for ${formatInteger(totalResolutionRecords)} records.`,
    );

    for (const entry of indexedEntries) {
      entry.record.derivedTags = deriveRecordTags({
        recordKey: entry.record.recordKey,
        name: entry.record.name,
        category: entry.record.category,
        subcategory: entry.record.subcategory,
        descriptionText: entry.record.descriptionText,
        blurbText: entry.record.blurbText,
        traits: entry.record.traits,
        families: entry.record.families,
        references: entry.resolvedReferences.map((reference) => ({
          recordKey: reference.targetRecordKey,
          packName: reference.targetRecord.packName,
          name: reference.targetRecord.name,
          category: reference.targetRecord.category,
          subcategory: reference.targetRecord.subcategory,
          traits: reference.targetRecord.traits,
        })),
      });

      resolvedRecordCount += 1;
      const now = Date.now();
      if (
        shouldLogProgressUpdate(
          resolvedRecordCount,
          totalResolutionRecords,
          lastLoggedResolvedRecordCount,
          lastResolutionProgressLogTime,
          resolutionProgressInterval,
          now,
        )
      ) {
        logResolutionProgress();
        lastLoggedResolvedRecordCount = resolvedRecordCount;
        lastResolutionProgressLogTime = now;
      }
    }

    for (const entry of canonicalDerivedAfflictions) {
      entry.record.derivedTags = deriveRecordTags({
        recordKey: entry.record.recordKey,
        name: entry.record.name,
        category: entry.record.category,
        subcategory: entry.record.subcategory,
        // Canonical derived afflictions can have empty descriptions even when their
        // linked staged-condition text is preserved in searchText.
        descriptionText: entry.record.descriptionText ?? entry.record.searchText,
        blurbText: entry.record.blurbText,
        traits: entry.record.traits,
        families: entry.record.families,
        references: [],
      });

      resolvedRecordCount += 1;
      const now = Date.now();
      if (
        shouldLogProgressUpdate(
          resolvedRecordCount,
          totalResolutionRecords,
          lastLoggedResolvedRecordCount,
          lastResolutionProgressLogTime,
          resolutionProgressInterval,
          now,
        )
      ) {
        logResolutionProgress();
        lastLoggedResolvedRecordCount = resolvedRecordCount;
        lastResolutionProgressLogTime = now;
      }
    }

    const creatureVariantInheritableTags = getVariantInheritableTags({ category: "creature" });
    if (creatureVariantInheritableTags.length > 0) {
      applyVariantBaseTagInheritance(
        indexedEntries.map((entry) => entry.record),
        creatureVariantInheritableTags,
      );
    }

    progressLogger?.(
      `Resolved ${formatInteger(aliasRows.length)} verified aliases, ${formatInteger(legacyLinkRows.length)} legacy-to-remaster links, and ${formatInteger(canonicalDerivedAfflictions.length)} derived affliction canonicals.`,
    );

    const suppressedRecordKeys = new Set(legacyLinkRows.map((row) => row.legacyRecordKey));
    const aliasesByCanonicalRecordKey = new Map<string, string[]>();
    for (const alias of aliasRows) {
      const bucket = aliasesByCanonicalRecordKey.get(alias.canonicalRecordKey) ?? [];
      bucket.push(alias.aliasText);
      aliasesByCanonicalRecordKey.set(alias.canonicalRecordKey, uniqueSorted(bucket));
    }

    resolutionDurationMs = Date.now() - resolutionStartTime;

    const writableEntries: WritableIndexEntry[] = [
      ...indexedEntries.map((entry) => ({
        record: entry.record,
        raw: entry.raw,
        actorData: entry.actorData,
        itemData: entry.itemData,
        spellData: entry.spellData,
        resolvedReferences: entry.resolvedReferences,
        aliasTexts: aliasesByCanonicalRecordKey.get(entry.record.recordKey) ?? [],
        isSearchCanonical: !suppressedRecordKeys.has(entry.record.recordKey),
      })),
      ...derivedAfflictions.records.map((entry) => ({
        record: entry.record,
        raw: entry.raw,
        actorData: entry.actorData,
        itemData: entry.itemData,
        spellData: entry.spellData,
        resolvedReferences: entry.resolvedReferences,
        aliasTexts: [],
        isSearchCanonical: entry.isSearchCanonical,
      })),
    ];

    recordCount = writableEntries.length;
    const canonicalEmbeddingCount = writableEntries.filter((entry) => entry.isSearchCanonical).length;
    const recordWriteProgressInterval = Math.max(100, Math.ceil(writableEntries.length / 10));
    const embeddingProgressInterval = Math.max(25, Math.ceil(Math.max(canonicalEmbeddingCount, 1) / 10));
    const pendingCanonicalEmbeddings: PendingCanonicalEmbeddingWithHash[] = [];
    let writtenRecordCount = 0;
    let processedCanonicalEmbeddingCount = 0;
    let lastRecordProgressLogTime = 0;
    let lastLoggedRecordCount = 0;
    let lastEmbeddingProgressLogTime = 0;
    let lastLoggedProcessedEmbeddingCount = 0;
    const recordStorageStartTime = Date.now();

    progressLogger?.("Writing indexed records and search metadata.");

    for (const entry of writableEntries) {
      const record = entry.record;
      const aliasTexts = entry.aliasTexts;
      const isSearchCanonical = entry.isSearchCanonical;
      const searchText = uniqueSorted([record.searchText, ...aliasTexts].filter(Boolean)).join("\n");

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
        isSearchCanonical ? 1 : 0,
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
          const normalizedValue = (() => {
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
          })() satisfies {
            valueType: "number" | "text" | "boolean";
            numberValue: number | null;
            textValue: string | null;
            boolValue: number | null;
          };

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
          const normalizedValue = (() => {
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
          })() satisfies {
            valueType: "number" | "text" | "boolean";
            numberValue: number | null;
            textValue: string | null;
            boolValue: number | null;
          };

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

      if (isSearchCanonical) {
        insertFts.run(record.recordKey, record.name, searchText);
        const encodedEmbeddingInput = buildSemanticEmbeddingText(record, entry.raw, aliasTexts);
        pendingCanonicalEmbeddings.push({
          record,
          encodedEmbeddingInput,
          semanticInputHash: hashSemanticInput(encodedEmbeddingInput),
        });
      }

      writtenRecordCount += 1;
      const now = Date.now();
      const shouldLogProgress =
        writtenRecordCount === writableEntries.length ||
        writtenRecordCount - lastLoggedRecordCount >= recordWriteProgressInterval ||
        now - lastRecordProgressLogTime >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldLogProgress) {
        progressStatusLogger?.(
          `[write] Stored records ${renderProgressBar(writtenRecordCount, writableEntries.length)} ${formatPercentage(writtenRecordCount, writableEntries.length)} (${formatInteger(writtenRecordCount)}/${formatInteger(writableEntries.length)} records).`,
        );
        lastRecordProgressLogTime = now;
        lastLoggedRecordCount = writtenRecordCount;
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

    recordStorageDurationMs = Date.now() - recordStorageStartTime;

    progressLogger?.(`Processing canonical embeddings in batches of ${EMBEDDING_BATCH_SIZE}.`);

    for (let index = 0; index < pendingCanonicalEmbeddings.length; index += EMBEDDING_BATCH_SIZE) {
      const batch = pendingCanonicalEmbeddings.slice(index, index + EMBEDDING_BATCH_SIZE);
      const pendingRegeneration: PendingCanonicalEmbeddingWithHash[] = [];

      const vecInsertStartTime = Date.now();
      for (const entry of batch) {
        const record = entry.record;
        const reusableEmbedding = reusableEmbeddingLookup?.get(record.recordKey) ?? null;

        if (
          reusableEmbedding &&
          reusableEmbedding.semanticInputHash === entry.semanticInputHash &&
          reusableEmbedding.dimensions === embeddingProvider.identity.dimensions
        ) {
          insertEmbedding.run(
            record.recordKey,
            embeddingProvider.identity.dimensions,
            entry.semanticInputHash,
            reusableEmbedding.vectorBlob,
          );
          insertVecEmbedding.run(
            record.recordKey,
            reusableEmbedding.vectorBlob,
            normalizeVecText(record.category),
            normalizeVecText(record.subcategory),
            normalizeVecText(record.packName),
            normalizeVecText(record.packLabel),
            normalizeVecText(record.documentType),
            normalizeVecText(record.type),
            normalizeVecInteger(record.level),
            normalizeVecText(record.rarity),
            normalizeVecText(record.sourceCategory),
            normalizeVecText(record.publicationTitle),
            BigInt(record.publicationRemaster ? 1 : 0),
            BigInt(record.hasDescription ? 1 : 0),
            BigInt(record.isUnique ? 1 : 0),
            normalizeVecText(record.size),
            normalizeVecText(record.itemCategory),
            normalizeVecInteger(record.priceCp),
            normalizeVecInteger(record.actionCost),
          );
          reusedCanonicalEmbeddingCount += 1;
          continue;
        }

        pendingRegeneration.push(entry);
      }

      if (pendingRegeneration.length > 0) {
        const embeddingStartTime = Date.now();
        const embeddings = await embeddingProvider.embedMany(
          pendingRegeneration.map((entry) => entry.encodedEmbeddingInput),
        );
        embeddingGenerationDurationMs += Date.now() - embeddingStartTime;

        for (const [batchIndex, entry] of pendingRegeneration.entries()) {
          const embedding = embeddings[batchIndex] ?? new Float32Array(embeddingProvider.identity.dimensions);
          const encodedEmbedding = encodeVector(embedding);
          const record = entry.record;

          insertEmbedding.run(
            record.recordKey,
            embeddingProvider.identity.dimensions,
            entry.semanticInputHash,
            encodedEmbedding,
          );
          insertVecEmbedding.run(
            record.recordKey,
            encodedEmbedding,
            normalizeVecText(record.category),
            normalizeVecText(record.subcategory),
            normalizeVecText(record.packName),
            normalizeVecText(record.packLabel),
            normalizeVecText(record.documentType),
            normalizeVecText(record.type),
            normalizeVecInteger(record.level),
            normalizeVecText(record.rarity),
            normalizeVecText(record.sourceCategory),
            normalizeVecText(record.publicationTitle),
            BigInt(record.publicationRemaster ? 1 : 0),
            BigInt(record.hasDescription ? 1 : 0),
            BigInt(record.isUnique ? 1 : 0),
            normalizeVecText(record.size),
            normalizeVecText(record.itemCategory),
            normalizeVecInteger(record.priceCp),
            normalizeVecInteger(record.actionCost),
          );
          regeneratedCanonicalEmbeddingCount += 1;
        }
      }
      vecInsertDurationMs += Date.now() - vecInsertStartTime;

      processedCanonicalEmbeddingCount += batch.length;
      const now = Date.now();
      const shouldLogProgress =
        processedCanonicalEmbeddingCount === canonicalEmbeddingCount ||
        processedCanonicalEmbeddingCount - lastLoggedProcessedEmbeddingCount >= embeddingProgressInterval ||
        now - lastEmbeddingProgressLogTime >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldLogProgress) {
        progressStatusLogger?.(
          `[embed] Canonical embeddings ${renderProgressBar(processedCanonicalEmbeddingCount, canonicalEmbeddingCount)} ${formatPercentage(processedCanonicalEmbeddingCount, canonicalEmbeddingCount)} (${formatInteger(processedCanonicalEmbeddingCount)}/${formatInteger(canonicalEmbeddingCount)} embeddings, reused ${formatInteger(reusedCanonicalEmbeddingCount)}, regenerated ${formatInteger(regeneratedCanonicalEmbeddingCount)}).`,
        );
        lastEmbeddingProgressLogTime = now;
        lastLoggedProcessedEmbeddingCount = processedCanonicalEmbeddingCount;
      }
    }

    progressLogger?.(
      `Canonical embedding reuse summary: reused ${formatInteger(reusedCanonicalEmbeddingCount)}, regenerated ${formatInteger(regeneratedCanonicalEmbeddingCount)}.`,
    );

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

    populateMetricCatalog(db);

    db.exec("COMMIT");
  } catch (error) {
    db.exec("ROLLBACK");
    throw error;
  }

  packs.sort((left, right) => left.label.localeCompare(right.label));
  return {
    packs,
    warnings,
    recordCount,
    reusedCanonicalEmbeddingCount,
    regeneratedCanonicalEmbeddingCount,
    stageTimings: [
      { label: "Scan and normalize records", durationMs: scanNormalizationDurationMs },
      { label: "Resolve families, references, tags, and aliases", durationMs: resolutionDurationMs },
      { label: "Write records and lexical search metadata", durationMs: recordStorageDurationMs },
      { label: "Generate canonical embeddings", durationMs: embeddingGenerationDurationMs },
      { label: "Insert vector rows", durationMs: vecInsertDurationMs },
    ],
  };
}
