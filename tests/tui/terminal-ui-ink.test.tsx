import React from "react";
import { EventEmitter } from "node:events";

import { render as renderInkApp } from "ink";
import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it } from "vitest";

import {
  createDerivedTagTerminalInputEvent,
  createDerivedTagTerminalListNavigationState,
  type DerivedTagTerminalOptionalSelectPromptResult,
  type DerivedTagTerminalSelectPromptResult,
  DerivedTagTerminalProvider,
  TerminalTextScreen,
  getDerivedTagTerminalListNavigationAction,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
} from "../../src/tui/terminal-ui.js";
import {
  useDerivedTagTerminalBackdropActive,
  useDerivedTagTerminalViewportSize,
} from "../../src/tui/framework/context.js";
import { TerminalMenuScreen } from "../../src/tui/shared-screens.js";

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

function findFrameLine(frame: string, snippet: string): number {
  return frame.split("\n").findIndex((line) => line.includes(snippet));
}

function promptListLinePrefix(kind: LayoutPromptKind): string {
  return kind === "multiselect" ? "[" : "Option ";
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

function CenteredModePromptHarness({
  presentation = "overlay",
}: {
  presentation?: "overlay" | "blanked";
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const backdropActive = useDerivedTagTerminalBackdropActive();
  const size = useDerivedTagTerminalSize();
  const viewportSize = useDerivedTagTerminalViewportSize();
  const [result, setResult] = React.useState("pending");
  const [modalSnapshot, setModalSnapshot] = React.useState<{ layoutHeight: number; viewportHeight: number } | null>(null);

  React.useEffect(() => {
    if (!terminal.modalActive) {
      return;
    }
    setModalSnapshot((current) => current ?? { layoutHeight: size.height, viewportHeight: viewportSize.height });
  }, [size.height, terminal.modalActive, viewportSize.height]);

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Choose Search Mode",
        prompt: "",
        presentation,
        choiceLayout: "horizontal",
        filtering: false,
        entries: [
          { value: "browse", label: "Browse", description: "Explore records without text search." },
          { value: "search", label: "Search", description: "Search named records and ranked text matches." },
          { value: "lookup", label: "Lookup", description: "Jump straight to exact or fuzzy name lookup." },
        ],
      })
      .then((selection) => {
        setResult(formatSelectResult(selection));
      });
  }, [presentation]);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[
        { text: "bg=visible" },
        { text: `bgHeight=${size.height}` },
        { text: `viewportHeight=${viewportSize.height}` },
        { text: `modalLayoutHeight=${modalSnapshot?.layoutHeight ?? "pending"}` },
        { text: `modalViewportHeight=${modalSnapshot?.viewportHeight ?? "pending"}` },
        { text: `backdrop=${String(backdropActive)}` },
        { text: `result=${result}` },
      ]}
    />
  );
}

function OverlayTransparencyHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();

  React.useEffect(() => {
    void terminal.promptSelectOption({
      title: "Choose Search Mode",
      prompt: "",
      presentation: "overlay",
      choiceLayout: "horizontal",
      filtering: false,
      entries: [
        { value: "browse", label: "Browse", description: "Explore records without text search." },
        { value: "search", label: "Search", description: "Search named records and ranked text matches." },
        { value: "lookup", label: "Lookup", description: "Jump straight to exact or fuzzy name lookup." },
      ],
    });
  }, [terminal]);

  return (
    <TerminalTextScreen
      title="Backdrop Harness"
      body={Array.from({ length: 16 }, (_, index) => ({
        text: `L${String(index).padStart(2, "0")} ${"·".repeat(84)} R${String(index).padStart(2, "0")}`,
        noWrap: true,
      }))}
    />
  );
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

function SessionPreemptedBySharedPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [sessionResult, setSessionResult] = React.useState("pending");
  const [sharedResult, setSharedResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.runPromptSession(async (session) => {
      const result = await session.promptSelectOption({
        title: "Session Prompt",
        prompt: "Choose a session-owned value",
        entries: [
          { value: "first", label: "First" },
          { value: "second", label: "Second" },
        ],
      });
      setSessionResult(formatSelectResult(result));
    });

    const timer = setTimeout(() => {
      void terminal
        .promptTextInput({
          title: "Shared Prompt",
          prompt: "Enter a shared value",
        })
        .then((value) => {
          setSharedResult(value ?? "cancelled");
        });
    }, 0);

    return () => {
      clearTimeout(timer);
    };
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `session=${sessionResult}` }, { text: `shared=${sharedResult}` }]}
    />
  );
}

function SessionSelfPreemptionHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [firstResult, setFirstResult] = React.useState("pending");
  const [secondResult, setSecondResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal.runPromptSession(async (session) => {
      const firstPrompt = session.promptSelectOption({
        title: "First Session Prompt",
        prompt: "Choose the first session value",
        entries: [
          { value: "alpha", label: "Alpha" },
          { value: "beta", label: "Beta" },
        ],
      });

      await new Promise<void>((resolve) => {
        setTimeout(resolve, 0);
      });

      const second = await session.promptTextInput({
        title: "Second Session Prompt",
        prompt: "Enter the second session value",
      });
      const first = await firstPrompt;
      setFirstResult(formatSelectResult(first));
      setSecondResult(second ?? "cancelled");
    });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[{ text: `first=${firstResult}` }, { text: `second=${secondResult}` }]}
    />
  );
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
      .then((selection) => {
        if (selection.kind === "selected") {
          setResult(selection.values.join(",") || "empty");
          return;
        }
        if (selection.kind === "commands") {
          setResult("commands");
          return;
        }
        setResult("cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function FilterableChoicePromptHarness({
  kind,
}: {
  kind: "select" | "multiselect";
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");
  const entries = React.useMemo(
    () => [
      { value: "common", label: "Common", description: "Common rarity" },
      { value: "rare", label: "Rare", description: "Rare rarity" },
      { value: "cold", label: "Cold", description: "Cold damage" },
    ],
    [],
  );

  React.useEffect(() => {
    if (kind === "select") {
      void terminal
        .promptSelectOption({
          title: "Filterable Select",
          prompt: "Choose a value",
          entries,
        })
        .then((selection) => setResult(formatSelectResult(selection)));
      return;
    }
    if (kind === "multiselect") {
      void terminal
        .promptMultiSelectOption({
          title: "Filterable Multi",
          prompt: "Choose values",
          entries,
        })
        .then((selection) => {
          if (selection.kind === "selected") {
            setResult(selection.values.join(",") || "empty");
            return;
          }
          if (selection.kind === "commands") {
            setResult("commands");
            return;
          }
          setResult("cancelled");
        });
      return;
    }
  }, [entries, kind]);

  return <TerminalTextScreen title="Harness" body={[{ text: `kind=${kind}` }, { text: `result=${result}` }]} />;
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

function SelectThenCommandPaletteHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        supportsCommands: true,
        entries: [
          { value: "field", label: "Metadata", description: "Choose a metadata field." },
          { value: "pack", label: "Pack", description: "Choose one or more packs." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind === "commands") {
          setResult("command-requested");
          return;
        }
        setResult(`select=${selection.kind === "selected" ? selection.value : "cancelled"}`);
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function SelectThenMultiSelectHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        entries: [
          { value: "pack", label: "Pack", description: "Choose one or more packs." },
          { value: "scope", label: "Scope", description: "Choose a category scope." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("select=cancelled");
          return;
        }

        const multiSelectResult = await terminal.promptMultiSelectOption({
          title: "Pack Picker",
          prompt: "Choose packs",
          entries: [
            { value: "pathfinder-npc-core", label: "Pathfinder NPC Core" },
            { value: "monster-core", label: "Monster Core" },
          ],
        });

        if (multiSelectResult.kind === "selected") {
          setResult(`packs=${multiSelectResult.values.join(",") || "empty"}`);
          return;
        }
        if (multiSelectResult.kind === "commands") {
          setResult("packs=commands");
          return;
        }
        setResult("packs=cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function SelectThenSelectHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        entries: [
          { value: "metricCompare", label: "Metric comparison", description: "Compare two numeric metrics." },
          { value: "scope", label: "Scope", description: "Choose a category scope." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("clause=cancelled");
          return;
        }

        const metricSelection = await terminal.promptSelectOption({
          title: "Left Metric",
          prompt: "Choose the left-hand metric",
          supportsCommands: true,
          entries: [
            { value: "hp.value", label: "hp.value", description: "Creature hit points." },
            { value: "ac.value", label: "ac.value", description: "Creature armor class." },
          ],
        });
        if (metricSelection.kind === "selected") {
          setResult(`metric=${metricSelection.value}`);
          return;
        }
        if (metricSelection.kind === "commands") {
          setResult("metric=commands");
          return;
        }
        setResult("metric=cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function OverlaySelectThenSelectHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const backdropActive = useDerivedTagTerminalBackdropActive();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        presentation: "overlay",
        entries: [
          { value: "metricCompare", label: "Metric comparison", description: "Compare two numeric metrics." },
          { value: "scope", label: "Scope", description: "Choose a category scope." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("clause=cancelled");
          return;
        }

        const metricSelection = await terminal.promptSelectOption({
          title: "Left Metric",
          prompt: "Choose the left-hand metric",
          presentation: "overlay",
          entries: [
            { value: "hp.value", label: "hp.value", description: "Creature hit points." },
            { value: "ac.value", label: "ac.value", description: "Creature armor class." },
          ],
        });
        if (metricSelection.kind === "selected") {
          setResult(`metric=${metricSelection.value}`);
          return;
        }
        setResult("metric=cancelled");
      });
  }, []);

  return (
    <TerminalTextScreen
      title="Harness"
      body={[
        { text: "bg=chain" },
        { text: `backdrop=${String(backdropActive)}` },
        { text: `result=${result}` },
      ]}
    />
  );
}

function SelectThenSelectThenCommandsHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        entries: [
          { value: "metricCompare", label: "Metric comparison", description: "Compare two numeric metrics." },
          { value: "pack", label: "Pack", description: "Choose one or more packs." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("select=cancelled");
          return;
        }

        const secondSelection = await terminal.promptSelectOption({
          title: "Left Metric",
          prompt: "Choose the left-hand metric",
          presentation: "screen",
          supportsCommands: true,
          entries: [
            { value: "hp.value", label: "hp.value", description: "2 matching canonical records." },
            { value: "ac.value", label: "ac.value", description: "1 matching canonical record." },
          ],
        });

        if (secondSelection.kind === "commands") {
          setResult("command-requested");
          return;
        }

        setResult(`select=${secondSelection.kind === "selected" ? secondSelection.value : "cancelled"}`);
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function SelectThenMultiSelectThenCommandsHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [result, setResult] = React.useState("pending");

  React.useEffect(() => {
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        entries: [
          { value: "pack", label: "Pack", description: "Choose one or more packs." },
          { value: "scope", label: "Scope", description: "Choose a category scope." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("select=cancelled");
          return;
        }

        const multiSelectResult = await terminal.promptMultiSelectOption({
          title: "Pack",
          prompt: "Choose packs",
          presentation: "screen",
          supportsCommands: true,
          entries: [
            { value: "pathfinder-npc-core", label: "Pathfinder NPC Core" },
            { value: "monster-core", label: "Monster Core" },
          ],
        });

        if (multiSelectResult.kind === "commands") {
          setResult("command-requested");
          return;
        }
        if (multiSelectResult.kind === "selected") {
          setResult(`packs=${multiSelectResult.values.join(",") || "empty"}`);
          return;
        }
        setResult("packs=cancelled");
      });
  }, []);

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }]} />;
}

function EventDrivenSelectChainHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [started, setStarted] = React.useState(false);
  const [result, setResult] = React.useState("pending");

  useDerivedTagTerminalInput((event) => {
    if (started || !event.isConfirmKey()) {
      return;
    }

    setStarted(true);
    void terminal
      .promptSelectOption({
        title: "Clause Picker",
        prompt: "Choose a clause kind",
        entries: [
          { value: "metricCompare", label: "Metric comparison", description: "Compare two numeric metrics." },
          { value: "pack", label: "Pack", description: "Choose one or more packs." },
        ],
      })
      .then(async (selection) => {
        if (selection.kind !== "selected") {
          setResult("select=cancelled");
          return;
        }

        const nextSelection = await terminal.promptSelectOption({
          title: "Left Metric",
          prompt: "Choose the left-hand metric",
          supportsCommands: true,
          entries: [
            { value: "hp.value", label: "hp.value", description: "2 matching canonical records." },
            { value: "ac.value", label: "ac.value", description: "1 matching canonical record." },
          ],
        });

        if (nextSelection.kind === "commands") {
          setResult("command-requested");
          return;
        }

        setResult(`select=${nextSelection.kind === "selected" ? nextSelection.value : "cancelled"}`);
      });
  });

  return <TerminalTextScreen title="Harness" body={[{ text: `result=${result}` }, { text: "Press Enter to start" }]} />;
}

function MenuScreenDrivenPromptHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const [result, setResult] = React.useState("pending");

  const items = [{ label: "Open Prompt Chain" }];

  return (
    <TerminalMenuScreen
      title="Harness"
      subtitle="Menu-driven prompt chain"
      leftTitle="Menu"
      rightTitle="Detail"
      items={items}
      selectedIndex={selectedIndex}
      interactionActions={[{ id: "select" }, { id: "back", label: "back" }]}
      status={{ text: `result=${result}` }}
      helpTitle="Help"
      helpBody={[{ text: "Help" }]}
      buildDetailLines={() => [{ text: "Press Enter to start" }]}
      onMove={(delta) => setSelectedIndex((current) => Math.max(0, Math.min(current + delta, items.length - 1)))}
      onBack={() => setResult("back")}
      onSelect={() => {
        void terminal
          .promptSelectOption({
            title: "Clause Picker",
            prompt: "Choose a clause kind",
            entries: [
              { value: "metricCompare", label: "Metric comparison", description: "Compare two numeric metrics." },
              { value: "pack", label: "Pack", description: "Choose one or more packs." },
            ],
          })
          .then(async (selection) => {
            if (selection.kind !== "selected") {
              setResult("select=cancelled");
              return;
            }

            const secondSelection = await terminal.promptSelectOption({
              title: "Left Metric",
              prompt: "Choose the left-hand metric",
              presentation: "screen",
              supportsCommands: true,
              entries: [
                { value: "hp.value", label: "hp.value", description: "2 matching canonical records." },
                { value: "ac.value", label: "ac.value", description: "1 matching canonical record." },
              ],
            });

            if (secondSelection.kind === "commands") {
              setResult("command-requested");
              return;
            }

            setResult(`select=${secondSelection.kind === "selected" ? secondSelection.value : "cancelled"}`);
          });
      }}
    />
  );
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

function HyperlinkHarness(): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();

  return (
    <TerminalTextScreen
      title="Harness"
      body={[
        { text: `support=${terminal.capabilities.hyperlinkSupport}` },
        { text: "Archives of Nethys", href: "https://2e.aonprd.com/" },
        {
          text: "",
          noWrap: true,
          segments: [
            { text: "Visit ", tone: "default" },
            { text: "AoN", tone: "accent", href: "https://2e.aonprd.com/" },
          ],
        },
      ]}
    />
  );
}

type LayoutPromptKind = "select" | "multiselect";

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
          .then((selection) => {
            if (selection.kind === "selected") {
              setResult(selection.values.join(",") || "empty");
              return;
            }
            if (selection.kind === "commands") {
              setResult("commands");
              return;
            }
            setResult("cancelled");
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

  it("resolves a session-owned prompt when a shared prompt preempts it", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SessionPreemptedBySharedPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Shared Prompt");
    expect(app.lastFrame()).not.toContain("Session Prompt");

    app.stdin.write("\r");
    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("session=cancelled");
    expect(app.lastFrame()).toContain("shared=cancelled");
  });

  it("lets one owned session deterministically preempt its own earlier prompt", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SessionSelfPreemptionHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Second Session Prompt");
    expect(app.lastFrame()).not.toContain("First Session Prompt");

    app.stdin.write("\r");
    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("first=cancelled");
    expect(app.lastFrame()).toContain("second=cancelled");
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

  it("emits OSC 8 hyperlinks when the provider reports support", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider hyperlinkSupport="supported">
        <HyperlinkHarness />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 12 },
    );

    await flushInkFrames(4);
    const frame = app.lastFrame() ?? "";
    expect(frame).toContain("support=supported");
    expect(frame).toContain("\u001b]8;;https://2e.aonprd.com/\u001b\\");
    expect(frame).toContain("\u001b]8;;\u001b\\");
    expect(frame).toContain("Archives of Nethys");
    expect(frame).toContain("AoN");
  });

  it("falls back to plain text when hyperlink support is unavailable", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider hyperlinkSupport="unsupported">
        <HyperlinkHarness />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 12 },
    );

    await flushInkFrames(4);
    const frame = app.lastFrame() ?? "";
    expect(frame).toContain("support=unsupported");
    expect(frame).toContain("Archives of Nethys");
    expect(frame).toContain("AoN: https://2e.aonprd.com/");
    expect(frame).not.toContain("\u001b]8;;https://2e.aonprd.com/\u001b\\");
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
    expect(app.lastFrame()).toContain("[✓] Common");

    app.stdin.write("j");
    await flushInkFrames();
    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[✓] Rare");

    app.stdin.write("\u007f");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=common,rare");
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

  it("renders centered horizontal choice dialogs through the shared select prompt path", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <CenteredModePromptHarness />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Choose Search Mode");
    expect(app.lastFrame()).toContain("bg=visible");
    expect(app.lastFrame()).toContain("[Browse]   Search   Lookup");
    expect(app.lastFrame()).toContain("←/→ change mode");

    app.stdin.write("\u001b[C");
    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Browse   [Search]   Lookup");
    expect(app.lastFrame()).toContain("Search named records and ranked text matches.");

    app.stdin.write("\r");
    await flushInkFrames(2);
    expect(app.lastFrame()).toContain("result=search");
  });

  it("uses one centered shell for overlay and blanked prompts while changing only background treatment", async () => {
    const overlayApp = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <CenteredModePromptHarness presentation="overlay" />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    const overlayFrame = overlayApp.lastFrame() ?? "";
    expect(overlayFrame).toContain("Choose Search Mode");
    expect(overlayFrame).toContain("[Browse]   Search   Lookup");
    expect(overlayFrame).toContain("backdrop=true");
    expect(overlayFrame).toContain("bg=visible");
    expect(overlayFrame).toContain("bgHeight=20");
    expect(overlayFrame).toContain("viewportHeight=20");

    const blankedApp = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <CenteredModePromptHarness presentation="blanked" />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    const blankedFrame = blankedApp.lastFrame() ?? "";
    expect(blankedFrame).toContain("Choose Search Mode");
    expect(blankedFrame).toContain("[Browse]   Search   Lookup");
    expect(blankedFrame).not.toContain("bg=visible");
    expect(blankedFrame).not.toContain("bgHeight=20");
    expect(blankedFrame).not.toContain("backdrop=true");

    const overlayPromptLine = findFrameLine(overlayFrame, "Choose Search Mode");
    const blankedPromptLine = findFrameLine(blankedFrame, "Choose Search Mode");
    expect(overlayPromptLine).toBeGreaterThan(2);
    expect(blankedPromptLine).toBe(overlayPromptLine);
  });

  it("preserves the full modal viewport height in blanked mode even when background layout height collapses", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <CenteredModePromptHarness presentation="blanked" />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Choose Search Mode");

    app.stdin.write("\r");
    await flushInkFrames(4);

    const frame = app.lastFrame() ?? "";
    expect(frame).toContain("result=browse");
    expect(frame).toContain("bgHeight=20");
    expect(frame).toContain("viewportHeight=20");
    expect(frame).toContain("modalLayoutHeight=0");
    expect(frame).toContain("modalViewportHeight=20");
  });

  it("keeps the overlay background visible outside the floating prompt box", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <OverlayTransparencyHarness />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    const frame = app.lastFrame() ?? "";
    const promptLineIndex = findFrameLine(frame, "Choose Search Mode");
    expect(promptLineIndex).toBeGreaterThan(2);

    const promptLine = frame.split("\n")[promptLineIndex] ?? "";
    expect(promptLine).toContain("Choose Search Mode");
    expect(promptLine).toMatch(/L\d{2}/);
    expect(promptLine).toMatch(/R\d{2}/);
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

  it("shares slash filtering across select and multiselect prompts", async () => {
    const selectApp = render(
      <DerivedTagTerminalProvider>
        <FilterableChoicePromptHarness kind="select" />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    selectApp.stdin.write("/");
    await flushInkFrames(2);
    for (const character of "rare") {
      selectApp.stdin.write(character);
    }
    await flushInkFrames(2);
    expect(selectApp.lastFrame()).toContain("Search /rare");
    expect(selectApp.lastFrame()).toContain("Rare");
    expect(selectApp.lastFrame()).not.toContain("Common");
    selectApp.stdin.write("\r");
    await flushInkFrames();
    selectApp.stdin.write("\r");
    await flushInkFrames();
    expect(selectApp.lastFrame()).toContain("result=rare");
    selectApp.unmount();

    const multiApp = render(
      <DerivedTagTerminalProvider>
        <FilterableChoicePromptHarness kind="multiselect" />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    multiApp.stdin.write("/");
    await flushInkFrames(2);
    for (const character of "rare") {
      multiApp.stdin.write(character);
    }
    await flushInkFrames(2);
    multiApp.stdin.write("\r");
    await flushInkFrames();
    multiApp.stdin.write("\r");
    await flushInkFrames();
    multiApp.stdin.write("\u007f");
    await flushInkFrames();
    expect(multiApp.lastFrame()).toContain("result=rare");
    multiApp.unmount();

  });

  it("uses the same inline list-capacity planning across select-like prompts", async () => {
    const promptKinds: LayoutPromptKind[] = ["select", "multiselect"];
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

  it("lets select and multiselect prompts grow beyond the legacy fixed inline height", async () => {
    const promptKinds: Array<Exclude<LayoutPromptKind, "command">> = ["select", "multiselect"];

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

  it("returns a command request from a commands-enabled select prompt", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenCommandPaletteHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");
    expect(app.lastFrame()).toContain("Metadata");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=command-requested");
  });

  it("replaces chained overlay prompts without exposing a background-only frame", async () => {
    const app = renderWithTerminalSize(
      <DerivedTagTerminalProvider>
        <OverlaySelectThenSelectHarness />
      </DerivedTagTerminalProvider>,
      { columns: 100, rows: 20 },
    );

    await flushInkFrames(4);
    expect(app.lastFrame()).toContain("Clause Picker");
    const frameStart = app.frames.length;

    app.stdin.write("\r");
    await flushInkFrames(12);

    const transitionFrames = app.frames.slice(frameStart);
    expect(transitionFrames.some((frame) => frame.includes("Left Metric"))).toBe(true);
    expect(
      transitionFrames.some(
        (frame) =>
          frame.includes("bg=chain") &&
          !frame.includes("Clause Picker") &&
          !frame.includes("Left Metric"),
      ),
    ).toBe(false);
  });

  it("can chain a select prompt directly into a multiselect prompt", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenMultiSelectHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");
    expect(app.lastFrame()).toContain("Pack");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Pack Picker");
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[✓] Pathfinder NPC Core");

    app.stdin.write("j");
    await flushInkFrames();
    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("[✓] Monster Core");

    app.stdin.write("\u007f");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=packs=pathfinder-npc-core,monster-core");
  });

  it("can request commands from a second select prompt in a chained flow", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenSelectThenCommandsHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Left Metric");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=command-requested");
  });

  it("can request commands from a chained multiselect prompt", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenMultiSelectThenCommandsHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Pack");
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=command-requested");
  });

  it("can request commands from a chained select prompt started by a live input handler", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <EventDrivenSelectChainHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Press Enter to start");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Clause Picker");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Left Metric");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=command-requested");
  });

  it("can request commands from a prompt chain launched through TerminalMenuScreen", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <MenuScreenDrivenPromptHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Open Prompt Chain");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Clause Picker");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Left Metric");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=command-requested");
  });

  it("can chain a select prompt directly into another select prompt and confirm inside the second picker", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenSelectHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");
    expect(app.lastFrame()).toContain("Metric comparison");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Left Metric");
    expect(app.lastFrame()).toContain("hp.value");

    app.stdin.write("\r");
    await flushInkFrames();
    expect(app.lastFrame()).toContain("result=metric=hp.value");
  });

  it("can chain a select prompt directly into another select prompt and request commands in the second picker", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <SelectThenSelectHarness />
      </DerivedTagTerminalProvider>,
    );

    await flushInkFrames();
    expect(app.lastFrame()).toContain("Clause Picker");
    expect(app.lastFrame()).toContain("Metric comparison");

    app.stdin.write("\r");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("Left Metric");
    expect(app.lastFrame()).toContain("hp.value");

    app.stdin.write(":");
    await flushInkFrames(3);
    expect(app.lastFrame()).toContain("result=metric=commands");
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
