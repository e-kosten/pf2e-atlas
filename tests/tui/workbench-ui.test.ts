import { describe, expect, it } from "vitest";

import { PF2E_APP_AREAS } from "../../src/tui/app-areas.js";
import { createPf2eAppState, pf2eAppReducer } from "../../src/tui/pf2e-app-state.js";

describe("derived tag migration workbench reducer", () => {
  it("orders top-level areas by user-facing workflow", () => {
    expect(PF2E_APP_AREAS.map((area) => area.label)).toEqual([
      "Browse/Search",
      "Ontology Browser",
      "Tag Refinement",
    ]);
  });

  it("wraps top-level area selection", () => {
    const initial = createPf2eAppState();

    const next = pf2eAppReducer(initial, {
      type: "move_area",
      delta: -1,
      itemCount: 3,
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
