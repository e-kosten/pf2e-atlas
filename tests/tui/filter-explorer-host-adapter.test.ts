import { describe, expect, it } from "vitest";

import {
  createComposeFilterExplorerHostAdapter,
  createInspectFilterExplorerHostAdapter,
  describeFilterExplorerHostNode,
} from "../../src/tui/filter-explorer/host-adapter.js";
import type { EntityPageTarget } from "../../src/app/ontology/entity-page.js";
import type {
  FilterExplorerControllerContext,
  FilterExplorerNode,
} from "../../src/tui/filter-explorer/types.js";
import type { PageDocumentModel } from "../../src/tui/page-document/model.js";

function createNode(overrides: Partial<FilterExplorerNode> = {}): FilterExplorerNode {
  return {
    id: "spell:trait:illusion",
    kind: "value",
    label: "Illusion",
    filterText: "illusion",
    detailLines: [{ text: "Illusion" }],
    ...overrides,
  };
}

function createComposeControllerContext(args: {
  currentNode?: FilterExplorerNode;
  discreteClauses?: FilterExplorerControllerContext["draft"]["discreteClauses"];
  scalarClauses?: FilterExplorerControllerContext["draft"]["scalarClauses"];
  selectedScalarClause?: FilterExplorerControllerContext["selectedScalarClause"];
}): FilterExplorerControllerContext {
  return {
    browser: {
      currentNode: args.currentNode,
    },
    draft: {
      discreteClauses: args.discreteClauses ?? [],
      scalarClauses: args.scalarClauses ?? {},
    },
    selectedScalarClause: args.selectedScalarClause,
  } as FilterExplorerControllerContext;
}

describe("filter explorer host adapter", () => {
  it("maps compose discrete selections to semantic state badges", () => {
    const node = createNode();
    const host = createComposeFilterExplorerHostAdapter({
      resolveTarget: () => ({
        field: "traits",
        fieldLabel: "Traits",
        value: "illusion",
        allowedOperators: ["include", "exclude"],
      }),
    });
    const controller = createComposeControllerContext({
      discreteClauses: [{ field: "traits", value: "illusion", operator: "include" }],
    });

    expect(
      describeFilterExplorerHostNode({
        host,
        node,
        isFocused: true,
        controller,
      }),
    ).toEqual({
      activationStyle: "toggle",
      stateBadge: { kind: "include" },
    });
  });

  it("surfaces compose scalar badge and focused summary through the host contract", () => {
    const node = createNode({
      id: "creature:actorMetrics:attributes.hp.max",
      kind: "metric",
      label: "Hit Points",
    });
    const host = createComposeFilterExplorerHostAdapter({
      resolveTarget: () => ({
        kind: "scalar",
        key: "actorMetric:attributes.hp.max",
        fieldLabel: "Creature Statistics",
        subjectLabel: "Hit Points",
        valueType: "number",
      }),
      onEditScalarTarget: () => undefined,
    });
    const controller = createComposeControllerContext({
      currentNode: node,
      selectedScalarClause: { operator: "gte", value: 12 },
    });

    expect(
      describeFilterExplorerHostNode({
        host,
        node,
        isFocused: true,
        controller,
      }),
    ).toEqual({
      activationStyle: "edit",
      stateBadge: { kind: "custom", text: "ƒ", tone: "accent" },
      suffixText: ">= 12",
    });
  });

  it("keeps inspect activation style host-owned instead of branching in the shared renderer", () => {
    const inspectNode = createNode({
      kind: "metric",
      query: {
        label: "Browse hit points",
        request: { mode: "browse", limit: 20 },
      },
    });
    const inspectHost = createInspectFilterExplorerHostAdapter({
      resolveTarget: () => ({
        kind: "scalar",
        key: "actorMetric:attributes.hp.max",
        fieldLabel: "Creature Statistics",
        subjectLabel: "Hit Points",
        valueType: "number",
      }),
      onEditScalarTarget: () => undefined,
    });

    expect(
      describeFilterExplorerHostNode({
        host: inspectHost,
        node: inspectNode,
        isFocused: true,
      }),
    ).toEqual({
      activationStyle: "edit",
      tone: undefined,
    });

    expect(
      describeFilterExplorerHostNode({
        host: createInspectFilterExplorerHostAdapter({}),
        node: createNode({
          query: {
            label: "Browse illusion spells",
            request: { mode: "browse", limit: 20 },
          },
        }),
        isFocused: true,
      }),
    ).toEqual({
      activationStyle: "open",
      tone: undefined,
    });

    expect(
      describeFilterExplorerHostNode({
        host: createInspectFilterExplorerHostAdapter({}),
        node: createNode(),
        isFocused: true,
      }),
    ).toEqual({
      activationStyle: "none",
      tone: undefined,
    });
  });

  it("passes inspect page-document and page-target seams through the shared host contract", () => {
    const pageDocument: PageDocumentModel = {
      recordKey: "spell:test-fireball",
      title: "Fireball",
      nodes: [],
      sections: [],
      sectionAnchors: [],
      targetNodes: [],
    };
    const searchPivotTarget: EntityPageTarget = {
      kind: "searchPivot",
      label: "Creatures (2)",
      request: { mode: "browse", limit: 20 },
    };
    const resolvePageDocument = () => pageDocument;
    const activatePageTarget = () => true;
    const host = createInspectFilterExplorerHostAdapter({
      resolvePageDocument,
      activatePageTarget,
    });

    expect(host.resolvePageDocument?.(createNode({ kind: "record" }))).toBe(pageDocument);
    expect(
      host.activatePageTarget?.({
        target: searchPivotTarget,
        controller: {} as FilterExplorerControllerContext,
      }),
    ).toBe(true);
  });
});
