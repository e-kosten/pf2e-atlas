import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AreaMenuScreen, type Pf2eTopLevelArea } from "../../src/tui/area-menu-screen.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";
import { TagRefinementMenuScreen } from "../../src/tui/tag-refinement-menu-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

afterEach(() => {
  cleanup();
});

describe("menu transition status hosts", () => {
  it("keeps the area menu mounted while showing a pending route transition", async () => {
    const areas: Pf2eTopLevelArea[] = [
      {
        id: "search",
        audience: "user",
        label: "Browse/Search",
        description: "Search and browse records.",
      },
      {
        id: "ontology_search",
        audience: "user",
        label: "Search Semantics",
        description: "Inspect ontology-backed search fields.",
      },
    ];

    const app = render(
      <DerivedTagTerminalProvider>
        <AreaMenuScreen
          title="PF2E Terminal"
          selectedAreaIndex={1}
          areas={areas}
          pendingReviewCount={2}
          onOpenSelectedArea={vi.fn()}
          onMove={vi.fn()}
          onQuit={vi.fn()}
          transitionStatus={{ kind: ROUTE_TRANSITION_STATUS_KIND.PENDING, message: "Opening Search Semantics..." }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Choose a first-class TUI area");
    expect(frame).toContain("Search Semantics");
    expect(frame).toContain("Area Details");
    expect(frame).toContain("Loading next view | Opening Search Semantics...");
  });

  it("keeps the tag-refinement menu mounted while showing a pending route transition", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <TagRefinementMenuScreen
          selectedIndex={0}
          queueItems={[]}
          onBack={vi.fn()}
          onMove={vi.fn()}
          onOpenSelected={vi.fn()}
          onQuickAction={vi.fn()}
          transitionStatus={{
            kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
            message: "Preparing legacy-seed review session...",
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    const frame = app.lastFrame();
    expect(frame).toContain("Pending Review Queue");
    expect(frame).toContain("Create legacy-seed review session");
    expect(frame).toContain("Pending Review Queue");
    expect(frame).toContain("Loading next view | Preparing legacy-seed review session...");
  });
});
