import { describe, expect, it } from "vitest";

import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import { getLoadedOntologyNodeChildren } from "../../src/app/ontology/node-helpers.js";
import { buildSearchFilterExplorerTargetResolver } from "../../src/tui/filter-explorer/search-draft-model.js";
import {
  levelSupportsSearchFilterExplorerFrequencySort,
  sortSearchFilterExplorerModel,
} from "../../src/tui/search-screen/filter-explorer-value-sorting.js";
import { reconcileSearchFilterExplorerModel } from "../../src/tui/search-screen/filter-explorer-model-reconciliation.js";
import type { Pf2eTerminalQueryFieldOption } from "../../src/tui/search/service.js";

function modelWithRootValues(nodes: readonly OntologyNode[]): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Filter Explorer",
    description: "Test filter explorer",
    rootNodes: nodes,
  };
}

function valueNode(field: string, value: string, count: number, label = value): OntologyNode {
  return {
    id: `spell:field:${field}:value:${value}`,
    kind: "value",
    label,
    filterText: label,
    listLabel: `${label} | ${count}`,
    detailLines: [{ text: label }],
  };
}

function traitNode(value: string, count: number): OntologyNode {
  return {
    id: `spell:traits:${value}`,
    kind: "trait",
    label: value,
    filterText: value,
    listLabel: `${value} | ${count}`,
    detailLines: [{ text: value }],
  };
}

describe("search filter explorer value sorting", () => {
  it("defaults count-bearing trait values to alphabetical semantic order", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "traits",
        label: "Traits",
        description: "Trait values.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ];
    const resolver = buildSearchFilterExplorerTargetResolver(fieldOptions);

    const sorted = sortSearchFilterExplorerModel(modelWithRootValues([
      traitNode("water", 20),
      traitNode("air", 1),
      traitNode("earth", 7),
    ]), {
      sortMode: "semantic",
      fieldOptions,
      resolveSelectionTarget: resolver,
    });

    expect(sorted.rootNodes.map((node) => node.label)).toEqual(["air", "earth", "water"]);
    expect(levelSupportsSearchFilterExplorerFrequencySort(sorted.rootNodes, {
      fieldOptions,
      resolveSelectionTarget: resolver,
    })).toBe(true);
  });

  it("can sort eligible value lists by frequency on request", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "traits",
        label: "Traits",
        description: "Trait values.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ];

    const sorted = sortSearchFilterExplorerModel(modelWithRootValues([
      traitNode("air", 1),
      traitNode("earth", 7),
      traitNode("water", 20),
    ]), {
      sortMode: "frequency",
      fieldOptions,
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
    });

    expect(sorted.rootNodes.map((node) => node.label)).toEqual(["water", "earth", "air"]);
  });

  it("uses declared semantic ordering for canonical fields and does not offer frequency sorting", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "rarity",
        label: "Rarity",
        description: "Rarity values.",
        fieldType: "enumString",
        editor: "sharedExplorer",
        valueOrdering: {
          kind: "canonical",
          order: ["common", "uncommon", "rare", "unique"],
        },
      },
    ];
    const resolver = buildSearchFilterExplorerTargetResolver(fieldOptions);

    const sorted = sortSearchFilterExplorerModel(modelWithRootValues([
      valueNode("rarity", "unique", 1),
      valueNode("rarity", "common", 10),
      valueNode("rarity", "rare", 5),
    ]), {
      sortMode: "frequency",
      fieldOptions,
      resolveSelectionTarget: resolver,
    });

    expect(sorted.rootNodes.map((node) => node.label)).toEqual(["common", "rare", "unique"]);
    expect(levelSupportsSearchFilterExplorerFrequencySort(sorted.rootNodes, {
      fieldOptions,
      resolveSelectionTarget: resolver,
    })).toBe(false);
  });

  it("sorts boolean values true first", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "sustained",
        label: "Sustained",
        description: "Sustained values.",
        fieldType: "boolean",
        editor: "sharedExplorer",
      },
    ];

    const sorted = sortSearchFilterExplorerModel(modelWithRootValues([
      valueNode("sustained", "false", 10),
      valueNode("sustained", "true", 1),
    ]), {
      sortMode: "semantic",
      fieldOptions,
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
    });

    expect(sorted.rootNodes.map((node) => node.label)).toEqual(["true", "false"]);
  });

  it("sorts only value levels inside field nodes", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "traits",
        label: "Traits",
        description: "Trait values.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ];
    const fieldNode: OntologyNode = {
      id: "spell:field:traits",
      kind: "field",
      label: "Traits",
      filterText: "traits",
      detailLines: [{ text: "Traits" }],
      childSource: {
        kind: "static",
        children: [traitNode("water", 10), traitNode("air", 1)],
      },
    };

    const sorted = sortSearchFilterExplorerModel(modelWithRootValues([fieldNode]), {
      sortMode: "semantic",
      fieldOptions,
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
    });

    expect(sorted.rootNodes.map((node) => node.label)).toEqual(["Traits"]);
    expect(getLoadedOntologyNodeChildren(sorted.rootNodes[0]).map((node) => node.label)).toEqual(["air", "water"]);
  });

  it("keeps reconciled selected zero-count values in semantic order", () => {
    const fieldOptions: Pf2eTerminalQueryFieldOption[] = [
      {
        value: "traits",
        label: "Traits",
        description: "Trait values.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ];
    const resolver = buildSearchFilterExplorerTargetResolver(fieldOptions);

    const model = reconcileSearchFilterExplorerModel({
      currentModel: modelWithRootValues([traitNode("water", 10), traitNode("air", 2)]),
      refreshedModel: modelWithRootValues([traitNode("water", 9)]),
      fieldState: {
        discreteSelections: {
          traits: {
            include: ["air"],
            exclude: [],
          },
        },
        scalarClauses: {},
      },
      fieldOptions,
      resolveSelectionTarget: resolver,
      sortMode: "semantic",
    });

    expect(model.rootNodes.map((node) => node.label)).toEqual(["air", "water"]);
    expect(model.rootNodes[0]?.listLabel).toBe("air | 0");
  });
});
