import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  discoverUntaggedCohorts,
} from "../../src/tags/untagged-cohort-discovery.js";
import {
  formatUntaggedCohortReport,
  parseOptions,
} from "../../src/tags/discover-untagged-cohorts.js";

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
}

function insertReference(db: DatabaseSync, fromRecordKey: string, toRecordKey: string, toRecordName: string): void {
  db.prepare(`
    INSERT OR IGNORE INTO records (
      record_key, name, normalized_name, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, 'spell', NULL, NULL, NULL, NULL, '[]', NULL, '[]', '[]', NULL, 1)
  `).run(
    toRecordKey,
    toRecordName,
    toRecordName.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
  );
  db.prepare("INSERT OR IGNORE INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(toRecordKey, blob([0, 0, 1]));
  db.prepare(`
    INSERT INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, NULL, ?, 'equipment-srd', 'equipment', 'equipment', 'rules')
  `).run(fromRecordKey, toRecordKey, `ref:${fromRecordKey}:${toRecordKey}`);
}

describe("discover untagged cohorts", () => {
  it("proposes coherent cohorts from all untagged records in a scoped category", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:scarf",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "A masquerade scarf helps you change outfits and hide your identity with superficial sewing tricks.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:outfit",
        name: "Quick-Change Outfit",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "This outfit uses sewing techniques to enable quick change and disguise work during a masquerade.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:cloak",
        name: "Clandestine Cloak",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "A subtle cloak that conceals your equipment.",
        vector: [0.92, 0.08, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:goggles",
        name: "Tracker's Goggles",
        category: "equipment",
        subcategory: "gear",
        traits: ["divination", "magical"],
        descriptionText: "These goggles aid tracking and pursuit across muddy roads.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:tag",
        name: "Tracking Tag",
        category: "equipment",
        subcategory: "gear",
        traits: ["magical"],
        descriptionText: "Place the tag on prey to improve tracking and pursuit.",
        vector: [0.02, 0.98, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:already-tagged",
        name: "Ordinary Mask",
        category: "equipment",
        subcategory: "gear",
        traits: ["mundane"],
        descriptionText: "A normal mask with no disguise magic.",
        vector: [0.7, 0.3, 0],
        tags: ["concealment"],
      });
      insertRecord(db, {
        recordKey: "equipment:tagged-rope",
        name: "Climbing Rope",
        category: "equipment",
        subcategory: "gear",
        traits: ["mundane"],
        descriptionText: "A sturdy rope for climbing cliffs.",
        vector: [0.1, 0.4, 0],
        tags: ["mobility"],
      });
      insertRecord(db, {
        recordKey: "equipment:tagged-lantern",
        name: "Lantern",
        category: "equipment",
        subcategory: "gear",
        traits: ["mundane"],
        descriptionText: "A lantern that illuminates dark tunnels.",
        vector: [0.2, 0.2, 0],
        tags: ["illumination"],
      });
      insertRecord(db, {
        recordKey: "equipment:tagged-saddle",
        name: "War Saddle",
        category: "equipment",
        subcategory: "gear",
        traits: ["mundane"],
        descriptionText: "A saddle for mounted travel.",
        vector: [0.1, 0.3, 0],
        tags: ["mounted_support"],
      });

      insertReference(db, "equipment:scarf", "spell:illusory-disguise", "Illusory Disguise");
      insertReference(db, "equipment:outfit", "spell:illusory-disguise", "Illusory Disguise");

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        cohortLimit: 4,
        anchorLimit: 10,
        minFeatureSupport: 2,
        minFeatureLift: 1.5,
      });

      expect(report.untaggedRecordCount).toBe(5);
      expect(report.anchorTerms.map((anchor) => anchor.value)).toContain("masquerade");
      expect(report.cohorts.length).toBeGreaterThan(0);
      expect(report.cohorts[0]?.size).toBeGreaterThanOrEqual(2);
      expect(report.cohorts.some((cohort) =>
        cohort.signature.some((term) => term.includes("masquerade")) &&
        cohort.representativeRecords.some((record) => record.name === "Masquerade Scarf"))).toBe(true);
    } finally {
      db.close();
    }
  });

  it("parses CLI options and renders a readable report", () => {
    const options = parseOptions([
      "--category", "equipment",
      "--subcategory", "gear",
      "--cohort-limit", "5",
      "--anchor-limit", "12",
      "--min-feature-support", "3",
      "--min-feature-lift", "2.5",
    ]);

    expect(options).toEqual({
      category: "equipment",
      subcategory: "gear",
      cohortLimit: 5,
      anchorLimit: 12,
      minFeatureSupport: 3,
      minFeatureLift: 2.5,
    });

    const rendered = formatUntaggedCohortReport({
      category: "equipment",
      subcategory: "gear",
      untaggedRecordCount: 12,
      baselineRecordCount: 30,
      anchorTerms: [
        { value: "masquerade", support: 4, baselineSupport: 5, lift: 4.8, score: 19.2 },
      ],
      cohorts: [
        {
          signature: ["masquerade", "target:illusory disguise"],
          size: 3,
          distinctVariantFamilies: 2,
          averageSimilarity: 0.82,
          sharedTraits: ["illusion"],
          anchorSupport: 4,
          anchorLift: 4.8,
          score: 0.74,
          recommendation: "rule-led",
          representativeRecords: [
            { recordKey: "equipment:1", name: "Masquerade Scarf", similarity: 0.88 },
          ],
          contrastRecords: [
            { recordKey: "equipment:2", name: "Clandestine Cloak", similarity: 0.73 },
          ],
        },
      ],
    });

    expect(rendered).toContain("Untagged cohort summary:");
    expect(rendered).toContain("Top anchors:");
    expect(rendered).toContain("Recommended cohorts:");
    expect(rendered).toContain("families=2");
  });

  it("down-ranks single-family variant ladders in favor of multi-family cohorts", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:wand-2",
        name: "Wand of Choking Mist (2nd-Rank)",
        category: "equipment",
        subcategory: "consumable",
        variantFamilyKey: "equipment:family:wand-of-choking-mist",
        variantBaseName: "Wand of Choking Mist",
        variantLabel: "2nd-Rank",
        variantAxes: ["rank"],
        traits: ["magical", "water"],
        descriptionText: "This wand casts mist and leaves choking vapors behind.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:wand-4",
        name: "Wand of Choking Mist (4th-Rank)",
        category: "equipment",
        subcategory: "consumable",
        variantFamilyKey: "equipment:family:wand-of-choking-mist",
        variantBaseName: "Wand of Choking Mist",
        variantLabel: "4th-Rank",
        variantAxes: ["rank"],
        traits: ["magical", "water"],
        descriptionText: "This wand casts cinder swarm and leaves choking vapors behind.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:figurine-bear",
        name: "Wondrous Figurine (Rubber Bear)",
        category: "equipment",
        subcategory: "gear",
        variantFamilyKey: "equipment:family:wondrous-figurine",
        variantBaseName: "Wondrous Figurine",
        variantLabel: "Rubber Bear",
        variantAxes: ["other"],
        traits: ["magical"],
        descriptionText: "This figurine becomes a circus bear when activated.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:figurine-lions",
        name: "Wondrous Figurine (Golden Lions)",
        category: "equipment",
        subcategory: "gear",
        variantFamilyKey: "equipment:family:wondrous-figurine",
        variantBaseName: "Wondrous Figurine",
        variantLabel: "Golden Lions",
        variantAxes: ["other"],
        traits: ["magical"],
        descriptionText: "This figurine becomes a pair of lions when activated.",
        vector: [0.01, 0.99, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:scarf",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "A masquerade scarf helps disguise your identity.",
        vector: [0, 0, 1],
      });
      insertRecord(db, {
        recordKey: "equipment:outfit",
        name: "Quick-Change Outfit",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "An outfit designed for disguise and social infiltration.",
        vector: [0, 0.05, 0.95],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        cohortLimit: 5,
        anchorLimit: 10,
        minFeatureSupport: 2,
        minFeatureLift: 1.1,
      });

      expect(report.cohorts.length).toBeGreaterThan(0);
      expect(report.cohorts.every((cohort) => cohort.distinctVariantFamilies <= cohort.size)).toBe(true);
      expect(report.cohorts[0]?.distinctVariantFamilies).toBeGreaterThanOrEqual(1);
      expect(report.cohorts.some((cohort) =>
        cohort.representativeRecords.filter((record) => record.name.startsWith("Wand of Choking Mist")).length <= 1)).toBe(true);
    } finally {
      db.close();
    }
  });

  it("dedupes contrast records by variant family", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:scarf",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "A masquerade scarf helps disguise your identity.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:outfit",
        name: "Quick-Change Outfit",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "An outfit designed for disguise and social infiltration.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:words-lesser",
        name: "Words of Wisdom (Lesser)",
        category: "equipment",
        subcategory: "gear",
        variantFamilyKey: "equipment:family:words-of-wisdom",
        variantBaseName: "Words of Wisdom",
        variantLabel: "Lesser",
        variantAxes: ["grade"],
        traits: ["mental", "magical"],
        descriptionText: "Words of wisdom sharpen your mind.",
        vector: [0.8, 0.2, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:words-moderate",
        name: "Words of Wisdom (Moderate)",
        category: "equipment",
        subcategory: "gear",
        variantFamilyKey: "equipment:family:words-of-wisdom",
        variantBaseName: "Words of Wisdom",
        variantLabel: "Moderate",
        variantAxes: ["grade"],
        traits: ["mental", "magical"],
        descriptionText: "Words of wisdom sharpen your mind further.",
        vector: [0.79, 0.21, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:words-greater",
        name: "Words of Wisdom (Greater)",
        category: "equipment",
        subcategory: "gear",
        variantFamilyKey: "equipment:family:words-of-wisdom",
        variantBaseName: "Words of Wisdom",
        variantLabel: "Greater",
        variantAxes: ["grade"],
        traits: ["mental", "magical"],
        descriptionText: "Words of wisdom sharpen your mind with potent insight.",
        vector: [0.78, 0.22, 0],
      });
      insertReference(db, "equipment:scarf", "spell:illusory-disguise", "Illusory Disguise");
      insertReference(db, "equipment:outfit", "spell:illusory-disguise", "Illusory Disguise");

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        cohortLimit: 4,
        anchorLimit: 10,
        minFeatureSupport: 2,
        minFeatureLift: 1.1,
      });

      expect(report.cohorts.every((cohort) =>
        cohort.contrastRecords.filter((record) => record.name.startsWith("Words of Wisdom")).length <= 1)).toBe(true);
    } finally {
      db.close();
    }
  });
});
