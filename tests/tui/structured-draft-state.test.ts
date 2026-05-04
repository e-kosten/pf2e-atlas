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
Path matrix for the structured-editor continuation model.

| Intent family | Child surface | Host entrypoint | Mutation kind | Resume/focus target |
| --- | --- | --- | --- | --- |
| Reopen/root continuation | host tree | structured draft session open | none | root resume selects visible root group |
| Group-local add clause | prompt builder | addQueryClauseAtPath | appendNodes | containing groupPath, then derive visible row |
| Group-local explorer field | shared explorer | runStructuredDraftExplorerContinuation | replaceGroupedField or appendNodes | groupPath; buckets derive from canonical members |
| Exact structural edit | host tree | runNodeAction/runInsertionAction | replaceNode/remove/wrap/move/lift/unwrap | node path only while semantically present |
| Metric-key or pack discovery | shared explorer | runStructuredDraftExplorerContinuation | bounded node result consumed by prompt flow | prompt resumes through host state, not explorer-local state |
| Unary not wrapper | host tree | wrapNot/addNotGroup | replaceNode or appendNodes | node/containing group; never a peer group anchor |

This file pins the resume-target and projection rows. Coordinator lifecycle coverage
lives in structured-editor-continuation-coordinator.test.ts, grouped mutation helper
coverage lives in structured-draft-metadata-actions.test.ts, and broad end-to-end
interaction coverage stays in search-screen.test.tsx.
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
        fieldMemberPaths: [
          [1, 0],
          [1, 1],
        ],
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
