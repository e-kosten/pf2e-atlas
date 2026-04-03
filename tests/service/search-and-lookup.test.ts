import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../../src/data/service.js";
import { RankingConfigStore } from "../../src/search/ranking-config.js";
import {
  createCapturingEmbeddingProviderFactory,
  createEmbeddingBatchTrackingProviderFactory,
  createFakeEmbeddingProviderFactory,
  initializeGitFixture,
  loadTestService,
  openPreparedTestService,
  TEST_HASH_EMBEDDING,
  writeJson,
} from "../helpers/pf2e-fixture.js";
import {
  cleanupCreatedRoots,
  createFixture,
  createHardFilterFixture,
} from "../helpers/pf2e-service-fixture.js";


describe("Pf2eDataService / Search and Lookup", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("supports lookup, listing, filtering, and derived metadata", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords({ pack: "actions" }).searchProfile).toBeNull();
    expect(service.listRecords({ pack: "actions" }).records).toHaveLength(4);
    expect(service.listRecords({ category: "creature", levelMin: 4, levelMax: 4, metadata: { field: "traits", op: "includesAny", values: ["undead"] } }).records.map((record) => record.name)).toEqual(["Ghost Commoner"]);
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "includesAll", values: ["fiend"] } })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "includesAll", values: ["fiend"] } })).searchProfile).toBeNull();
    expect((await service.search({ category: "creature", metadata: { field: "size", op: "eq", value: "sm" } })).records.every((record) => record.size === "sm")).toBe(true);
    expect((await service.search({ searchProfile: "lexical", query: "aberration", category: "creature" })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "spell", metadata: { field: "traditions", op: "includesAny", values: ["primal"] }, actionCost: 2 })).records[0]?.name).toBe("Sea Blessing");
    expect((await service.search({ category: "spell", metadata: { field: "spellKinds", op: "includesAny", values: ["focus"] } })).records.map((record) => record.name)).toEqual(["Focus Burst"]);
    expect((await service.search({ category: "rule", subcategory: "condition" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Blinded", "Dazzled", "Hidden"]),
    );
    expect((await service.search({ category: "hazard" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Mournful Hallway", "Spear Launcher"]),
    );
    expect((await service.search({ category: "hazard", subcategory: "trap" })).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect(service.listRecords({ pack: "Pathfinder Monster Core", category: "hazard", subcategory: "trap" }).records.map((record) => record.name)).toEqual(["Spear Launcher"]);
    expect((await service.search({ category: "affliction" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Cackling Delirium", "Calcifying Rot"]),
    );
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "excludesAny", values: ["water"] } })).records.some((record) => record.traits.includes("water"))).toBe(false);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "hasDescription", op: "eq", value: true } })).records.every((record) => record.hasDescription)).toBe(true);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "sourceCategory", op: "notIn", values: ["adventure"] } })).records[0]?.sourceCategory).toBe("core");
    expect((await service.search({ category: "creature", metadata: { field: "sourceCategory", op: "eq", value: "core" } })).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect((await service.search({ query: "ghost ship", category: "creature" })).mode).toBe("hybrid");
    expect((await service.search({ query: "ghost ship", category: "creature" })).searchProfile).toBe("balanced");
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_poison"] } }).records.map((record) => record.name)).toEqual(["Antidote (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_recovery"] } }).records.map((record) => record.name)).toEqual(["Bottled Catharsis (Serenity)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["energy_resistance"] } }).records.map((record) => record.name)).toEqual(["Potion of Cold Resistance (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["escape_support"] } }).records.map((record) => record.name)).toEqual(["Escape Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["senses_support"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Potion of Disguise (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["navigation"] } }).records.map((record) => record.name)).toEqual(["Traveler's Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)", "Tracker's Stew"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Aroma Concealer"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["lock_bypass"] } }).records.map((record) => record.name)).toEqual(["Concealable Thieves' Tools"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Boots of Free Running (Greater)", "Climbing Kit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Tracker's Goggles", "Tracking Tag"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Trackless"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", metadata: { field: "derivedTags", op: "includesAny", values: ["carry_support"] } }).records.map((record) => record.name)).toEqual(["Spacious Pouch (Type I)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAll", values: ["beneficial", "anti_disease"] } }).records.map((record) => record.name)).toEqual(["Antiplague (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "excludesAny", values: ["offensive"] } }).records.map((record) => record.name)).not.toContain("Sightless Tincture");
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Masquerade Scarf", "Quick-Change Outfit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Shacklebreaker", "Swallow-Spike"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).not.toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace", "False Manacles", "Manacles of Persuasion"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).not.toContain("Shacklebreaker");
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Alarm Ward"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(["Snaring Glyph"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] } }).records.map((record) => record.name)).toEqual(["Cackling Delirium"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility_impairment"] } }).records.map((record) => record.name)).toEqual(["Calcifying Rot"]);
    expect(service.listRecords({
      category: "equipment",
      metadata: {
        and: [
          { field: "weaponGroup", op: "eq", value: "bomb" },
          { field: "hands", op: "eq", value: 1 },
        ],
      },
    }).records.map((record) => record.name)).toEqual(["Ghost Charge Prototype"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["aquatic_context"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Sailor", "Pelagic Stalker", "Ship Captain"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["scene_adjacent"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ship Captain", "Wealthy Vigilante"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["ghost"] } }).records.map((record) => record.name)).toEqual(["Ghost Commoner"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["lich"] } }).records.map((record) => record.name)).toEqual(["Mythic Lich"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAny", values: ["seafarer"] } }).records.map((record) => record.name)).toEqual(["Bosun"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "families", op: "includesAll", values: ["mythic", "lich"] } }).records.map((record) => record.name)).toEqual(["Mythic Lich"]);
    expect(service.listRecords({ category: "creature", levelMin: 5, levelMax: 5, metadata: { field: "families", op: "excludesAny", values: ["vampire"] } }).records.map((record) => record.name)).not.toContain("Morlock Thrall");

    const cythnigot = service.lookup("Cythnigot", { category: "creature" }).match;
    expect(cythnigot?.hasDescription).toBe(true);
    expect(cythnigot?.descriptionSnippet).toBe("Small aberration.");
    expect(cythnigot?.sourceCategory).toBe("core");
    expect(cythnigot?.category).toBe("creature");
    const seaBlessing = service.lookup("Sea Blessing", { category: "spell" }).match;
    expect(seaBlessing?.subcategory).toBeNull();
    expect(seaBlessing?.traditions).toEqual(["primal"]);
    const focusBurst = service.lookup("Focus Burst", { category: "spell" }).match;
    expect(focusBurst?.subcategory).toBeNull();
    expect(focusBurst?.spellKinds).toEqual(["focus"]);
    const illusoryDisguise = service.lookup("Illusory Disguise", { category: "spell" }).match;
    expect(illusoryDisguise?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const antidote = service.lookup("Antidote (Lesser)", { category: "equipment" }).match;
    expect(antidote?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "anti_poison"]));
    expect(antidote?.derivedTags).not.toEqual(expect.arrayContaining(["offensive", "thrown_offense"]));
    const bottledCatharsis = service.lookup("Bottled Catharsis (Serenity)", { category: "equipment" }).match;
    expect(bottledCatharsis?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "condition_support", "mental_recovery"]));
    const coldResistancePotion = service.lookup("Potion of Cold Resistance (Moderate)", { category: "equipment" }).match;
    expect(coldResistancePotion?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "energy_resistance", "buff_support", "self_buff"]));
    const bloodhoundMask = service.lookup("Bloodhound Mask (Greater)", { category: "equipment" }).match;
    expect(bloodhoundMask?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "senses_support", "self_buff", "tracking"]));
    expect(bloodhoundMask?.derivedTags).not.toContain("anti_tracking");
    const aromaConcealer = service.lookup("Aroma Concealer", { category: "equipment" }).match;
    expect(aromaConcealer?.derivedTags).toContain("anti_tracking");
    expect(aromaConcealer?.derivedTags).not.toContain("tracking");
    const ichthyosisMutagen = service.lookup("Ichthyosis Mutagen", { category: "equipment" }).match;
    expect(ichthyosisMutagen?.derivedTags).not.toContain("tracking");
    expect(ichthyosisMutagen?.derivedTags).not.toContain("anti_tracking");
    const escapeFulu = service.lookup("Escape Fulu", { category: "equipment" }).match;
    expect(escapeFulu?.derivedTags).toEqual(expect.arrayContaining(["beneficial", "escape_support", "buff_support", "self_buff"]));
    const travelersFulu = service.lookup("Traveler's Fulu", { category: "equipment" }).match;
    expect(travelersFulu?.derivedTags).toContain("navigation");
    const trackersStew = service.lookup("Tracker's Stew", { category: "equipment" }).match;
    expect(trackersStew?.derivedTags).toContain("tracking");
    const disguisePotion = service.lookup("Potion of Disguise (Moderate)", { category: "equipment" }).match;
    expect(disguisePotion?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const shipCaptain = service.lookup("Ship Captain", { category: "creature" }).match;
    expect(shipCaptain?.derivedTags).toEqual(expect.arrayContaining(["nautical", "profession_npc", "scene_adjacent"]));
    const wealthyVigilante = service.lookup("Wealthy Vigilante", { category: "creature" }).match;
    expect(wealthyVigilante?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "scene_adjacent"]));
    const ghostCommoner = service.lookup("Ghost Commoner", { category: "creature" }).match;
    expect(ghostCommoner?.families).toEqual(["ghost"]);
    const mythicLich = service.lookup("Mythic Lich", { category: "creature" }).match;
    expect(mythicLich?.families).toEqual(["lich", "mythic"]);
    const morlockThrall = service.lookup("Morlock Thrall", { category: "creature" }).match;
    expect(morlockThrall?.families).toEqual(["vampire"]);
    expect(morlockThrall?.derivedTags).toContain("undead_threat");
    const bosun = service.lookup("Bosun", { category: "creature" }).match;
    expect(bosun?.families).toEqual(["seafarer"]);
    const pelagicStalker = service.lookup("Pelagic Stalker", { category: "creature" }).match;
    expect(pelagicStalker?.derivedTags).toContain("aquatic_context");
    const spaciousPouch = service.lookup("Spacious Pouch (Type I)", { category: "equipment" }).match;
    expect(spaciousPouch?.derivedTags).toContain("carry_support");
    const bootsOfFreeRunning = service.lookup("Boots of Free Running (Greater)", { category: "equipment" }).match;
    expect(bootsOfFreeRunning?.derivedTags).toContain("mobility");
    expect(bootsOfFreeRunning?.derivedTags).not.toContain("climbing");
    const trackersGoggles = service.lookup("Tracker's Goggles", { category: "equipment" }).match;
    expect(trackersGoggles?.derivedTags).toEqual(expect.arrayContaining(["navigation", "survival", "tracking"]));
    const trackingTag = service.lookup("Tracking Tag", { category: "equipment" }).match;
    expect(trackingTag?.derivedTags).toContain("tracking");
    const trackless = service.lookup("Trackless", { category: "equipment" }).match;
    expect(trackless?.derivedTags).toContain("anti_tracking");
    expect(trackless?.derivedTags).not.toContain("tracking");
    const masqueradeScarf = service.lookup("Masquerade Scarf", { category: "equipment" }).match;
    expect(masqueradeScarf?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const quickChangeOutfit = service.lookup("Quick-Change Outfit", { category: "equipment" }).match;
    expect(quickChangeOutfit?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const shacklebreaker = service.lookup("Shacklebreaker", { category: "equipment" }).match;
    expect(shacklebreaker?.derivedTags).toContain("restraint_escape");
    expect(shacklebreaker?.derivedTags).not.toContain("restraint_capture");
    const swallowSpike = service.lookup("Swallow-Spike", { category: "equipment" }).match;
    expect(swallowSpike?.derivedTags).toContain("restraint_escape");
    const lawbringersLasso = service.lookup("Lawbringer's Lasso", { category: "equipment" }).match;
    expect(lawbringersLasso?.derivedTags).toContain("restraint_capture");
    expect(lawbringersLasso?.derivedTags).not.toContain("restraint_escape");
    const injigosLovingEmbrace = service.lookup("Injigo's Loving Embrace", { category: "equipment" }).match;
    expect(injigosLovingEmbrace?.derivedTags).toContain("restraint_capture");
    expect(injigosLovingEmbrace?.derivedTags).not.toContain("restraint_escape");
    const falseManacles = service.lookup("False Manacles", { category: "equipment" }).match;
    expect(falseManacles?.derivedTags).toContain("restraint_capture");
    expect(falseManacles?.derivedTags).not.toContain("restraint_escape");
    const manaclesOfPersuasion = service.lookup("Manacles of Persuasion", { category: "equipment" }).match;
    expect(manaclesOfPersuasion?.derivedTags).toContain("restraint_capture");
    const handcuffs = service.lookup("Handcuffs (Average)", { category: "equipment" }).match;
    expect(handcuffs?.derivedTags).toContain("restraint_capture");
    expect(handcuffs?.derivedTags).not.toContain("restraint_escape");
    const catchPole = service.lookup("Catch Pole", { category: "equipment" }).match;
    expect(catchPole?.derivedTags).toContain("restraint_capture");
    expect(catchPole?.derivedTags).not.toContain("restraint_escape");
    const alarmWard = service.lookup("Alarm Ward", { category: "hazard" }).match;
    expect(alarmWard?.derivedTags).toContain("alarm");
    const snaringGlyph = service.lookup("Snaring Glyph", { category: "hazard" }).match;
    expect(snaringGlyph?.derivedTags).toContain("restraint_capture");
    const cacklingDelirium = service.lookup("Cackling Delirium", { category: "affliction" }).match;
    expect(cacklingDelirium?.subcategory).toBe("curse");
    expect(cacklingDelirium?.derivedTags).toContain("mental_impairment");
    const calcifyingRot = service.lookup("Calcifying Rot", { category: "affliction" }).match;
    expect(calcifyingRot?.subcategory).toBe("disease");
    expect(calcifyingRot?.derivedTags).toContain("mobility_impairment");
    const ghostChargePrototype = service.lookup("Ghost Charge Prototype", { category: "equipment" }).match;
    expect(ghostChargePrototype?.weaponGroup).toBe("bomb");
    expect(ghostChargePrototype?.hands).toBe(1);
    expect(ghostChargePrototype?.damageTypes).toEqual(["positive"]);
    expect(service.lookup("Blinded", { category: "rule", subcategory: "condition" }).match?.category).toBe("rule");
  });

  it("keeps deterministic listing distinct from structured ranked search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const listed = service.listRecords({
      category: "creature",
      levelMin: 2,
      levelMax: 2,
    }).records.map((record) => `${record.name}::${record.packLabel}`);
    const searched = (await service.search({
      category: "creature",
      levelMin: 2,
      levelMax: 2,
    })).records.map((record) => `${record.name}::${record.packLabel}`);

    expect(listed).not.toEqual(searched);
    expect(listed.indexOf("Bilge Skeleton::Quest for the Frozen Flame")).toBeLessThan(
      listed.indexOf("Diver::Pathfinder Monster Core"),
    );
    expect(searched.indexOf("Bilge Skeleton::Quest for the Frozen Flame")).toBeGreaterThan(
      searched.indexOf("Diver::Pathfinder Monster Core"),
    );
  });

  it("normalizes legacy plural aliases and supports scoped mixed-family filters", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const legacyCategoryLookup = service.lookup("Cythnigot", { category: "creatures" }).match;
    expect(legacyCategoryLookup?.category).toBe("creature");

    const scopedResults = await service.search({
      scopes: [
        { category: "feats" },
        { category: "rules", subcategories: ["actions"] },
      ],
      limit: 20,
    });
    expect(scopedResults.records.some((record) => record.name === "Deep Focus" && record.category === "feat")).toBe(true);
    expect(scopedResults.records.some((record) => record.name === "Refocus" && record.category === "rule")).toBe(true);
    expect(scopedResults.records.some((record) => record.category === "creature")).toBe(false);

    await expect(service.search({
      scopes: [{ category: "feat", subcategories: ["action"] }],
    })).rejects.toThrow(/does not belong to category "feat"/i);
  });

  it("requires a text query or structured filters for ranked search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    await expect(service.search({})).rejects.toThrow("pf2e_search requires search text and/or at least one structured filter.");
    await expect(service.search({ searchProfile: "concept" })).rejects.toThrow("pf2e_search requires search text and/or at least one structured filter.");
  });

  it("maps user-facing search profiles onto the underlying retrieval modes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const lexicalResults = await service.search({
      searchProfile: "lexical",
      query: "aberration",
      category: "creature",
    });
    expect(lexicalResults.searchProfile).toBe("lexical");
    expect(lexicalResults.mode).toBe("lexical");
    expect(lexicalResults.records[0]?.name).toBe("Cythnigot");

    const balancedResults = await service.search({
      searchProfile: "balanced",
      query: "ghost ship",
      category: "creature",
    });
    expect(balancedResults.searchProfile).toBe("balanced");
    expect(balancedResults.mode).toBe("hybrid");

    const conceptResults = await service.search({
      searchProfile: "concept",
      query: "ghost ship",
      category: "creature",
      explain: true,
    });
    expect(conceptResults.searchProfile).toBe("concept");
    expect(conceptResults.mode).toBe("hybrid");
    expect(conceptResults.explain?.searchProfile).toBe("concept");
    expect(conceptResults.explain?.fusionMethod).toBe("weightedRrf");
    expect(conceptResults.explain?.fusionProfile).toBe("concept");
    expect(conceptResults.explain?.fusionConfig).toEqual({
      rrfK: 60,
      lexicalWeight: 0.3,
      semanticWeight: 0.7,
      lexicalTopK: 100,
      semanticTopK: 150,
    });
  });

  it("uses normalized text for lexical scoring and raw query text for embeddings", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const embeddingCalls: string[] = [];

    const service = await loadTestService(fixture, {
      embeddingProviderFactory: createCapturingEmbeddingProviderFactory(embeddingCalls, {
        provider: "hash",
        model: "capture-model",
        revision: null,
        dimensions: 8,
      }),
    });

    const query = "  Ghost-ship: body horror?!  ";
    const result = await service.search({
      searchProfile: "concept",
      query,
      category: "creature",
      explain: true,
    });

    expect(embeddingCalls.at(-1)).toBe("Ghost-ship: body horror?!");
    expect(result.explain?.semanticQuery).toBe("Ghost-ship: body horror?!");
    expect(result.explain?.lexicalQuery).toBe("ghost ship body horror");
    expect(result.explain?.query?.normalizedQuery).toBe("ghost ship body horror");
  });

  it("surfaces haunted-ship swarm candidates in broad themed search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const broadQuery =
      "ghost ship cursed voyage fear fog darkness possession maddening whispers vermin in the hold wrong-feeling stowaways body horror haunted physically unclean";
    const broadResults = await service.search({
      category: "creature",
      levelMin: 1,
      levelMax: 5,
      rarity: "common",
      query: broadQuery,
      limit: 20,
      explain: true,
    });
    const broadNames = broadResults.records.map((record) => record.name);
    const crawlingIndex = broadNames.indexOf("Crawling Hand Swarm");

    expect(broadResults.mode).toBe("hybrid");
    expect(crawlingIndex).toBeGreaterThanOrEqual(0);

    const crawlingExplain = broadResults.explain?.records.find((record) => record.name === "Crawling Hand Swarm");
    expect(broadResults.explain?.query?.queryTokens).toEqual(expect.arrayContaining(["ghost", "ship", "body", "horror"]));
    expect(Array.isArray(crawlingExplain?.matchedTraits)).toBe(true);
    expect(Array.isArray(crawlingExplain?.matchedNameTokens)).toBe(true);
    expect(typeof crawlingExplain?.lexicalRerankScore).toBe("number");
    expect(crawlingExplain?.fusionScore).not.toBeNull();
    expect(crawlingExplain?.rerankAdjustments.sourcePenalty ?? 0).toBe(0);

    const lexicalResults = await service.search({
      category: "creature",
      levelMin: 1,
      levelMax: 5,
      searchProfile: "lexical",
      query: "undead swarm body horror haunted ship crawling infestation severed limbs cursed voyage",
      limit: 20,
    });
    const lexicalNames = lexicalResults.records.map((record) => record.name);
    const lexicalCrawlingIndex = lexicalNames.indexOf("Crawling Hand Swarm");
    const lexicalDiverIndex = lexicalNames.indexOf("Diver");
    const lexicalLionIndex = lexicalNames.indexOf("Lion");
    expect(lexicalCrawlingIndex).toBeGreaterThanOrEqual(0);
    expect(lexicalDiverIndex).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
    expect(lexicalLionIndex).toSatisfy((index) => index === -1 || index > lexicalCrawlingIndex);
  });

  it("applies small source-quality preferences and stronger thematic unique penalties", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const bilgeResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(bilgeResults.records[0]?.sourceCategory).toBe("core");

    const coreBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.rerankAdjustments.sourceQuality > 0);
    const adventureBilgeExplain = bilgeResults.explain?.records.find((record) => record.name === "Bilge Skeleton" && record.rerankAdjustments.sourceQuality < 0);
    expect(coreBilgeExplain?.rerankAdjustments.sourceQuality).toBe(0.04);
    expect(adventureBilgeExplain?.rerankAdjustments.sourceQuality).toBe(-0.01);

    const sentinelResults = await service.search({
      category: "creature",
      query: "sentinel guardian ancient ruins watch intruders",
      limit: 10,
      explain: true,
    });
    const sentinelNames = sentinelResults.records.map((record) => record.name);
    const commonIndex = sentinelNames.indexOf("Amber Sentinel");
    const uncommonIndex = sentinelNames.indexOf("Azure Sentinel");
    const rareIndex = sentinelNames.indexOf("Gloam Sentinel");
    const uniqueIndex = sentinelNames.indexOf("Last Sentinel");

    expect(commonIndex).toBeGreaterThanOrEqual(0);
    expect(uncommonIndex).toBeGreaterThanOrEqual(0);
    expect(rareIndex).toBeGreaterThanOrEqual(0);
    expect(uniqueIndex).toBeGreaterThan(rareIndex);

    const uniqueExplain = sentinelResults.explain?.records.find((record) => record.name === "Last Sentinel");
    const rareExplain = sentinelResults.explain?.records.find((record) => record.name === "Gloam Sentinel");
    expect(uniqueExplain?.rerankAdjustments.rarityPreference).toBe(-0.2);
    expect(rareExplain?.rerankAdjustments.rarityPreference).toBe(0.01);

    const exactUniqueResults = await service.search({
      category: "creature",
      nameQuery: "Last Sentinel",
      explain: true,
    });
    expect(exactUniqueResults.records[0]?.name).toBe("Last Sentinel");
    const exactUniqueExplain = exactUniqueResults.explain?.records.find((record) => record.name === "Last Sentinel");
    expect(exactUniqueExplain?.rerankAdjustments.rarityPreference).toBe(-0.03);
  });

  it("hot-reloads ranking weights without rebuilding the service", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const rankingConfigPath = path.join(fixture.root, "pf2e-ranking.json");
    const rankingConfigStore = await RankingConfigStore.create(rankingConfigPath, { watch: false });
    const service = await loadTestService(fixture, { rankingConfigStore });

    const baselineResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(baselineResults.records[0]?.sourceCategory).toBe("core");
    expect(baselineResults.explain?.rankingConfig.source).toBe("default");

    const baselineRevision = service.getRankingConfigStatus().revision;
    await writeJson(rankingConfigPath, {
      sourceQuality: {
        core: -0.5,
        adventure: 0.5,
      },
    });
    await rankingConfigStore.reload();

    const updatedResults = await service.search({
      category: "creature",
      nameQuery: "Bilge Skeleton",
      explain: true,
    });
    expect(updatedResults.records[0]?.sourceCategory).toBe("adventure");
    expect(updatedResults.explain?.rankingConfig.source).toBe("file");
    expect(updatedResults.explain?.rankingConfig.revision).toBeGreaterThan(baselineRevision);
    expect(updatedResults.explain?.records.some((record) => record.rerankAdjustments.sourceQuality === 0.5)).toBe(true);
    service.close();
  });

  it("excludes dedicated Pathfinder Society content while retaining base equivalents", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listPacks().map((pack) => pack.name)).not.toContain("macros");
    expect(service.listPacks().map((pack) => pack.name)).not.toContain("action-macros");
    expect(service.lookup("Grimstalker", { category: "creature" }).match?.name).toBe("Grimstalker");
    expect(service.lookup("Ghoul", { category: "creature" }).match?.name).toBe("Ghoul");
    expect(service.lookup("Zebub", { category: "creature" }).match?.name).toBe("Zebub");
    expect(service.lookup("Raise Shield", { category: "rule", subcategory: "action" }).match?.name).toBe("Raise a Shield");

    expect((await service.search({ nameQuery: "Grimstalker (PFS 3-13)", category: "creature" })).records.map((record) => record.name)).not.toContain("Grimstalker (PFS 3-13)");
    expect((await service.search({ nameQuery: "Ghoul (PFS Intro 2)", category: "creature" })).records.map((record) => record.name)).not.toContain("Ghoul (PFS Intro 2)");
    expect((await service.search({ nameQuery: "Zebub (PFS)", category: "creature" })).records.map((record) => record.name)).not.toContain("Zebub (PFS)");
    expect((await service.search({ nameQuery: "Magical Mentor" })).records.map((record) => record.name)).not.toContain("Magical Mentor");
    expect((await service.search({ nameQuery: "Effect: Magical Mentor" })).records.map((record) => record.name)).not.toContain("Effect: Magical Mentor");
    expect((await service.search({ nameQuery: "Treat Wounds" })).records.map((record) => record.name)).not.toContain("Treat Wounds");
    expect((await service.search({ nameQuery: "Trip: Athletics" })).records.map((record) => record.name)).not.toContain("Trip: Athletics");

    const featResults = (await service.search({
      category: "feat",
      query: "mentor training support teamwork guidance",
      limit: 10,
    })).records.map((record) => record.name);
    expect(featResults).toContain("Proud Mentor");
    expect(featResults).not.toContain("Magical Mentor");
  });

  it("indexes verified aliases onto remaster canonical records and exposes linked legacy records", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const embedCalls: string[] = [];
    const service = await loadTestService(fixture, {
      embeddingProviderFactory: createCapturingEmbeddingProviderFactory(embedCalls, {
        provider: "hash",
        model: "feature-hash-192",
        revision: null,
        dimensions: 4,
      }),
    });

    expect(service.lookup("Attack of Opportunity", { category: "rule", subcategory: "action" }).match?.name).toBe("Reactive Strike");
    expect(service.lookup("Strike Back", { category: "rule", subcategory: "action" }).match?.name).toBe("Reactive Strike");
    expect(service.lookup("flat-footed", { category: "rule", subcategory: "condition" }).match?.name).toBe("Off-Guard");
    expect(service.lookup("Aasimar").match?.name).toBe("Nephilim");
    expect(service.lookup("Ifrit").match?.name).toBe("Naari");
    expect(service.lookup("Feather Token (Swan Boat)").match?.name).toBe("Marvelous Miniature (Boat)");
    expect(service.lookup("Bag of Holding", { category: "equipment" }).match?.name).toBe("Spacious Pouch (Type I)");
    expect(service.lookup("Attack of Opportunity", { category: "rule", subcategory: "action" }).match?.aliases).toContain("Attack of Opportunity");
    expect(service.lookup("Strike Back", { category: "rule", subcategory: "action" }).match?.aliases).toContain("Strike Back");
    expect(service.lookup("flat-footed", { category: "rule", subcategory: "condition" }).match?.aliases).toContain("flat-footed");
    expect(service.lookup("Aasimar").match?.aliases).toContain("Aasimar");
    expect(service.lookup("Ifrit").match?.aliases).toContain("Ifrit");
    expect(service.lookup("Bag of Holding", { category: "equipment" }).match?.aliases).toContain("Bag of Holding");

    const attackSearch = await service.search({
      category: "rule",
      subcategory: "action",
      nameQuery: "Attack of Opportunity",
    });
    expect(attackSearch.records.map((record) => record.name)).toContain("Reactive Strike");
    expect(attackSearch.records.map((record) => record.name)).not.toContain("Attack of Opportunity");

    const offGuard = service.lookup("Off-Guard", { category: "rule", subcategory: "condition" }).match;
    expect(offGuard?.aliases).toContain("flat-footed");
    expect(offGuard?.legacyRecordLinks).toEqual([
      {
        recordKey: "conditionitems:Flat-Footed",
        name: "Flat-Footed",
      },
    ]);
    expect(service.getRecord(offGuard!.legacyRecordLinks[0]!.recordKey)?.name).toBe("Flat-Footed");

    expect(embedCalls.some((text) => text.includes("Attack of Opportunity") && text.includes("Reactive Strike"))).toBe(true);
    expect(embedCalls.some((text) => text.includes("flat-footed") && text.includes("Off-Guard"))).toBe(true);
    expect(embedCalls.some((text) => text.includes("Aasimar") && text.includes("Nephilim"))).toBe(true);

    const nephilim = service.lookup("Nephilim").match;
    expect(nephilim?.aliases).toContain("Tiefling");
    expect(nephilim?.aliases).not.toContain("and Tiefling");

    const naari = service.lookup("Naari").match;
    expect(naari?.aliases).toContain("Ifrit");
    expect(naari?.aliases.some((alias) => alias.includes("are now"))).toBe(false);

    const boat = service.lookup("Marvelous Miniature (Boat)").match;
    expect(boat?.aliases).toContain("Feather Token (Swan Boat)");

    expect(service.lookup("Sight-Theft Grit", { category: "equipment" }).match?.name).not.toBe("Surging Serum (Lesser)");
    expect(service.lookup("Surging Serum (Lesser)", { category: "equipment" }).match?.aliases).not.toContain("Sight-Theft Grit");
  });
});
