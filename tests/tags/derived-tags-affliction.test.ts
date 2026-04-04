import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: affliction", () => {
  it("derives affliction impact tags", () => {
    expect(deriveRecordTags({
      name: "Cackling Delirium",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "Mocking whispers leave the victim confused, frightened, and unable to trust their own senses.",
      traits: ["curse", "mental"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Calcifying Rot",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "The disease stiffens joints, reduces the victim's Speed, and can leave them immobilized.",
      traits: ["disease"],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Giant Wasp Venom",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "Giant wasp venom interferes with a victim's movement. Stage 1 damage and Clumsy 1.",
      traits: ["poison"],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Dancing Lamentation",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This toxin erratically stimulates the limbs. At the start of each turn, the victim takes one or more Steps in a random direction if able. This movement is forced.",
      traits: ["poison"],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Arsenic",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This toxin is a compound of arsenic and other substances. You can't reduce your sickened condition while affected.",
      traits: ["poison"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Bubonic Plague",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "Stage 1 Fatigued. Stage 2 Drained 1 and Fatigued. Stage 3 Drained 2 and Enfeebled 1.",
      traits: ["disease"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Blackfrost",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "This affliction can't be reduced below stage 1, nor the damage from it healed, until successfully treated with a similar effect.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTags({
      name: "Ghast Fever",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "Stage 2 regains half as many Hit Points from all healing. Stage 4 gains no benefit from healing.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTags({
      name: "Ravenous Wasting",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "The victim cannot be healed while the wasting persists.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTags({
      name: "Ashen Rot",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "Stage 3 regain half as many Hit Points from all healing effects.",
      traits: ["disease"],
    })).toContain("healing_suppression");

    expect(deriveRecordTags({
      name: "Mindmurk Oil",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This oil dulls the mind, creating a euphoric stupor that inhibits the imbiber's decision-making skills and mental capacity.",
      traits: ["poison"],
    })).toContain("cognitive_impairment");

    expect(deriveRecordTags({
      name: "Cytillesh Oil",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This thick substance is distilled from the mind-robbing cytillesh fungus.",
      traits: ["poison"],
    })).toContain("cognitive_impairment");

    expect(deriveRecordTags({
      name: "Isolation Draught",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This clear tonic slowly shuts down the imbiber's senses. Stage 2 deafened. Stage 3 blinded and deafened.",
      traits: ["poison"],
    })).toContain("sensory_impairment");

    expect(deriveRecordTags({
      name: "Knockout Dram",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This soporific chemical quickly induces a deep unconsciousness and makes the victim sleep normally.",
      traits: ["poison"],
    })).toContain("sedation");

    expect(deriveRecordTags({
      name: "Dreamtime Tea",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This lemony tea blended from rare herbs puts the drinker into a trancelike state and stage 2 unconsciousness.",
      traits: ["poison"],
    })).toContain("sedation");

    expect(deriveRecordTags({
      name: "Black Apoxia",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "The victim can't breathe properly as the disease drains air from the lungs and leaves them breathless.",
      traits: ["disease"],
    })).toContain("respiratory_impairment");

    expect(deriveRecordTags({
      name: "Crystal Corruption",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "Creatures afflicted by this curse slowly turn to solid crystal until they are petrified.",
      traits: ["curse"],
    })).toContain("transformative_corruption");

    expect(deriveRecordTags({
      name: "Crystalline Corruption",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The curse gradually turns the victim into crystalline matter until their body is no longer wholly flesh.",
      traits: ["curse"],
    })).toContain("transformative_corruption");

    expect(deriveRecordTags({
      name: "Bloom Curse",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The curse makes the victim's skin sprout petals and roots as their body slowly becomes plant matter.",
      traits: ["curse"],
    })).toContain("transformative_corruption");

    expect(deriveRecordTags({
      name: "Bloodfire Fever",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "Stage 1 blood loss. Stage 2 fatigued and weakened as the fever drains vitality from the body.",
      traits: ["disease"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Boiling Blood",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "Stage 1 the victim's blood boils and they become exhausted. Stage 2 hemorrhaging and drained.",
      traits: ["poison"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Jelly Blood",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The curse weakens the body until the victim is wasting away with blood loss.",
      traits: ["curse"],
    })).toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Cackling Delirium",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "Mocking whispers leave the victim confused, frightened, and unable to trust their own senses.",
      traits: ["curse", "mental"],
    })).not.toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Cackling Delirium",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "Mocking whispers leave the victim confused, frightened, and unable to trust their own senses.",
      traits: ["curse", "mental"],
    })).not.toEqual(expect.arrayContaining([
      "healing_suppression",
      "cognitive_impairment",
      "sensory_impairment",
      "sedation",
      "physical_debilitation",
      "transformative_corruption",
    ]));

    expect(deriveRecordTags({
      name: "Dreamtime Tea",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This lemony tea blended from rare herbs puts the drinker into a trancelike state and stage 2 unconsciousness.",
      traits: ["poison"],
    })).not.toContain("physical_debilitation");

    expect(deriveRecordTags({
      name: "Rotting Curse",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The victim's flesh rots and decays away, leaving necrotic wounds that worsen each day.",
      traits: ["curse"],
    })).toContain("rot_decay");

    expect(deriveRecordTags({
      name: "Wasp Larva",
      category: "affliction",
      subcategory: "disease",
      descriptionText: "A parasitic larva burrows inside the host's body and hatches after several agonizing stages.",
      traits: ["disease"],
    })).toContain("infestation_implant");

    expect(deriveRecordTags({
      name: "Liar's Demise",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "The poison forces the victim to speak only the truth and can use no actions but to answer questions put to them.",
      traits: ["poison"],
    })).toContain("compulsion");

    expect(deriveRecordTags({
      name: "Twisted Loyalties",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The target's loyalties twist until it treats allies as enemies and is driven to betray its companions.",
      traits: ["curse", "mental"],
    })).toContain("compulsion");

    expect(deriveRecordTags({
      name: "Mind-Rotting Toxin",
      category: "affliction",
      subcategory: "poison",
      descriptionText: "This toxin clouds the mind and leaves the victim stupefied with fractured thoughts.",
      traits: ["poison"],
    })).not.toContain("rot_decay");

    expect(deriveRecordTags({
      name: "Deep Breath Hex",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "The curse leaves the victim fatigued and short of breath after any strenuous activity.",
      traits: ["curse"],
    })).not.toEqual(expect.arrayContaining(["respiratory_impairment", "sedation"]));
  });
});
