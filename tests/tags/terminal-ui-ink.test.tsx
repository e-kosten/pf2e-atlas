import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  getNormalizedKeyName,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
} from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
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
  }, [terminal]);

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

    app.stdin.write("j");
    await flushInk();
    expect(app.lastFrame()).toContain("Selected: First");
    expect(app.lastFrame()).not.toContain("appJ=1");
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

    app.stdin.write("x");
    await flushInk();
    expect(app.lastFrame()).toContain("page=detail");
  });

  it("normalizes ctrl letter combinations from both Ink key paths", () => {
    expect(getNormalizedKeyName("\u0015", {} as never)).toBe("ctrl_u");
    expect(getNormalizedKeyName("d", { ctrl: true } as never)).toBe("ctrl_d");
  });
});
