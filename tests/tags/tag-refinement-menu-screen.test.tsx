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

  it("runs page-specific actions through the command palette", async () => {
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
    expect(app.lastFrame()).toContain("Tag Refinement Commands");

    for (const character of "legacy seed") {
      app.stdin.write(character);
    }
    await flushInk();
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
});
