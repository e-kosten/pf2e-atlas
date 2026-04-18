import { DatabaseSync } from "node:sqlite";

import { beforeEach, describe, expect, it } from "vitest";

import { REVIEWED_DISCOVERY_RECORDS } from "../../src/tags/discovery/discovery-reviewed-records.js";
import { discoverUntaggedCohorts } from "../../src/tags/discovery/untagged-cohort-discovery.js";
import { formatHelp, formatUntaggedCohortReport, parseOptions } from "../../src/tags/cli/discover-untagged-cohorts.js";

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
}

function insertReference(db: DatabaseSync, fromRecordKey: string, toRecordKey: string, toRecordName: string): void {
  db.prepare(
    `
    INSERT OR IGNORE INTO records (
      record_key, name, normalized_name, pack_name, publication_title, folder_id, source_path, category, subcategory,
      variant_family_key, variant_base_name, variant_label, variant_axes_json,
      level, traits_json, derived_tags_json, description_text, is_search_canonical
    )
    VALUES (?, ?, ?, 'equipment-srd', NULL, NULL, NULL, 'spell', NULL, NULL, NULL, NULL, '[]', NULL, '[]', '[]', NULL, 1)
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
    ) VALUES (?, ?, NULL, ?, 'equipment-srd', 'equipment', 'equipment', 'rules')
  `,
  ).run(fromRecordKey, toRecordKey, `ref:${fromRecordKey}:${toRecordKey}`);
}

describe("discover untagged cohorts", () => {
  beforeEach(() => {
    REVIEWED_DISCOVERY_RECORDS.creature ??= {};
    REVIEWED_DISCOVERY_RECORDS.creature.setting ??= {};
    REVIEWED_DISCOVERY_RECORDS.creature.setting.not_family_salient = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.insufficient_evidence = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.mixed_family_cues = [];
    REVIEWED_DISCOVERY_RECORDS.creature.setting.manual_lore_only = [];
  });

  it("proposes coherent cohorts from all untagged records in a scoped category", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:scarf",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText:
          "A masquerade scarf helps you change outfits and hide your identity with superficial sewing tricks.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:outfit",
        name: "Quick-Change Outfit",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText:
          "This outfit uses sewing techniques to enable quick change and disguise work during a masquerade.",
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
      expect(
        report.cohorts.some(
          (cohort) =>
            cohort.signature.some((term) => term.includes("masquerade")) &&
            cohort.representativeRecords.some((record) => record.name === "Masquerade Scarf"),
        ),
      ).toBe(true);
    } finally {
      db.close();
    }
  });

  it("can use four-word phrases as cohort anchors when configured", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:veil",
        name: "Court Veil",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "This veil supports hidden court masquerade attire during elite infiltration.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:ribbon",
        name: "Masquerade Ribbon",
        category: "equipment",
        subcategory: "gear",
        traits: ["illusion", "magical"],
        descriptionText: "The ribbon completes hidden court masquerade attire for covert galas.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:torch",
        name: "Signal Torch",
        category: "equipment",
        subcategory: "gear",
        traits: ["fire"],
        descriptionText: "A bright torch for signaling allies.",
        vector: [0, 1, 0],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        minGramLength: 4,
        maxGramLength: 4,
        cohortLimit: 3,
      });

      expect(report.cohorts.length).toBeGreaterThan(0);
    } finally {
      db.close();
    }
  });

  it("parses CLI options and renders a readable report", () => {
    const options = parseOptions([
      "--category",
      "equipment",
      "--subcategory",
      "gear",
      "--family",
      "purpose",
      "--family-gap-signals",
      "--include-reviewed",
      "--review-reason",
      "manual_lore_only",
      "--cohort-limit",
      "5",
      "--anchor-limit",
      "12",
      "--min-feature-support",
      "3",
      "--min-feature-lift",
      "2.5",
      "--min-gram-length",
      "4",
      "--max-gram-length",
      "5",
    ]);

    expect(options).toEqual({
      category: "equipment",
      subcategory: "gear",
      family: "purpose",
      familyGapSignals: true,
      includeReviewed: true,
      reviewReason: "manual_lore_only",
      cohortLimit: 5,
      anchorLimit: 12,
      minFeatureSupport: 3,
      minFeatureLift: 2.5,
      minGramLength: 4,
      maxGramLength: 5,
    });

    const rendered = formatUntaggedCohortReport({
      category: "equipment",
      subcategory: "gear",
      family: "purpose",
      untaggedRecordCount: 12,
      baselineRecordCount: 30,
      coveredRecordCount: 18,
      liveTags: ["fortress_setting", "temple_setting"],
      reviewedRecords: {
        mode: "excluded",
        reviewReason: null,
        scopedCount: 4,
        appliedCount: 4,
        reasonCounts: [{ reason: "not_family_salient", count: 4 }],
      },
      anchorTerms: [
        {
          value: "masquerade",
          support: 4,
          baselineSupport: 5,
          lift: 4.8,
          score: 19.2,
          existingTagOverlaps: ["temple_setting"],
        },
      ],
      cohorts: [
        {
          signature: ["masquerade", "target:illusory disguise"],
          size: 3,
          distinctVariantFamilies: 2,
          averageSimilarity: 0.82,
          sourceCount: 0,
          topSources: [],
          publicationCount: 0,
          topPublications: [],
          sourceSliceCount: 0,
          topSourceSlices: [],
          dominantSourceShare: 0,
          sourceScope: null,
          sharedTraits: ["illusion"],
          nonNameAnchors: ["target:illusory disguise"],
          reviewFlags: [],
          anchorSupport: 4,
          anchorLift: 4.8,
          score: 0.74,
          recommendation: "rule-led",
          classification: "existing_tag_coverage_gap",
          familyGapRecommendation: "extend-existing-tag",
          overlappingTags: ["temple_setting"],
          representativeRecords: [{ recordKey: "equipment:1", name: "Masquerade Scarf", similarity: 0.88 }],
          contrastRecords: [{ recordKey: "equipment:2", name: "Clandestine Cloak", similarity: 0.73 }],
        },
      ],
    });

    expect(rendered).toContain("Untagged cohort summary:");
    expect(rendered).toContain("Family: purpose");
    expect(rendered).toContain("Covered family records: 18");
    expect(rendered).toContain("Excluded reviewed records: 4/4");
    expect(rendered).toContain("Reviewed reason counts: not_family_salient=4");
    expect(rendered).toContain("Top anchors:");
    expect(rendered).toContain("Recommended cohorts:");
    expect(rendered).toContain("classification=existing_tag_coverage_gap");
    expect(rendered).toContain("family_gap=extend-existing-tag");
    expect(rendered).toContain("families=2");
    expect(rendered).toContain("source_scope=(none)");
    expect(rendered).toContain("flags=(none)");
  });

  it("can focus on records missing tags from one derived-tag family", () => {
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
        recordKey: "creature:citadel-wraith",
        name: "Citadel Wraith",
        category: "creature",
        traits: ["undead"],
        descriptionText: "This wraith stalks ruined bastions and abandoned keeps.",
        vector: [0.99, 0.01, 0],
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
        recordKey: "creature:keep-sentinel",
        name: "Keep Sentinel",
        category: "creature",
        traits: ["construct"],
        descriptionText: "An animate guardian stands watch over an old keep and its bastion.",
        vector: [0.97, 0.03, 0],
        tags: ["enforcer_npc"],
      });
      insertRecord(db, {
        recordKey: "creature:bastion-shade",
        name: "Bastion Shade",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A shade lurks in derelict bastions and forgotten keeps.",
        vector: [0.96, 0.04, 0],
        tags: ["haunt_theme"],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "creature",
        family: "setting",
        cohortLimit: 3,
        anchorLimit: 8,
        minFeatureSupport: 2,
        minFeatureLift: 1.1,
      });

      expect(report.family).toBe("setting");
      expect(report.untaggedRecordCount).toBe(3);
      expect(report.baselineRecordCount).toBe(5);
    } finally {
      db.close();
    }
  });

  it("excludes reviewed family-gap negatives by default and can audit one reviewed reason bucket", () => {
    const db = createDiscoveryDb();
    try {
      REVIEWED_DISCOVERY_RECORDS.creature!.setting!.not_family_salient = [
        { recordKey: "creature:generic-raider" },
        { recordKey: "creature:generic-marauder" },
      ];

      insertRecord(db, {
        recordKey: "creature:covered-fortress",
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
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "creature:keep-sentinel",
        name: "Keep Sentinel",
        category: "creature",
        traits: ["construct"],
        descriptionText: "An animate guardian stands watch over an old keep and its bastion.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "creature:generic-raider",
        name: "Generic Raider",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A marauder with no stable habitat cues and no obvious setting tie.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "creature:generic-marauder",
        name: "Generic Marauder",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A brute defined by role rather than place.",
        vector: [0.02, 0.98, 0],
      });

      const defaultReport = discoverUntaggedCohorts(db, {
        category: "creature",
        family: "setting",
        cohortLimit: 3,
        anchorLimit: 8,
      });
      expect(defaultReport.untaggedRecordCount).toBe(2);
      expect(defaultReport.reviewedRecords).toEqual(
        expect.objectContaining({
          mode: "excluded",
          scopedCount: 2,
          appliedCount: 2,
        }),
      );

      const reviewedReport = discoverUntaggedCohorts(db, {
        category: "creature",
        family: "setting",
        includeReviewed: true,
        reviewReason: "not_family_salient",
        cohortLimit: 3,
        anchorLimit: 8,
      });
      expect(reviewedReport.untaggedRecordCount).toBe(2);
      expect(reviewedReport.reviewedRecords).toEqual(
        expect.objectContaining({
          mode: "filtered",
          reviewReason: "not_family_salient",
          scopedCount: 2,
          appliedCount: 2,
        }),
      );
      expect(
        reviewedReport.cohorts.every((cohort) =>
          cohort.representativeRecords.every(
            (record) =>
              record.recordKey === "creature:generic-raider" || record.recordKey === "creature:generic-marauder",
          ),
        ),
      ).toBe(true);
    } finally {
      db.close();
    }
  });

  it("uses family-gap-signals to suppress taxonomy markers and classify new versus existing setting concepts", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "creature:covered-forest",
        name: "Forest Guardian",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A guardian of deep forest groves and jungle trails.",
        vector: [1, 0, 0],
        tags: ["forest_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:covered-temple",
        name: "Temple Warden",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A warden who serves in ancient temple sanctuaries.",
        vector: [0.95, 0.05, 0],
        tags: ["temple_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:covered-shadow",
        name: "Shadow Courier",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A courier who crosses the shadow plane at will.",
        vector: [0.92, 0.08, 0],
        tags: ["shadow_plane_setting"],
      });
      insertRecord(db, {
        recordKey: "creature:shadow-scout",
        name: "Shadow Scout",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "This scout stalks prey on the shadow plane beneath penumbral skies.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "creature:umbral-hunter",
        name: "Umbral Hunter",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "An umbral hunter crosses the plane of shadow with silent wings.",
        vector: [0.02, 0.98, 0],
      });
      insertRecord(db, {
        recordKey: "creature:penumbral-stalker",
        name: "Penumbral Stalker",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "A penumbral stalker emerges from the shadow plane to hunt alone.",
        vector: [0.01, 0.99, 0.01],
      });
      insertRecord(db, {
        recordKey: "creature:jungle-prowler",
        name: "Jungle Prowler",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "A dragon that lurks in jungle canopy and forest ruins.",
        vector: [0, 0, 1],
      });
      insertRecord(db, {
        recordKey: "creature:jungle-stalker",
        name: "Jungle Stalker",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "This dragon prowls jungle paths through thick forest canopy.",
        vector: [0.03, 0.02, 0.95],
      });
      insertRecord(db, {
        recordKey: "creature:erebus-scout",
        name: "Erebus Scout",
        category: "creature",
        traits: ["spirit"],
        descriptionText: "A scout slips through erebus in search of unquiet souls.",
        vector: [0.01, 0.94, 0.05],
      });
      insertRecord(db, {
        recordKey: "creature:erebus-stalker",
        name: "Erebus Stalker",
        category: "creature",
        traits: ["spirit"],
        descriptionText: "A stalker from erebus drifts behind the dead and forgotten.",
        vector: [0.02, 0.93, 0.05],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "creature",
        family: "setting",
        familyGapSignals: true,
        cohortLimit: 4,
        anchorLimit: 8,
        minFeatureSupport: 2,
        minFeatureLift: 1.05,
      });

      expect(report.anchorTerms.map((anchor) => anchor.value)).not.toContain("dragon");
      expect(
        report.cohorts.some(
          (cohort) => cohort.classification === "new_concept_candidate" && cohort.signature.includes("erebus"),
        ),
      ).toBe(true);
      expect(
        report.cohorts.some(
          (cohort) =>
            cohort.classification === "existing_tag_coverage_gap" &&
            cohort.overlappingTags?.includes("shadow_plane_setting"),
        ),
      ).toBe(true);
      expect(
        report.cohorts.some(
          (cohort) =>
            cohort.classification === "existing_tag_coverage_gap" && cohort.overlappingTags?.includes("forest_setting"),
        ),
      ).toBe(true);
    } finally {
      db.close();
    }
  });

  it("keeps source-local dragon variant ladders out of the top setting cohorts", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "fortress-core:covered-warden",
        name: "Fortress Warden",
        category: "creature",
        traits: ["humanoid"],
        descriptionText: "A warden posted inside an old fortress keep.",
        vector: [1, 0, 0],
        tags: ["fortress_setting"],
      });
      insertRecord(db, {
        recordKey: "fortress-a:watchtower-shadow",
        name: "Watchtower Shadow",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A shadow lurks in an abandoned fortress keep and its ruined watchtower.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "fortress-b:castle-sentry",
        name: "Castle Sentry",
        category: "creature",
        traits: ["construct"],
        descriptionText: "A sentry patrols a derelict fortress keep and its crumbling castle battlements.",
        vector: [0.97, 0.03, 0],
      });
      insertRecord(db, {
        recordKey: "fortress-c:bastion-ghost",
        name: "Bastion Ghost",
        category: "creature",
        traits: ["undead"],
        descriptionText: "A ghost haunts a forgotten fortress keep above the old bastion gate.",
        vector: [0.96, 0.04, 0],
      });
      insertRecord(db, {
        recordKey: "fortress-d:keep-predator",
        name: "Keep Predator",
        category: "creature",
        traits: ["beast"],
        descriptionText: "This predator nests among a fortress keep and broken watchtower ramparts.",
        vector: [0.95, 0.05, 0],
      });
      insertRecord(db, {
        recordKey: "storm-pack:storm-dragon-young",
        name: "Storm Dragon (Young)",
        category: "creature",
        traits: ["dragon"],
        variantFamilyKey: "creature:family:storm-dragon",
        variantBaseName: "Storm Dragon",
        variantLabel: "Young",
        variantAxes: ["age"],
        descriptionText: "This dragon circles the sky above black clouds and violent winds.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "storm-pack:storm-dragon-adult",
        name: "Storm Dragon (Adult)",
        category: "creature",
        traits: ["dragon"],
        variantFamilyKey: "creature:family:storm-dragon",
        variantBaseName: "Storm Dragon",
        variantLabel: "Adult",
        variantAxes: ["age"],
        descriptionText: "This dragon circles the sky above black clouds and violent winds.",
        vector: [0.01, 0.99, 0],
      });
      insertRecord(db, {
        recordKey: "storm-pack:storm-dragon-ancient",
        name: "Storm Dragon (Ancient)",
        category: "creature",
        traits: ["dragon"],
        variantFamilyKey: "creature:family:storm-dragon",
        variantBaseName: "Storm Dragon",
        variantLabel: "Ancient",
        variantAxes: ["age"],
        descriptionText: "This dragon circles the sky above black clouds and violent winds.",
        vector: [0.02, 0.98, 0],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "creature",
        family: "setting",
        familyGapSignals: true,
        cohortLimit: 6,
        anchorLimit: 10,
        minFeatureSupport: 2,
        minFeatureLift: 1.05,
      });

      expect(report.cohorts[0]?.overlappingTags ?? []).toContain("fortress_setting");
      expect(report.cohorts[0]?.recommendation).not.toBe("reject");
      expect(
        report.cohorts.some((cohort) =>
          cohort.representativeRecords.some((record) => record.name.includes("Storm Dragon")),
        ),
      ).toBe(false);
    } finally {
      db.close();
    }
  });

  it("rejects invalid CLI gram ranges", () => {
    expect(() => parseOptions(["--category", "equipment", "--max-gram-length", "6"])).toThrow(/max-gram-length/i);

    expect(() => parseOptions(["--category", "creature", "--family-gap-signals"])).toThrow(/family-gap-signals/i);
  });

  it("renders help with family-scoped semantics", () => {
    const help = formatHelp();

    expect(help).toContain("Usage:");
    expect(help).toContain("--family <derived-tag-family>");
    expect(help).toContain("--family-gap-signals");
    expect(help).toContain("With --family, it scans records missing tags from that family");
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
      expect(
        report.cohorts.some(
          (cohort) =>
            cohort.representativeRecords.filter((record) => record.name.startsWith("Wand of Choking Mist")).length <= 1,
        ),
      ).toBe(true);
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

      expect(
        report.cohorts.every(
          (cohort) => cohort.contrastRecords.filter((record) => record.name.startsWith("Words of Wisdom")).length <= 1,
        ),
      ).toBe(true);
    } finally {
      db.close();
    }
  });

  it("does not mark lexical-only treasure cohorts as rule-led", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "equipment:gem-1",
        name: "Sapphire Necklace",
        category: "equipment",
        subcategory: "treasure",
        descriptionText: "A blue sapphire necklace set in silver filigree.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:gem-2",
        name: "Sapphire Ring",
        category: "equipment",
        subcategory: "treasure",
        descriptionText: "A polished sapphire ring with a delicate band.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:gem-3",
        name: "Star Sapphire Brooch",
        category: "equipment",
        subcategory: "treasure",
        descriptionText: "A star sapphire brooch pinned to a velvet clasp.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:contrast-1",
        name: "Porcelain Doll",
        category: "equipment",
        subcategory: "treasure",
        descriptionText: "A porcelain doll with painted cheeks.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:contrast-2",
        name: "Gold Chalice",
        category: "equipment",
        subcategory: "treasure",
        descriptionText: "A gold chalice etched with tiny birds.",
        vector: [0.02, 0.98, 0],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        subcategory: "treasure",
        cohortLimit: 3,
        anchorLimit: 8,
        minFeatureSupport: 2,
        minFeatureLift: 1.1,
      });

      expect(report.cohorts[0]?.signature).toContain("sapphire");
      expect(report.cohorts[0]?.recommendation).not.toBe("rule-led");
    } finally {
      db.close();
    }
  });

  it("does not promote single-source named lines with weak non-name evidence to rule-led", () => {
    const db = createDiscoveryDb();
    try {
      insertRecord(db, {
        recordKey: "ap-war:chime-blasting",
        name: "Warcaller's Chime of Blasting",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and blasting runes.",
        vector: [1, 0, 0],
      });
      insertRecord(db, {
        recordKey: "ap-war:chime-dread",
        name: "Warcaller's Chime of Dread",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and dreadful omens.",
        vector: [0.99, 0.01, 0],
      });
      insertRecord(db, {
        recordKey: "ap-war:chime-refuge",
        name: "Warcaller's Chime of Refuge",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A warcaller's chime with carvings that depict battle scenes and refuge wards.",
        vector: [0.98, 0.02, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:cloak",
        name: "Traveler's Cloak",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A durable cloak for long journeys.",
        vector: [0, 1, 0],
      });
      insertRecord(db, {
        recordKey: "equipment:lamp",
        name: "Signal Lamp",
        category: "equipment",
        subcategory: "gear",
        descriptionText: "A hooded lamp for signaling at a distance.",
        vector: [0.01, 0.99, 0],
      });

      const report = discoverUntaggedCohorts(db, {
        category: "equipment",
        subcategory: "gear",
        cohortLimit: 3,
        anchorLimit: 10,
        minFeatureSupport: 2,
        minFeatureLift: 1.1,
      });

      expect(report.cohorts[0]?.reviewFlags).toContain("lexical-only");
      expect(report.cohorts[0]?.reviewFlags).toContain("source-local");
      expect(report.cohorts[0]?.recommendation).not.toBe("rule-led");
    } finally {
      db.close();
    }
  });
});
