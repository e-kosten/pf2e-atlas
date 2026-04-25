import { describe, expect, it, vi } from "vitest";

import {
  promptLevelRangeDraft,
  promptNumericScalarClause,
} from "../../src/tui/filter-explorer/scalar-editor.js";

describe("scalar-editor prompts", () => {
  it("parses centered numeric matcher input for shared scalar clauses", async () => {
    const promptTextInput = vi.fn().mockResolvedValue(">=5");
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
    ).resolves.toEqual({ op: "gte", value: 5 });

    expect(promptTextInput).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Action Cost Clause",
        presentation: "centered",
      }),
    );
    expect(pauseForAnyKey).not.toHaveBeenCalled();
  });

  it("accepts >= level syntax in the centered level matcher", async () => {
    const promptTextInput = vi.fn().mockResolvedValue(">=7");
    const pauseForAnyKey = vi.fn();

    await expect(
      promptLevelRangeDraft(
        { promptTextInput },
        { pauseForAnyKey },
        {
          defaultValue: ">=5",
        },
      ),
    ).resolves.toEqual({ levelMin: 7, levelMax: null });

    expect(promptTextInput).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Level Range",
        presentation: "centered",
      }),
    );
  });
});
