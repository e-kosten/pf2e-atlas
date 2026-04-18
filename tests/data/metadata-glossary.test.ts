import { describe, expect, it } from "vitest";

import { extractTraitGlossaryEntries, getMetadataGlossaryArtifactPath } from "../../src/data/metadata-glossary.js";

describe("metadata glossary", () => {
  it("extracts trait labels and descriptions from PF2E config and language data", () => {
    const traitsConfigSource = `
      const exampleTraits = {
        fire: "PF2E.TraitFire",
        "cold-iron": "PF2E.TraitColdIron",
      };

      const traitDescriptions = {
        fire: "PF2E.TraitDescriptionFire",
      };
    `;

    const entries = extractTraitGlossaryEntries(traitsConfigSource, {
      PF2E: {
        TraitFire: "Fire",
        TraitColdIron: "Cold Iron",
        TraitDescriptionFire: "Effects with the fire trait deal fire damage.",
      },
    });

    expect(entries.fire).toEqual({
      value: "fire",
      label: "Fire",
      description: "Effects with the fire trait deal fire damage.",
    });
    expect(entries["cold-iron"]).toEqual({
      value: "cold-iron",
      label: "Cold Iron",
      description: null,
    });
  });

  it("derives the metadata glossary artifact path from the index path", () => {
    expect(getMetadataGlossaryArtifactPath("/tmp/pf2e-index.sqlite")).toBe("/tmp/pf2e-index.metadata-glossary.json");
    expect(getMetadataGlossaryArtifactPath("/tmp/pf2e-index")).toBe("/tmp/pf2e-index.metadata-glossary.json");
  });
});
