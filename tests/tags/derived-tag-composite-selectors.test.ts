import { describe, expect, it } from "vitest";

import type { DerivedTagAuthoredCategoryOntology, DerivedTagOntologyTag } from "../../src/domain/index.js";
import { flattenDerivedTagAuthoredCategoryOntology, fromFamily, fromTag } from "../../src/tags/ontology/utils.js";

function findTag(tags: DerivedTagOntologyTag[], tag: string): DerivedTagOntologyTag | undefined {
  return tags.find((entry) => entry.tag === tag);
}

describe("derived tag authored composite selectors", () => {
  it("expands family selectors to leaf tags and supports additional tag selectors", () => {
    const ontology = {
      category: "spell",
      families: {
        reconnaissance: {
          axis: "utility",
          description: "Recon test family.",
          tags: [
            {
              tag: "scouting",
              description: "Scout.",
              assignmentMode: "hybrid",
            },
            {
              tag: "tracking",
              description: "Track.",
              assignmentMode: "hybrid",
            },
            {
              tag: "reconnaissance",
              description: "Recon umbrella.",
              assignmentMode: "composite",
              compositeOfAnyTags: ["scouting", "tracking"],
            },
          ],
        },
        control: {
          axis: "battlefield",
          description: "Control test family.",
          tags: [
            {
              tag: "countermagic",
              description: "Counter magic.",
              assignmentMode: "hybrid",
            },
          ],
        },
        utility: {
          axis: "utility",
          description: "Utility test family.",
          tags: [
            {
              tag: "security",
              description: "Security umbrella.",
              assignmentMode: "composite",
              compositeOfAny: [fromFamily("reconnaissance"), fromTag("countermagic")],
            },
          ],
        },
      },
    } satisfies DerivedTagAuthoredCategoryOntology<"spell">;

    const flattened = flattenDerivedTagAuthoredCategoryOntology(ontology);
    expect(findTag(flattened.tags, "security")?.compositeOfAnyTags).toEqual(["scouting", "tracking", "countermagic"]);
  });

  it("supports exclusions and optional inclusion of composite family tags", () => {
    const ontology = {
      category: "spell",
      families: {
        revelation: {
          axis: "utility",
          description: "Reveal test family.",
          tags: [
            {
              tag: "magic_detection",
              description: "Magic reveal.",
              assignmentMode: "hybrid",
            },
            {
              tag: "truth_reveal",
              description: "Truth reveal.",
              assignmentMode: "hybrid",
            },
            {
              tag: "revelation",
              description: "Reveal umbrella.",
              assignmentMode: "composite",
              compositeOfAnyTags: ["magic_detection", "truth_reveal"],
            },
            {
              tag: "filtered_reveal",
              description: "Filtered reveal umbrella.",
              assignmentMode: "composite",
              compositeOfAny: [fromFamily("revelation", { excludeTags: ["truth_reveal"] })],
            },
            {
              tag: "all_reveal",
              description: "All reveal umbrella.",
              assignmentMode: "composite",
              compositeOfAny: [fromFamily("revelation", { include: "all_tags" })],
            },
          ],
        },
      },
    } satisfies DerivedTagAuthoredCategoryOntology<"spell">;

    const flattened = flattenDerivedTagAuthoredCategoryOntology(ontology);
    expect(findTag(flattened.tags, "filtered_reveal")?.compositeOfAnyTags).toEqual(["magic_detection"]);
    expect(findTag(flattened.tags, "all_reveal")?.compositeOfAnyTags).toEqual([
      "magic_detection",
      "truth_reveal",
      "revelation",
      "filtered_reveal",
    ]);
  });

  it("rejects unknown family exclusions", () => {
    const ontology = {
      category: "spell",
      families: {
        revelation: {
          axis: "utility",
          description: "Reveal test family.",
          tags: [
            {
              tag: "magic_detection",
              description: "Magic reveal.",
              assignmentMode: "hybrid",
            },
            {
              tag: "filtered_reveal",
              description: "Filtered reveal umbrella.",
              assignmentMode: "composite",
              compositeOfAny: [fromFamily("revelation", { excludeTags: ["missing_tag"] })],
            },
          ],
        },
      },
    } satisfies DerivedTagAuthoredCategoryOntology<"spell">;

    expect(() => flattenDerivedTagAuthoredCategoryOntology(ontology)).toThrow(
      /excludes unknown family tag "missing_tag"/i,
    );
  });
});
