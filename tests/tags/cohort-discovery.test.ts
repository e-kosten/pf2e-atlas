import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import { discoverRuleableCohorts } from "../../src/tags/discovery/cohort-discovery.js";

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
    variantFamilyKey?: string | null;
    variantBaseName?: string | null;
    variantLabel?: string | null;
    variantAxes?: string[];
    packName?: string | null;
    publicationTitle?: string | null;
    folderId?: string | null;
    sourcePath?: string | null;
    traits?: string[];
    descriptionText?: string | null;
    vector: number[];
    tags?: string[];
    aliases?: string[];
  },
): void {
  db.prepare(
    `
    INSERT INTO records (
      record_key, name, normalized_name, pack_name, publication_title, folder_id, source_path, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, 1)
  `,
  ).run(
    input.recordKey,
    input.name,
    input.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, " ")
      .trim()
      .replace(/\s+/g, " "),
    input.packName ?? input.recordKey.split(":")[0] ?? input.recordKey,
    input.publicationTitle ?? null,
    input.folderId ?? null,
    input.sourcePath ?? null,
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
    db.prepare(
      `
      INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
      VALUES (?, ?, ?, 'test', 'alias')
    `,
    ).run(
      input.recordKey,
      alias,
      alias
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, " ")
        .trim()
        .replace(/\s+/g, " "),
    );
  }
}

function insertReference(db: DatabaseSync, fromRecordKey: string, toRecordKey: string, toRecordName: string): void {
  db.prepare(
    `
    INSERT OR IGNORE INTO records (
      record_key, name, normalized_name, pack_name, publication_title, folder_id, source_path, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, 'actionspf2e', NULL, NULL, NULL, 'rule', 'action', NULL, NULL, NULL, '[]', NULL, '[]', '[]', NULL, 1)
  `,
  ).run(
    toRecordKey,
    toRecordName,
    toRecordName
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, " ")
      .trim()
      .replace(/\s+/g, " "),
  );
  db.prepare("INSERT OR IGNORE INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(
    toRecordKey,
    blob([0, 0, 1]),
  );
  db.prepare(
    `
    INSERT INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, NULL, ?, 'actionspf2e', 'spell', 'spell', 'rules')
  `,
  ).run(fromRecordKey, toRecordKey, `ref:${fromRecordKey}:${toRecordKey}`);
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
      expect(report.anchorTerms.map((term) => term.value)).toContain("target:ignite");
      expect(report.anchorTerms.map((term) => term.value)).toEqual(expect.arrayContaining(["illusion"]));
      expect(report.cohorts[0]?.size).toBeGreaterThanOrEqual(1);
      expect(report.cohorts[0]?.recommendation).toMatch(/rule-led|hybrid|manual-only/);
      expect(report.contrastRecords.length).toBeGreaterThan(0);
    } finally {
      db.close();
    }
  });

  it("dedupes contrast records by variant family", () => {
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
        recordKey: "spell:contrast-lesser",
        name: "Words of Wisdom (Lesser)",
        category: "spell",
        variantFamilyKey: "spell:words-of-wisdom",
        variantBaseName: "Words of Wisdom",
        variantLabel: "Lesser",
        traits: ["fire", "abjuration"],
        descriptionText: "Gain wise counsel behind a brief veil of fire.",
        vector: [0.95, 0.05, 0],
      });
      insertRecord(db, {
        recordKey: "spell:contrast-greater",
        name: "Words of Wisdom (Greater)",
        category: "spell",
        variantFamilyKey: "spell:words-of-wisdom",
        variantBaseName: "Words of Wisdom",
        variantLabel: "Greater",
        traits: ["fire", "abjuration"],
        descriptionText: "Gain greater wise counsel behind a brighter veil of fire.",
        vector: [0.949, 0.051, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "spell",
        tag: "mask_motif",
        candidateLimit: 6,
        cohortLimit: 3,
      });

      expect(
        report.contrastRecords.filter((record) => record.name.startsWith("Words of Wisdom")).length,
      ).toBeLessThanOrEqual(1);
    } finally {
      db.close();
    }
  });

  it("keeps explicit category scope broad when a tag spans multiple subcategories", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:boots",
        name: "Windstep Boots",
        category: "equipment",
        subcategory: "gear",
        traits: ["magical"],
        descriptionText: "These boots help you move with sudden bursts of speed.",
        vector: [1, 0, 0],
        tags: ["mobility"],
      });
      insertRecord(db, {
        recordKey: "equipment:skiff",
        name: "Storm Skiff",
        category: "equipment",
        subcategory: "vehicle",
        traits: ["magical"],
        descriptionText: "A skiff that surges quickly across rough water.",
        vector: [0.98, 0.02, 0],
        tags: ["mobility"],
      });
      insertRecord(db, {
        recordKey: "equipment:cloak",
        name: "Updraft Cloak",
        category: "equipment",
        subcategory: "gear",
        traits: ["magical"],
        descriptionText: "A cloak that helps you catch strong updrafts.",
        vector: [0.97, 0.03, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:sled",
        name: "Glacier Sled",
        category: "equipment",
        subcategory: "vehicle",
        traits: ["magical"],
        descriptionText: "A sled built to skim over packed snow and ice.",
        vector: [0.96, 0.04, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "equipment",
        tag: "mobility",
        candidateLimit: 6,
        cohortLimit: 3,
      });

      expect(report.category).toBe("equipment");
      expect(report.subcategory).toBeNull();
      expect(report.candidateCount).toBe(2);
    } finally {
      db.close();
    }
  });

  it("does not promote semantic-only seeded cohorts to rule-led", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "spell:seed-1",
        name: "North Blaze",
        category: "spell",
        traits: ["fire"],
        descriptionText: "A cinder spiral races down an alley.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "spell:seed-2",
        name: "South Veil",
        category: "spell",
        traits: ["illusion"],
        descriptionText: "A silk mirage hides whispered memories.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-1",
        name: "Center Echo",
        category: "spell",
        traits: ["sonic"],
        descriptionText: "A resonant pulse ripples through a plaza.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "spell:candidate-2",
        name: "Far Lantern",
        category: "spell",
        traits: ["light"],
        descriptionText: "A lantern glow drifts across the courtyard.",
        vector: [0.97, 0.03, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "spell",
        exemplarRecordKeys: ["spell:seed-1", "spell:seed-2"],
        candidateLimit: 6,
        cohortLimit: 3,
      });

      expect(report.cohorts.length).toBeGreaterThan(0);
      expect(report.cohorts[0]?.sharedAnchors).toEqual([]);
      expect(report.cohorts[0]?.recommendation).not.toBe("rule-led");
    } finally {
      db.close();
    }
  });

  it("prioritizes candidates that match stronger exemplar anchors over broad shared traits", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:seed-1",
        name: "Returning Fang",
        category: "equipment",
        subcategory: "weapon",
        traits: ["agile", "finesse", "thrown-10", "versatile-s"],
        descriptionText: "This returning dagger is balanced for precise throws.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:seed-2",
        name: "Twilight Knife",
        category: "equipment",
        subcategory: "weapon",
        traits: ["agile", "finesse", "thrown-10", "versatile-s"],
        descriptionText: "This returning dagger flashes back to your hand after a throw.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:candidate-focused",
        name: "Shadow Fang",
        category: "equipment",
        subcategory: "weapon",
        traits: ["agile", "finesse", "thrown-10", "versatile-s"],
        descriptionText: "A returning dagger that rewards swift, precise throws.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:candidate-broad",
        name: "Hooked Talon",
        category: "equipment",
        subcategory: "weapon",
        traits: ["agile"],
        descriptionText: "A hooked blade used for rapid slashes in close quarters.",
        vector: [0.985, 0.015, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "equipment",
        subcategory: "weapon",
        exemplarRecordKeys: ["equipment:seed-1", "equipment:seed-2"],
        candidateLimit: 2,
        cohortLimit: 2,
      });

      expect(report.cohorts[0]?.representativeRecords[0]?.name).toBe("Shadow Fang");
      expect(report.cohorts[0]?.sharedAnchors).not.toEqual(["agile"]);
    } finally {
      db.close();
    }
  });

  it("does not promote single-source name-series seeded cohorts to rule-led", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "ap-war:seed-1",
        name: "Warcaller's Chime of Blasting",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and blasting runes.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "ap-war:seed-2",
        name: "Warcaller's Chime of Dread",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and dreadful omens.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "ap-war:candidate-1",
        name: "Warcaller's Chime of Refuge",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and refuge wards.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "ap-war:candidate-2",
        name: "Warcaller's Chime of Restoration",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and healing sigils.",
        vector: [0.97, 0.03, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:contrast",
        name: "Traveler's Cloak",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A durable cloak for long journeys.",
        vector: [0, 1, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        exemplarRecordKeys: ["ap-war:seed-1", "ap-war:seed-2"],
        candidateLimit: 6,
        cohortLimit: 3,
      });

      expect(report.cohorts[0]?.reviewFlags).toContain("name-series");
      expect(report.cohorts[0]?.reviewFlags).toContain("source-local");
      expect(report.cohorts[0]?.recommendation).not.toBe("rule-led");
    } finally {
      db.close();
    }
  });

  it("prefers shared source-path slices over pack names for locality warnings", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "pack-alpha:seed-1",
        packName: "pack-alpha",
        publicationTitle: "Pathfinder Lost Omens The Mwangi Expanse",
        sourcePath: "/tmp/vendor/pf2e/packs/pf2e/pack-alpha/mwangi-expanse/mask-1.json",
        name: "Mwangi Jungle Mask",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion"],
        descriptionText: "A ceremonial mask used in Mwangi jungle rites.",
        vector: [1, 0, 0],
        tags: ["mask_motif"],
      });
      insertRecord(db, {
        recordKey: "pack-beta:seed-2",
        packName: "pack-beta",
        publicationTitle: "Pathfinder Lost Omens The Mwangi Expanse",
        sourcePath: "/tmp/vendor/pf2e/packs/pf2e/pack-beta/mwangi-expanse/veil-1.json",
        name: "Mwangi Jungle Veil",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion"],
        descriptionText: "A veil used in Mwangi jungle rites and masquerades.",
        vector: [0.99, 0.01, 0],
        tags: ["mask_motif"],
      });
      insertRecord(db, {
        recordKey: "pack-alpha:candidate-1",
        packName: "pack-alpha",
        publicationTitle: "Pathfinder Lost Omens The Mwangi Expanse",
        sourcePath: "/tmp/vendor/pf2e/packs/pf2e/pack-alpha/mwangi-expanse/mask-2.json",
        name: "Mwangi Revel Mask",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion"],
        descriptionText: "A revel mask used in Mwangi jungle celebrations.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "pack-beta:candidate-2",
        packName: "pack-beta",
        publicationTitle: "Pathfinder Lost Omens The Mwangi Expanse",
        sourcePath: "/tmp/vendor/pf2e/packs/pf2e/pack-beta/mwangi-expanse/veil-2.json",
        name: "Mwangi Revel Veil",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion"],
        descriptionText: "A revel veil used in Mwangi jungle celebrations.",
        vector: [0.97, 0.03, 0],
      });

      const report = discoverRuleableCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        tag: "mask_motif",
        candidateLimit: 6,
        cohortLimit: 3,
      });

      expect(report.cohorts[0]?.sourceScope).toBe("source-slice");
      expect(report.cohorts[0]?.topSourceSlices).toContain("mwangi-expanse");
      expect(report.cohorts[0]?.reviewFlags).toContain("source-local");
    } finally {
      db.close();
    }
  });
});
