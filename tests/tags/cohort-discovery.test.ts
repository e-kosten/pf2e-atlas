import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import { discoverRuleableCohorts } from "../../src/tags/cohort-discovery.js";

function vector(values: number[]): Float32Array {
  return Float32Array.from(values);
}

function blob(values: number[]): Uint8Array {
  return new Uint8Array(vector(values).buffer.slice(0));
}

function createDiscoveryDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      normalized_name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      variant_family_key TEXT,
      variant_base_name TEXT,
      variant_label TEXT,
      variant_axes_json TEXT NOT NULL,
      level INTEGER,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT NOT NULL,
      description_text TEXT,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE embeddings (
      record_key TEXT PRIMARY KEY,
      vector_blob BLOB NOT NULL
    );
    CREATE TABLE record_aliases (
      canonical_record_key TEXT NOT NULL,
      alias_text TEXT NOT NULL,
      normalized_alias TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL
    );
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
    CREATE TABLE reference_edges (
      from_record_key TEXT NOT NULL,
      to_record_key TEXT NOT NULL,
      display_text TEXT,
      reference_text TEXT NOT NULL,
      from_pack_name TEXT NOT NULL,
      from_record_type TEXT NOT NULL,
      from_document_type TEXT NOT NULL,
      from_source_category TEXT NOT NULL
    );
  `);
  return db;
}

function insertRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    category: string;
    subcategory?: string | null;
    variantFamilyKey?: string | null;
    variantBaseName?: string | null;
    variantLabel?: string | null;
    variantAxes?: string[];
    traits?: string[];
    descriptionText?: string | null;
    vector: number[];
    tags?: string[];
    aliases?: string[];
  },
): void {
  db.prepare(`
    INSERT INTO records (
      record_key, name, normalized_name, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, 1)
  `).run(
    input.recordKey,
    input.name,
    input.name.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
    input.category,
    input.subcategory ?? null,
    input.variantFamilyKey ?? null,
    input.variantBaseName ?? null,
    input.variantLabel ?? null,
    JSON.stringify(input.variantAxes ?? []),
    JSON.stringify(input.traits ?? []),
    JSON.stringify(input.tags ?? []),
    input.descriptionText ?? null,
  );
  db.prepare("INSERT INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(input.recordKey, blob(input.vector));
  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
  for (const alias of input.aliases ?? []) {
    db.prepare(`
      INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
      VALUES (?, ?, ?, 'test', 'alias')
    `).run(
      input.recordKey,
      alias,
      alias.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
    );
  }
}

function insertReference(db: DatabaseSync, fromRecordKey: string, toRecordKey: string, toRecordName: string): void {
  db.prepare(`
    INSERT OR IGNORE INTO records (
      record_key, name, normalized_name, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, 'rule', 'action', NULL, NULL, NULL, '[]', NULL, '[]', '[]', NULL, 1)
  `).run(
    toRecordKey,
    toRecordName,
    toRecordName.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
  );
  db.prepare("INSERT OR IGNORE INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(toRecordKey, blob([0, 0, 1]));
  db.prepare(`
    INSERT INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, NULL, ?, 'actionspf2e', 'spell', 'spell', 'rules')
  `).run(fromRecordKey, toRecordKey, `ref:${fromRecordKey}:${toRecordKey}`);
}

describe("ruleable cohort discovery", () => {
  it("groups semantic neighbors into evidence-backed cohorts and surfaces contrast records", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "spell:seed-1",
        name: "Blazing Mask",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "Create a burning mask that deals 2d6 fire damage and conceals your face.",
        vector: [1, 0, 0],
        tags: ["mask_motif"],
        aliases: ["Blaze Mask"],
      });
      insertRecord(db, {
        recordKey: "spell:seed-2",
        name: "Ashen Veil",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A veil of cinders forms a mask and deals 4d6 fire damage.",
        vector: [0.98, 0.02, 0],
        tags: ["mask_motif"],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-1",
        name: "Cinder Masquerade",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A cinder mask hides your features and deals 6d6 fire damage.",
        vector: [0.97, 0.03, 0],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-2",
        name: "Flame Disguise",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A blazing disguise mask scorches foes for 3d6 fire damage.",
        vector: [0.96, 0.04, 0],
      });
      insertRecord(db, {
        recordKey: "spell:contrast",
        name: "Heat Ward",
        category: "spell",
        traits: ["fire", "abjuration"],
        descriptionText: "A ward that protects against heat damage.",
        vector: [0.94, 0.06, 0],
      });

      insertReference(db, "spell:seed-1", "rule:ignite", "Ignite");
      insertReference(db, "spell:seed-2", "rule:ignite", "Ignite");
      insertReference(db, "spell:candidate-1", "rule:ignite", "Ignite");

      const report = discoverRuleableCohorts(db, {
        category: "spell",
        tag: "mask_motif",
        candidateLimit: 5,
        cohortLimit: 3,
      });

      expect(report.sourceTag).toBe("mask_motif");
      expect(report.anchorTerms.map((term) => term.value)).toEqual(expect.arrayContaining(["fire", "target:ignite"]));
      expect(report.cohorts[0]?.size).toBeGreaterThanOrEqual(1);
      expect(report.cohorts[0]?.recommendation).toMatch(/rule-led|hybrid|manual-only/);
      expect(report.contrastRecords.map((record) => record.name)).toContain("Heat Ward");
    } finally {
      db.close();
    }
  });
});
