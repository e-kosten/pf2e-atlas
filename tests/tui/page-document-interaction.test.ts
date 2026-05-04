import { describe, expect, it } from "vitest";

import {
  createPageDocumentInteractionState,
  enterPageDocumentTargetMode,
  focusPageDocumentSection,
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
        targetId: "section:references:target:0:0",
        nodeId: "section:references:target:0:0",
        sectionId: "references",
        target: { kind: "record", label: "Spell Effect: Fireball", recordKey: "spell:fireball-effect", action: "open" },
        location: { kind: "line", nodeId: "section:references:target:0:0" },
      },
      {
        targetId: "section:references:target:0:1",
        nodeId: "section:references:target:0:1",
        sectionId: "references",
        target: { kind: "record", label: "Spell Effect: Delayed Blast", recordKey: "spell:delayed-blast", action: "open" },
        location: { kind: "line", nodeId: "section:references:target:0:1" },
      },
    ],
  };
}

function createInlineTargetDocument(): PageDocumentModel {
  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    nodes: [
      { id: "header:title", kind: "title", line: { text: "Fireball" }, anchorRole: "content" },
      {
        id: "header:traits",
        kind: "traits",
        sectionId: "header",
        line: {
          text: "Traits: Concentrate, Fire, Manipulate",
          segments: [
            { text: "Traits: " },
            { text: "Concentrate" },
            { text: ", " },
            { text: "Fire" },
            { text: ", " },
            { text: "Manipulate" },
          ],
        },
        inlineTargets: [
          {
            targetId: "header:traits:target:0",
            segmentId: "header:traits:segment:0",
            segmentIndex: 1,
            target: { kind: "searchPivot", label: "Trait: Concentrate", request: { mode: "browse", limit: 50 } },
          },
          {
            targetId: "header:traits:target:1",
            segmentId: "header:traits:segment:1",
            segmentIndex: 3,
            target: { kind: "searchPivot", label: "Trait: Fire", request: { mode: "browse", limit: 50 } },
          },
          {
            targetId: "header:traits:target:2",
            segmentId: "header:traits:segment:2",
            segmentIndex: 5,
            target: { kind: "searchPivot", label: "Trait: Manipulate", request: { mode: "browse", limit: 50 } },
          },
        ],
        anchorRole: "target",
      },
      {
        id: "header:row:target",
        kind: "target",
        sectionId: "header",
        line: { text: "Open in Archives of Nethys" },
        target: { kind: "external", label: "Open in Archives of Nethys", href: "https://example.com" },
        anchorRole: "target",
      },
      {
        id: "section:summary:heading",
        kind: "sectionHeading",
        sectionId: "summary",
        line: { text: "Summary" },
        anchorRole: "sectionStart",
      },
      {
        id: "section:summary:target:0:0",
        kind: "target",
        sectionId: "summary",
        line: { text: "Related Spell" },
        target: { kind: "record", label: "Related Spell", recordKey: "spell:related", action: "open" },
        anchorRole: "target",
      },
    ],
    sections: [
      {
        id: "header",
        kind: "identity",
        title: "Identity",
        startNodeIndex: 0,
        endNodeIndex: 2,
        targetNodeIds: [
          "header:traits:target:0",
          "header:traits:target:1",
          "header:traits:target:2",
          "header:row:target",
        ],
      },
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        startNodeIndex: 3,
        endNodeIndex: 4,
        targetNodeIds: ["section:summary:target:0:0"],
      },
    ],
    sectionAnchors: [
      { sectionId: "header", nodeIndex: 1 },
      { sectionId: "summary", nodeIndex: 3 },
    ],
    targetNodes: [
      {
        targetId: "header:traits:target:0",
        nodeId: "header:traits",
        sectionId: "header",
        target: { kind: "searchPivot", label: "Trait: Concentrate", request: { mode: "browse", limit: 50 } },
        location: { kind: "span", nodeId: "header:traits", segmentId: "header:traits:segment:0" },
      },
      {
        targetId: "header:traits:target:1",
        nodeId: "header:traits",
        sectionId: "header",
        target: { kind: "searchPivot", label: "Trait: Fire", request: { mode: "browse", limit: 50 } },
        location: { kind: "span", nodeId: "header:traits", segmentId: "header:traits:segment:1" },
      },
      {
        targetId: "header:traits:target:2",
        nodeId: "header:traits",
        sectionId: "header",
        target: { kind: "searchPivot", label: "Trait: Manipulate", request: { mode: "browse", limit: 50 } },
        location: { kind: "span", nodeId: "header:traits", segmentId: "header:traits:segment:2" },
      },
      {
        targetId: "header:row:target",
        nodeId: "header:row:target",
        sectionId: "header",
        target: { kind: "external", label: "Open in Archives of Nethys", href: "https://example.com" },
        location: { kind: "line", nodeId: "header:row:target" },
      },
      {
        targetId: "section:summary:target:0:0",
        nodeId: "section:summary:target:0:0",
        sectionId: "summary",
        target: { kind: "record", label: "Related Spell", recordKey: "spell:related", action: "open" },
        location: { kind: "line", nodeId: "section:summary:target:0:0" },
      },
    ],
  };
}

function createWrappedInlineTargetDocument(): PageDocumentModel {
  const target = { kind: "record" as const, label: "Delayed Blast Fireball", recordKey: "spell:delayed-blast-fireball", action: "open" as const };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    nodes: [
      { id: "header:title", kind: "title", line: { text: "Fireball" }, anchorRole: "content" },
      {
        id: "section:description:heading",
        kind: "sectionHeading",
        sectionId: "description",
        line: { text: "Description" },
        anchorRole: "sectionStart",
      },
      {
        id: "section:description:text:0",
        kind: "text",
        sectionId: "description",
        line: {
          text: "A long line that wraps before the inline target named Delayed Blast Fireball appears.",
          segments: [
            { text: "A long line that wraps before the inline target named " },
            { text: "Delayed Blast Fireball" },
            { text: " appears." },
          ],
        },
        inlineTargets: [
          {
            targetId: "section:description:text:0:target:0",
            segmentId: "section:description:text:0:segment:0",
            segmentIndex: 1,
            target,
          },
        ],
        anchorRole: "target",
      },
    ],
    sections: [
      {
        id: "description",
        kind: "description",
        title: "Description",
        startNodeIndex: 1,
        endNodeIndex: 2,
        targetNodeIds: ["section:description:text:0:target:0"],
      },
    ],
    sectionAnchors: [{ sectionId: "description", nodeIndex: 1 }],
    targetNodes: [
      {
        targetId: "section:description:text:0:target:0",
        nodeId: "section:description:text:0",
        sectionId: "description",
        target,
        location: { kind: "span", nodeId: "section:description:text:0", segmentId: "section:description:text:0:segment:0" },
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

    const moved = movePageDocumentSection({
      document,
      scroll: 0,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });

    expect(moved.scroll).toBe(1);
    expect(moved.state.mode).toEqual({ kind: "section", sectionId: "description" });
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

    const start = movePageDocumentSectionBoundary({
      document,
      boundary: "start",
      bodyHeight: 6,
      maxScroll: 6,
    });
    const end = movePageDocumentSectionBoundary({
      document,
      boundary: "end",
      bodyHeight: 6,
      maxScroll: 6,
    });

    expect(start.scroll).toBe(0);
    expect(start.state.mode).toEqual({ kind: "section", sectionId: "summary" });
    expect(end.scroll).toBe(3);
    expect(end.state.mode).toEqual({ kind: "section", sectionId: "references" });
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

  it("traverses multiple inline targets on one line before later row targets", () => {
    const document = createInlineTargetDocument();
    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 0,
      bodyHeight: 6,
      maxScroll: 6,
      sectionId: "header",
    });
    const second = movePageDocumentTarget({
      document,
      state: entered.state,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });
    const third = movePageDocumentTarget({
      document,
      state: second.state,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });
    const row = movePageDocumentTarget({
      document,
      state: third.state,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });

    expect(getSelectedPageDocumentTarget({ document, state: entered.state })?.target.label).toBe("Trait: Concentrate");
    expect(getSelectedPageDocumentTarget({ document, state: second.state })?.target.label).toBe("Trait: Fire");
    expect(getSelectedPageDocumentTarget({ document, state: third.state })?.target.label).toBe("Trait: Manipulate");
    expect(getSelectedPageDocumentTarget({ document, state: row.state })?.target.label).toBe("Open in Archives of Nethys");
  });

  it("moves section focus over sections with inline span targets and row targets", () => {
    const document = createInlineTargetDocument();
    const next = movePageDocumentSection({
      document,
      state: focusPageDocumentSection("header"),
      scroll: 0,
      bodyHeight: 6,
      maxScroll: 6,
      delta: 1,
    });
    const previous = movePageDocumentSection({
      document,
      state: next.state,
      scroll: next.scroll,
      bodyHeight: 6,
      maxScroll: 6,
      delta: -1,
    });

    expect(next.section?.id).toBe("summary");
    expect(next.state.mode).toEqual({ kind: "section", sectionId: "summary" });
    expect(next.scroll).toBe(1);
    expect(previous.section?.id).toBe("header");
    expect(previous.state.mode).toEqual({ kind: "section", sectionId: "header" });
    expect(previous.scroll).toBe(0);
  });

  it("honors inline targets for target boundary movement", () => {
    const document = createInlineTargetDocument();
    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 0,
      bodyHeight: 6,
      maxScroll: 6,
      sectionId: "header",
    });
    const end = movePageDocumentTargetBoundary({
      document,
      state: entered.state,
      bodyHeight: 6,
      maxScroll: 6,
      boundary: "end",
    });
    const start = movePageDocumentTargetBoundary({
      document,
      state: end.state,
      bodyHeight: 6,
      maxScroll: 6,
      boundary: "start",
    });

    expect(getSelectedPageDocumentTarget({ document, state: end.state })?.target.label).toBe(
      "Open in Archives of Nethys",
    );
    expect(getSelectedPageDocumentTarget({ document, state: start.state })?.target.label).toBe("Trait: Concentrate");
  });

  it("scrolls a wrapped inline span target's containing rendered line into view", () => {
    const document = createWrappedInlineTargetDocument();
    const nodeStartRows = [0, 1, 12];

    const entered = enterPageDocumentTargetMode({
      document,
      scroll: 0,
      bodyHeight: 3,
      maxScroll: 30,
      nodeStartRows,
      sectionId: "description",
    });

    expect(getSelectedPageDocumentTarget({ document, state: entered.state })?.target.label).toBe(
      "Delayed Blast Fireball",
    );
    expect(entered.scroll).toBe(11);
    expect(nodeStartRows[2]).toBeGreaterThanOrEqual(entered.scroll);
    expect(nodeStartRows[2]).toBeLessThan(entered.scroll + 3);
  });
});
