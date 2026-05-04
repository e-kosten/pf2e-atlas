import { describe, expect, it } from "vitest";

import {
  createPageDocumentInteractionState,
  enterPageDocumentTargetMode,
  getActivePageDocumentSection,
  getFocusedPageDocumentSection,
  getPageDocumentReadingAnchorNodeIndex,
  getPageDocumentReadingAnchorOffset,
  getPageDocumentSectionScrollTarget,
  getSelectedPageDocumentTarget,
  leavePageDocumentTargetMode,
  movePageDocumentSection,
  movePageDocumentSectionBoundary,
  movePageDocumentTarget,
  movePageDocumentTargetBoundary,
} from "../../src/tui/page-document/interaction.js";
import type { PageDocumentModel } from "../../src/tui/page-document/model.js";

function createDocument(): PageDocumentModel {
  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    nodes: [
      { id: "header:title", kind: "title", line: { text: "Fireball" }, anchorRole: "content" },
      { id: "section:summary:heading", kind: "sectionHeading", sectionId: "summary", line: { text: "Summary" }, anchorRole: "sectionStart" },
      { id: "section:summary:text:0", kind: "text", sectionId: "summary", line: { text: "Summary text" }, anchorRole: "content" },
      { id: "section:description:heading", kind: "sectionHeading", sectionId: "description", line: { text: "Description" }, anchorRole: "sectionStart" },
      { id: "section:description:text:0", kind: "text", sectionId: "description", line: { text: "Description text" }, anchorRole: "content" },
      { id: "section:references:heading", kind: "sectionHeading", sectionId: "references", line: { text: "References" }, anchorRole: "sectionStart" },
      { id: "section:references:target:0:0", kind: "target", sectionId: "references", line: { text: "Spell Effect: Fireball" }, anchorRole: "target", target: { kind: "record", label: "Spell Effect: Fireball", recordKey: "spell:fireball-effect", action: "open" } },
      { id: "section:references:target:0:1", kind: "target", sectionId: "references", line: { text: "Spell Effect: Delayed Blast" }, anchorRole: "target", target: { kind: "record", label: "Spell Effect: Delayed Blast", recordKey: "spell:delayed-blast", action: "open" } },
    ],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        startNodeIndex: 1,
        endNodeIndex: 2,
        targetNodeIds: [],
      },
      {
        id: "description",
        kind: "description",
        title: "Description",
        startNodeIndex: 3,
        endNodeIndex: 4,
        targetNodeIds: [],
      },
      {
        id: "references",
        kind: "references",
        title: "References",
        startNodeIndex: 5,
        endNodeIndex: 7,
        targetNodeIds: ["section:references:target:0:0", "section:references:target:0:1"],
      },
    ],
    sectionAnchors: [
      { sectionId: "summary", nodeIndex: 1 },
      { sectionId: "description", nodeIndex: 3 },
      { sectionId: "references", nodeIndex: 5 },
    ],
    targetNodes: [
      {
        nodeId: "section:references:target:0:0",
        sectionId: "references",
        target: { kind: "record", label: "Spell Effect: Fireball", recordKey: "spell:fireball-effect", action: "open" },
      },
      {
        nodeId: "section:references:target:0:1",
        sectionId: "references",
        target: { kind: "record", label: "Spell Effect: Delayed Blast", recordKey: "spell:delayed-blast", action: "open" },
      },
    ],
  };
}

describe("page document interaction", () => {
  it("derives the reading anchor from a stable viewport offset", () => {
    expect(getPageDocumentReadingAnchorOffset(9)).toBe(3);
    expect(
      getPageDocumentReadingAnchorNodeIndex({
        document: createDocument(),
        scroll: 2,
        bodyHeight: 9,
      }),
    ).toBe(5);
  });

  it("derives the active section from the reading anchor instead of stale focus", () => {
    const document = createDocument();

    expect(getActivePageDocumentSection({ document, scroll: 0, bodyHeight: 6 })?.id).toBe("summary");
    expect(getActivePageDocumentSection({ document, scroll: 2, bodyHeight: 6 })?.id).toBe("description");
    expect(getActivePageDocumentSection({ document, scroll: 4, bodyHeight: 6 })?.id).toBe("references");
  });

  it("computes section scroll targets against the same reading anchor", () => {
    const document = createDocument();

    expect(
      getPageDocumentSectionScrollTarget({
        document,
        sectionId: "description",
        bodyHeight: 6,
        maxScroll: 6,
      }),
    ).toBe(1);
    expect(
      getPageDocumentSectionScrollTarget({
        document,
        sectionId: "references",
        bodyHeight: 6,
        maxScroll: 6,
      }),
    ).toBe(3);
  });

  it("can compute section and target scroll targets in rendered row coordinates", () => {
    const document = createDocument();
    const nodeStartRows = [5, 6, 7, 8, 9, 10, 14, 18];

    expect(
      getPageDocumentSectionScrollTarget({
        document,
        sectionId: "references",
        bodyHeight: 6,
        maxScroll: 30,
        nodeStartRows,
      }),
    ).toBe(8);

    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 8,
      bodyHeight: 6,
      maxScroll: 30,
      nodeStartRows,
    });
    const moved = movePageDocumentTarget({
      document,
      state: entered.state,
      bodyHeight: 6,
      maxScroll: 30,
      delta: 1,
      nodeStartRows,
    });

    expect(getSelectedPageDocumentTarget({ document, state: entered.state })?.target.label).toBe("Spell Effect: Fireball");
    expect(moved.scroll).toBe(16);
  });

  it("moves section focus by scrolling to the next section anchor", () => {
    const document = createDocument();

    expect(
      movePageDocumentSection({
        document,
        scroll: 0,
        bodyHeight: 6,
        maxScroll: 6,
        delta: 1,
      }),
    ).toBe(1);
  });

  it("keeps focused section ownership while target mode is active", () => {
    const document = createDocument();
    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 4,
      bodyHeight: 6,
      maxScroll: 6,
    });

    expect(
      getFocusedPageDocumentSection({
        document,
        state: entered.state,
        scroll: 0,
        bodyHeight: 6,
      })?.id,
    ).toBe("references");
  });

  it("supports section boundary movement against the shared reading anchor", () => {
    const document = createDocument();

    expect(
      movePageDocumentSectionBoundary({
        document,
        boundary: "start",
        bodyHeight: 6,
        maxScroll: 6,
      }),
    ).toBe(0);
    expect(
      movePageDocumentSectionBoundary({
        document,
        boundary: "end",
        bodyHeight: 6,
        maxScroll: 6,
      }),
    ).toBe(3);
  });

  it("enters target mode from the active section and restores section mode on exit", () => {
    const document = createDocument();

    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 4,
      bodyHeight: 6,
      maxScroll: 6,
    });
    expect(entered.state.mode).toEqual({
      kind: "target",
      sectionId: "references",
      targetIndex: 0,
    });
    expect(getSelectedPageDocumentTarget({ document, state: entered.state })?.target.label).toBe("Spell Effect: Fireball");
    expect(leavePageDocumentTargetMode()).toEqual(createPageDocumentInteractionState());
  });

  it("moves target focus within the active section and scrolls to keep it readable", () => {
    const document = createDocument();
    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 4,
      bodyHeight: 6,
      maxScroll: 6,
    });

    const moved = movePageDocumentTarget({
      document,
      state: entered.state,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });

    expect(moved.state.mode).toEqual({
      kind: "target",
      sectionId: "references",
      targetIndex: 1,
    });
    expect(getSelectedPageDocumentTarget({ document, state: moved.state })?.target.label).toBe("Spell Effect: Delayed Blast");
    expect(moved.scroll).toBe(5);
  });

  it("supports target boundary movement inside the active section", () => {
    const document = createDocument();
    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 4,
      bodyHeight: 6,
      maxScroll: 6,
    });
    const moved = movePageDocumentTargetBoundary({
      document,
      state: entered.state,
      bodyHeight: 6,
      maxScroll: 6,
      boundary: "end",
    });

    expect(moved.state.mode).toEqual({
      kind: "target",
      sectionId: "references",
      targetIndex: 1,
    });
    expect(moved.scroll).toBe(5);
  });
});
