import { describe, expect, it, vi } from "vitest";

import { resolveOntologyNodeChildren } from "../../src/app/ontology/node-helpers.js";
import type { OntologyNode } from "../../src/domain/ontology-types.js";

describe("ontology node helpers", () => {
  it("resolves only the requested lazy node and caches its immediate children", async () => {
    const nestedLoad = vi.fn(async () => []);
    const siblingLoad = vi.fn(async () => []);
    const loadChildren = vi.fn(async (): Promise<readonly OntologyNode[]> => [
      {
        id: "child",
        kind: "value",
        label: "Child",
        filterText: "child",
        detailLines: [{ text: "Child" }],
        childSource: { kind: "lazy", load: nestedLoad },
      },
    ]);
    const node: OntologyNode = {
      id: "parent",
      kind: "field",
      label: "Parent",
      filterText: "parent",
      detailLines: [{ text: "Parent" }],
      childSource: { kind: "lazy", load: loadChildren },
    };
    const sibling: OntologyNode = {
      id: "sibling",
      kind: "field",
      label: "Sibling",
      filterText: "sibling",
      detailLines: [{ text: "Sibling" }],
      childSource: { kind: "lazy", load: siblingLoad },
    };

    const firstChildren = await resolveOntologyNodeChildren(node);
    const secondChildren = await resolveOntologyNodeChildren(node);

    expect(firstChildren.map((child) => child.id)).toEqual(["child"]);
    expect(secondChildren).toBe(firstChildren);
    expect(loadChildren).toHaveBeenCalledTimes(1);
    expect(nestedLoad).not.toHaveBeenCalled();
    expect(siblingLoad).not.toHaveBeenCalled();

    await resolveOntologyNodeChildren(sibling);
    expect(siblingLoad).toHaveBeenCalledTimes(1);
  });
});
