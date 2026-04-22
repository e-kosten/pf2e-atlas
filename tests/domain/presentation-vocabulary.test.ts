import { describe, expect, it } from "vitest";

import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
  formatMetadataFieldTypeLabel,
  formatOntologySearchVocabularyLabel,
  humanizeOntologySearchIdentifier,
} from "../../src/domain/presentation-vocabulary.js";

describe("presentation vocabulary", () => {
  it("renders canonical shared metadata field labels", () => {
    expect(formatMetadataFieldLabel("derivedTags")).toBe("Derived Tags");
    expect(formatMetadataFieldLabel("actionCost")).toBe("Action Cost");
    expect(formatMetadataFieldLabel("sourceCategory")).toBe("Source Category");
  });

  it("renders canonical shared field-type labels", () => {
    expect(formatMetadataFieldTypeLabel("enumString")).toBe("Enumerated String");
    expect(formatMetadataFieldTypeLabel("set")).toBe("String Set");
    expect(describeMetadataFieldType("text")).toBe("free text");
  });

  it("falls back to shared humanized labels for other ontology/search vocabulary", () => {
    expect(formatOntologySearchVocabularyLabel("publicationRemaster")).toBe("Publication Remaster");
    expect(formatOntologySearchVocabularyLabel("customMetricGroup")).toBe("Custom Metric Group");
  });

  it("humanizes punctuation-delimited ontology/search identifiers through one shared fallback", () => {
    expect(humanizeOntologySearchIdentifier("party_role")).toBe("Party Role");
    expect(humanizeOntologySearchIdentifier("characterCreation")).toBe("Character Creation");
    expect(humanizeOntologySearchIdentifier("creature:family:giant-bat")).toBe("Creature Family Giant Bat");
    expect(humanizeOntologySearchIdentifier("ability.score.value")).toBe("Ability Score Value");
  });
});
