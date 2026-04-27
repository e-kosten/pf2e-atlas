import { describe, expect, it, vi } from "vitest";

import {
  buildCompiledFilterExplorerInspectResult,
  buildFilterExplorerInspectResult,
  openFilterExplorerInspectQuery,
  openFilterExplorerInspectResult,
  resolveFilterExplorerLaunchIntent,
  shouldOpenImmediateFilterExplorerInspectResult,
} from "../../src/tui/filter-explorer/controller-inspect.js";
import {
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerInspectAndOpenMode,
  type FilterExplorerSelectTargetOutcome,
} from "../../src/tui/filter-explorer/index.js";
import { canonicalFilterToMetadataNode } from "../../src/tui/search/query-parts.js";
import type { FilterExplorerKeyContext } from "../../src/tui/filter-explorer/controller-types.js";
import type {
  FilterExplorerNode,
  FilterExplorerOptions,
} from "../../src/tui/filter-explorer/types.js";
import { browseRequest, searchRequest, scopeFilter } from "../helpers/search-request-fixture.js";

function createNode(overrides: Partial<FilterExplorerNode> = {}): FilterExplorerNode {
  return {
    id: "creature:hp",
    kind: "value",
    label: "Hit Points",
    filterText: "hit points",
    detailLines: [{ text: "Hit Points" }],
    query: {
      label: "Browse by hit points",
      request: browseRequest({ filter: scopeFilter("creature"), limit: 20 }),
    },
    ...overrides,
  };
}

function createKeyContext(): FilterExplorerKeyContext {
  return {
    state: {
      activePane: "list",
      browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
      layoutMode: "split",
      searchInput: "",
      searchMode: false,
    },
    effectiveState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
    selection: { ancestors: [], currentNodes: [], currentNode: undefined, currentParent: undefined },
    currentNode: undefined,
    currentNodeHasChildren: false,
    breadcrumb: "",
    bodyHeight: 20,
    detailWidth: 40,
    detailLines: [],
    visibleDetailLines: [],
    detailTitle: "Detail",
    layoutMode: "split",
    maxDetailScroll: 0,
    detailJumpSize: 5,
    detailPageSize: 10,
    selectionJumpSize: 5,
    searchIndicator: "",
    event: {
      input: "\r",
      key: { return: true },
      isBackNavigationKey: () => true,
      isCommandPaletteKey: () => false,
      isConfirmKey: () => true,
      isConfirmOrToggleKey: () => true,
      isExactPrintableKey: () => false,
      isExecuteKey: () => true,
      isFocusToggleKey: () => false,
      isHelpKey: () => false,
      isLayoutToggleKey: () => false,
      isMoveDownKey: () => false,
      isMoveLeftKey: () => false,
      isMoveRightKey: () => false,
      isMoveUpKey: () => false,
      isPageDownKey: () => false,
      isPageUpKey: () => false,
      isSearchKey: () => false,
      isTerminalBoundaryEndKey: () => false,
      isTerminalBoundaryStartKey: () => false,
      isTerminalJumpBackwardKey: () => false,
      isTerminalJumpForwardKey: () => false,
      isTerminalQuitKey: () => false,
      getCycleDirection: () => undefined,
    } as FilterExplorerKeyContext["event"],
  };
}

function createOptions(mode: FilterExplorerInspectAndOpenMode): FilterExplorerOptions {
  return {
    model: {
      id: "search-semantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [createNode()],
    },
    mode,
    onOutcome: vi.fn(),
  };
}

describe("filter explorer controller inspect", () => {
  it("derives launch intent from query kind and mode defaults", () => {
    const mode: FilterExplorerInspectAndOpenMode = { kind: "inspect-and-open" };

    expect(
      resolveFilterExplorerLaunchIntent(mode, {
        request: searchRequest({ search: { query: "" }, limit: 20 }),
      }),
    ).toBe(FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
    expect(
      resolveFilterExplorerLaunchIntent(
        {
          kind: "inspect-and-open",
          defaultListRecordLaunchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
        },
        createNode().query!,
      ),
    ).toBe(FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
  });

  it("builds compiled scalar inspect queries for numeric metric targets", () => {
    const result = buildFilterExplorerInspectResult(
      {
        kind: "inspect-and-open",
        resolveInspectTarget: () => ({
          kind: "scalar",
          key: "actorMetric:attributes.hp.max",
          fieldLabel: "Hit Points",
          subjectLabel: "Hit Points",
          valueType: "number",
        }),
      },
      createNode(),
    );

    const compiled = buildCompiledFilterExplorerInspectResult(result!, {
      operator: "between",
      min: 40,
      max: 80,
    });

    expect(compiled).toMatchObject({
      launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.RESULTS,
      query: {
        label: "Browse records where Hit Points between 40 and 80",
        request: {
          limit: 20,
          mode: "browse",
        },
      },
    });
    expect(canonicalFilterToMetadataNode(compiled?.query.request.filter)).toEqual({
      and: [
        {
          field: "actorMetric",
          metric: "attributes.hp.max",
          op: ">=",
          value: 40,
        },
        {
          field: "actorMetric",
          metric: "attributes.hp.max",
          op: "<=",
          value: 80,
        },
      ],
    });
  });

  it("opens inspect queries with editor intent snapshots", () => {
    const node = createNode();
    const options = createOptions({ kind: "inspect-and-open" });
    const result = buildFilterExplorerInspectResult(options.mode, node);
    const handled = openFilterExplorerInspectQuery({
      options,
      keyContext: createKeyContext(),
      result,
    });

    expect(handled).toBe(true);
    expect(options.onOutcome).toHaveBeenCalledWith(
      {
        kind: "selectTarget",
        activationStyle: "open",
        result: {
          ...result,
          launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
        },
        queryIntent: {
          query: node.query,
          launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
        },
      },
      expect.objectContaining({
        activePane: "list",
        browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
      }),
    );
  });

  it("compiles scalar inspect edits before opening results", async () => {
    const options = createOptions({
      kind: "inspect-and-open",
      onEditScalarTarget: () => ({ operator: "gte", value: 12 }),
      resolveInspectTarget: () => ({
        kind: "scalar",
        key: "actorMetric:speed.land",
        fieldLabel: "Land Speed",
        subjectLabel: "Land Speed",
        valueType: "number",
      }),
    });
    const result = buildFilterExplorerInspectResult(options.mode, createNode());

    const handled = openFilterExplorerInspectResult({
      options,
      keyContext: createKeyContext(),
      result,
    });

    expect(handled).toBe(true);
    await Promise.resolve();

    expect(options.onOutcome).toHaveBeenCalledWith(
      {
        kind: "selectTarget",
        activationStyle: "edit",
        result: {
          ...result,
          launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.RESULTS,
          query: {
            ...createNode().query,
            label: "Browse records where Land Speed >= 12",
            request: {
              ...createNode().query!.request,
              filter: expect.anything(),
            },
          },
        },
        queryIntent: {
          query: {
            ...createNode().query,
            label: "Browse records where Land Speed >= 12",
            request: {
              ...createNode().query!.request,
              filter: expect.anything(),
            },
          },
          launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.RESULTS,
        },
      },
      expect.any(Object),
    );
    const [[call]] = (options.onOutcome as ReturnType<typeof vi.fn>).mock.calls;
    expect(canonicalFilterToMetadataNode(call.queryIntent.query.request.filter)).toEqual({
      field: "actorMetric",
      metric: "speed.land",
      op: ">=",
      value: 12,
    });
  });

  it("opens non-record list nodes immediately but record nodes only on leaves", () => {
    const nonRecordNode = createNode({ kind: "value" });
    const recordNode = createNode({ kind: "record" });
    const result = buildFilterExplorerInspectResult({ kind: "inspect-and-open" }, nonRecordNode);

    expect(shouldOpenImmediateFilterExplorerInspectResult(nonRecordNode, result)).toBe(true);
    expect(
      shouldOpenImmediateFilterExplorerInspectResult(
        recordNode,
        buildFilterExplorerInspectResult({ kind: "inspect-and-open" }, recordNode),
      ),
    ).toBe(false);
  });
});
