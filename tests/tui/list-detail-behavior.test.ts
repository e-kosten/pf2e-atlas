import { describe, expect, it, vi } from "vitest";

import {
  TERMINAL_LIST_DETAIL_FOCUS_POLICY,
  applyTerminalListDetailRightBehavior,
  getTerminalListDetailDeadEndMessage,
  resolveTerminalListDetailRightBehavior,
} from "../../src/tui/list-detail-behavior.js";

describe("list detail behavior", () => {
  it("resolves available right intents as shared success actions", () => {
    const perform = vi.fn();

    const resolution = resolveTerminalListDetailRightBehavior({
      rightIntent: "drill",
      destination: {
        availability: "available",
        perform,
      },
    });

    expect(resolution).toEqual({
      kind: "success",
      focusPolicy: TERMINAL_LIST_DETAIL_FOCUS_POLICY,
      rightIntent: "drill",
    });
  });

  it("treats preview already visible as a notify dead end with explicit-only focus", () => {
    const resolution = resolveTerminalListDetailRightBehavior({
      rightIntent: "preview",
      destination: { availability: "already-satisfied" },
      deadEndPolicy: "notify",
    });

    expect(resolution).toEqual({
      kind: "dead-end",
      availability: "already-satisfied",
      deadEndPolicy: "notify",
      focusPolicy: TERMINAL_LIST_DETAIL_FOCUS_POLICY,
      rightIntent: "preview",
      notification: {
        message: "Preview is already visible.",
        tone: "warning",
      },
    });
  });

  it("routes shared dead-end notifications through the provided notification seam", () => {
    const showNotification = vi.fn();

    const resolution = applyTerminalListDetailRightBehavior({
      contract: {
        rightIntent: "drill",
        destination: { availability: "unavailable" },
        deadEndPolicy: "notify",
      },
      showNotification,
    });

    expect(resolution.kind).toBe("dead-end");
    expect(showNotification).toHaveBeenCalledWith({
      message: "No deeper destination is available for the focused entry.",
      tone: "warning",
    });
  });

  it("supports noop dead ends without emitting notifications", () => {
    const showNotification = vi.fn();

    const resolution = applyTerminalListDetailRightBehavior({
      contract: {
        rightIntent: "preview",
        destination: { availability: "unavailable" },
        deadEndPolicy: "noop",
      },
      showNotification,
    });

    expect(resolution).toEqual({
      kind: "dead-end",
      availability: "unavailable",
      deadEndPolicy: "noop",
      focusPolicy: TERMINAL_LIST_DETAIL_FOCUS_POLICY,
      rightIntent: "preview",
      notification: null,
    });
    expect(showNotification).not.toHaveBeenCalled();
  });

  it("keeps shared dead-end messages on one behavior-owned path", () => {
    expect(
      getTerminalListDetailDeadEndMessage({
        rightIntent: "none",
        availability: "unavailable",
      }),
    ).toBe("No rightward action is available for the focused entry.");
  });
});
