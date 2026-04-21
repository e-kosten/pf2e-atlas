import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import {
  readDerivedTagMigrationSession,
  writeDerivedTagMigrationSession,
} from "../../src/tags/editorial/sessions/session-store.js";
import type { DerivedTagMigrationSession } from "../../src/tags/editorial/types.js";
import type { OntologyExplorerEntityRecord } from "../../src/app/ontology/entity-record.js";

function createEntityRecord(
  overrides: Partial<OntologyExplorerEntityRecord> &
    Pick<OntologyExplorerEntityRecord, "recordKey" | "name" | "category">,
): OntologyExplorerEntityRecord {
  return {
    recordKey: overrides.recordKey,
    packName: overrides.packName ?? overrides.recordKey.split(":")[0] ?? "",
    name: overrides.name,
    type: overrides.type ?? "unknown",
    category: overrides.category,
    subcategory: overrides.subcategory ?? null,
    documentType: overrides.documentType ?? "unknown",
    level: overrides.level ?? null,
    rarity: overrides.rarity ?? null,
    traits: overrides.traits ?? [],
    derivedTags: overrides.derivedTags ?? [],
    families: overrides.families ?? [],
    descriptionText: overrides.descriptionText ?? null,
    blurbText: overrides.blurbText ?? null,
    sourceCategory: overrides.sourceCategory ?? "unknown",
    publicationTitle: overrides.publicationTitle ?? null,
    publicationRemaster: overrides.publicationRemaster ?? false,
    isUnique: overrides.isUnique ?? false,
    size: overrides.size ?? null,
    languages: overrides.languages ?? [],
    speedTypes: overrides.speedTypes ?? [],
    senses: overrides.senses ?? [],
    immunities: overrides.immunities ?? [],
    resistances: overrides.resistances ?? [],
    weaknesses: overrides.weaknesses ?? [],
    itemCategory: overrides.itemCategory ?? null,
    baseItem: overrides.baseItem ?? null,
    priceCp: overrides.priceCp ?? null,
    actionCost: overrides.actionCost ?? null,
    usage: overrides.usage ?? null,
    hands: overrides.hands ?? null,
    damageTypes: overrides.damageTypes ?? [],
    weaponGroup: overrides.weaponGroup ?? null,
    armorGroup: overrides.armorGroup ?? null,
    traditions: overrides.traditions ?? [],
    spellKinds: overrides.spellKinds ?? [],
    saveType: overrides.saveType ?? null,
    areaType: overrides.areaType ?? null,
    rangeText: overrides.rangeText ?? null,
    durationText: overrides.durationText ?? null,
    targetText: overrides.targetText ?? null,
    areaValue: overrides.areaValue ?? null,
    sustained: overrides.sustained ?? false,
    basicSave: overrides.basicSave ?? false,
    disableText: overrides.disableText ?? null,
    disableSkills: overrides.disableSkills ?? [],
    isComplex: overrides.isComplex ?? false,
  };
}

describe("migration session store", () => {
  const tempRoots: string[] = [];

  afterEach(async () => {
    await Promise.all(tempRoots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
  });

  it("round-trips current migration sessions", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "pf2e-session-store-"));
    tempRoots.push(root);

    const session: DerivedTagMigrationSession = {
      manifest: {
        id: "session-current",
        mode: "proposal_review",
        category: "creature",
        subcategory: "character",
        family: "setting",
        tag: "urban_setting",
        createdAt: "2026-04-18T00:00:00.000Z",
        recordCount: 1,
      },
      records: [
        {
          entityRecord: createEntityRecord({
            recordKey: "creature:city-guard",
            name: "City Guard",
            category: "creature",
            subcategory: "character",
            traits: ["human"],
            derivedTags: ["urban_setting"],
            families: ["setting"],
            descriptionText: "Protects the city gates.",
          }),
          currentSources: {
            urban_setting: "assignment",
          },
          selectionReasons: [
            {
              source: "llm_assignment_review_queue",
              family: "setting",
              tag: "urban_setting",
              note: "Needs migration review.",
            },
          ],
        },
      ],
      decisions: [
        {
          recordKey: "creature:city-guard",
          name: "City Guard",
          category: "creature",
          resolutionStatus: "needs_review",
          decisions: [
            {
              kind: "assignment",
              family: "setting",
              tag: "urban_setting",
              mode: "include",
              status: "needs_review",
              confidence: "medium",
              rationale: "Looks like a city-aligned record.",
              source: "llm",
            },
          ],
        },
      ],
      reviewState: {
        currentIndex: 0,
        unresolvedOnly: true,
        updatedAt: "2026-04-18T00:00:00.000Z",
      },
    };

    await writeDerivedTagMigrationSession(root, session);

    await expect(readDerivedTagMigrationSession(root, session.manifest.id)).resolves.toEqual(session);
  });

  it("normalizes legacy session records into entity records", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "pf2e-session-store-"));
    tempRoots.push(root);

    const sessionDirectory = path.join(root, "scratch", "migration-sessions", "legacy-session");
    await mkdir(sessionDirectory, { recursive: true });
    await writeFile(
      path.join(sessionDirectory, "manifest.json"),
      `${JSON.stringify({
        id: "legacy-session",
        mode: "legacy_seed",
        category: "equipment",
        subcategory: "gear",
        createdAt: "2026-04-18T00:00:00.000Z",
        recordCount: 1,
      })}\n`,
      "utf8",
    );
    await writeFile(
      path.join(sessionDirectory, "records.jsonl"),
      `${JSON.stringify({
        recordKey: "equipment:lantern",
        name: "Lantern",
        category: "equipment",
        subcategory: "gear",
        packName: "equipment",
        level: 0,
        traits: ["mundane"],
        families: ["utility"],
        currentDerivedTags: ["illumination"],
        currentSources: { illumination: "seed_migration" },
        descriptionText: "Lights the way.",
        blurbText: null,
        selectionReasons: [{ source: "legacy_seed", tag: "illumination", note: "Legacy seed coverage." }],
      })}\n`,
      "utf8",
    );
    await writeFile(path.join(sessionDirectory, "decisions.jsonl"), "", "utf8");
    await writeFile(
      path.join(sessionDirectory, "review-state.json"),
      `${JSON.stringify({
        currentIndex: 0,
        unresolvedOnly: true,
        updatedAt: "2026-04-18T00:00:00.000Z",
      })}\n`,
      "utf8",
    );

    const session = await readDerivedTagMigrationSession(root, "legacy-session");

    expect(session.records).toHaveLength(1);
    expect(session.records[0]?.entityRecord.recordKey).toBe("equipment:lantern");
    expect(session.records[0]?.entityRecord.derivedTags).toEqual(["illumination"]);
    expect(session.records[0]?.entityRecord.subcategory).toBe("gear");
    expect(session.records[0]?.currentSources).toEqual({ illumination: "seed_migration" });
  });

  it("rejects invalid session manifests", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "pf2e-session-store-"));
    tempRoots.push(root);

    const sessionDirectory = path.join(root, "scratch", "migration-sessions", "invalid-session");
    await mkdir(sessionDirectory, { recursive: true });
    await writeFile(
      path.join(sessionDirectory, "manifest.json"),
      `${JSON.stringify({
        id: "invalid-session",
        mode: "legacy_seed",
        category: "relic",
        createdAt: "2026-04-18T00:00:00.000Z",
        recordCount: 0,
      })}\n`,
      "utf8",
    );
    await writeFile(path.join(sessionDirectory, "records.jsonl"), "", "utf8");
    await writeFile(path.join(sessionDirectory, "decisions.jsonl"), "", "utf8");
    await writeFile(
      path.join(sessionDirectory, "review-state.json"),
      `${JSON.stringify({
        currentIndex: 0,
        unresolvedOnly: true,
        updatedAt: "2026-04-18T00:00:00.000Z",
      })}\n`,
      "utf8",
    );

    await expect(readDerivedTagMigrationSession(root, "invalid-session")).rejects.toThrow(/Invalid search category/i);
  });
});
