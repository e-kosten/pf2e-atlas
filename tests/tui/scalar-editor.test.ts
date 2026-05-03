import { describe, expect, it, vi } from "vitest";

import {
  promptLevelRangeDraft,
  promptNumericScalarClause,
} from "../../src/tui/filter-explorer/scalar-editor.js";

describe("scalar-editor prompts", () => {
  it("parses overlay numeric matcher input for shared scalar clauses", async () => {
    const promptTextInput = vi.fn().mockResolvedValue(">5");
    const pauseForAnyKey = vi.fn();

    await expect(
      promptNumericScalarClause(
        { promptTextInput },
        { pauseForAnyKey },
        {
          title: "Action Cost Clause",
          currentClause: null,
        },
      ),
    ).resolves.toEqual({ op: "gt", value: 5 });

    expect(promptTextInput).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Action Cost Clause",
        previewTitle: "Preview",
        buildPreviewLines: expect.any(Function),
        presentation: "overlay",
      }),
    );
    expect(pauseForAnyKey).not.toHaveBeenCalled();
  });

  it("accepts strict upper-bound level syntax in the overlay level matcher", async () => {
    const promptTextInput = vi.fn().mockResolvedValue("<7");
    const pauseForAnyKey = vi.fn();

    await expect(
      promptLevelRangeDraft(
        { promptTextInput },
        { pauseForAnyKey },
        {
          defaultValue: ">=5",
        },
      ),
    ).resolves.toEqual({ kind: "lt", value: 7 });

    expect(promptTextInput).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Level Range",
        previewTitle: "Preview",
        buildPreviewLines: expect.any(Function),
        presentation: "overlay",
      }),
    );
  });
});
