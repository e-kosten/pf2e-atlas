import { describe, expect, it } from "vitest";

import {
  createPf2eAppState,
  pf2eAppReducer,
} from "../../src/tui/pf2e-app.js";

describe("derived tag migration workbench reducer", () => {
  it("wraps top-level area selection", () => {
    const initial = createPf2eAppState();

    const next = pf2eAppReducer(initial, {
      type: "move_area",
      delta: -1,
    });

    expect(next.selectedAreaIndex).toBe(2);
  });

  it("clamps tag-refinement selection when the menu shrinks", () => {
    const initial = {
      ...createPf2eAppState({ kind: "tag_refinement" }),
      tagRefinementSelectedIndex: 6,
    };

    const next = pf2eAppReducer(initial, {
      type: "set_tag_refinement_index",
      index: 6,
      itemCount: 3,
    });

    expect(next.tagRefinementSelectedIndex).toBe(2);
  });
});
