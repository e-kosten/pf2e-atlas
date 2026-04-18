import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import { TagRefinementMenuScreen } from "../../src/tui/tag-refinement-menu-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

describe("tag refinement menu screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("runs page-specific actions through the shared action rail", async () => {
    const onQuickAction = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <TagRefinementMenuScreen
          selectedIndex={0}
          queueItems={[]}
          onBack={vi.fn()}
          onMove={vi.fn()}
          onOpenSelected={vi.fn()}
          onQuickAction={onQuickAction}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Create Legacy-Seed Review Session");
    expect(app.lastFrame()).not.toContain("[ACTIONS]");
    app.stdin.write("\r");
    await flushInk();

    expect(onQuickAction).toHaveBeenCalledWith("legacy_seed");
  });

  it("does not keep old page-specific letters as live commands", async () => {
    const onQuickAction = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <TagRefinementMenuScreen
          selectedIndex={0}
          queueItems={[]}
          onBack={vi.fn()}
          onMove={vi.fn()}
          onOpenSelected={vi.fn()}
          onQuickAction={onQuickAction}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("s");
    await flushInk();

    expect(onQuickAction).not.toHaveBeenCalled();
  });

  it("keeps left-arrow ownership inside the action rail", async () => {
    const onBack = vi.fn();
    const onQuickAction = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <TagRefinementMenuScreen
          selectedIndex={0}
          queueItems={[]}
          onBack={onBack}
          onMove={vi.fn()}
          onOpenSelected={vi.fn()}
          onQuickAction={onQuickAction}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write(":");
    await flushInk();
    app.stdin.write("\u001b[D");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onBack).not.toHaveBeenCalled();
    expect(onQuickAction).toHaveBeenCalledWith("proposal_review");
  });
});
