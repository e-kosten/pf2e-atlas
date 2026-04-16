import { DatabaseSync } from "node:sqlite";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { REVIEWED_DISCOVERY_RECORDS } from "../../src/tags/discovery/discovery-reviewed-records.js";
vi.mock("../../src/tags/index.js", async () => {
  const actual = await vi.importActual<typeof import("../../src/tags/index.js")>("../../src/tags/index.js");
  return {
    ...actual,
    getDerivedTagExemplarRecordKeys: vi.fn(() => []),
  };
});

import { getDerivedTagExemplarRecordKeys } from "../../src/tags/index.js";
import { discoverRuleableCohorts } from "../../src/tags/discovery/cohort-discovery.js";
import { analyzeDiscoveryEvidence } from "../../src/tags/evaluation/evidence-analyzer.js";
import { evaluateDerivedTagGaps } from "../../src/tags/evaluation/gap-evaluator.js";

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
      pack_name TEXT,
      publication_title TEXT,
      folder_id TEXT,
      source_path TEXT,
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
    packName?: string | null;
    publicationTitle?: string | null;
    folderId?: string | null;
    sourcePath?: string | null;
    traits?: string[];
    descriptionText?: string | null;
    vector: number[];
    tags?: string[];
  },
): void {
  db.prepare(`
    INSERT INTO records (
      record_key, name, normalized_name, pack_name, publication_title, folder_id, source_path, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, '[]', NULL, ?, ?, ?, 1)
  `).run(
    input.recordKey,
    input.name,
    input.name.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
    input.packName ?? input.recordKey.split(":")[0] ?? input.recordKey,
    input.publicationTitle ?? null,
    input.folderId ?? null,
    input.sourcePath ?? null,
    input.category,
    input.subcategory ?? null,
    JSON.stringify(input.traits ?? []),
    JSON.stringify(input.tags ?? []),
    input.descriptionText ?? null,
  );
  db.prepare("INSERT INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(input.recordKey, blob(input.vector));
  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

const mockedGetDerivedTagExemplarRecordKeys = vi.mocked(getDerivedTagExemplarRecordKeys);

describe("exemplar-backed discovery integration", () => {
  beforeEach(() => {
    mockedGetDerivedTagExemplarRecordKeys.mockReset();
    mockedGetDerivedTagExemplarRecordKeys.mockReturnValue([]);
    REVIEWED_DISCOVERY_RECORDS.creature ??= {};
    REVIEWED_DISCOVERY_RECORDS.creature.setting ??= {};
    REVIEWED_DISCOVERY_RECORDS.creature.setting.not_family_salient = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.insufficient_evidence = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.mixed_family_cues = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.manual_lore_only = [];
  });

  it("uses configured exemplars as discovery evidence exemplars for a tag", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "spell:seed-1",
        name: "Blazing Mask",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "Create a burning mask that deals 2d6 fire damage.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "spell:seed-2",
        name: "Ashen Veil",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A veil of cinders forms a mask and deals 4d6 fire damage.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "spell:baseline",
        name: "Bracing Ward",
        category: "spell",
        traits: ["abjuration"],
        descriptionText: "Gain resistance and protective cover.",
        vector: [0, 1, 0],
      });

      mockedGetDerivedTagExemplarRecordKeys.mockReturnValue(["spell:seed-1", "spell:seed-2"]);

      const report = analyzeDiscoveryEvidence(db, {
        category: "spell",
        tag: "mask_motif",
        limit: 4,
      });

      expect(report.cohortSize).toBe(2);
      expect(report.representativeRecords.map((record) => record.recordKey)).toEqual([
        "spell:seed-1",
        "spell:seed-2",
      ]);
    } finally {
      db.close();
    }
  });

  it("can analyze one derived-tag family without naming a specific tag", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "creature:fortress-ghost",
        name: "Fortress Ghost",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A ghost haunts abandoned fortresses and broken citadels.",
        vector: [1, 0, 0],
        tags: ["fortress_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:graveyard-guardian",
        name: "Graveyard Guardian",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A spirit prowls old mausoleums and burial grounds.",
        vector: [0.98, 0.02, 0],
        tags: ["graveyard_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:mercenary",
        name: "Mercenary",
        category: "creature",
        traits: ["human"],
        descriptionText: "A veteran soldier watches the city gates.",
        vector: [0, 1, 0],
        tags: ["combatant_npc"],
      });

      const report = analyzeDiscoveryEvidence(db, {
        category: "creature",
        family: "setting",
        limit: 4,
      });

      expect(report.family).toBe("setting");
      expect(report.cohortSize).toBe(2);
      expect(report.representativeRecords.map((record) => record.recordKey)).toEqual([
        "creature:fortress-ghost",
        "creature:graveyard-guardian",
      ]);
    } finally {
      db.close();
    }
  });

  it("excludes reviewed-negative family-gap records by default and can audit one reviewed bucket", () => {
    const db = createDiscoveryDb();
    try {
      REVIEWED_DISCOVERY_RECORDS.creature!.setting!.not_family_salient = [
        { recordKey: "creature:generic-thug" },
      ];

      insertRecord(db, {
        recordKey: "creature:fortress-ghost",
        name: "Fortress Ghost",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A ghost haunts abandoned fortresses and broken citadels.",
        vector: [1, 0, 0],
        tags: ["fortress_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:wall-phantom",
        name: "Wall Phantom",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A phantom patrols fortress walls and crumbling battlements.",
        vector: [0.98, 0.02, 0],
        tags: ["undead_adjacent"],
      });
      insertRecord(db, {
        recordKey: "creature:generic-thug",
        name: "Generic Thug",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A generic thug with no stable habitat cues.",
        vector: [0, 1, 0],
        tags: ["combatant_npc"],
      });

      const defaultReport = analyzeDiscoveryEvidence(db, {
        category: "creature",
        family: "setting",
        familyGapSignals: true,
        limit: 4,
      });
      expect(defaultReport.familyGap?.uncoveredCount).toBe(1);
      expect(defaultReport.reviewedRecords).toEqual(expect.objectContaining({
        mode: "excluded",
        scopedCount: 1,
        appliedCount: 1,
      }));

      const reviewedReport = analyzeDiscoveryEvidence(db, {
        category: "creature",
        family: "setting",
        familyGapSignals: true,
        includeReviewed: true,
        reviewReason: "not_family_salient",
        limit: 4,
      });
      expect(reviewedReport.cohortSize).toBe(1);
      expect(reviewedReport.representativeRecords.map((record) => record.recordKey)).toEqual([
        "creature:generic-thug",
      ]);
      expect(reviewedReport.reviewedRecords).toEqual(expect.objectContaining({
        mode: "filtered",
        reviewReason: "not_family_salient",
        scopedCount: 1,
        appliedCount: 1,
      }));
    } finally {
      db.close();
    }
  });

  it("uses configured exemplars as cohort exemplars when a tag has no indexed matches yet", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "spell:seed-1",
        name: "Blazing Mask",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "Create a burning mask that deals 2d6 fire damage.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "spell:seed-2",
        name: "Ashen Veil",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A veil of cinders forms a mask and deals 4d6 fire damage.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-1",
        name: "Cinder Masquerade",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A cinder mask hides your features and deals 6d6 fire damage.",
        vector: [0.97, 0.03, 0],
      });

      mockedGetDerivedTagExemplarRecordKeys.mockReturnValue(["spell:seed-1", "spell:seed-2"]);

      const report = discoverRuleableCohorts(db, {
        category: "spell",
        tag: "mask_motif",
        candidateLimit: 5,
        cohortLimit: 3,
      });

      expect(report.sourceTag).toBe("mask_motif");
      expect(report.exemplarCount).toBe(2);
      expect(report.resolvedExemplars.map((record) => record.recordKey)).toEqual([
        "spell:seed-1",
        "spell:seed-2",
      ]);
    } finally {
      db.close();
    }
  });

  it("treats configured exemplars as gap exemplars without echoing them back as candidates", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "spell:seed-1",
        name: "Blazing Mask",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "Create a burning mask that deals 2d6 fire damage.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "spell:seed-2",
        name: "Ashen Veil",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A veil of cinders forms a mask and deals 4d6 fire damage.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-1",
        name: "Cinder Masquerade",
        category: "spell",
        traits: ["fire", "illusion"],
        descriptionText: "A cinder mask hides your features and deals 6d6 fire damage.",
        vector: [0.97, 0.03, 0],
      });

      mockedGetDerivedTagExemplarRecordKeys.mockReturnValue(["spell:seed-1", "spell:seed-2"]);

      const report = evaluateDerivedTagGaps(db, {
        tag: "mask_motif",
        category: "spell",
        limit: 5,
      });

      expect(report.exemplarCount).toBe(2);
      expect(report.candidates.map((record) => record.recordKey)).toContain("spell:candidate-1");
      expect(report.candidates.map((record) => record.recordKey)).not.toContain("spell:seed-1");
      expect(report.candidates.map((record) => record.recordKey)).not.toContain("spell:seed-2");
    } finally {
      db.close();
    }
  });
});
