import { describe, expect, it } from "vitest";

import {
  createWorkbenchState,
  workbenchReducer,
} from "../../src/tags/migration/workbench-ui.js";

describe("derived tag migration workbench reducer", () => {
  it("wraps top-level area selection", () => {
    const initial = createWorkbenchState();

    const next = workbenchReducer(initial, {
      type: "move_area",
      delta: -1,
    });

    expect(next.selectedAreaIndex).toBe(2);
  });

  it("clamps tag-refinement selection when the menu shrinks", () => {
    const initial = {
      ...createWorkbenchState({ kind: "tag_refinement" }),
      tagRefinementSelectedIndex: 6,
    };

    const next = workbenchReducer(initial, {
      type: "set_tag_refinement_index",
      index: 6,
      itemCount: 3,
    });

    expect(next.tagRefinementSelectedIndex).toBe(2);
  });
});
