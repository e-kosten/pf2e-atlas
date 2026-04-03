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
      name: "Cackling Delirium",
      category: "affliction",
      subcategory: "curse",
      descriptionText: "Mocking whispers leave the victim confused, frightened, and unable to trust their own senses.",
      traits: ["curse", "mental"],
    })).not.toContain("physical_debilitation");
  });
});
