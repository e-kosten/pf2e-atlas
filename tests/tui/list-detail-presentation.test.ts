import { describe, expect, it } from "vitest";

import {
  buildTerminalListDetailScreenModel,
  measureTerminalListDetailPresentation,
} from "../../src/tui/list-detail-presentation.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";

describe("list detail presentation", () => {
  it("measures detail capacity and clamps detail scroll", () => {
    const metrics = measureTerminalListDetailPresentation({
      terminalWidth: 100,
      terminalHeight: 12,
      footerLineCount: 2,
      detailLines: Array.from({ length: 20 }, (_, index) => ({
        text: `Detail line ${index + 1}`,
        noWrap: true,
      })),
      detailScroll: 99,
      layoutMode: "split",
      leftWidth: 40,
    });

    expect(metrics.bodyHeight).toBeGreaterThan(0);
    expect(metrics.pageSize).toBe(Math.max(1, metrics.bodyHeight - 1));
    expect(metrics.selectionJumpSize).toBe(Math.max(1, Math.floor(metrics.bodyHeight / 2)));
    expect(metrics.detailScroll).toBe(metrics.maxDetailScroll);
    expect(metrics.visibleDetailLines.length).toBeLessThanOrEqual(metrics.bodyHeight);
  });

  it("builds a two-pane screen model with active pane flags", () => {
    const screen = buildTerminalListDetailScreenModel({
      title: "Shared Screen",
      subtitle: "Subtitle",
      activePane: "detail",
      layoutMode: "split",
      leftWidth: 32,
      leftPane: {
        title: "List",
        lines: [{ text: "Entry 1" }],
      },
      rightPane: {
        title: "Detail",
      },
      metrics: {
        visibleDetailLines: [{ text: "Detail 1" }],
      },
      footer: [{ text: "Bindings", tone: "dim" }],
    });

    expect(screen.kind).toBe("two-pane");
    if (screen.kind !== "two-pane") {
      throw new Error("expected two-pane model");
    }
    expect(screen.props.left.active).toBe(false);
    expect(screen.props.right.active).toBe(true);
    expect(screen.props.right.lines).toEqual([{ text: "Detail 1" }]);
  });

  it("builds a detail-only screen model and appends transition footer status", () => {
    const screen = buildTerminalListDetailScreenModel({
      title: "Shared Screen",
      subtitle: "Subtitle",
      activePane: "list",
      layoutMode: "detail-only",
      leftWidth: 32,
      leftPane: {
        title: "List",
        lines: [{ text: "Entry 1" }],
      },
      rightPane: {
        title: "Detail",
        detailOnlyTitle: "[FOCUSED DETAIL] Detail",
      },
      metrics: {
        visibleDetailLines: [{ text: "Detail 1" }],
      },
      footer: [{ text: "Bindings", tone: "dim" }],
      transitionStatus: {
        kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
        message: "Loading detail",
        frame: 0,
      },
    });

    expect(screen.kind).toBe("detail-only");
    if (screen.kind !== "detail-only") {
      throw new Error("expected detail-only model");
    }
    expect(screen.props.pane.title).toBe("[FOCUSED DETAIL] Detail");
    expect(screen.props.footer).toHaveLength(2);
    expect(screen.props.footer?.[1]?.text).toContain("Loading next view");
  });
});
