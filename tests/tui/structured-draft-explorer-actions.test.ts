import { describe, expect, it } from "vitest";

import { canOpenStructuredDraftExactNodeFieldFallback } from "../../src/tui/search-screen/structured-draft/structured-draft-explorer-actions.js";
import type { Pf2eTerminalQueryFieldOption } from "../../src/tui/search/service.js";
import {
  allOfFilter,
  browseQuery,
  metadataPredicateFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

describe("structured draft explorer actions", () => {
  const traitsSharedExplorerFieldOption = {
    value: "traits",
    label: "Traits",
    description: "Browse traits.",
    fieldType: "set",
    editor: "sharedExplorer",
  } satisfies Pf2eTerminalQueryFieldOption;

  it("allows exact-node shared-explorer fallback only for the current canonical leaf", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;

    expect(
      canOpenStructuredDraftExactNodeFieldFallback({
        query,
        path: [1],
        fieldOption: traitsSharedExplorerFieldOption,
        currentNode: { field: "traits", op: "includes", value: "evil" },
      }),
    ).toBe(true);

    expect(
      canOpenStructuredDraftExactNodeFieldFallback({
        query,
        path: [0],
        fieldOption: traitsSharedExplorerFieldOption,
        currentNode: { field: "traits", op: "includes", value: "evil" },
      }),
    ).toBe(false);
    expect(
      canOpenStructuredDraftExactNodeFieldFallback({
        query,
        path: [],
        fieldOption: traitsSharedExplorerFieldOption,
        currentNode: { field: "traits", op: "includes", value: "evil" },
      }),
    ).toBe(false);
  });
});
