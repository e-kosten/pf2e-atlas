import { describe, expect, it } from "vitest";

import {
  createTerminalChoiceSizingDescriptor,
  createTerminalMessageSizingDescriptor,
  planTerminalModalLayout,
} from "../../src/tui/terminal-modal-layout.js";

describe("planTerminalModalLayout", () => {
  it("keeps a short dialog inline on a large terminal", () => {
    const result = planTerminalModalLayout({
      terminalWidth: 100,
      terminalHeight: 30,
      kind: "dialog",
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: 4,
      }),
    });

    expect(result.presentation).toBe("inline");
    expect(result.totalHeight).toBe(8);
    expect(result.bodyHeight).toBe(4);
    expect(result.reservedMainScreenHeight).toBeGreaterThanOrEqual(6);
    expect(result.overflowPolicy).toEqual({
      body: "fit",
      list: "fit",
      detail: "fit",
    });
  });

  it("switches a long dialog to screen when inline would consume too much height", () => {
    const result = planTerminalModalLayout({
      terminalWidth: 80,
      terminalHeight: 14,
      kind: "dialog",
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: 10,
      }),
    });

    expect(result.presentation).toBe("screen");
    expect(result.totalHeight).toBe(14);
    expect(result.bodyHeight).toBe(10);
    expect(result.maxInlineTotalHeight).toBe(8);
  });

  it.each(["command", "select", "multiselect"] as const)(
    "applies the same choice-planning rules for %s",
    (kind) => {
      const result = planTerminalModalLayout({
        terminalWidth: 100,
        terminalHeight: 24,
        kind,
        headerRows: 3,
        footerRows: 2,
        descriptor: createTerminalChoiceSizingDescriptor({
          staticBodyLineCount: 2,
          list: {
            itemCount: 16,
            chromeRows: 0,
            minVisibleRowCount: 4,
            preferredVisibleRowCount: 8,
            maxVisibleRowCount: 12,
          },
          detail: {
            lineCount: 9,
            chromeRows: 0,
            minVisibleLineCount: 2,
            preferredVisibleLineCount: 6,
            maxVisibleLineCount: 10,
          },
        }),
      });

      expect(result.presentation).toBe("inline");
      expect(result.paneMode).toBe("two-pane");
      expect(result.visibleListCapacity).toBe(8);
      expect(result.regions).toEqual({
        staticRows: 2,
        listRows: 8,
        detailRows: 8,
      });
      expect(result.overflowPolicy).toEqual({
        body: "fit",
        list: "window",
        detail: "scroll",
      });
    },
  );

  it("collapses choice prompts to single-column mode on narrow terminals", () => {
    const result = planTerminalModalLayout({
      terminalWidth: 40,
      terminalHeight: 24,
      kind: "select",
      headerRows: 3,
      footerRows: 2,
      descriptor: createTerminalChoiceSizingDescriptor({
        staticBodyLineCount: 2,
        list: {
          itemCount: 10,
          chromeRows: 0,
        },
        detail: {
          lineCount: 8,
          chromeRows: 0,
        },
      }),
    });

    expect(result.presentation).toBe("inline");
    expect(result.paneMode).toBe("single-column");
    expect(result.paneWidths).toBeUndefined();
    expect(result.regions.listRows).toBeGreaterThan(result.regions.detailRows);
    expect(result.visibleListCapacity).toBe(result.regions.listRows);
  });

  it("never chooses inline if it would leave too little room for the underlying screen", () => {
    const result = planTerminalModalLayout({
      terminalWidth: 100,
      terminalHeight: 20,
      kind: "dialog",
      headerRows: 3,
      footerRows: 1,
      descriptor: createTerminalMessageSizingDescriptor({
        bodyLineCount: 12,
      }),
    });

    expect(result.presentation).toBe("screen");
    expect(result.maxInlineTotalHeight).toBe(14);
    expect(result.reservedMainScreenHeight).toBe(0);
  });
});
