import { describe, expect, it } from "vitest";

import {
  analyzeDiscoveryEvidenceFromRecords,
  analyzeFamilyGapEvidenceFromRecords,
} from "../../src/tags/evaluation/evidence-analyzer.js";
import type { DiscoveryAnalysisRecord } from "../../src/tags/discovery/discovery-records.js";

function record(
  input: Partial<DiscoveryAnalysisRecord> & Pick<DiscoveryAnalysisRecord, "recordKey" | "name" | "category">,
): DiscoveryAnalysisRecord {
  const sourceKey = input.sourceKey ?? input.recordKey.split(":")[0] ?? input.recordKey;
  return {
    recordKey: input.recordKey,
    sourceKey,
    packName: input.packName ?? sourceKey,
    publicationTitle: input.publicationTitle ?? null,
    folderId: input.folderId ?? null,
    sourcePath: input.sourcePath ?? null,
    sourcePathSlice: input.sourcePathSlice ?? null,
    name: input.name,
    category: input.category,
    subcategory: input.subcategory ?? null,
    variantFamilyKey: input.variantFamilyKey ?? null,
    variantBaseName: input.variantBaseName ?? null,
    variantLabel: input.variantLabel ?? null,
    variantAxes: input.variantAxes ?? [],
    level: input.level ?? null,
    traits: input.traits ?? [],
    derivedTags: input.derivedTags ?? [],
    descriptionText: input.descriptionText ?? null,
    vector: input.vector ?? new Float32Array(0),
    references: input.references ?? [],
  };
}

describe("derived-tag evidence analyzer", () => {
  it("surfaces normalized phrases and reference signals that distinguish a cohort", () => {
    const cohort = [
      record({
        recordKey: "spell:1",
        name: "Blazing Cone",
        category: "spell",
        traits: ["fire", "evocation"],
        descriptionText: "Deals 2d6 fire damage in a 30-foot cone and lights torches.",
        references: [
          {
            targetRecordKey: "rule:1",
            targetName: "Ignite",
            targetCategory: "rule",
            targetSubcategory: "action",
            fromPackName: "actionspf2e",
            fromRecordType: "action",
            fromSourceCategory: "rules",
          },
        ],
      }),
      record({
        recordKey: "spell:2",
        name: "Ashen Cone",
        category: "spell",
        traits: ["fire", "evocation"],
        descriptionText: "Deals 4d6 fire damage in a 60-foot cone and lights braziers.",
        references: [
          {
            targetRecordKey: "rule:1",
            targetName: "Ignite",
            targetCategory: "rule",
            targetSubcategory: "action",
            fromPackName: "actionspf2e",
            fromRecordType: "action",
            fromSourceCategory: "rules",
          },
        ],
      }),
    ];
    const baseline = [
      ...cohort,
      record({
        recordKey: "spell:3",
        name: "Bracing Ward",
        category: "spell",
        traits: ["abjuration"],
        descriptionText: "Gain resistance and protective cover.",
      }),
    ];

    const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, {
      limit: 4,
      exampleLimit: 2,
    });

    expect(report.descriptionPhrases.map((term) => term.value)).toContain("fire damage");
    expect(report.traits.map((term) => term.value)).toContain("fire");
    expect(report.references.map((term) => term.value)).toContain("target:ignite");
  });

  it("filters foundry markup and activation boilerplate from surfaced evidence", () => {
    const cohort = [
      record({
        recordKey: "equipment:1",
        name: "Skyhook Harness",
        category: "equipment",
        descriptionText:
          "Activate 1 command. Effect @UUID[Compendium.pf2e.equipment-srd.Item.Rope]{Skyhook} line launches upward.",
      }),
      record({
        recordKey: "equipment:2",
        name: "Wallhook Rig",
        category: "equipment",
        descriptionText: "Activate 1 command. Effect the skyhook line catches on high ledges.",
      }),
    ];
    const baseline = [
      ...cohort,
      record({
        recordKey: "equipment:3",
        name: "Beacon Lantern",
        category: "equipment",
        descriptionText: "Activate 1 command. Effect the lantern shines brightly.",
      }),
    ];

    const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, {
      limit: 6,
      exampleLimit: 2,
    });

    expect(report.descriptionTokens.map((term) => term.value)).toContain("skyhook");
    expect(report.descriptionTokens.map((term) => term.value)).not.toContain("activate");
    expect(report.descriptionTokens.map((term) => term.value)).not.toContain("uuid");
    expect(report.descriptionTokens.map((term) => term.value)).not.toContain("compendium");
  });

  it("prefers repeated traits and references over singleton description phrases", () => {
    const cohort = [
      record({
        recordKey: "spell:1",
        name: "Rune Lash",
        category: "spell",
        traits: ["force"],
        descriptionText: "A lash of force arcs from your hand. A hidden catacomb sigil flares in the target's shadow.",
        references: [
          {
            targetRecordKey: "rule:1",
            targetName: "Force Barrage",
            targetCategory: "rule",
            targetSubcategory: "action",
            fromPackName: "actionspf2e",
            fromRecordType: "action",
            fromSourceCategory: "rules",
          },
        ],
      }),
      record({
        recordKey: "spell:2",
        name: "Rune Spear",
        category: "spell",
        traits: ["force"],
        descriptionText: "A spear of force pierces forward. The ancient plaza inscription briefly glows at impact.",
        references: [
          {
            targetRecordKey: "rule:1",
            targetName: "Force Barrage",
            targetCategory: "rule",
            targetSubcategory: "action",
            fromPackName: "actionspf2e",
            fromRecordType: "action",
            fromSourceCategory: "rules",
          },
        ],
      }),
      record({
        recordKey: "spell:3",
        name: "Rune Arc",
        category: "spell",
        traits: ["force"],
        descriptionText:
          "A crackling line of force whips through the air. The silver observatory arch hums softly nearby.",
        references: [
          {
            targetRecordKey: "rule:1",
            targetName: "Force Barrage",
            targetCategory: "rule",
            targetSubcategory: "action",
            fromPackName: "actionspf2e",
            fromRecordType: "action",
            fromSourceCategory: "rules",
          },
        ],
      }),
    ];
    const baseline = [
      ...cohort,
      record({
        recordKey: "spell:4",
        name: "Stone Ward",
        category: "spell",
        traits: ["abjuration"],
        descriptionText: "A layer of stone protects you from harm.",
      }),
    ];

    const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, {
      limit: 6,
      exampleLimit: 2,
    });

    const forceTerm = report.traits.find((term) => term.value === "force");
    const referenceTerm = report.references.find((term) => term.value === "target:force barrage");
    const singletonDescriptionTerm = report.descriptionPhrases.find((term) => term.value === "hidden catacomb sigil");

    expect(forceTerm?.score ?? 0).toBeGreaterThan(singletonDescriptionTerm?.score ?? 0);
    expect(referenceTerm?.score ?? 0).toBeGreaterThan(singletonDescriptionTerm?.score ?? 0);
  });

  it("supports configurable phrase lengths up to five grams", () => {
    const cohort = [
      record({
        recordKey: "creature:1",
        name: "Mourning Herald",
        category: "creature",
        descriptionText: "It walks beneath a moonlit ruined bell tower while whispering omens.",
      }),
      record({
        recordKey: "creature:2",
        name: "Bell Tower Shade",
        category: "creature",
        descriptionText: "This spirit circles the moonlit ruined bell tower as it whispers omens.",
      }),
    ];
    const baseline = [
      ...cohort,
      record({
        recordKey: "creature:3",
        name: "Sunny Traveler",
        category: "creature",
        descriptionText: "A traveler rests beside a market fountain at noon.",
      }),
    ];

    const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, {
      limit: 6,
      minGramLength: 4,
      maxGramLength: 5,
    });

    expect(report.descriptionPhrases.map((term) => term.value)).toContain("moonlit ruined bell tower");
    expect(report.descriptionPhrases.map((term) => term.value)).not.toContain("ruined bell tower");
  });

  it("splits family-gap evidence into new concepts, existing-tag overlaps, and suppressed taxonomy", () => {
    const uncovered = [
      record({
        recordKey: "creature:1",
        name: "Umbral Hunter",
        category: "creature",
        descriptionText: "This hunter stalks prey on the shadow plane.",
      }),
      record({
        recordKey: "creature:2",
        name: "Darklands Prowler",
        category: "creature",
        descriptionText: "A prowler dwells in the Darklands below Orv.",
      }),
      record({
        recordKey: "creature:3",
        name: "Jungle Shade",
        category: "creature",
        traits: ["dragon"],
        descriptionText: "This creature lurks in the jungle canopy.",
      }),
      record({
        recordKey: "creature:4",
        name: "Erebus Watcher",
        category: "creature",
        descriptionText: "An erebus watcher prowls the border between life and death.",
      }),
      record({
        recordKey: "creature:5",
        name: "Erebus Stalker",
        category: "creature",
        descriptionText: "A stalker from erebus drifts just beyond the reach of mortal souls.",
      }),
      record({
        recordKey: "creature:6",
        name: "Cloud Raider",
        category: "creature",
        descriptionText: "A raider circles the sky until prey slips away from the city walls.",
      }),
    ];
    const covered = [
      record({
        recordKey: "creature:covered-1",
        name: "Forest Stag",
        category: "creature",
        derivedTags: ["forest_setting"],
        descriptionText: "A guardian of the forest canopy and jungle paths.",
      }),
      record({
        recordKey: "creature:covered-2",
        name: "Shadow Envoy",
        category: "creature",
        derivedTags: ["shadow_plane_setting"],
        descriptionText: "A messenger who serves on the shadow plane.",
      }),
      record({
        recordKey: "creature:covered-3",
        name: "Cavern Lurker",
        category: "creature",
        derivedTags: ["underground_setting"],
        descriptionText: "A hunter of cavern roads deep in the Darklands.",
      }),
      record({
        recordKey: "creature:covered-4",
        name: "Temple Sentinel",
        category: "creature",
        derivedTags: ["temple_setting"],
        descriptionText: "A sentinel of the sacred temple halls.",
      }),
    ];
    const baseline = [...uncovered, ...covered];

    const report = analyzeFamilyGapEvidenceFromRecords(
      uncovered,
      covered,
      baseline,
      "setting",
      ["forest_setting", "temple_setting"],
      { limit: 6, exampleLimit: 2 },
    );

    expect(
      report.existingTagCoverageGaps.some(
        (term) => term.value === "shadow plane" && term.existingTagOverlaps.includes("shadow_plane_setting"),
      ),
    ).toBe(true);
    expect(
      report.existingTagCoverageGaps.some(
        (term) =>
          (term.value === "darklands" || term.value === "orv") &&
          term.existingTagOverlaps.includes("underground_setting"),
      ),
    ).toBe(true);
    expect(report.suppressedTerms.length).toBeGreaterThan(0);
  });
});
