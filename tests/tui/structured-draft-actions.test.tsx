import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { describe, expect, it } from "vitest";

import { useSearchStructuredDraftActions } from "../../src/tui/search-screen/structured-draft/structured-draft-actions.js";
import { createStructuredDraftGroupResumeTarget } from "../../src/tui/search-screen/structured-draft/structured-draft-state.js";
import type { SearchWorkspaceUser } from "../../src/tui/search-screen/workspace/workspace-action-types.js";
import type { Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import {
  allOfFilter,
  metadataPredicateFilter,
  notFilter,
  packFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

type StructuredDraftActions = ReturnType<typeof useSearchStructuredDraftActions>;

const user = {
  search: {
    getPackLabel: (packValue: string) => packValue,
    getQueryFieldOptions: () => [
      { value: "traits", label: "Traits", editor: "sharedExplorer" },
      { value: "families", label: "Families", editor: "sharedExplorer" },
    ],
    normalizeQuery: (query: Pf2eTerminalSearchQuery) => query,
  },
} as SearchWorkspaceUser;

function renderStructuredDraftActions(initialQuery: Pf2eTerminalSearchQuery): {
  getActions: () => StructuredDraftActions;
  renderer: ReactTestRenderer;
} {
  let actions: StructuredDraftActions | null = null;

  function Harness(): null {
    const [query, setQuery] = React.useState(initialQuery);
    actions = useSearchStructuredDraftActions({
      applyQueryUpdate: (update) => setQuery((current) => update(current)),
      currentQuery: query,
      user,
    });
    return null;
  }

  let renderer: ReactTestRenderer | null = null;
  act(() => {
    renderer = create(<Harness />);
  });
  return {
    getActions: () => {
      if (!actions) {
        throw new Error("Structured draft actions did not render.");
      }
      return actions;
    },
    renderer: renderer!,
  };
}

describe("structured draft action resume targeting", () => {
  it("stores the canonical resume target after projection reshapes the requested group path", () => {
    const initialQuery: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" }),
          packFilter("monster-core"),
        ]),
      ]),
    };
    const reshapedQuery: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        notFilter(
          allOfFilter([
            metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" }),
            packFilter("monster-core"),
          ]),
        ),
      ]),
    };
    const { getActions, renderer } = renderStructuredDraftActions(initialQuery);

    act(() => {
      getActions().openStructuredDraftSession({ kind: "queryNode", path: [1] });
    });
    act(() => {
      getActions().replaceStructuredDraftProjection(() => reshapedQuery, {
        resumeTarget: createStructuredDraftGroupResumeTarget([1]),
      });
    });

    expect(getActions().structuredDraftState?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
    expect(getActions().structuredDraftEntries[getActions().structuredDraftState?.selectedIndex ?? -1]).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [],
    });

    renderer.unmount();
  });
});
