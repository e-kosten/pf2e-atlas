import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  createDerivedTagTerminalListNavigationState,
  type DerivedTagTerminalPolicySelection,
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  getDerivedTagTerminalListNavigationAction,
  getNormalizedKeyName,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
} from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

async function flushInkFrames(count = 2): Promise<void> {
  for (let index = 0; index < count; index += 1) {
    await flushInk();
  }
}

function SelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [appJPresses, setAppJPresses] = React.useState(0);
  const [result, setResult] = React.useState("pending");

  useDerivedTagTerminalInput((input) => {
    if (input === "j") {
      setAppJPresses((count) => count + 1);
    }
  });

  React.useEffect(() => {
    void terminal.promptSelectOption({
      title: "Harness Prompt",
      prompt: "Pick a value",
      entries: [
        { value: "first", label: "First" },
        { value: "second", label: "Second" },
      ],
    }).then((value) => {
      setResult(value ?? "cancelled");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[
        { text: `result=${result}` },
        { text: `appJ=${appJPresses}` },
      ]}
    />
  );
}

function TextPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.promptTextInput({
      title: "Text Harness",
      prompt: "Enter a short value",
      hint: "Quick inline prompt",
    }).then((value) => {
      setResult(value ?? "cancelled");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `result=${result}` }]}
    />
  );
}

function DialogStateHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [page, setPage] = React.useState("home");

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);

    if (normalized === "d") {
      setPage("detail");
      return;
    }
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Help",
        body: [{ text: "Press any key to return." }],
      });
    }
  });

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `page=${page}` }]}
    />
  );
}

function LongDialogHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Long Help",
        body: Array.from({ length: 8 }, (_, index) => ({
          text: `Line ${index + 1}: detailed help content.`,
        })),
      });
    }
  });

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: "page=home" }]}
    />
  );
}

function MultiSelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.promptMultiSelectOption({
      title: "Multi Harness",
      prompt: "Toggle values",
      entries: [
        { value: "common", label: "Common" },
        { value: "rare", label: "Rare" },
      ],
    }).then((values) => {
      setResult(values.join(",") || "empty");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `result=${result}` }]}
    />
  );
}

function PolicyPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.promptPolicySelectOption({
      title: "Policy Harness",
      prompt: "Cycle values",
      allowedStates: ["any", "all", "exclude"],
      entries: [
        { value: "fire", label: "Fire" },
        { value: "cold", label: "Cold" },
      ],
    }).then((selection: DerivedTagTerminalPolicySelection<string>) => {
      setResult(`any=${selection.any.join(",") || "-"}|all=${selection.all.join(",") || "-"}|exclude=${selection.exclude.join(",") || "-"}`);
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `result=${result}` }]}
    />
  );
}

function LongSelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.promptSelectOption({
      title: "Long Harness",
      prompt: "Pick a longer-list value",
      entries: Array.from({ length: 12 }, (_, index) => ({
        value: `item-${index + 1}`,
        label: `Item ${index + 1}`,
      })),
    }).then((value) => {
      setResult(value ?? "cancelled");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `result=${result}` }]}
    />
  );
}

function CommandPaletteHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.promptCommandPalette({
      title: "Command Palette",
      prompt: "Filter commands",
      entries: [
        { value: "mode", label: "Mode", description: "Choose the search mode.", aliases: ["m"] },
        { value: "facet", label: "Edit Facet Filter", description: "Edit ontology-backed facet filters.", aliases: ["f"] },
      ],
    }).then((value) => {
      setResult(value ?? "cancelled");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `result=${result}` }]}
    />
  );
}

describe("derived tag terminal ink runtime", () => {
  afterEach(() => {
    cleanup();
  });

  it("suspends screen handlers while a select prompt is active", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Pick a value");
    expect(app.lastFrame()).not.toContain("result=pending");
    expect(app.lastFrame()).not.toContain("appJ=0");

    app.stdin.write("j");
    await flushInk();
    expect(app.lastFrame()).toContain("Selected: First");
    expect(app.lastFrame()).not.toContain("appJ=1");
  });

  it("keeps quick text prompts inline with the current screen", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <TextPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Enter a short value");
    expect(app.lastFrame()).toContain("Quick inline prompt");
    expect(app.lastFrame()).toContain("result=pending");
    expect(app.lastFrame()).toContain("Harness");
  });

  it("preserves screen state after closing a dialog modal", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <DialogStateHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("page=home");

    app.stdin.write("d");
    await flushInk();
    expect(app.lastFrame()).toContain("page=detail");

    app.stdin.write("?");
    await flushInk();
    expect(app.lastFrame()).toContain("Help");
    expect(app.lastFrame()).toContain("page=detail");

    app.stdin.write("x");
    await flushInk();
    expect(app.lastFrame()).toContain("page=detail");
  });

  it("grows inline dialogs to fit longer help content", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <LongDialogHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    app.stdin.write("?");
    await flushInkFrames();

    expect(app.lastFrame()).toContain("Long Help");
    expect(app.lastFrame()).toContain("Line 1: detailed help content.");
    expect(app.lastFrame()).toContain("Line 8: detailed help content.");
    expect(app.lastFrame()).toContain("page=home");
  });

  it("accumulates multiselect choices until an exit key closes the prompt", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <MultiSelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Toggle values");
    expect(app.lastFrame()).not.toContain("result=pending");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[x] Common");

    app.stdin.write("j");
    await flushInkFrames();
    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[x] Rare");

    app.stdin.write("\u007f");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=common,rare");
  });

  it("cycles policy states in a single prompt view", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <PolicyPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Cycle values");
    expect(app.lastFrame()).not.toContain("result=pending");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[ANY] Fire");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[ALL] Fire");

    app.stdin.write("j");
    await flushInkFrames();
    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[ANY] Cold");

    app.stdin.write("\u007f");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=any=cold|all=fire|exclude=-");
  });

  it("supports shared jump navigation inside select prompts", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <LongSelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Pick a longer-list value");

    app.stdin.write("\u0004");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("Selected: Item 6");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=item-6");
  });

  it("filters and selects commands through the shared command palette", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <CommandPaletteHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Command Palette");
    expect(app.lastFrame()).toContain("Mode");
    expect(app.lastFrame()).toContain("Enter/→ select");

    app.stdin.write("f");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("Filter: f");
    expect(app.lastFrame()).toContain("Edit Facet Filter");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=facet");
  });

  it("normalizes ctrl letter combinations from both Ink key paths", () => {
    expect(getNormalizedKeyName("\r", {} as never)).toBe("enter");
    expect(getNormalizedKeyName("\u001b", {} as never)).toBe("escape");
    expect(getNormalizedKeyName("\u0015", {} as never)).toBe("ctrl_u");
    expect(getNormalizedKeyName("d", { ctrl: true } as never)).toBe("ctrl_d");
  });

  it("prefers Ink arrow-key flags over raw escape input", () => {
    expect(getNormalizedKeyName("\u001b", { rightArrow: true } as never)).toBe("right");
    expect(getNormalizedKeyName("\u001b", { leftArrow: true } as never)).toBe("left");
  });

  it("normalizes raw ANSI escape sequences for common navigation keys", () => {
    expect(getNormalizedKeyName("\u001b[A", {} as never)).toBe("up");
    expect(getNormalizedKeyName("\u001b[B", {} as never)).toBe("down");
    expect(getNormalizedKeyName("\u001b[C", {} as never)).toBe("right");
    expect(getNormalizedKeyName("\u001b[D", {} as never)).toBe("left");
    expect(getNormalizedKeyName("\u001b[5~", {} as never)).toBe("page_up");
    expect(getNormalizedKeyName("\u001b[6~", {} as never)).toBe("page_down");
  });

  it("supports raw right-select and raw left-return inside select prompts", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    app.stdin.write("\u001b[C");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=first");

    app.unmount();

    const cancelApp = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    cancelApp.stdin.write("\u001b[D");
    await flushInkFrames();
    expect(cancelApp.lastFrame()).toContain("result=cancelled");
  });

  it("treats vim horizontal keys as the same confirm and cancel semantics", async () => {
    const confirmApp = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    confirmApp.stdin.write("l");
    await flushInkFrames();
    expect(confirmApp.lastFrame()).toContain("result=first");

    confirmApp.unmount();

    const cancelApp = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    cancelApp.stdin.write("h");
    await flushInkFrames();
    expect(cancelApp.lastFrame()).toContain("result=cancelled");
  });

  it("treats vim horizontal keys as the same list-navigation semantics", () => {
    expect(getDerivedTagTerminalListNavigationAction("l", {
      pageSize: 10,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    })).toEqual({ kind: "confirm" });
    expect(getDerivedTagTerminalListNavigationAction("h", {
      pageSize: 10,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    })).toEqual({ kind: "cancel" });
  });

  it("keeps space available for selection while using f for shared page-down navigation", () => {
    expect(getDerivedTagTerminalListNavigationAction("space", {
      pageSize: 10,
    })).toBeUndefined();
    expect(getDerivedTagTerminalListNavigationAction("f", {
      pageSize: 10,
    })).toEqual({ kind: "move", delta: 10 });
  });

  it("resolves shared gg and G list-boundary navigation", () => {
    const options = { pageSize: 10 };

    const firstG = resolveDerivedTagTerminalListNavigationAction("g", {} as never, options);
    expect(firstG.action).toBeUndefined();
    expect(firstG.state.pendingBoundaryPrefix).toBe("g");

    const secondG = resolveDerivedTagTerminalListNavigationAction("g", {} as never, options, firstG.state);
    expect(secondG.action).toEqual({ kind: "boundary", boundary: "start" });
    expect(secondG.state).toEqual(createDerivedTagTerminalListNavigationState());

    const upperG = resolveDerivedTagTerminalListNavigationAction("G", {} as never, options);
    expect(upperG.action).toEqual({ kind: "boundary", boundary: "end" });
    expect(upperG.state).toEqual(createDerivedTagTerminalListNavigationState());
  });
});
