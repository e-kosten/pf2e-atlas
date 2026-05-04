import { describe, expect, it } from "vitest";

import {
  dispatchDerivedTagTerminalPointerEvent,
  parseDerivedTagTerminalPointerEvent,
} from "../../src/tui/framework/pointer-events.js";

describe("terminal pointer events", () => {
  it("parses SGR wheel sequences from both raw and Ink-normalized input", () => {
    expect(parseDerivedTagTerminalPointerEvent("\u001b[<64;25;5M")).toEqual({
      kind: "wheel",
      x: 24,
      y: 4,
      deltaY: -1,
    });
    expect(parseDerivedTagTerminalPointerEvent("[<65;10;3M")).toEqual({
      kind: "wheel",
      x: 9,
      y: 2,
      deltaY: 1,
    });
  });

  it("dispatches to the highest-priority matching region first", () => {
    const calls: string[] = [];
    const handled = dispatchDerivedTagTerminalPointerEvent(
      [
        {
          rect: { x: 0, y: 0, width: 20, height: 10 },
          priority: 0,
          order: 1,
          onPointerEvent: () => {
            calls.push("background");
            return true;
          },
        },
        {
          rect: { x: 5, y: 2, width: 8, height: 4 },
          priority: 100,
          order: 2,
          onPointerEvent: () => {
            calls.push("overlay");
            return true;
          },
        },
      ],
      {
        kind: "wheel",
        x: 6,
        y: 3,
        deltaY: -1,
      },
    );

    expect(handled).toBe(true);
    expect(calls).toEqual(["overlay"]);
  });
});
