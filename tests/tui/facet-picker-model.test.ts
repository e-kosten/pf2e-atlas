import { describe, expect, it } from "vitest";

import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import { getOntologyNodeChildren } from "../../src/app/ontology/node-helpers.js";
import { buildSearchFacetPickerModel } from "../../src/tui/ontology-explorer/facet-picker-model.js";

const searchSemanticsDomain: OntologyDomainModel = {
  id: "searchSemantics",
  label: "Search Semantics",
  description: "Facet picker regression domain",
  rootNodes: [
    {
      id: "searchSemantics:equipment",
      kind: "category",
      label: "Equipment",
      shortLabel: "equipment",
      filterText: "equipment",
      detailTitle: "Search Semantics",
      detailLines: [{ text: "Equipment", tone: "section" }],
      children: [
        {
          id: "equipment:metadataFields",
          kind: "group",
          label: "Metadata Fields",
          filterText: "metadata fields",
          detailTitle: "Metadata Fields",
          detailLines: [{ text: "Root metadata", tone: "section" }],
          children: [
            {
              id: "equipment:field:weaponGroup",
              kind: "field",
              label: "weaponGroup",
              filterText: "weapon group root",
              detailTitle: "Metadata Field Details",
              detailLines: [{ text: "Subcategory: (all)" }],
              children: [
                {
                  id: "equipment:weaponGroup:bomb",
                  kind: "value",
                  label: "bomb",
                  filterText: "bomb",
                  detailLines: [{ text: "bomb", tone: "section" }],
                },
              ],
            },
          ],
        },
        {
          id: "equipment:subcategories",
          kind: "group",
          label: "Subcategories",
          filterText: "subcategories",
          detailTitle: "Subcategories",
          detailLines: [{ text: "Subcategories", tone: "section" }],
          children: [
            {
              id: "equipment:subcategory:weapon",
              kind: "subcategory",
              label: "weapon",
              filterText: "weapon",
              detailTitle: "Subcategory Boundary",
              detailLines: [{ text: "weapon", tone: "section" }],
              children: [
                {
                  id: "equipment:weapon:metadataFields",
                  kind: "group",
                  label: "Metadata Fields",
                  filterText: "weapon metadata fields",
                  detailTitle: "Metadata Fields",
                  detailLines: [{ text: "Weapon metadata", tone: "section" }],
                  children: [
                    {
                      id: "equipment:weapon:field:weaponGroup",
                      kind: "field",
                      label: "weaponGroup",
                      filterText: "weapon group scoped",
                      detailTitle: "Metadata Field Details",
                      detailLines: [{ text: "Subcategory: weapon" }],
                      children: [
                        {
                          id: "equipment:weaponGroup:sword",
                          kind: "value",
                          label: "sword",
                          filterText: "sword",
                          detailLines: [{ text: "sword", tone: "section" }],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
  ],
};

describe("buildSearchFacetPickerModel", () => {
  it("prefers the active subcategory metadata branch over the category-root field node", () => {
    const model = buildSearchFacetPickerModel(searchSemanticsDomain, {
      category: "equipment",
      subcategory: "weapon",
      fieldOptions: [
        {
          value: "weaponGroup",
          label: "Weapon Group",
          description: "Equipment weapon groups.",
          fieldType: "enumString",
        },
      ],
    });

    expect(model.rootNodes).toHaveLength(1);
    expect(model.rootNodes[0]?.id).toBe("searchSemantics:equipment");
    expect(model.rootNodes[0]?.detailLines.map((line) => line.text)).toContain("Query scope: equipment / weapon");

    const subcategoryFieldNode = getOntologyNodeChildren(model.rootNodes[0])
      .flatMap((node) => getOntologyNodeChildren(node))
      .flatMap((node) => getOntologyNodeChildren(node))
      .flatMap((node) => getOntologyNodeChildren(node))
      .find((node) => node.id === "equipment:weapon:field:weaponGroup");

    expect(subcategoryFieldNode?.detailLines.map((line) => line.text)).toContain("Subcategory: weapon");
    expect(getOntologyNodeChildren(subcategoryFieldNode)[0]?.selection).toEqual({
      field: "weaponGroup",
      fieldLabel: "Weapon Group",
      value: "sword",
      allowedStates: ["any", "exclude"],
    });
  });
});
