import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { describe, expect, it, vi } from "vitest";

import {
  isMetricFieldOptionValue,
  useStructuredDraftPromptActions,
  type ClauseKind,
} from "../../src/tui/search-screen/structured-draft/structured-draft-prompt-actions.js";
import type { SearchWorkspacePromptAdapters } from "../../src/tui/search-screen/workspace/workspace-action-types.js";
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import { browseQuery } from "../helpers/search-request-fixture.js";

type PromptActions = ReturnType<typeof useStructuredDraftPromptActions>;

const baseQuery: Pf2eTerminalSearchQuery = browseQuery("Browse").request;

function renderPromptActions({
  fieldOptions = [],
  openPromptFieldClause = vi.fn(),
  loadCategoryOptions = vi.fn(async () => [
    { value: null, label: "Any", description: "Any category." },
    { value: "creature", label: "Creature | 12", description: "12 matching canonical records." },
  ]),
  loadSubcategoryOptions = vi.fn(async () => [
    { value: null, label: "Any", description: "Any subcategory." },
    { value: "familiar", label: "Familiar | 3", description: "3 matching canonical records." },
  ]),
  promptSelectOption = vi.fn(),
  promptTextInput = vi.fn(),
  selectPromptMetricKey = vi.fn(async () => ({ kind: "cancel" })),
}: {
  fieldOptions?: Pf2eTerminalQueryFieldOption[];
  openPromptFieldClause?: ReturnType<typeof vi.fn>;
  loadCategoryOptions?: ReturnType<typeof vi.fn>;
  loadSubcategoryOptions?: ReturnType<typeof vi.fn>;
  promptSelectOption?: ReturnType<typeof vi.fn>;
  promptTextInput?: ReturnType<typeof vi.fn>;
  selectPromptMetricKey?: ReturnType<typeof vi.fn>;
} = {}): {
  getActions: () => PromptActions;
  promptSession: SearchWorkspacePromptAdapters;
  renderer: ReactTestRenderer;
  terminal: { pauseForAnyKey: ReturnType<typeof vi.fn> };
} {
  let actions: PromptActions | null = null;
  const terminal = {
    pauseForAnyKey: vi.fn(),
    runPromptSession: vi.fn(),
  };
  const user = {
    search: {
      getActionCostOptions: vi.fn(() => []),
      getCategoryOptions: vi.fn(() => [
        { value: null, label: "Any", description: "Any category." },
        { value: "creature", label: "Creature | 12", description: "12 creature records." },
      ]),
      getSubcategoryOptions: vi.fn(() => [
        { value: null, label: "Any", description: "Any subcategory." },
        { value: "familiar", label: "Familiar | 3", description: "3 familiar records." },
      ]),
      loadCategoryOptions,
      loadSubcategoryOptions,
      loadMetricKeyOptions: vi.fn(async () => []),
      prepareFilterExplorerDraft: vi.fn(() => ({
        draft: { discreteClauses: [], scalarClauses: {} },
      })),
    },
  };

  function Harness(): null {
    actions = useStructuredDraftPromptActions({
      editFieldClause: vi.fn(),
      getScopedFieldOptions: () => fieldOptions,
      openPromptFieldClause,
      selectPromptMetricKey,
      terminal,
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
        throw new Error("Prompt actions did not render.");
      }
      return actions;
    },
    promptSession: {
      promptMultiSelectOption: vi.fn(),
      promptOptionalSelectOption: vi.fn(),
      promptSelectOption,
      promptTextInput,
    },
    renderer: renderer!,
    terminal,
  };
}

describe("structured draft prompt actions", () => {
  it("offers pack and rarity clauses before a scope exists", async () => {
    const promptSelectOption = vi.fn(async () => ({ kind: "selected", value: "rarity" }));
    const { getActions, promptSession, renderer } = renderPromptActions({ promptSelectOption });

    const result = await getActions().promptForClauseKind(promptSession, baseQuery);

    expect(result).toEqual({ kind: "apply", value: "rarity" });
    const entries = promptSelectOption.mock.calls[0]?.[0].entries.map((entry: { value: ClauseKind }) => entry.value);
    expect(entries).toContain("pack");
    expect(entries).toContain("rarity");

    renderer.unmount();
  });

  it("builds a scope clause from prompt-local category and subcategory choices", async () => {
    const promptSelectOption = vi
      .fn()
      .mockResolvedValueOnce({ kind: "selected", value: "creature" })
      .mockResolvedValueOnce({ kind: "selected", value: "specific" })
      .mockResolvedValueOnce({ kind: "selected", value: "familiar" });
    const { getActions, promptSession, renderer } = renderPromptActions({ promptSelectOption });

    const result = await getActions().promptForClauseNode(promptSession, baseQuery, "scope");

    expect(result).toEqual({
      kind: "apply",
      value: { kind: "scope", category: "creature", subcategory: { kind: "eq", value: "familiar" } },
    });
    expect(promptSelectOption.mock.calls[0]?.[0].actionEntries).toContainEqual({
      id: "useCatalogCounts",
      label: "Use Catalog Counts",
      description: "Show counts from the wider applicability slice.",
    });
    expect(promptSelectOption.mock.calls[0]?.[0].entries).toContainEqual({
      value: "creature",
      label: "Creature | 12",
      description: "12 matching canonical records.",
    });
    expect(promptSelectOption.mock.calls[2]?.[0].entries).toContainEqual({
      value: "familiar",
      label: "Familiar | 3",
      description: "3 matching canonical records.",
    });

    renderer.unmount();
  });

  it("switches scope prompt counts through the select action rail", async () => {
    const loadCategoryOptions = vi.fn(async (_query, discoveryMode: "matching" | "catalog") => [
      { value: null, label: "Any", description: "Any category." },
      {
        value: "creature",
        label: discoveryMode === "matching" ? "Creature | 2" : "Creature | 12",
        description:
          discoveryMode === "matching" ? "2 matching canonical records." : "12 applicable canonical records.",
      },
    ]);
    const loadSubcategoryOptions = vi.fn(async () => [
      { value: null, label: "Any", description: "Any subcategory." },
    ]);
    const promptSelectOption = vi
      .fn()
      .mockResolvedValueOnce({ kind: "action", actionId: "useCatalogCounts" })
      .mockResolvedValueOnce({ kind: "selected", value: "creature" })
      .mockResolvedValueOnce({ kind: "selected", value: "any" });
    const { getActions, promptSession, renderer } = renderPromptActions({
      loadCategoryOptions,
      loadSubcategoryOptions,
      promptSelectOption,
    });

    const result = await getActions().promptForClauseNode(promptSession, baseQuery, "scope");

    expect(result).toEqual({
      kind: "apply",
      value: { kind: "scope", category: "creature", subcategory: { kind: "any" } },
    });
    expect(loadCategoryOptions.mock.calls.map((call) => call[1])).toEqual(["matching", "catalog"]);
    expect(promptSelectOption.mock.calls[1]?.[0].entries).toContainEqual({
      value: "creature",
      label: "Creature | 12",
      description: "12 applicable canonical records.",
    });

    renderer.unmount();
  });

  it("returns a range level clause from scalar prompt input", async () => {
    const promptTextInput = vi.fn(async () => "3-8");
    const { getActions, promptSession, renderer, terminal } = renderPromptActions({ promptTextInput });

    const result = await getActions().promptForClauseNode(promptSession, baseQuery, "level");

    expect(result).toEqual({
      kind: "apply",
      value: { kind: "level", match: { kind: "between", min: 3, max: 8 } },
    });
    expect(terminal.pauseForAnyKey).not.toHaveBeenCalled();

    renderer.unmount();
  });

  it("keeps invalid scalar input local and cancels without emitting a node", async () => {
    const promptTextInput = vi.fn(async () => "not-a-number");
    const { getActions, promptSession, renderer, terminal } = renderPromptActions({ promptTextInput });

    const result = await getActions().promptForClauseNode(promptSession, baseQuery, "price");

    expect(result).toEqual({ kind: "cancel" });
    expect(terminal.pauseForAnyKey).toHaveBeenCalledWith(
      "Use `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.",
    );

    renderer.unmount();
  });

  it("keeps grouped metadata fields out of prompt-local field-node construction", async () => {
    const openPromptFieldClause = vi.fn();
    const traitsField = {
      value: "traits",
      label: "Traits",
      description: "Browse traits.",
      fieldType: "set",
      editor: "sharedExplorer",
    } satisfies Pf2eTerminalQueryFieldOption;
    const { getActions, promptSession, renderer, terminal } = renderPromptActions({
      fieldOptions: [traitsField],
      openPromptFieldClause,
    });

    const result = await getActions().promptForClauseNode(promptSession, baseQuery, "field");

    expect(result).toEqual({ kind: "cancel" });
    expect(openPromptFieldClause).not.toHaveBeenCalled();
    expect(terminal.pauseForAnyKey).toHaveBeenCalledWith("No scoped field filters are available for the current query.");
    renderer.unmount();
  });

  it("routes metric field option classification through the prompt owner helper", () => {
    expect(isMetricFieldOptionValue("actorMetric")).toBe(true);
    expect(isMetricFieldOptionValue("itemMetric")).toBe(true);
    expect(isMetricFieldOptionValue("traits")).toBe(false);
  });
});
