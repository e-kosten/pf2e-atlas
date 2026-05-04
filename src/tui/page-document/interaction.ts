import type { PageDocumentModel, PageDocumentSectionModel, PageDocumentTargetNode } from "./model.js";

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}

function getSectionAnchorNodeIndex(
  document: PageDocumentModel,
  sectionId: string,
): number | null {
  return document.sectionAnchors.find((anchor) => anchor.sectionId === sectionId)?.nodeIndex ?? null;
}

function getNodeScrollPosition(nodeIndex: number, nodeStartRows: readonly number[] | undefined): number {
  return nodeStartRows?.[nodeIndex] ?? nodeIndex;
}

function getNodeIndexAtScrollPosition(document: PageDocumentModel, rowIndex: number, nodeStartRows: readonly number[]): number {
  let activeNodeIndex = 0;

  for (let nodeIndex = 0; nodeIndex < document.nodes.length; nodeIndex += 1) {
    const nodeRow = nodeStartRows[nodeIndex] ?? nodeIndex;
    if (nodeRow > rowIndex) {
      break;
    }
    activeNodeIndex = nodeIndex;
  }

  return activeNodeIndex;
}

export function getPageDocumentReadingAnchorOffset(bodyHeight: number): number {
  return Math.max(0, Math.floor(Math.max(1, bodyHeight) / 3));
}

export function getPageDocumentReadingAnchorNodeIndex(options: {
  document: PageDocumentModel;
  scroll: number;
  bodyHeight: number;
  nodeStartRows?: readonly number[];
}): number {
  const anchorOffset = getPageDocumentReadingAnchorOffset(options.bodyHeight);
  const anchorRow = options.scroll + anchorOffset;
  if (options.nodeStartRows) {
    return getNodeIndexAtScrollPosition(options.document, anchorRow, options.nodeStartRows);
  }
  return clamp(anchorRow, 0, Math.max(0, options.document.nodes.length - 1));
}

export function getActivePageDocumentSection(options: {
  document: PageDocumentModel;
  scroll: number;
  bodyHeight: number;
  nodeStartRows?: readonly number[];
  maxScroll?: number;
}): PageDocumentSectionModel | null {
  if (options.scroll === 0) {
    return options.document.sections[0] ?? null;
  }
  if (options.maxScroll != null && options.maxScroll > 0 && options.scroll >= options.maxScroll) {
    return options.document.sections.at(-1) ?? null;
  }

  const anchorNodeIndex = getPageDocumentReadingAnchorNodeIndex(options);
  let activeSection: PageDocumentSectionModel | null = null;

  for (const section of options.document.sections) {
    const sectionAnchorNodeIndex = getSectionAnchorNodeIndex(options.document, section.id);
    if (sectionAnchorNodeIndex == null || sectionAnchorNodeIndex > anchorNodeIndex) {
      break;
    }
    activeSection = section;
  }

  return activeSection ?? options.document.sections[0] ?? null;
}

function getPageDocumentSectionById(
  document: PageDocumentModel,
  sectionId: string,
): PageDocumentSectionModel | null {
  return document.sections.find((section) => section.id === sectionId) ?? null;
}

export function getFocusedPageDocumentSection(options: {
  document: PageDocumentModel;
  state: PageDocumentInteractionState;
  scroll: number;
  bodyHeight: number;
  nodeStartRows?: readonly number[];
  maxScroll?: number;
}): PageDocumentSectionModel | null {
  if (options.state.mode.kind === "target") {
    return getPageDocumentSectionById(options.document, options.state.mode.sectionId);
  }
  if (options.state.mode.sectionId) {
    return getPageDocumentSectionById(options.document, options.state.mode.sectionId);
  }

  return getActivePageDocumentSection(options);
}

export function getPageDocumentSectionScrollTarget(options: {
  document: PageDocumentModel;
  sectionId: string;
  bodyHeight: number;
  maxScroll: number;
  nodeStartRows?: readonly number[];
}): number {
  const sectionAnchorNodeIndex = getSectionAnchorNodeIndex(options.document, options.sectionId);
  if (sectionAnchorNodeIndex == null) {
    return 0;
  }

  const anchorOffset = getPageDocumentReadingAnchorOffset(options.bodyHeight);
  const sectionAnchorScrollPosition = getNodeScrollPosition(sectionAnchorNodeIndex, options.nodeStartRows);
  return clamp(sectionAnchorScrollPosition - anchorOffset, 0, Math.max(0, options.maxScroll));
}

export type PageDocumentInteractionMode =
  | { kind: "section"; sectionId?: string }
  | {
      kind: "target";
      sectionId: string;
      targetIndex: number;
    };

export type PageDocumentInteractionState = {
  mode: PageDocumentInteractionMode;
};

export type PageDocumentSectionMovementResult = {
  state: PageDocumentInteractionState;
  scroll: number;
  section: PageDocumentSectionModel | null;
};

export function createPageDocumentInteractionState(): PageDocumentInteractionState {
  return {
    mode: { kind: "section" },
  };
}

export function focusPageDocumentSection(sectionId: string): PageDocumentInteractionState {
  return {
    mode: {
      kind: "section",
      sectionId,
    },
  };
}

function getTargetNodesForSection(
  document: PageDocumentModel,
  sectionId: string,
): PageDocumentTargetNode[] {
  return document.targetNodes.filter((node) => node.sectionId === sectionId);
}

function getPageDocumentNodeScrollTarget(options: {
  document: PageDocumentModel;
  nodeId: string;
  bodyHeight: number;
  maxScroll: number;
  nodeStartRows?: readonly number[];
}): number {
  const nodeIndex = options.document.nodes.findIndex((node) => node.id === options.nodeId);
  if (nodeIndex < 0) {
    return 0;
  }

  const anchorOffset = getPageDocumentReadingAnchorOffset(options.bodyHeight);
  const nodeScrollPosition = getNodeScrollPosition(nodeIndex, options.nodeStartRows);
  return clamp(nodeScrollPosition - anchorOffset, 0, Math.max(0, options.maxScroll));
}

export function movePageDocumentSection(options: {
  document: PageDocumentModel;
  scroll: number;
  bodyHeight: number;
  maxScroll: number;
  delta: number;
  nodeStartRows?: readonly number[];
  state?: PageDocumentInteractionState;
}): PageDocumentSectionMovementResult {
  const state = options.state ?? createPageDocumentInteractionState();
  if (state.mode.kind === "target") {
    return {
      state,
      scroll: options.scroll,
      section: getPageDocumentSectionById(options.document, state.mode.sectionId),
    };
  }

  const activeSection =
    state.mode.sectionId
      ? getPageDocumentSectionById(options.document, state.mode.sectionId)
      : getActivePageDocumentSection({ ...options, maxScroll: options.maxScroll });
  if (!activeSection) {
    return {
      state,
      scroll: options.scroll,
      section: null,
    };
  }

  const currentIndex = options.document.sections.findIndex((section) => section.id === activeSection.id);
  const nextIndex = clamp(currentIndex + options.delta, 0, Math.max(0, options.document.sections.length - 1));
  const nextSection = options.document.sections[nextIndex];

  if (!nextSection) {
    return {
      state,
      scroll: options.scroll,
      section: null,
    };
  }

  return {
    state: focusPageDocumentSection(nextSection.id),
    scroll: getPageDocumentSectionScrollTarget({
      document: options.document,
      sectionId: nextSection.id,
      bodyHeight: options.bodyHeight,
      maxScroll: options.maxScroll,
      nodeStartRows: options.nodeStartRows,
    }),
    section: nextSection,
  };
}

export function movePageDocumentSectionBoundary(options: {
  document: PageDocumentModel;
  boundary: "start" | "end";
  bodyHeight: number;
  maxScroll: number;
  nodeStartRows?: readonly number[];
  scroll?: number;
  state?: PageDocumentInteractionState;
}): PageDocumentSectionMovementResult {
  const state = options.state ?? createPageDocumentInteractionState();
  if (state.mode.kind === "target") {
    return {
      state,
      scroll: options.scroll ?? 0,
      section: getPageDocumentSectionById(options.document, state.mode.sectionId),
    };
  }

  const targetSection =
    options.boundary === "start" ? options.document.sections[0] : options.document.sections.at(-1);

  if (!targetSection) {
    return {
      state,
      scroll: options.scroll ?? 0,
      section: null,
    };
  }

  return {
    state: focusPageDocumentSection(targetSection.id),
    scroll: getPageDocumentSectionScrollTarget({
      document: options.document,
      sectionId: targetSection.id,
      bodyHeight: options.bodyHeight,
      maxScroll: options.maxScroll,
      nodeStartRows: options.nodeStartRows,
    }),
    section: targetSection,
  };
}

export function enterPageDocumentTargetMode(options: {
  document: PageDocumentModel;
  scroll: number;
  bodyHeight: number;
  maxScroll: number;
  nodeStartRows?: readonly number[];
  sectionId?: string;
}): {
  state: PageDocumentInteractionState;
  scroll: number;
} {
  const activeSection = options.sectionId
    ? getPageDocumentSectionById(options.document, options.sectionId)
    : getActivePageDocumentSection(options);
  if (!activeSection) {
    return {
      state: createPageDocumentInteractionState(),
      scroll: options.scroll,
    };
  }

  const targets = getTargetNodesForSection(options.document, activeSection.id);
  const firstTarget = targets[0];
  if (!firstTarget) {
    return {
      state: createPageDocumentInteractionState(),
      scroll: options.scroll,
    };
  }

  return {
    state: {
      mode: {
        kind: "target",
        sectionId: activeSection.id,
        targetIndex: 0,
      },
    },
    scroll: getPageDocumentNodeScrollTarget({
      document: options.document,
      nodeId: firstTarget.nodeId,
      bodyHeight: options.bodyHeight,
      maxScroll: options.maxScroll,
      nodeStartRows: options.nodeStartRows,
    }),
  };
}

export function leavePageDocumentTargetMode(): PageDocumentInteractionState {
  return createPageDocumentInteractionState();
}

export function getSelectedPageDocumentTarget(options: {
  document: PageDocumentModel;
  state: PageDocumentInteractionState;
}): PageDocumentTargetNode | null {
  if (options.state.mode.kind !== "target") {
    return null;
  }

  return getTargetNodesForSection(options.document, options.state.mode.sectionId)[options.state.mode.targetIndex] ?? null;
}

export function movePageDocumentTarget(options: {
  document: PageDocumentModel;
  state: PageDocumentInteractionState;
  bodyHeight: number;
  maxScroll: number;
  delta: number;
  nodeStartRows?: readonly number[];
}): {
  state: PageDocumentInteractionState;
  scroll: number;
} {
  if (options.state.mode.kind !== "target") {
    return {
      state: options.state,
      scroll: 0,
    };
  }

  const targets = getTargetNodesForSection(options.document, options.state.mode.sectionId);
  const nextTargetIndex = clamp(options.state.mode.targetIndex + options.delta, 0, Math.max(0, targets.length - 1));
  const nextTarget = targets[nextTargetIndex];
  if (!nextTarget) {
    return {
      state: options.state,
      scroll: 0,
    };
  }

  return {
    state: {
      mode: {
        kind: "target",
        sectionId: options.state.mode.sectionId,
        targetIndex: nextTargetIndex,
      },
    },
    scroll: getPageDocumentNodeScrollTarget({
      document: options.document,
      nodeId: nextTarget.nodeId,
      bodyHeight: options.bodyHeight,
      maxScroll: options.maxScroll,
      nodeStartRows: options.nodeStartRows,
    }),
  };
}

export function movePageDocumentTargetBoundary(options: {
  document: PageDocumentModel;
  state: PageDocumentInteractionState;
  bodyHeight: number;
  maxScroll: number;
  boundary: "start" | "end";
  nodeStartRows?: readonly number[];
}): {
  state: PageDocumentInteractionState;
  scroll: number;
} {
  if (options.state.mode.kind !== "target") {
    return {
      state: options.state,
      scroll: 0,
    };
  }

  const targets = getTargetNodesForSection(options.document, options.state.mode.sectionId);
  const targetIndex = options.boundary === "start" ? 0 : Math.max(0, targets.length - 1);
  const target = targets[targetIndex];
  if (!target) {
    return {
      state: options.state,
      scroll: 0,
    };
  }

  return {
    state: {
      mode: {
        kind: "target",
        sectionId: options.state.mode.sectionId,
        targetIndex,
      },
    },
    scroll: getPageDocumentNodeScrollTarget({
      document: options.document,
      nodeId: target.nodeId,
      bodyHeight: options.bodyHeight,
      maxScroll: options.maxScroll,
      nodeStartRows: options.nodeStartRows,
    }),
  };
}
