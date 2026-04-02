import { describe, expect, it } from "vitest";

import { DERIVED_TAG_CATALOG, deriveRecordTags } from "../src/derived-tags.js";

describe("derived tag rules", () => {
  it("derives expanded consumable support and offense tags", () => {
    expect(deriveRecordTags({
      name: "Darkvision Elixir",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The drinker gains darkvision and a bonus to Perception for 1 hour.",
      traits: ["alchemical", "consumable", "elixir"],
    })).toEqual(expect.arrayContaining(["beneficial", "senses_support", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Dreamer's Tonic",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This restorative tonic helps recover from mental conditions and steady the emotions.",
      traits: ["alchemical", "consumable"],
    })).toEqual(expect.arrayContaining(["beneficial", "condition_support", "mental_recovery"]));

    expect(deriveRecordTags({
      name: "Fire Ward Elixir",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "The drinker gains resistance to fire for 1 hour.",
      traits: ["alchemical", "consumable", "elixir"],
    })).toEqual(expect.arrayContaining(["beneficial", "energy_resistance", "buff_support", "self_buff"]));

    expect(deriveRecordTags({
      name: "Spider Venom",
      category: "equipment",
      subcategory: "consumable",
      descriptionText: "This contact poison is smeared on a weapon and afflicts the target through skin contact.",
      traits: ["alchemical", "consumable", "poison"],
    })).toEqual(expect.arrayContaining(["offensive", "weapon_applied", "contact_offense"]));
  });

  it("derives expanded gear-purpose tags", () => {
    expect(deriveRecordTags({
      name: "Disguise Kit",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "A costume and cosmetics kit used to create a false identity and pass as local nobility.",
      traits: [],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Trail Pack",
      category: "equipment",
      subcategory: "backpack",
      descriptionText: "A roomy backpack for carrying supplies and weathering long wilderness travel.",
      traits: [],
    })).toEqual(expect.arrayContaining(["carry_support", "survival"]));

    expect(deriveRecordTags({
      name: "Survey Lantern",
      category: "equipment",
      subcategory: "gear",
      descriptionText: "A lantern with a compass hood used to illuminate ruins and track your heading underground.",
      traits: [],
    })).toEqual(expect.arrayContaining(["illumination", "navigation"]));
  });

  it("derives expanded creature context tags without adding redundant composites", () => {
    expect(deriveRecordTags({
      name: "Graveyard Watcher",
      category: "creature",
      subcategory: null,
      descriptionText: "This cemetery guard patrols the crypts beneath the old city.",
      traits: [],
    })).toEqual(expect.arrayContaining(["graveyard", "underground", "urban", "profession_npc", "scene_adjacent"]));

    expect(deriveRecordTags({
      name: "Bog Wisp",
      category: "creature",
      subcategory: null,
      descriptionText: "A fey spirit that haunts marshy bogs and flooded mires.",
      traits: ["fey"],
    })).toEqual(expect.arrayContaining(["fey_threat", "swamp"]));

    expect(deriveRecordTags({
      name: "Icebound Mariner",
      category: "creature",
      subcategory: null,
      descriptionText: "A sailor raider from the frozen sea who prowls icy coasts and shipwrecks.",
      traits: [],
    })).toEqual(expect.arrayContaining(["nautical", "aquatic_context", "arctic"]));
  });

  it("publishes a compact derived-tag catalog", () => {
    expect(DERIVED_TAG_CATALOG).toEqual(expect.arrayContaining([
      expect.objectContaining({
        category: "equipment",
        subcategories: ["consumable"],
        family: "function",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "mental_recovery", description: expect.any(String) }),
          expect.objectContaining({ value: "senses_support", description: expect.any(String) }),
          expect.objectContaining({ value: "energy_resistance", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "equipment",
        family: "purpose",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "disguise", description: expect.any(String) }),
          expect.objectContaining({ value: "navigation", description: expect.any(String) }),
          expect.objectContaining({ value: "carry_support", description: expect.any(String) }),
        ]),
      }),
      expect.objectContaining({
        category: "creature",
        family: "context",
        tags: expect.arrayContaining([
          expect.objectContaining({ value: "swamp", description: expect.any(String) }),
          expect.objectContaining({ value: "underground", description: expect.any(String) }),
          expect.objectContaining({ value: "graveyard", description: expect.any(String) }),
        ]),
      }),
    ]));
  });
});
