import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
} from "../../src/tags/migration/terminal-ui.js";

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

describe("derived tag terminal ink runtime", () => {
  afterEach(() => {
    cleanup();
  });

  it("routes modal select input through the prompt and suspends screen handlers", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Pick a value");

    app.stdin.write("j");
    await flushInk();
    expect(app.lastFrame()).toContain("Selected: Second");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("result=second");
    expect(app.lastFrame()).toContain("appJ=0");
  });
});
