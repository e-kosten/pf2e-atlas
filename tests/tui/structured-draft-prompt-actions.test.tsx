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
  openPromptPackClause = vi.fn(async () => ({ kind: "cancel" })),
  openPromptRarityClause = vi.fn(async () => ({ kind: "cancel" })),
  promptSelectOption = vi.fn(),
  promptTextInput = vi.fn(),
  selectPromptMetricKey = vi.fn(async () => ({ kind: "cancel" })),
}: {
  fieldOptions?: Pf2eTerminalQueryFieldOption[];
  openPromptFieldClause?: ReturnType<typeof vi.fn>;
  openPromptPackClause?: ReturnType<typeof vi.fn>;
  openPromptRarityClause?: ReturnType<typeof vi.fn>;
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
        { value: "creature", label: "Creature", description: "Creature records." },
      ]),
      getSubcategoryOptions: vi.fn(() => [
        { value: null, label: "Any", description: "Any subcategory." },
        { value: "familiar", label: "Familiar", description: "Familiars." },
      ]),
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
      openPromptPackClause,
      openPromptRarityClause,
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

  it("gets pack and rarity nodes through injected high-level explorer actions", async () => {
    const openPromptPackClause = vi.fn(async () => ({ kind: "apply", value: { kind: "pack", value: "equipment" } }));
    const openPromptRarityClause = vi.fn(async () => ({
      kind: "apply",
      value: { kind: "rarity", match: { kind: "eq", value: "common" } },
    }));
    const { getActions, promptSession, renderer } = renderPromptActions({
      openPromptPackClause,
      openPromptRarityClause,
    });

    const packResult = await getActions().promptForClauseNode(promptSession, baseQuery, "pack");
    const rarityResult = await getActions().promptForClauseNode(promptSession, baseQuery, "rarity");

    expect(packResult).toEqual({ kind: "apply", value: { kind: "pack", value: "equipment" } });
    expect(rarityResult).toEqual({
      kind: "apply",
      value: { kind: "rarity", match: { kind: "eq", value: "common" } },
    });
    expect(openPromptPackClause).toHaveBeenCalledWith(baseQuery, undefined);
    expect(openPromptRarityClause).toHaveBeenCalledWith(baseQuery, undefined);

    renderer.unmount();
  });

  it("routes metric field option classification through the prompt owner helper", () => {
    expect(isMetricFieldOptionValue("actorMetric")).toBe(true);
    expect(isMetricFieldOptionValue("itemMetric")).toBe(true);
    expect(isMetricFieldOptionValue("traits")).toBe(false);
  });
});
