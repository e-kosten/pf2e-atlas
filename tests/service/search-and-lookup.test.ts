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
    expect(service.listRecords({ category: "creature", levelMin: 4, levelMax: 4, metadata: { field: "traits", op: "includesAny", values: ["undead"] } }).records.map((record) => record.name)).toEqual(["Cairn Wight", "Ghost Commoner"]);
    const fiendCreatures = await service.search({ category: "creature", metadata: { field: "traits", op: "includesAll", values: ["fiend"] } });
    expect(fiendCreatures.records.map((record) => record.name)).toEqual(expect.arrayContaining(["Caldera Oni", "Cythnigot"]));
    expect(fiendCreatures.searchProfile).toBeNull();
    expect((await service.search({ category: "creature", metadata: { field: "size", op: "eq", value: "sm" } })).records.every((record) => record.size === "sm")).toBe(true);
    expect((await service.search({ searchProfile: "lexical", query: "aberration", category: "creature" })).records[0]?.name).toBe("Cythnigot");
    expect((await service.search({ category: "spell", metadata: { field: "traditions", op: "includesAny", values: ["primal"] }, actionCost: 2 })).records[0]?.name).toBe("Sea Blessing");
    expect((await service.search({ category: "spell", metadata: { field: "spellKinds", op: "includesAny", values: ["focus"] } })).records.map((record) => record.name)).toEqual(["Focus Burst"]);
    expect((await service.search({ category: "spell", metadata: { field: "saveType", op: "eq", value: "reflex" } })).records.map((record) => record.name)).toEqual(["Hydraulic Push"]);
    expect((await service.search({ category: "spell", metadata: { field: "areaType", op: "eq", value: "burst" } })).records.map((record) => record.name)).toEqual(["Hydraulic Push"]);
    expect((await service.search({ category: "spell", metadata: { field: "durationUnit", op: "eq", value: "minute" } })).records.map((record) => record.name)).toEqual(["Fear", "Painted Scout"]);
    expect((await service.search({ category: "spell", metadata: { field: "sustained", op: "eq", value: true } })).records.map((record) => record.name)).toEqual(["Painted Scout"]);
    expect((await service.search({ category: "spell", metadata: { field: "basicSave", op: "eq", value: true } })).records.map((record) => record.name)).toEqual(["Hydraulic Push"]);
    expect((await service.search({ category: "spell", metadata: { field: "areaValue", op: "gte", value: 10 } })).records.map((record) => record.name)).toEqual(["Fear"]);
    expect((await service.search({ category: "rule", subcategory: "condition" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Blinded", "Dazzled", "Hidden"]),
    );
    expect((await service.search({ category: "hazard" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Mournful Hallway", "Spear Launcher"]),
    );
    expect((await service.search({ category: "hazard", subcategory: "trap" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Images of Failure", "Mental Assault", "Spear Launcher"]),
    );
    expect(service.listRecords({ pack: "Pathfinder Monster Core", category: "hazard", subcategory: "trap" }).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Images of Failure", "Mental Assault", "Spear Launcher"]),
    );
    expect((await service.search({ category: "affliction" })).records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Cackling Delirium", "Calcifying Rot"]),
    );
    expect((await service.search({ category: "creature", metadata: { field: "traits", op: "excludesAny", values: ["water"] } })).records.some((record) => record.traits.includes("water"))).toBe(false);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "hasDescription", op: "eq", value: true } })).records.every((record) => record.hasDescription)).toBe(true);
    expect((await service.search({ category: "creature", nameQuery: "Ghost Sailor", metadata: { field: "sourceCategory", op: "notIn", values: ["adventure"] } })).records[0]?.sourceCategory).toBe("core");
    expect((await service.search({ category: "creature", metadata: { field: "sourceCategory", op: "eq", value: "core" } })).records.every((record) => record.sourceCategory === "core")).toBe(true);
    expect((await service.search({ query: "ghost ship", category: "creature" })).mode).toBe("hybrid");
    expect((await service.search({ query: "ghost ship", category: "creature" })).searchProfile).toBe("balanced");
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["epidemic_pestilence"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bubonic Plague"]));
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["void_soul_corruption"] } }).records.map((record) => record.name)).toEqual(["Reaper's Shadow"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["nightmare_torment"] } }).records.map((record) => record.name)).toEqual(["Endless Nightmare"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_poison"] } }).records.map((record) => record.name)).toEqual(["Antidote (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_recovery"] } }).records.map((record) => record.name)).toEqual(["Bottled Catharsis (Serenity)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["energy_resistance"] } }).records.map((record) => record.name)).toEqual(["Potion of Cold Resistance (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["escape_support"] } }).records.map((record) => record.name)).toEqual(["Escape Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["senses_support"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Potion of Disguise (Moderate)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["navigation"] } }).records.map((record) => record.name)).toEqual(["Traveler's Fulu"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bloodhound Mask (Greater)", "Tracker's Stew"]));
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Aroma Concealer"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["sustenance"] } }).records.map((record) => record.name)).toEqual(["Ration Tonic"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["lock_bypass"] } }).records.map((record) => record.name)).toEqual(["Concealable Thieves' Tools"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Boots of Free Running (Greater)", "Climbing Kit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["tracking"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Tracker's Goggles", "Tracking Tag"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["anti_tracking"] } }).records.map((record) => record.name)).toEqual(["Trackless"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["mounted_support"] } }).records.map((record) => record.name)).toEqual(["War Saddle"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["sustenance"] } }).records.map((record) => record.name)).toEqual(["Ring of Sustenance"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["aquatic_support"] } }).records.map((record) => record.name)).toEqual(["Sailor's Collar"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", metadata: { field: "derivedTags", op: "includesAny", values: ["carry_support"] } }).records.map((record) => record.name)).toEqual(["Spacious Pouch (Type I)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", metadata: { field: "derivedTags", op: "includesAny", values: ["extradimensional_storage"] } }).records.map((record) => record.name)).toEqual(["Spacious Pouch (Type I)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "backpack", metadata: { field: "derivedTags", op: "includesAny", values: ["weapon_staging"] } }).records.map((record) => record.name)).toEqual(["Gunner's Bandolier"]);
    expect(service.listRecords({ category: "equipment", subcategory: "ammo", metadata: { field: "derivedTags", op: "includesAny", values: ["ammo_management"] } }).records.map((record) => record.name)).toEqual(["Repeating Crossbow Magazine"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAll", values: ["beneficial", "anti_disease"] } }).records.map((record) => record.name)).toEqual(["Antiplague (Lesser)"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "excludesAny", values: ["offensive"] } }).records.map((record) => record.name)).not.toContain("Sightless Tincture");
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Masquerade Scarf", "Quick-Change Outfit"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Shacklebreaker", "Swallow-Spike"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_escape"] } }).records.map((record) => record.name)).not.toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Catch Pole", "Handcuffs (Average)", "Lawbringer's Lasso", "Injigo's Loving Embrace", "False Manacles", "Manacles of Persuasion"]));
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).not.toContain("Shacklebreaker");
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Alarm"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["disguise"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] } }).records.map((record) => record.name)).toEqual(["Illusory Disguise"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["scouting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Painted Scout", "Web of Eyes"]));
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["navigation"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Return Beacon"]));
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Air Walk"]));
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] } }).records.map((record) => record.name)).toEqual(["Fear"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["sensory_impairment"] } }).records.map((record) => record.name)).toEqual(["Blindness"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["forced_movement"] } }).records.map((record) => record.name)).toEqual(["Hydraulic Push"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(["Phantom Prison"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["countermagic"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Veil of Privacy"]));
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["persistent_damage"] } }).records.map((record) => record.name)).toEqual(["Acid Arrow"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["initiative_support"] } }).records.map((record) => record.name)).toEqual(["Anticipate Peril"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["eidolon_support"] } }).records.map((record) => record.name)).toEqual(["Protect Companion"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Alarm Ward"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Images of Failure", "Mental Assault"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["fire_hazard"] } }).records.map((record) => record.name)).toEqual(["Explosive Barrels"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["poison_hazard"] } }).records.map((record) => record.name)).toEqual(["Gas Trap"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["pitfall"] } }).records.map((record) => record.name)).toEqual(["Drowning Pit"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["collapse_hazard"] } }).records.map((record) => record.name)).toEqual(["Collapsing Bridge"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["forced_movement"] } }).records.map((record) => record.name)).toEqual(["Rushing Wind"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["spawned_attackers"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Mask Summoning Rune", "Shadow Guards"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["phantom_assailants"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Phantom Soldiers"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["spawned_attackers"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Dream-Poisoned Door"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["control_interface"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Hallowed Wheel"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["barrier_lockdown"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Crushing Gate Trap"]));
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["navigation_disruption"] } }).records.map((record) => record.name)).toEqual(["Confounding Portal"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["overhead_strike"] } }).records.map((record) => record.name)).toEqual(["Falling Debris"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Alarm Snare", "Sentry Fulu", "Warning Snare"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["alarm"] } }).records.map((record) => record.name)).toEqual(["Floorbell"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["signaling"] } }).records.map((record) => record.name)).toEqual(["Flare Beacon (Moderate)"]);
    expect(service.listRecords({ category: "spell", metadata: { field: "derivedTags", op: "includesAny", values: ["message_delivery"] } }).records.map((record) => record.name)).toEqual(["Message Rune", "Sending"]);
    expect(service.listRecords({ category: "equipment", subcategory: "gear", metadata: { field: "derivedTags", op: "includesAny", values: ["countermagic"] } }).records.map((record) => record.name)).toEqual(["Countering Charm"]);
    expect(service.listRecords({ category: "equipment", subcategory: "consumable", metadata: { field: "derivedTags", op: "includesAny", values: ["magic_protection"] } }).records.map((record) => record.name)).toEqual(["Antimagic Oil"]);
    expect(service.listRecords({ category: "hazard", metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] } }).records.map((record) => record.name)).toEqual(["Ash Web", "Snaring Glyph"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] } }).records.map((record) => record.name)).toEqual(["Cackling Delirium"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["mobility_impairment"] } }).records.map((record) => record.name)).toEqual(["Calcifying Rot"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["respiratory_impairment"] } }).records.map((record) => record.name)).toEqual(["Black Apoxia"]);
    expect(service.listRecords({ category: "affliction", metadata: { field: "derivedTags", op: "includesAny", values: ["transformative_corruption"] } }).records.map((record) => record.name)).toEqual(["Crystal Corruption"]);
    expect(service.listRecords({
      category: "equipment",
      metadata: {
        and: [
          { field: "weaponGroup", op: "eq", value: "bomb" },
          { field: "hands", op: "eq", value: 1 },
        ],
      },
    }).records.map((record) => record.name)).toEqual(["Ghost Charge Prototype"]);
    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "baseItem", op: "eq", value: "alchemical-bomb" },
    }).records.map((record) => record.name)).toContain("Ghost Charge Prototype");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["aquatic_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Sailor", "Pelagic Stalker", "Ship Captain"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["freshwater_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Amelekana", "Electric Eel", "Water Orm", "Gathganara"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["coastal_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Coastal Prowler", "Sea Drake", "Ship Captain"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["astral_setting"] } }).records.map((record) => record.name)).toContain("Astradaemon");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["first_world_setting"] } }).records.map((record) => record.name)).toContain("Blodeuwedd");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["boneyard_setting"] } }).records.map((record) => record.name)).toContain("Catrina");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["heaven_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Empyreal Dragon", "Rekhep"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["nirvana_setting"] } }).records.map((record) => record.name)).toContain("Guloval");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["elysium_setting"] } }).records.map((record) => record.name)).toContain("Ghaele");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["celestial_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Empyreal Dragon", "Rekhep", "Guloval", "Ghaele"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["hell_setting"] } }).records.map((record) => record.name)).toContain("Insidiator");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["abyss_setting"] } }).records.map((record) => record.name)).toContain("Vrock");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["fiendish_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Insidiator", "Vrock"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["shadow_plane_setting"] } }).records.map((record) => record.name)).toContain("Shae");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["maelstrom_setting"] } }).records.map((record) => record.name)).toContain("Naunet");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["island_setting"] } }).records.map((record) => record.name)).toContain("Island Watcher");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["forest_setting"] } }).records.map((record) => record.name)).toContain("Jungle Stalker");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["plains_setting"] } }).records.map((record) => record.name)).toContain("Plains Runner");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["canyon_setting"] } }).records.map((record) => record.name)).toContain("Canyon Stalker");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["swamp_setting"] } }).records.map((record) => record.name)).toContain("Boggard Mire Scout");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["urban_setting"] } }).records.map((record) => record.name)).toContain("Anugobu Apprentice");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["temple_setting"] } }).records.map((record) => record.name)).not.toContain("Anugobu Apprentice");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["underground_setting"] } }).records.map((record) => record.name)).not.toContain("Anugobu Apprentice");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["graveyard_setting"] } }).records.map((record) => record.name)).toContain("Cairn Wight");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["ruins_setting"] } }).records.map((record) => record.name)).toContain("Temple Scavenger");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["temple_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Temple Custodian", "Temple Scavenger"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["fortress_setting"] } }).records.map((record) => record.name)).toContain("Fortress Warden");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["wasteland_setting"] } }).records.map((record) => record.name)).toContain("Wasteland Reclaimer");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["volcanic_setting"] } }).records.map((record) => record.name)).toEqual(["Caldera Oni"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["nautical_setting"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Pirate Captain", "Ship Captain"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["undead_adjacent"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Ghost Pirate Captain", "Cairn Wight", "Morlock Thrall"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["possession_threat"] } }).records.map((record) => record.name)).toEqual(["Body Snatcher"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["life_drain_threat"] } }).records.map((record) => record.name)).toEqual(["Soul Drinker"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["spawn_creator"] } }).records.map((record) => record.name)).toEqual(["Brood Mother"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["petrification_threat"] } }).records.map((record) => record.name)).toEqual(["Stone Gaze Basilisk"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["regeneration_threat"] } }).records.map((record) => record.name)).toEqual(["Marsh Troll"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["ambush_grabber"] } }).records.map((record) => record.name)).toEqual(["Web Lurker"]);
    const civicNpcNames = service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["civic_npc"] } }).records.map((record) => record.name);
    expect(civicNpcNames).toEqual(expect.arrayContaining(["Guild Engineer", "Ship Captain", "Priest of Pharasma", "High Priest of Pharasma"]));
    expect(civicNpcNames).not.toContain("Boggard Mire Scout");
    expect(civicNpcNames).not.toContain("Fortress Warden");
    expect(civicNpcNames).not.toContain("Hellknight Gaoler");
    expect(civicNpcNames).not.toContain("Wealthy Vigilante");
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["combatant_npc"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Bandit", "Fortress Warden", "Hellknight Gaoler"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["rural_setting"] } }).records.map((record) => record.name)).toEqual(["Scarecrow"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["carnival_show"] } }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Court Jester", "Mechanical Carny"]));
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["living_toy"] } }).records.map((record) => record.name)).toEqual(["Soulbound Doll"]);
    expect(service.listRecords({ category: "creature", metadata: { field: "derivedTags", op: "includesAny", values: ["trickster_chaos"] } }).records.map((record) => record.name)).toEqual(["Fire Scamp"]);
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
    const fear = service.lookup("Fear", { category: "spell" }).match;
    expect(fear?.saveType).toBe("will");
    expect(fear?.areaType).toBe("cone");
    expect(fear?.durationText).toBe("1 minute");
    expect(fear?.durationUnit).toBe("minute");
    expect(fear?.targetText).toBe("1 creature");
    expect(fear?.areaValue).toBe(30);
    expect(fear?.basicSave).toBe(false);
    const hydraulicPush = service.lookup("Hydraulic Push", { category: "spell" }).match;
    expect(hydraulicPush?.saveType).toBe("reflex");
    expect(hydraulicPush?.areaType).toBe("burst");
    expect(hydraulicPush?.rangeText).toBe("60 feet");
    expect(hydraulicPush?.targetText).toBe("1 creature");
    expect(hydraulicPush?.areaValue).toBe(5);
    expect(hydraulicPush?.basicSave).toBe(true);
    const alarm = service.lookup("Alarm", { category: "spell" }).match;
    expect(alarm?.derivedTags).toContain("alarm");
    const illusoryDisguise = service.lookup("Illusory Disguise", { category: "spell" }).match;
    expect(illusoryDisguise?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const webOfEyes = service.lookup("Web of Eyes", { category: "spell" }).match;
    expect(webOfEyes?.derivedTags).toContain("scouting");
    const paintedScout = service.lookup("Painted Scout", { category: "spell" }).match;
    expect(paintedScout?.derivedTags).toContain("scouting");
    expect(paintedScout?.sustained).toBe(true);
    expect(paintedScout?.durationUnit).toBe("minute");
    const airWalk = service.lookup("Air Walk", { category: "spell" }).match;
    expect(airWalk?.derivedTags).toContain("mobility");
    const returnBeacon = service.lookup("Return Beacon", { category: "spell" }).match;
    expect(returnBeacon?.derivedTags).toContain("navigation");
    const messageRune = service.lookup("Message Rune", { category: "spell" }).match;
    expect(messageRune?.derivedTags).toContain("message_delivery");
    const veilOfPrivacy = service.lookup("Veil of Privacy", { category: "spell" }).match;
    expect(veilOfPrivacy?.derivedTags).toContain("countermagic");
    expect(veilOfPrivacy?.derivedTags).not.toContain("scouting");
    const acidArrow = service.lookup("Acid Arrow", { category: "spell" }).match;
    expect(acidArrow?.derivedTags).toContain("persistent_damage");
    const anticipatePeril = service.lookup("Anticipate Peril", { category: "spell" }).match;
    expect(anticipatePeril?.derivedTags).toContain("initiative_support");
    const protectCompanion = service.lookup("Protect Companion", { category: "spell" }).match;
    expect(protectCompanion?.derivedTags).toContain("eidolon_support");
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
    const warSaddle = service.lookup("War Saddle", { category: "equipment" }).match;
    expect(warSaddle?.derivedTags).toContain("mounted_support");
    const rationTonic = service.lookup("Ration Tonic", { category: "equipment" }).match;
    expect(rationTonic?.derivedTags).toContain("sustenance");
    expect(rationTonic?.derivedTags).not.toContain("aquatic_support");
    const sailorsCollar = service.lookup("Sailor's Collar", { category: "equipment" }).match;
    expect(sailorsCollar?.derivedTags).toContain("aquatic_support");
    const ringOfSustenance = service.lookup("Ring of Sustenance", { category: "equipment" }).match;
    expect(ringOfSustenance?.derivedTags).toContain("sustenance");
    const disguisePotion = service.lookup("Potion of Disguise (Moderate)", { category: "equipment" }).match;
    expect(disguisePotion?.derivedTags).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));
    const shipCaptain = service.lookup("Ship Captain", { category: "creature" }).match;
    expect(shipCaptain?.derivedTags).toEqual(expect.arrayContaining(["nautical_setting", "coastal_setting", "profession_npc", "civic_npc"]));
    const ghostPirateCaptain = service.lookup("Ghost Pirate Captain", { category: "creature" }).match;
    expect(ghostPirateCaptain?.derivedTags).toEqual(expect.arrayContaining(["nautical_setting", "aquatic_setting", "undead_adjacent"]));
    const amelekana = service.lookup("Amelekana", { category: "creature" }).match;
    expect(amelekana?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(amelekana?.derivedTags).not.toContain("coastal_setting");
    const electricEel = service.lookup("Electric Eel", { category: "creature" }).match;
    expect(electricEel?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(electricEel?.derivedTags).not.toContain("coastal_setting");
    const waterOrm = service.lookup("Water Orm", { category: "creature" }).match;
    expect(waterOrm?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(waterOrm?.derivedTags).not.toContain("coastal_setting");
    const gathganara = service.lookup("Gathganara", { category: "creature" }).match;
    expect(gathganara?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const defacedNaiadQueen = service.lookup("Defaced Naiad Queen", { category: "creature" }).match;
    expect(defacedNaiadQueen?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const coldmirePond = service.lookup("Coldmire Pond", { category: "creature" }).match;
    expect(coldmirePond?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const boilingSpring = service.lookup("Boiling Spring", { category: "creature" }).match;
    expect(boilingSpring?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const coastalProwler = service.lookup("Coastal Prowler", { category: "creature" }).match;
    expect(coastalProwler?.derivedTags).toContain("coastal_setting");
    expect(coastalProwler?.derivedTags).not.toContain("aquatic_setting");
    const islandWatcher = service.lookup("Island Watcher", { category: "creature" }).match;
    expect(islandWatcher?.derivedTags).toContain("island_setting");
    const anugobuApprentice = service.lookup("Anugobu Apprentice", { category: "creature" }).match;
    expect(anugobuApprentice?.derivedTags).toContain("island_setting");
    expect(anugobuApprentice?.derivedTags).toContain("urban_setting");
    expect(anugobuApprentice?.derivedTags).not.toEqual(expect.arrayContaining(["temple_setting", "underground_setting"]));
    const jungleStalker = service.lookup("Jungle Stalker", { category: "creature" }).match;
    expect(jungleStalker?.derivedTags).toContain("forest_setting");
    const plainsRunner = service.lookup("Plains Runner", { category: "creature" }).match;
    expect(plainsRunner?.derivedTags).toContain("plains_setting");
    const canyonStalker = service.lookup("Canyon Stalker", { category: "creature" }).match;
    expect(canyonStalker?.derivedTags).toContain("canyon_setting");
    const boggardMireScout = service.lookup("Boggard Mire Scout", { category: "creature" }).match;
    expect(boggardMireScout?.derivedTags).toContain("swamp_setting");
    const cairnWight = service.lookup("Cairn Wight", { category: "creature" }).match;
    expect(cairnWight?.derivedTags).toEqual(expect.arrayContaining(["graveyard_setting", "undead_adjacent"]));
    const templeScavenger = service.lookup("Temple Scavenger", { category: "creature" }).match;
    expect(templeScavenger?.derivedTags).toEqual(expect.arrayContaining(["ruins_setting", "temple_setting"]));
    const templeCustodian = service.lookup("Temple Custodian", { category: "creature" }).match;
    expect(templeCustodian?.derivedTags).toContain("temple_setting");
    const fortressWarden = service.lookup("Fortress Warden", { category: "creature" }).match;
    expect(fortressWarden?.derivedTags).toEqual(expect.arrayContaining(["fortress_setting", "profession_npc", "combatant_npc"]));
    expect(fortressWarden?.derivedTags).not.toContain("civic_npc");
    const wastelandReclaimer = service.lookup("Wasteland Reclaimer", { category: "creature" }).match;
    expect(wastelandReclaimer?.derivedTags).toContain("wasteland_setting");
    const calderaOni = service.lookup("Caldera Oni", { category: "creature" }).match;
    expect(calderaOni?.derivedTags).toContain("volcanic_setting");
    const wealthyVigilante = service.lookup("Wealthy Vigilante", { category: "creature" }).match;
    expect(wealthyVigilante?.derivedTags).toContain("profession_npc");
    expect(wealthyVigilante?.derivedTags).not.toContain("civic_npc");
    const priestOfPharasma = service.lookup("Priest of Pharasma", { category: "creature" }).match;
    expect(priestOfPharasma?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));
    const highPriestOfPharasma = service.lookup("High Priest of Pharasma", { category: "creature" }).match;
    expect(highPriestOfPharasma?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));
    const travelingPriestOfDesna = service.lookup("Traveling Priest of Desna", { category: "creature" }).match;
    expect(travelingPriestOfDesna?.derivedTags).toContain("profession_npc");
    expect(travelingPriestOfDesna?.derivedTags).not.toContain("civic_npc");
    const guildEngineer = service.lookup("Guild Engineer", { category: "creature" }).match;
    expect(guildEngineer?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "civic_npc"]));
    expect(guildEngineer?.derivedTags).not.toContain("combatant_npc");
    expect(boggardMireScout?.derivedTags).toEqual(expect.arrayContaining(["swamp_setting", "profession_npc"]));
    expect(boggardMireScout?.derivedTags).not.toContain("civic_npc");
    const bandit = service.lookup("Bandit", { category: "creature" }).match;
    expect(bandit?.derivedTags).toContain("combatant_npc");
    const hellknightGaoler = service.lookup("Hellknight Gaoler", { category: "creature" }).match;
    expect(hellknightGaoler?.derivedTags).toEqual(expect.arrayContaining(["profession_npc", "combatant_npc"]));
    expect(hellknightGaoler?.derivedTags).not.toContain("civic_npc");
    const scarecrow = service.lookup("Scarecrow", { category: "creature" }).match;
    expect(scarecrow?.derivedTags).toContain("rural_setting");
    const animatedTeaCart = service.lookup("Animated Tea Cart", { category: "creature" }).match;
    expect(animatedTeaCart?.derivedTags).toContain("animated_object");
    const oldManStatue = service.lookup("Old Man Statue", { category: "creature" }).match;
    expect(oldManStatue?.derivedTags).toContain("animated_statue");
    const courtJester = service.lookup("Court Jester", { category: "creature" }).match;
    expect(courtJester?.derivedTags).toContain("carnival_show");
    const mechanicalCarny = service.lookup("Mechanical Carny", { category: "creature" }).match;
    expect(mechanicalCarny?.derivedTags).toContain("carnival_show");
    expect(mechanicalCarny?.derivedTags).not.toContain("living_toy");
    const soulboundDoll = service.lookup("Soulbound Doll", { category: "creature" }).match;
    expect(soulboundDoll?.derivedTags).toContain("living_toy");
    const fireScamp = service.lookup("Fire Scamp", { category: "creature" }).match;
    expect(fireScamp?.derivedTags).toContain("trickster_chaos");
    const ghostCommoner = service.lookup("Ghost Commoner", { category: "creature" }).match;
    expect(ghostCommoner?.families).toEqual(["ghost"]);
    const mythicLich = service.lookup("Mythic Lich", { category: "creature" }).match;
    expect(mythicLich?.families).toEqual(["lich", "mythic"]);
    const morlockThrall = service.lookup("Morlock Thrall", { category: "creature" }).match;
    expect(morlockThrall?.families).toEqual(["vampire"]);
    expect(morlockThrall?.derivedTags).toContain("undead_adjacent");
    const bosun = service.lookup("Bosun", { category: "creature" }).match;
    expect(bosun?.families).toEqual(["seafarer"]);
    const pelagicStalker = service.lookup("Pelagic Stalker", { category: "creature" }).match;
    expect(pelagicStalker?.derivedTags).toContain("aquatic_setting");
    const castruccioIrovetti = service.lookup("Castruccio Irovetti", { category: "creature" }).match;
    expect(castruccioIrovetti?.derivedTags).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const apothecaryBee = service.lookup("Apothecary Bee", { category: "creature" }).match;
    expect(apothecaryBee?.derivedTags).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const astradaemon = service.lookup("Astradaemon", { category: "creature" }).match;
    expect(astradaemon?.derivedTags).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(astradaemon?.derivedTags).toContain("astral_setting");
    const shrineCaretaker = service.lookup("Shrine Caretaker", { category: "creature" }).match;
    expect(shrineCaretaker?.derivedTags).toContain("temple_setting");
    expect(shrineCaretaker?.derivedTags).not.toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    const seaDrake = service.lookup("Sea Drake", { category: "creature" }).match;
    expect(seaDrake?.derivedTags).toEqual(expect.arrayContaining(["aquatic_setting", "coastal_setting"]));
    const blodeuwedd = service.lookup("Blodeuwedd", { category: "creature" }).match;
    expect(blodeuwedd?.derivedTags).toContain("first_world_setting");
    const bodySnatcher = service.lookup("Body Snatcher", { category: "creature" }).match;
    expect(bodySnatcher?.derivedTags).toContain("possession_threat");
    const soulDrinker = service.lookup("Soul Drinker", { category: "creature" }).match;
    expect(soulDrinker?.derivedTags).toContain("life_drain_threat");
    const broodMother = service.lookup("Brood Mother", { category: "creature" }).match;
    expect(broodMother?.derivedTags).toContain("spawn_creator");
    const stoneGazeBasilisk = service.lookup("Stone Gaze Basilisk", { category: "creature" }).match;
    expect(stoneGazeBasilisk?.derivedTags).toContain("petrification_threat");
    const marshTroll = service.lookup("Marsh Troll", { category: "creature" }).match;
    expect(marshTroll?.derivedTags).toContain("regeneration_threat");
    const webLurker = service.lookup("Web Lurker", { category: "creature" }).match;
    expect(webLurker?.derivedTags).toContain("ambush_grabber");
    const catrina = service.lookup("Catrina", { category: "creature" }).match;
    expect(catrina?.derivedTags).toContain("boneyard_setting");
    const rekhep = service.lookup("Rekhep", { category: "creature" }).match;
    expect(rekhep?.derivedTags).toEqual(expect.arrayContaining(["heaven_setting", "celestial_setting"]));
    const guloval = service.lookup("Guloval", { category: "creature" }).match;
    expect(guloval?.derivedTags).toEqual(expect.arrayContaining(["nirvana_setting", "celestial_setting"]));
    const ghaele = service.lookup("Ghaele", { category: "creature" }).match;
    expect(ghaele?.derivedTags).toEqual(expect.arrayContaining(["elysium_setting", "celestial_setting"]));
    const empyrealDragon = service.lookup("Empyreal Dragon", { category: "creature" }).match;
    expect(empyrealDragon?.derivedTags).toEqual(expect.arrayContaining(["heaven_setting", "celestial_setting"]));
    expect(empyrealDragon?.derivedTags).not.toEqual(expect.arrayContaining(["nirvana_setting", "elysium_setting"]));
    const insidiator = service.lookup("Insidiator", { category: "creature" }).match;
    expect(insidiator?.derivedTags).toEqual(expect.arrayContaining(["hell_setting", "fiendish_setting"]));
    const vrock = service.lookup("Vrock", { category: "creature" }).match;
    expect(vrock?.derivedTags).toEqual(expect.arrayContaining(["abyss_setting", "fiendish_setting"]));
    const shae = service.lookup("Shae", { category: "creature" }).match;
    expect(shae?.derivedTags).toContain("shadow_plane_setting");
    const naunet = service.lookup("Naunet", { category: "creature" }).match;
    expect(naunet?.derivedTags).toContain("maelstrom_setting");
    const hooktongue = service.lookup("Hooktongue", { category: "creature" }).match;
    expect(hooktongue?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(hooktongue?.derivedTags).not.toContain("coastal_setting");
    const oldHerok = service.lookup("Old Herok", { category: "creature" }).match;
    expect(oldHerok?.derivedTags).toEqual(expect.arrayContaining(["freshwater_setting", "aquatic_setting"]));
    expect(oldHerok?.derivedTags).not.toContain("coastal_setting");
    const bubonicPlague = service.lookup("Bubonic Plague", { category: "affliction" }).match;
    expect(bubonicPlague?.derivedTags).toContain("epidemic_pestilence");
    const reapersShadow = service.lookup("Reaper's Shadow", { category: "affliction" }).match;
    expect(reapersShadow?.derivedTags).toContain("void_soul_corruption");
    const endlessNightmare = service.lookup("Endless Nightmare", { category: "affliction" }).match;
    expect(endlessNightmare?.derivedTags).toContain("nightmare_torment");
    const spaciousPouch = service.lookup("Spacious Pouch (Type I)", { category: "equipment" }).match;
    expect(spaciousPouch?.derivedTags).toEqual(expect.arrayContaining(["carry_support", "extradimensional_storage"]));
    const gunnersBandolier = service.lookup("Gunner's Bandolier", { category: "equipment" }).match;
    expect(gunnersBandolier?.derivedTags).toContain("weapon_staging");
    const repeatingCrossbowMagazine = service.lookup("Repeating Crossbow Magazine", { category: "equipment" }).match;
    expect(repeatingCrossbowMagazine?.derivedTags).toContain("ammo_management");
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
    const alarmSnare = service.lookup("Alarm Snare", { category: "equipment" }).match;
    expect(alarmSnare?.derivedTags).toContain("alarm");
    const sentryFulu = service.lookup("Sentry Fulu", { category: "equipment" }).match;
    expect(sentryFulu?.derivedTags).toContain("alarm");
    const warningSnare = service.lookup("Warning Snare", { category: "equipment" }).match;
    expect(warningSnare?.derivedTags).toContain("alarm");
    const floorbell = service.lookup("Floorbell", { category: "equipment" }).match;
    expect(floorbell?.derivedTags).toContain("alarm");
    const flareBeacon = service.lookup("Flare Beacon (Moderate)", { category: "equipment" }).match;
    expect(flareBeacon?.derivedTags).toContain("signaling");
    const counteringCharm = service.lookup("Countering Charm", { category: "equipment" }).match;
    expect(counteringCharm?.derivedTags).toContain("countermagic");
    expect(counteringCharm?.derivedTags).not.toContain("magic_protection");
    const antimagicOil = service.lookup("Antimagic Oil", { category: "equipment" }).match;
    expect(antimagicOil?.derivedTags).toEqual(expect.arrayContaining(["countermagic", "magic_protection"]));
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
    const mentalAssault = service.lookup("Mental Assault", { category: "hazard" }).match;
    expect(mentalAssault?.derivedTags).toContain("mental_impairment");
    const imagesOfFailure = service.lookup("Images of Failure", { category: "hazard" }).match;
    expect(imagesOfFailure?.derivedTags).toContain("mental_impairment");
    const snaringGlyph = service.lookup("Snaring Glyph", { category: "hazard" }).match;
    expect(snaringGlyph?.derivedTags).toContain("restraint_capture");
    const explosiveBarrels = service.lookup("Explosive Barrels", { category: "hazard" }).match;
    expect(explosiveBarrels?.derivedTags).toContain("fire_hazard");
    const gasTrap = service.lookup("Gas Trap", { category: "hazard" }).match;
    expect(gasTrap?.derivedTags).toContain("poison_hazard");
    const drowningPit = service.lookup("Drowning Pit", { category: "hazard" }).match;
    expect(drowningPit?.derivedTags).toContain("pitfall");
    const collapsingBridge = service.lookup("Collapsing Bridge", { category: "hazard" }).match;
    expect(collapsingBridge?.derivedTags).toContain("collapse_hazard");
    const rushingWind = service.lookup("Rushing Wind", { category: "hazard" }).match;
    expect(rushingWind?.derivedTags).toContain("forced_movement");
    const phantomSoldiers = service.lookup("Phantom Soldiers", { category: "hazard" }).match;
    expect(phantomSoldiers?.derivedTags).toContain("phantom_assailants");
    const dreamPoisonedDoor = service.lookup("Dream-Poisoned Door", { category: "hazard" }).match;
    expect(dreamPoisonedDoor?.derivedTags).toContain("spawned_attackers");
    const hallowedWheel = service.lookup("Hallowed Wheel", { category: "hazard" }).match;
    expect(hallowedWheel?.derivedTags).toContain("control_interface");
    const crushingGateTrap = service.lookup("Crushing Gate Trap", { category: "hazard" }).match;
    expect(crushingGateTrap?.derivedTags).toContain("barrier_lockdown");
    const cacklingDelirium = service.lookup("Cackling Delirium", { category: "affliction" }).match;
    expect(cacklingDelirium?.subcategory).toBe("curse");
    expect(cacklingDelirium?.derivedTags).toContain("mental_impairment");
    const calcifyingRot = service.lookup("Calcifying Rot", { category: "affliction" }).match;
    expect(calcifyingRot?.subcategory).toBe("disease");
    expect(calcifyingRot?.derivedTags).toContain("mobility_impairment");
    const ghostChargePrototype = service.lookup("Ghost Charge Prototype", { category: "equipment" }).match;
    expect(ghostChargePrototype?.baseItem).toBe("alchemical-bomb");
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

  it("supports exact link filters in deterministic listing and ranked search", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);
    const track = service.lookup("Track").match;
    const coverTracks = service.lookup("Cover Tracks").match;

    expect(track).toBeTruthy();
    expect(coverTracks).toBeTruthy();

    const anyLinked = service.listRecords({
      category: "equipment",
      linksTo: [track!.recordKey, coverTracks!.recordKey],
      linksToMode: "any",
    }).records.map((record) => record.name);
    expect(anyLinked).toEqual(expect.arrayContaining([
      "Aroma Concealer",
      "Bloodhound Mask (Greater)",
      "Tracker's Goggles",
      "Tracker's Stew",
    ]));
    expect(anyLinked).not.toContain("Tracking Tag");
    expect(anyLinked).not.toContain("Trackless");

    const allLinked = service.listRecords({
      category: "equipment",
      linksTo: [track!.recordKey, coverTracks!.recordKey],
      linksToMode: "all",
    }).records.map((record) => record.name);
    expect(allLinked).toEqual(["Tracker's Stew"]);

    const excludedLinked = service.listRecords({
      category: "equipment",
      linksTo: [track!.recordKey],
      excludeLinksTo: [coverTracks!.recordKey],
    }).records.map((record) => record.name);
    expect(excludedLinked).toEqual(expect.arrayContaining([
      "Aroma Concealer",
      "Bloodhound Mask (Greater)",
      "Tracker's Goggles",
    ]));
    expect(excludedLinked).not.toContain("Tracker's Stew");

    const searched = await service.search({
      category: "equipment",
      nameQuery: "goggles",
      linksTo: [track!.recordKey],
    });
    expect(searched.records[0]?.name).toBe("Tracker's Goggles");

    expect(() => service.listRecords({ linksToMode: "all" })).toThrow("linksToMode requires linksTo.");
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
    expect(() => service.listRecords({ excludeQuery: "ghost" })).toThrow("excludeQuery is only supported for pf2e_search.");
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

  it("applies excludeQuery as a hard lexical filter over indexed search text", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const baseline = await service.search({
      searchProfile: "lexical",
      category: "hazard",
      query: "crossbow rope door trap",
      limit: 20,
    });
    expect(baseline.records.map((record) => record.name)).toContain("Spear Launcher");

    const filtered = await service.search({
      searchProfile: "lexical",
      category: "hazard",
      query: "crossbow rope door trap",
      excludeQuery: "crossbow",
      limit: 20,
    });
    expect(filtered.records.map((record) => record.name)).not.toContain("Spear Launcher");
  });

  it("applies excludeQuery to hybrid search results and surfaces normalized exclusion analysis", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const baseline = await service.search({
      category: "creature",
      query: "ghost sailor ship",
      limit: 20,
    });
    expect(baseline.records.map((record) => record.name)).toContain("Ghost Sailor");

    const filtered = await service.search({
      category: "creature",
      query: "ghost sailor ship",
      excludeQuery: " GHOST!!! ",
      limit: 20,
      explain: true,
    });
    expect(filtered.mode).toBe("hybrid");
    expect(filtered.records.map((record) => record.name)).not.toContain("Ghost Sailor");
    expect(filtered.explain?.excludeQuery).toEqual({
      rawQuery: "GHOST!!!",
      normalizedQuery: "ghost",
      queryTokens: ["ghost"],
    });
  });

  it("applies excludeQuery to structured pf2e_search flows without query text", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const baseline = await service.search({
      category: "creature",
      nameQuery: "Ghost Sailor",
    });
    expect(baseline.records.map((record) => record.name)).toContain("Ghost Sailor");

    const filtered = await service.search({
      category: "creature",
      nameQuery: "Ghost Sailor",
      excludeQuery: "sailor",
    });
    expect(filtered.records.map((record) => record.name)).not.toContain("Ghost Sailor");
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
      limit: 50,
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
      limit: 20,
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

  it("supports generalized actor-metric and item-metric filters and comparisons", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "ability.int.mod", op: ">=", value: 5 },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetricCompare", leftMetric: "ability.int.mod", op: ">", rightMetric: "ability.cha.mod" },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "save.best", op: "==", value: "will" },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "save.worst", op: "==", value: "ref" },
    }).records.map((record) => record.name)).toEqual(["Stubborn Brute"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "skill.arcana.proficient", op: "==", value: true },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "skill.arcana.proficient", op: "==", value: false },
    }).records.map((record) => record.name)).toEqual(["Silver Tongue Duelist", "Stubborn Brute"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "skill.arcana.rank", op: ">=", value: 4 },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "senses", op: "includesAny", values: ["darkvision"] },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "speed.fly.value", op: ">=", value: 40 },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "actorMetric", metric: "sense.scent.range", op: ">=", value: 30 },
    }).records.map((record) => record.name)).toEqual(["Tactical Mastermind"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetric", metric: "stealth.dc", op: ">=", value: 20 },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox", "Haunting Choir"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetric", metric: "hardness.value", op: ">=", value: 1 },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetricCompare", leftMetric: "ac.value", op: ">", rightMetric: "save.ref.mod" },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox", "Haunting Choir"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetric", metric: "save.best", op: "==", value: "will" },
    }).records.map((record) => record.name)).toEqual(["Haunting Choir"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "isComplex", op: "eq", value: true },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "disableSkills", op: "includesAny", values: ["thievery"] },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetric", metric: "disable.thievery.rank.min", op: ">=", value: 1 },
    }).records.map((record) => record.name)).toEqual(["Clockwork Killbox"]);

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "actorMetric", metric: "disable.religion.rank.min", op: ">=", value: 2 },
    }).records.map((record) => record.name)).toEqual(["Haunting Choir"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "weapon.reload", op: "==", value: 1 },
    }).records.map((record) => record.name)).toEqual(["Repeating Hand Crossbow"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "weapon.range_increment", op: ">=", value: 100 },
    }).records.map((record) => record.name)).toEqual(["Siege Laser"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "armor.ac_bonus", op: ">=", value: 4 },
    }).records.map((record) => record.name)).toEqual(["Fortress Plate"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "shield.hardness", op: ">=", value: 10 },
    }).records.map((record) => record.name)).toEqual(["Tower Bulwark"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetricCompare", leftMetric: "shield.hp", op: ">", rightMetric: "shield.bt" },
    }).records.map((record) => record.name)).toEqual(["Buckler Aegis", "Tower Bulwark"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "shield.ac_bonus", op: ">=", value: 2 },
    }).records.map((record) => record.name)).toEqual(["Tower Bulwark"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "armor.dex_cap", op: "==", value: 1 },
    }).records.map((record) => record.name)).toEqual(["Fortress Plate"]);

    expect(service.listRecords({
      category: "equipment",
      metadata: { field: "itemMetric", metric: "armor.strength", op: ">=", value: 4 },
    }).records.map((record) => record.name)).toEqual(["Fortress Plate"]);

    expect(service.lookup("Clockwork Killbox", { category: "hazard" }).match?.actorMetrics).toMatchObject({
      "ac.value": 20,
      "disable.crafting.dc.min": 18,
      "disable.thievery.rank.min": 1,
      "hardness.value": 8,
      "hp.bt": 16,
      "stealth.dc": 20,
    });

    expect(service.lookup("Clockwork Killbox", { category: "hazard" }).match).toMatchObject({
      disableSkills: ["crafting", "thievery"],
      isComplex: true,
    });

    expect(service.lookup("Tower Bulwark", { category: "equipment" }).match?.itemMetrics).toMatchObject({
      "shield.ac_bonus": 2,
      "shield.hardness": 10,
      "shield.hp": 40,
      "shield.bt": 20,
    });
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

  it("supports expanded derived tags across categories", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["healing_suppression"] },
    }).records.map((record) => record.name)).toContain("Ghast Fever");
    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["sedation"] },
    }).records.map((record) => record.name)).toContain("Knockout Dram");
    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["cognitive_impairment"] },
    }).records.map((record) => record.name)).toContain("Mind-Rotting Toxin");
    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["rot_decay"] },
    }).records.map((record) => record.name)).toContain("Rotting Curse");
    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["infestation_implant"] },
    }).records.map((record) => record.name)).toContain("Wasp Larva");
    expect(service.listRecords({
      category: "affliction",
      metadata: { field: "derivedTags", op: "includesAny", values: ["compulsion"] },
    }).records.map((record) => record.name)).toContain("Liar's Demise");

    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["protective_ward"] },
    }).records.map((record) => record.name)).toContain("Sanctuary Circle");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["death_prevention"] },
    }).records.map((record) => record.name)).toContain("Breath of Life");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["transformation"] },
    }).records.map((record) => record.name)).toContain("Animal Form");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["animal_form"] },
    }).records.map((record) => record.name)).toContain("Animal Form");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["mobility"] },
    }).records.map((record) => record.name)).toEqual(expect.arrayContaining(["Teleport", "Water Walk"]));
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["fear_pressure"] },
    }).records.map((record) => record.name)).toContain("Fear");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["battlefield_disruption"] },
    }).records.map((record) => record.name)).toContain("Phantom Prison");
    expect(service.listRecords({
      category: "spell",
      metadata: { field: "derivedTags", op: "includesAny", values: ["condition_support"] },
    }).records.map((record) => record.name)).toContain("Clear Mind");

    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["acid_hazard"] },
    }).records.map((record) => record.name)).toContain("Acid Mist");
    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["sound_hazard"] },
    }).records.map((record) => record.name)).toContain("Buzzing Latch Rune");
    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["ward_trigger"] },
    }).records.map((record) => record.name)).toContain("Mask Summoning Rune");
    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["respiratory_hazard"] },
    }).records.map((record) => record.name)).toContain("Smoke-Filled Hallway");
    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["water_hazard"] },
    }).records.map((record) => record.name)).toContain("Sudden Geysers");
    expect(service.listRecords({
      category: "hazard",
      metadata: { field: "derivedTags", op: "includesAny", values: ["illusion_assault"] },
    }).records.map((record) => record.name)).toContain("Distortion Mirror");

    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["illumination"] },
    }).records.map((record) => record.name)).toContain("Beacon Shot");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["elemental_payload"] },
    }).records.map((record) => record.name)).toContain("Elemental Ammunition");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["spell_payload"] },
    }).records.map((record) => record.name)).toContain("Disintegration Bolt");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["restraint_capture"] },
    }).records.map((record) => record.name)).toContain("Bola Shot");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["mobility_impairment"] },
    }).records.map((record) => record.name)).toContain("Glue Bullet");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["sensory_impairment"] },
    }).records.map((record) => record.name)).toContain("Blindpepper Bolt");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "ammo",
      metadata: { field: "derivedTags", op: "includesAny", values: ["mental_impairment"] },
    }).records.map((record) => record.name)).toContain("Mindlock Shot");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "consumable",
      metadata: { field: "derivedTags", op: "includesAny", values: ["sedation"] },
    }).records.map((record) => record.name)).toContain("Slumber Wine");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "weapon",
      metadata: { field: "derivedTags", op: "includesAny", values: ["concealable"] },
    }).records.map((record) => record.name)).toContain("Cane Pistol");
    expect(service.listRecords({
      category: "equipment",
      subcategory: "armor",
      metadata: { field: "derivedTags", op: "includesAny", values: ["stealth_support"] },
    }).records.map((record) => record.name)).toContain("Shadow Shroud");

    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["mask_motif"] },
    }).records.map((record) => record.name)).toContain("Masked Mourner");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["faceless_horror"] },
    }).records.map((record) => record.name)).toContain("Faceless Butcher");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["disguised_pretender"] },
    }).records.map((record) => record.name)).toContain("False Herald");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["animated_object"] },
    }).records.map((record) => record.name)).toContain("Animated Armor");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["living_artwork"] },
    }).records.map((record) => record.name)).toContain("Living Mural");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["profession_npc"] },
    }).records.map((record) => record.name)).toContain("Tree Singer");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["mask_motif"] },
    }).records.map((record) => record.name)).toContain("Taljjae");
    expect(service.listRecords({
      category: "creature",
      metadata: { field: "derivedTags", op: "includesAny", values: ["faceless_horror"] },
    }).records.map((record) => record.name)).toContain("The Vanish Man");

    const ghastFever = service.lookup("Ghast Fever", { category: "affliction" }).match;
    expect(ghastFever?.derivedTags).toContain("healing_suppression");
    const rottingCurse = service.lookup("Rotting Curse", { category: "affliction" }).match;
    expect(rottingCurse?.derivedTags).toContain("rot_decay");
    const waspLarva = service.lookup("Wasp Larva", { category: "affliction" }).match;
    expect(waspLarva?.derivedTags).toContain("infestation_implant");
    const liarsDemise = service.lookup("Liar's Demise", { category: "affliction" }).match;
    expect(liarsDemise?.derivedTags).toContain("compulsion");

    const breathOfLife = service.lookup("Breath of Life", { category: "spell" }).match;
    expect(breathOfLife?.derivedTags).toContain("death_prevention");
    const animalForm = service.lookup("Animal Form", { category: "spell" }).match;
    expect(animalForm?.derivedTags).toEqual(expect.arrayContaining(["transformation", "battle_form", "animal_form"]));
    const teleport = service.lookup("Teleport", { category: "spell" }).match;
    expect(teleport?.derivedTags).toContain("mobility");
    const fear = service.lookup("Fear", { category: "spell" }).match;
    expect(fear?.derivedTags).toContain("fear_pressure");
    const phantomPrison = service.lookup("Phantom Prison", { category: "spell" }).match;
    expect(phantomPrison?.derivedTags).toContain("battlefield_disruption");
    const clearMind = service.lookup("Clear Mind", { category: "spell" }).match;
    expect(clearMind?.derivedTags).toContain("condition_support");

    const acidMist = service.lookup("Acid Mist", { category: "hazard" }).match;
    expect(acidMist?.derivedTags).toContain("acid_hazard");
    const maskSummoningRune = service.lookup("Mask Summoning Rune", { category: "hazard" }).match;
    expect(maskSummoningRune?.derivedTags).toContain("ward_trigger");
    const smokeFilledHallway = service.lookup("Smoke-Filled Hallway", { category: "hazard" }).match;
    expect(smokeFilledHallway?.derivedTags).toContain("respiratory_hazard");
    const suddenGeysers = service.lookup("Sudden Geysers", { category: "hazard" }).match;
    expect(suddenGeysers?.derivedTags).toContain("water_hazard");
    const ashWeb = service.lookup("Ash Web", { category: "hazard" }).match;
    expect(ashWeb?.derivedTags).toContain("restraint_capture");

    const beaconShot = service.lookup("Beacon Shot", { category: "equipment" }).match;
    expect(beaconShot?.subcategory).toBe("ammo");
    expect(beaconShot?.derivedTags).toEqual(expect.arrayContaining(["illumination", "signaling"]));
    const disintegrationBolt = service.lookup("Disintegration Bolt", { category: "equipment" }).match;
    expect(disintegrationBolt?.derivedTags).toContain("spell_payload");
    const canePistol = service.lookup("Cane Pistol", { category: "equipment" }).match;
    expect(canePistol?.derivedTags).toContain("concealable");
    const glueBullet = service.lookup("Glue Bullet", { category: "equipment" }).match;
    expect(glueBullet?.derivedTags).toContain("mobility_impairment");
    const blindpepperBolt = service.lookup("Blindpepper Bolt", { category: "equipment" }).match;
    expect(blindpepperBolt?.derivedTags).toContain("sensory_impairment");
    const mindlockShot = service.lookup("Mindlock Shot", { category: "equipment" }).match;
    expect(mindlockShot?.derivedTags).toContain("mental_impairment");
    const slumberWine = service.lookup("Slumber Wine", { category: "equipment" }).match;
    expect(slumberWine?.derivedTags).toEqual(expect.arrayContaining(["offensive", "ingested_offense", "sedation"]));
    const shadowShroud = service.lookup("Shadow Shroud", { category: "equipment" }).match;
    expect(shadowShroud?.subcategory).toBe("armor");
    expect(shadowShroud?.derivedTags).toContain("stealth_support");

    const falseHerald = service.lookup("False Herald", { category: "creature" }).match;
    expect(falseHerald?.derivedTags).toContain("disguised_pretender");
    const treeSinger = service.lookup("Tree Singer", { category: "creature" }).match;
    expect(treeSinger?.derivedTags).toContain("profession_npc");
    const livingMural = service.lookup("Living Mural", { category: "creature" }).match;
    expect(livingMural?.derivedTags).toContain("living_artwork");
    const taljjae = service.lookup("Taljjae", { category: "creature" }).match;
    expect(taljjae?.derivedTags).toContain("mask_motif");
    const vanishMan = service.lookup("The Vanish Man", { category: "creature" }).match;
    expect(vanishMan?.derivedTags).toContain("faceless_horror");
    const animatedArmor = service.lookup("Animated Armor", { category: "creature" }).match;
    expect(animatedArmor?.derivedTags).toContain("animated_object");
    expect(animatedArmor?.derivedTags).not.toContain("bound_object");
  });
});
