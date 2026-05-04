import { describe, expect, it } from "vitest";

import { buildStructuredDraftEntries } from "../../src/tui/search-screen/structured-draft/structured-draft-support.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftNodeResumeTarget,
  createStructuredDraftResumeTargetForContainingGroup,
  createStructuredDraftResumeTargetForNodePath,
  createStructuredDraftRootResumeTarget,
  getStructuredDraftSelectionIndexForResumeTarget,
} from "../../src/tui/search-screen/structured-draft/structured-draft-state.js";
import type { Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import type { SearchStructuredDraftEntry } from "../../src/tui/search/structured-draft-session.js";
import {
  allOfFilter,
  anyOfFilter,
  metadataPredicateFilter,
  notFilter,
  packFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

/*
Path matrix for the resume-target state owner.

| Intent family | Child surface | Durable target | Selection invariant |
| --- | --- | --- | --- |
| Reopen/root continuation | host tree | root | select visible root group |
| Group-local continuation | prompt or explorer | groupPath | select containing group and derive active buckets from canonical paths |
| Exact structural node action | host tree | node path | select exact node while it still exists |
| Unary not wrapper | host tree | node/containing group | never derive a peer group anchor from the not wrapper itself |
| Grouped field bucket | shared explorer | groupPath + member paths | select the projected bucket from canonical member paths, not durable projected identity |

Continuation-coordinator and broad interaction-flow coverage belongs in later Slice A
host/coordinator tests; this file only pins the state/resume-target invariants.
*/

function buildEntries(
  query: Pf2eTerminalSearchQuery,
  target: Parameters<typeof buildStructuredDraftEntries>[1],
): SearchStructuredDraftEntry[] {
  return buildStructuredDraftEntries(query, target, {
    groupedFieldValues: new Set(["traits", "families"]),
  });
}

function selectedEntry(
  entries: SearchStructuredDraftEntry[],
  target: Parameters<typeof getStructuredDraftSelectionIndexForResumeTarget>[1],
): SearchStructuredDraftEntry | undefined {
  return entries[getStructuredDraftSelectionIndexForResumeTarget(entries, target, 0)];
}

describe("structured draft resume target state", () => {
  it("selects the visible root group for root resume targets", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
      ]),
    };
    const target = createStructuredDraftRootResumeTarget();

    expect(selectedEntry(buildEntries(query, target), target)).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [],
    });
  });

  it("uses groupPath as the live continuation anchor for grouped field editing", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "amphibious" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" }),
          packFilter("monster-core"),
        ]),
      ]),
    };
    const target = createStructuredDraftGroupResumeTarget([1]);
    const entries = buildEntries(query, target);

    expect(selectedEntry(entries, target)).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [1],
    });
    expect(entries).toContainEqual(
      expect.objectContaining({
        kind: "queryFieldBucket",
        groupPath: [1],
        field: "traits",
        fieldMemberPaths: [[1, 0], [1, 1]],
      }),
    );
  });

  it("keeps exact node paths available for structural node-scoped resume", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" }),
          packFilter("monster-core"),
        ]),
      ]),
    };
    const target = createStructuredDraftNodeResumeTarget([1, 1]);

    expect(selectedEntry(buildEntries(query, target), target)).toMatchObject({
      kind: "queryNode",
      treePath: [1, 1],
      label: "Pack: monster-core",
    });
  });

  it("does not derive a peer group resume target from a unary not wrapper", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "undead" })),
      ]),
    };

    expect(createStructuredDraftResumeTargetForNodePath(query.filter, [1])).toEqual({
      kind: "node",
      path: [1],
    });
    expect(createStructuredDraftResumeTargetForContainingGroup(query.filter, [1])).toEqual({
      kind: "group",
      groupPath: [],
    });
  });

  it("selects grouped buckets from canonical member paths when leaf rows are projected away", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "families", op: "includes", value: "focus" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "visual" }),
      ]),
    };
    const target = createStructuredDraftNodeResumeTarget([3]);
    const selected = selectedEntry(buildEntries(query, target), target);

    expect(selected).toMatchObject({
      kind: "queryFieldBucket",
      groupPath: [],
      field: "traits",
      memberPaths: [[2], [3]],
      label: "Traits: Include illusion, visual",
    });
  });

  it("derives a group resume target from boolean groups without encoding projected bucket identity", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: allOfFilter([
        scopeFilter("creature"),
        anyOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "amphibious" }),
        ]),
      ]),
    };

    expect(createStructuredDraftResumeTargetForNodePath(query.filter, [1])).toEqual({
      kind: "group",
      groupPath: [1],
    });
  });
});
