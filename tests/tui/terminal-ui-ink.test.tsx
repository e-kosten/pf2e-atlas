import React from "react";
import { EventEmitter } from "node:events";

import { render as renderInkApp } from "ink";
import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  createDerivedTagTerminalInputEvent,
  createDerivedTagTerminalListNavigationState,
  type DerivedTagTerminalOptionalSelectPromptResult,
  type DerivedTagTerminalPolicySelection,
  type DerivedTagTerminalSelectPromptResult,
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  getDerivedTagTerminalListNavigationAction,
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

class SizedStdout extends EventEmitter {
  public readonly frames: string[] = [];
  public isTTY = true;
  public destroyed = false;
  public writableEnded = false;
  public writable = true;

  private lastWrittenFrame?: string;

  public constructor(
    private columnCount: number,
    private rowCount: number,
  ) {
    super();
  }

  get columns(): number {
    return this.columnCount;
  }

  get rows(): number {
    return this.rowCount;
  }

  public lastFrame(): string | undefined {
    return this.lastWrittenFrame;
  }

  public write(
    frame: string,
    encodingOrCallback?: string | ((error?: Error) => void),
    callback?: (error?: Error) => void,
  ): boolean {
    this.frames.push(frame);
    this.lastWrittenFrame = frame;
    const resolvedCallback = typeof encodingOrCallback === "function" ? encodingOrCallback : callback;
    resolvedCallback?.();
    return true;
  }

  public setSize(columns: number, rows: number): void {
    this.columnCount = columns;
    this.rowCount = rows;
    this.emit("resize");
  }
}

class SizedStderr extends EventEmitter {
  public readonly frames: string[] = [];
  public isTTY = true;
  public destroyed = false;
  public writableEnded = false;
  public writable = true;

  private lastWrittenFrame?: string;

  public lastFrame(): string | undefined {
    return this.lastWrittenFrame;
  }

  public write(
    frame: string,
    encodingOrCallback?: string | ((error?: Error) => void),
    callback?: (error?: Error) => void,
  ): boolean {
    this.frames.push(frame);
    this.lastWrittenFrame = frame;
    const resolvedCallback = typeof encodingOrCallback === "function" ? encodingOrCallback : callback;
    resolvedCallback?.();
    return true;
  }
}

class SizedStdin extends EventEmitter {
  public isTTY = true;
  private data: string | null = null;

  public write(nextData: string): void {
    this.data = nextData;
    this.emit("readable");
    this.emit("data", nextData);
  }

  public setEncoding(): void {}

  public setRawMode(): void {}

  public resume(): void {}

  public pause(): void {}

  public ref(): void {}

  public unref(): void {}

  public read(): string | null {
    const currentData = this.data;
    this.data = null;
    return currentData;
  }
}

type SizedInkInstance = {
  cleanup: () => void;
  frames: string[];
  lastFrame: () => string | undefined;
  rerender: (tree: React.ReactElement) => void;
  stderr: SizedStderr;
  stdin: SizedStdin;
  stdout: SizedStdout;
  unmount: () => void;
};

const sizedInkInstances: SizedInkInstance[] = [];

function renderWithTerminalSize(
  tree: React.ReactElement,
  { columns, rows }: { columns: number; rows: number },
): SizedInkInstance {
  const stdout = new SizedStdout(columns, rows);
  const stderr = new SizedStderr();
  const stdin = new SizedStdin();
  const inkInstance = renderInkApp(tree, {
    stdout: stdout as never,
    stderr: stderr as never,
    stdin: stdin as never,
    debug: true,
    exitOnCtrlC: false,
    patchConsole: false,
  });

  const instance: SizedInkInstance = {
    cleanup: () => {
      inkInstance.cleanup();
      const instanceIndex = sizedInkInstances.indexOf(instance);
      if (instanceIndex >= 0) {
        sizedInkInstances.splice(instanceIndex, 1);
      }
    },
    frames: stdout.frames,
    lastFrame: () => stdout.lastFrame(),
    rerender: inkInstance.rerender,
    stderr,
    stdin,
    stdout,
    unmount: inkInstance.unmount,
  };

  sizedInkInstances.push(instance);
  return instance;
}

function countFrameLinesContaining(frame: string, prefix: string): number {
  return frame
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith(prefix)).length;
}

function promptListLinePrefix(kind: LayoutPromptKind): string {
  return kind === "multiselect" || kind === "policy" ? "[" : "Option ";
}

function formatSelectResult<T>(result: DerivedTagTerminalSelectPromptResult<T>): string {
  return result.kind === "selected" ? String(result.value) : "cancelled";
}

function formatOptionalSelectResult<T>(result: DerivedTagTerminalOptionalSelectPromptResult<T>): string {
  if (result.kind === "selected") {
    return String(result.value);
  }
  return result.kind;
}

function SelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [appJPresses, setAppJPresses] = React.useState(0);
  const [result, setResult] = React.useState("pending");

  useDerivedTagTerminalInput((event) => {
    if (event.isExactPrintableKey("j")) {
      setAppJPresses((count) => count + 1);
    }
  });

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Harness Prompt",
        prompt: "Pick a value",
        entries: [
          { value: "first", label: "First" },
          { value: "second", label: "Second" },
        ],
      })
      .then((result) => {
        setResult(formatSelectResult(result));
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }, { text: `appJ=${appJPresses}` }]} />;
}

function TextPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptTextInput({
        title: "Text Harness",
        prompt: "Enter a short value",
        hint: "Quick inline prompt",
      })
      .then((value) => {
        setResult(value ?? "cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function DialogStateHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [page, setPage] = React.useState("home");

  useDerivedTagTerminalInput((event) => {
    if (event.isExactPrintableKey("d")) {
      setPage("detail");
      return;
    }
    if (event.isHelpKey()) {
      void terminal.showDialog({
        title: "Help",
        body: [{ text: "Press any key to return." }],
      });
    }
  });

  return <TerminalTextScreen title="Harness" body={[{ text: `page=${page}` }]} />;
}

function LongDialogHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();

  useDerivedTagTerminalInput((event) => {
    if (event.isHelpKey()) {
      void terminal.showDialog({
        title: "Long Help",
        body: Array.from({ length: 8 }, (_, index) => ({
          text: `Line ${index + 1}: detailed help content.`,
        })),
      });
    }
  });

  return <TerminalTextScreen title="Harness" body={[{ text: "page=home" }]} />;
}

function MultiSelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptMultiSelectOption({
        title: "Multi Harness",
        prompt: "Toggle values",
        entries: [
          { value: "common", label: "Common" },
          { value: "rare", label: "Rare" },
        ],
      })
      .then((values) => {
        setResult(values.join(",") || "empty");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function PolicyPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptPolicySelectOption({
        title: "Policy Harness",
        prompt: "Cycle values",
        allowedStates: ["any", "all", "exclude"],
        entries: [
          { value: "fire", label: "Fire" },
          { value: "cold", label: "Cold" },
        ],
      })
      .then((selection: DerivedTagTerminalPolicySelection<string>) => {
        setResult(
          `any=${selection.any.join(",") || "-"}|all=${selection.all.join(",") || "-"}|exclude=${selection.exclude.join(",") || "-"}`,
        );
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function LongSelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Long Harness",
        prompt: "Pick a longer-list value",
        entries: Array.from({ length: 12 }, (_, index) => ({
          value: `item-${index + 1}`,
          label: `Item ${index + 1}`,
        })),
      })
      .then((result) => {
        setResult(formatSelectResult(result));
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function OptionalSelectPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptOptionalSelectOption({
        title: "Optional Harness",
        prompt: "Pick a value or keep the full scope",
        allOption: {
          label: "All values",
          description: "Keep the full scope.",
        },
        entries: [
          { value: "first", label: "First" },
          { value: "second", label: "Second" },
        ],
      })
      .then((selection) => {
        setResult(formatOptionalSelectResult(selection));
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function CommandPaletteHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptCommandPalette({
        title: "Command Palette",
        prompt: "Filter commands",
        entries: [
          { value: "mode", label: "Mode", description: "Choose the search mode.", aliases: ["m"] },
          {
            value: "facet",
            label: "Edit Query Field",
            description: "Edit ontology-backed query field selections.",
            aliases: ["f"],
          },
        ],
      })
      .then((value) => {
        setResult(value ?? "cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function DisabledCommandPaletteHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptCommandPalette({
        title: "Command Palette",
        prompt: "Filter commands",
        entries: [
          {
            value: "disabled",
            label: "Disabled Command",
            description: "Run the currently unavailable command.",
            disabled: true,
            disabledReason: "Unavailable until the current scope changes.",
          },
          {
            value: "enabled",
            label: "Enabled Command",
            description: "Available right now.",
          },
        ],
      })
      .then((value) => {
        setResult(value ?? "cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function AutoDialogHarness({ title, bodyLineCount }: { title: string; bodyLineCount: number }): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();

  React.useEffect(() => {
    void terminal.showDialog({
      title,
      body: Array.from({ length: bodyLineCount }, (_, index) => ({
        text: `Dialog line ${index + 1}`,
      })),
    });
  }, [bodyLineCount, terminal, title]);

  return (
    <TerminalTextScreen
      title="Harness"
      body={Array.from({ length: 8 }, (_, index) => ({
        text: `base-${index + 1}`,
      }))}
    />
  );
}

type LayoutPromptKind = "command" | "select" | "multiselect" | "policy";

function ModalLayoutPromptHarness({
  kind,
  presentation,
  entryCount = 8,
}: {
  kind: LayoutPromptKind;
  presentation?: "inline" | "screen";
  entryCount?: number;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    const entries = Array.from({ length: entryCount }, (_, index) => ({
      value: `option-${index + 1}`,
      label: `Option ${index + 1}`,
      description: `Description ${index + 1}`,
      detailLines: [{ text: `Detail ${index + 1}` }],
    }));

    switch (kind) {
      case "command":
        void terminal
          .promptCommandPalette({
            title: "Command Layout",
            prompt: "Choose an option",
            presentation,
            entries: entries.map((entry) => ({
              value: entry.value,
              label: entry.label,
              description: entry.description,
              detailLines: entry.detailLines,
            })),
          })
          .then((value) => {
            setResult(value ?? "cancelled");
          });
        break;
      case "select":
        void terminal
          .promptSelectOption({
            title: "Select Layout",
            prompt: "Choose an option",
            presentation,
            entries,
          })
          .then((selection) => {
            setResult(formatSelectResult(selection));
          });
        break;
      case "multiselect":
        void terminal
          .promptMultiSelectOption({
            title: "Multi Layout",
            prompt: "Choose options",
            presentation,
            entries,
          })
          .then((values) => {
            setResult(values.join(",") || "empty");
          });
        break;
      case "policy":
        void terminal
          .promptPolicySelectOption({
            title: "Policy Layout",
            prompt: "Choose policies",
            presentation,
            allowedStates: ["any", "all", "exclude"],
            entries,
          })
          .then((selection) => {
            setResult(
              `any=${selection.any.join(",") || "-"}|all=${selection.all.join(",") || "-"}|exclude=${selection.exclude.join(",") || "-"}`,
            );
          });
        break;
    }
  }, [entryCount, kind, presentation, terminal]);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[
        { text: `kind=${kind}` },
        { text: `result=${result}` },
        { text: "base-1" },
        { text: "base-2" },
        { text: "base-3" },
        { text: "base-4" },
      ]}
    />
  );
}

describe("derived tag terminal ink runtime", () => {
  afterEach(() => {
    cleanup();
    while (sizedInkInstances.length > 0) {
      const instance = sizedInkInstances.pop();
      instance?.unmount();
      instance?.cleanup();
    }
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

  it("keeps short dialogs inline on large terminals", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <AutoDialogHarness title="Short Help" bodyLineCount={2} />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 24 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Short Help");
    expect(app.lastFrame()).toContain("base-1");
    expect(app.lastFrame()).toContain("base-2");
    expect(app.lastFrame()).toContain("base-3");
  });

  it("switches long dialogs to screen presentation when inline would crowd out the screen", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <AutoDialogHarness title="Deep Help" bodyLineCount={16} />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 12 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Deep Help");
    expect(app.lastFrame()).toContain("Dialog line 1");
    expect(app.lastFrame()).toContain("Press any key to continue.");
    expect(app.lastFrame()).not.toContain("base-1");
  });

  it("preserves a usable amount of the underlying screen for inline dialogs", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <AutoDialogHarness title="Inline Help" bodyLineCount={3} />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 14 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Inline Help");
    expect(app.lastFrame()).toContain("base-1");
    expect(app.lastFrame()).toContain("base-2");
    expect(app.lastFrame()).toContain("base-3");
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

  it("supports typed all-selections without sentinel values", async () => {
    const allApp = render(
      <DerivedTagTerminalProvider>
        <OptionalSelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(allApp.lastFrame()).toContain("Pick a value or keep the full scope");

    allApp.stdin.write("\r");
    await flushInkFrames();
    expect(allApp.lastFrame()).toContain("result=all");

    allApp.unmount();

    const selectedApp = render(
      <DerivedTagTerminalProvider>
        <OptionalSelectPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    selectedApp.stdin.write("j");
    await flushInkFrames();
    selectedApp.stdin.write("\r");
    await flushInkFrames();
    expect(selectedApp.lastFrame()).toContain("result=first");
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
    expect(app.lastFrame()).toContain("Edit Query Field");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=facet");
  });

  it("uses the same inline list-capacity planning across command and select-like prompts", async () => {
    const promptKinds: LayoutPromptKind[] = ["command", "select", "multiselect", "policy"];
    const visibleCounts: number[] = [];

    for (const kind of promptKinds) {
      const app = renderWithTerminalSize(
        <DerivedTagTerminalProvider>
          <ModalLayoutPromptHarness kind={kind} presentation="inline" entryCount={8} />
        </DerivedTagTerminalProvider>,
        { columns: 100, rows: 24 },
      );

      await flushInkFrames(4);
      const frame = app.lastFrame() ?? "";
      visibleCounts.push(countFrameLinesContaining(frame, promptListLinePrefix(kind)));
      expect(frame).toContain("base-1");
      app.unmount();
      app.cleanup();
    }

    expect(visibleCounts.every((count) => count >= 5)).toBe(true);
    expect(new Set(visibleCounts)).toEqual(new Set([visibleCounts[0]!]));
  });

  it("lets select, multiselect, and policy prompts grow beyond the legacy fixed inline height", async () => {
    const promptKinds: Array<Exclude<LayoutPromptKind, "command">> = ["select", "multiselect", "policy"];

    for (const kind of promptKinds) {
      const app = renderWithTerminalSize(
        <DerivedTagTerminalProvider>
          <ModalLayoutPromptHarness kind={kind} presentation="inline" entryCount={8} />
        </DerivedTagTerminalProvider>,
        { columns: 100, rows: 24 },
      );

      await flushInkFrames(4);
      const frame = app.lastFrame() ?? "";
      const visibleCount = countFrameLinesContaining(frame, promptListLinePrefix(kind));
      expect(visibleCount).toBeGreaterThanOrEqual(5);
      app.unmount();
      app.cleanup();
    }
  });

  it("collapses forced-inline choice prompts instead of rendering a cramped two-pane split on narrow terminals", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <ModalLayoutPromptHarness kind="select" presentation="inline" entryCount={6} />
      </DerivedTagTerminalProvider>,
      { columns: 42, rows: 20 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Select Layout");
    expect(app.lastFrame()).toContain("Option 1");
    expect(app.lastFrame()).not.toContain("│");
  });

  it("prefers the first enabled command when a palette opens", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <DisabledCommandPaletteHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Enabled Command");
    expect(app.lastFrame()).toContain("Command 2/2");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=enabled");
  });

  it("keeps disabled commands visible but does not dispatch them", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <DisabledCommandPaletteHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();

    for (const character of "disabled") {
      app.stdin.write(character);
    }
    await flushInkFrames();
    expect(app.lastFrame()).toContain("Disabled Command | unavailable");
    expect(app.lastFrame()).toContain("Unavailable: Unavailable until the current scope changes.");
    expect(app.lastFrame()).not.toContain("This command is currently unavailable.");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("Command Palette");
    expect(app.lastFrame()).toContain("result=pending");
  });

  it("normalizes ctrl letter combinations from both Ink key paths", () => {
    expect(createDerivedTagTerminalInputEvent("\r", {} as never).textInputAction).toBe("submit");
    expect(createDerivedTagTerminalInputEvent("\u001b", {} as never).textInputAction).toBe("cancel");
    expect(createDerivedTagTerminalInputEvent("\u0015", {} as never).isTerminalJumpBackwardKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("d", { ctrl: true } as never).printable).toBeUndefined();
  });

  it("prefers Ink arrow-key flags over raw escape input", () => {
    expect(createDerivedTagTerminalInputEvent("\u001b", { rightArrow: true } as never).isMoveRightKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b", { leftArrow: true } as never).isMoveLeftKey()).toBe(true);
  });

  it("normalizes raw ANSI escape sequences for common navigation keys", () => {
    expect(createDerivedTagTerminalInputEvent("\u001b[A", {} as never).isMoveUpKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b[B", {} as never).isMoveDownKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b[C", {} as never).isMoveRightKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b[D", {} as never).isMoveLeftKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b[5~", {} as never).isPageUpKey()).toBe(true);
    expect(createDerivedTagTerminalInputEvent("\u001b[6~", {} as never).isPageDownKey()).toBe(true);
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
    expect(
      getDerivedTagTerminalListNavigationAction(createDerivedTagTerminalInputEvent("l", {} as never), {
        pageSize: 10,
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      }),
    ).toEqual({ kind: "confirm" });
    expect(
      getDerivedTagTerminalListNavigationAction(createDerivedTagTerminalInputEvent("h", {} as never), {
        pageSize: 10,
        includeCancelKeys: true,
        includeHorizontalCancelKeys: true,
      }),
    ).toEqual({ kind: "cancel" });
  });

  it("keeps space available for selection while using f for shared page-down navigation", () => {
    expect(
      getDerivedTagTerminalListNavigationAction(createDerivedTagTerminalInputEvent(" ", {} as never), {
        pageSize: 10,
      }),
    ).toBeUndefined();
    expect(
      getDerivedTagTerminalListNavigationAction(createDerivedTagTerminalInputEvent("f", {} as never), {
        pageSize: 10,
      }),
    ).toEqual({ kind: "move", delta: 10 });
  });

  it("resolves shared gg and G list-boundary navigation", () => {
    const options = { pageSize: 10 };

    const firstG = resolveDerivedTagTerminalListNavigationAction(
      createDerivedTagTerminalInputEvent("g", {} as never),
      options,
    );
    expect(firstG.action).toBeUndefined();
    expect(firstG.state.pendingBoundaryPrefix).toBe("g");

    const secondG = resolveDerivedTagTerminalListNavigationAction(
      createDerivedTagTerminalInputEvent("g", {} as never),
      options,
      firstG.state,
    );
    expect(secondG.action).toEqual({ kind: "boundary", boundary: "start" });
    expect(secondG.state).toEqual(createDerivedTagTerminalListNavigationState());

    const upperG = resolveDerivedTagTerminalListNavigationAction(
      createDerivedTagTerminalInputEvent("G", {} as never),
      options,
    );
    expect(upperG.action).toEqual({ kind: "boundary", boundary: "end" });
    expect(upperG.state).toEqual(createDerivedTagTerminalListNavigationState());
  });
});
