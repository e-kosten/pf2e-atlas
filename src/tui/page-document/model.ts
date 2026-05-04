import type {
  EntityPageDocument,
  EntityPageSection,
  EntityPageTarget,
  EntityPageTextSegment,
} from "../../app/ontology/entity-page.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { RecordKey } from "../../domain/record-types.js";
import type { DerivedTagTerminalLine, DerivedTagTerminalSegment } from "../framework/types.js";

export type PageDocumentNodeKind =
  | "title"
  | "identity"
  | "external"
  | "traits"
  | "sectionHeading"
  | "text"
  | "fact"
  | "target";

export type PageDocumentNode = {
  id: string;
  kind: PageDocumentNodeKind;
  line: DerivedTagTerminalLine;
  sectionId?: string;
  target?: EntityPageTarget;
  inlineTargets?: PageDocumentInlineTarget[];
  anchorRole: "content" | "sectionStart" | "target";
};

export type PageDocumentSectionModel = {
  id: string;
  kind: EntityPageSection["kind"];
  title?: string;
  startNodeIndex: number;
  endNodeIndex: number;
  targetNodeIds: string[];
};

export type PageDocumentSectionAnchor = {
  sectionId: string;
  nodeIndex: number;
};

export type PageDocumentTargetNode = {
  targetId: string;
  nodeId: string;
  sectionId: string;
  target: EntityPageTarget;
  location:
    | { kind: "line"; nodeId: string }
    | { kind: "span"; nodeId: string; segmentId: string };
};

export type PageDocumentModel = {
  recordKey: RecordKey;
  title: string;
  nodes: PageDocumentNode[];
  sections: PageDocumentSectionModel[];
  sectionAnchors: PageDocumentSectionAnchor[];
  targetNodes: PageDocumentTargetNode[];
};

type PageDocumentInlineTarget = {
  targetId: string;
  segmentId: string;
  segmentIndex: number;
  target: EntityPageTarget;
};

function toTargetLine(target: EntityPageTarget): DerivedTagTerminalLine {
  if (target.kind === "external") {
    return {
      text: target.label,
      indent: 2,
      href: target.href,
      plainTextFallback: target.plainTextFallback,
    };
  }

  return {
    text: target.label,
    indent: 2,
  };
}

function toInlineTraitLine(traits: readonly string[], targets: readonly EntityPageTarget[]): {
  line: DerivedTagTerminalLine;
  inlineTargets: PageDocumentInlineTarget[];
} {
  const segments: DerivedTagTerminalSegment[] = [{ text: "Traits: " }];
  const inlineTargets: PageDocumentInlineTarget[] = [];

  traits.forEach((trait, traitIndex) => {
    if (traitIndex > 0) {
      segments.push({ text: ", " });
    }

    const target = targets[traitIndex];
    const segmentIndex = segments.length;
    segments.push({ text: formatOntologySearchVocabularyLabel(trait) });
    if (target) {
      inlineTargets.push({
        targetId: `header:traits:target:${traitIndex}`,
        segmentId: `header:traits:segment:${traitIndex}`,
        segmentIndex,
        target,
      });
    }
  });

  return {
    line: {
      text: segments.map((segment) => segment.text).join(""),
      indent: 2,
      segments,
    },
    inlineTargets,
  };
}

function toInlineTextLine(args: {
  nodeId: string;
  text: string;
  segments: readonly EntityPageTextSegment[] | undefined;
}): {
  line: DerivedTagTerminalLine;
  inlineTargets: PageDocumentInlineTarget[];
} {
  if (!args.segments || args.segments.length === 0) {
    return {
      line: { text: args.text, indent: 2 },
      inlineTargets: [],
    };
  }

  const renderedSegments: DerivedTagTerminalSegment[] = args.segments.map((segment) => ({
    text: segment.text,
    tone: segment.tone,
  }));
  const inlineTargets: PageDocumentInlineTarget[] = [];
  args.segments.forEach((segment, segmentIndex) => {
    if (!segment.target) {
      return;
    }
    inlineTargets.push({
      targetId: `${args.nodeId}:target:${inlineTargets.length}`,
      segmentId: `${args.nodeId}:segment:${segmentIndex}`,
      segmentIndex,
      target: segment.target,
    });
  });

  return {
    line: {
      text: renderedSegments.map((segment) => segment.text).join(""),
      indent: 2,
      segments: renderedSegments,
    },
    inlineTargets,
  };
}

export function buildPageDocumentModel(
  document: EntityPageDocument,
  options: { includeHeader?: boolean } = {},
): PageDocumentModel {
  const includeHeader = options.includeHeader ?? true;
  const nodes: PageDocumentNode[] = [];
  const sections: PageDocumentSectionModel[] = [];
  const sectionAnchors: PageDocumentSectionAnchor[] = [];
  const targetNodes: PageDocumentTargetNode[] = [];

  const pushNode = (node: PageDocumentNode): number => {
    nodes.push(node);
    return nodes.length - 1;
  };

  const pushInlineTargets = (node: PageDocumentNode, sectionId: string, targetIds: string[]): void => {
    for (const inlineTarget of node.inlineTargets ?? []) {
      targetIds.push(inlineTarget.targetId);
      targetNodes.push({
        targetId: inlineTarget.targetId,
        nodeId: node.id,
        sectionId,
        target: inlineTarget.target,
        location: {
          kind: "span",
          nodeId: node.id,
          segmentId: inlineTarget.segmentId,
        },
      });
    }
  };

  if (includeHeader) {
    pushNode({
      id: "header:title",
      kind: "title",
      line: { text: document.title, tone: "section" },
      anchorRole: "content",
    });
    if (document.identityLine) {
      pushNode({
        id: "header:identity",
        kind: "identity",
        line: { text: document.identityLine, indent: 2 },
        anchorRole: "content",
      });
    }
  }

  const headerStartNodeIndex = nodes.length;
  const headerTargetIds: string[] = [];
  let headerAnchorNodeIndex: number | null = null;

  if (document.aonLink) {
    const nodeId = "header:external:aon";
    const nodeIndex = pushNode({
      id: nodeId,
      kind: "external",
      line: toTargetLine(document.aonLink),
      target: document.aonLink,
      anchorRole: "target",
    });
    headerAnchorNodeIndex ??= nodeIndex;
    headerTargetIds.push(nodeId);
    targetNodes.push({
      targetId: nodeId,
      nodeId,
      sectionId: "header",
      target: document.aonLink,
      location: { kind: "line", nodeId },
    });
  }

  if (document.traits.length > 0) {
    const traitLine = toInlineTraitLine(document.traits, document.traitTargets ?? []);
    const node: PageDocumentNode = {
      id: "header:traits",
      kind: "traits",
      sectionId: "header",
      line: traitLine.line,
      inlineTargets: traitLine.inlineTargets,
      anchorRole: traitLine.inlineTargets.length > 0 ? "target" : "content",
    };
    const nodeIndex = pushNode(node);
    headerAnchorNodeIndex ??= nodeIndex;
    pushInlineTargets(node, "header", headerTargetIds);
  }

  if (headerTargetIds.length > 0) {
    sections.push({
      id: "header",
      kind: "identity",
      title: "Identity",
      startNodeIndex: Math.min(headerStartNodeIndex, Math.max(0, nodes.length - 1)),
      endNodeIndex: Math.max(headerStartNodeIndex, nodes.length - 1),
      targetNodeIds: headerTargetIds,
    });
    if (headerAnchorNodeIndex != null) {
      sectionAnchors.push({
        sectionId: "header",
        nodeIndex: headerAnchorNodeIndex,
      });
    }
  }

  for (const section of document.sections) {
    const startNodeIndex = nodes.length;
    const targetNodeIds: string[] = [];
    let anchorNodeIndex: number | null = null;

    if (section.title) {
      anchorNodeIndex = pushNode({
        id: `section:${section.id}:heading`,
        kind: "sectionHeading",
        sectionId: section.id,
        line: { text: section.title, tone: "section" },
        anchorRole: "sectionStart",
      });
    }

    section.blocks.forEach((block, blockIndex) => {
      if (block.kind === "text") {
        const nodeId = `section:${section.id}:text:${blockIndex}`;
        const textLine = toInlineTextLine({
          nodeId,
          text: block.text,
          segments: block.segments,
        });
        const node: PageDocumentNode = {
          id: nodeId,
          kind: "text",
          sectionId: section.id,
          line: textLine.line,
          inlineTargets: textLine.inlineTargets,
          anchorRole: textLine.inlineTargets.length > 0 ? "target" : "content",
        };
        const nodeIndex = pushNode(node);
        anchorNodeIndex ??= nodeIndex;
        pushInlineTargets(node, section.id, targetNodeIds);
        return;
      }

      if (block.kind === "factList") {
        block.facts.forEach((fact, factIndex) => {
          const nodeIndex = pushNode({
            id: `section:${section.id}:fact:${blockIndex}:${factIndex}`,
            kind: "fact",
            sectionId: section.id,
            line: { text: `${fact.label}: ${fact.value}`, indent: 2 },
            anchorRole: "content",
          });
          anchorNodeIndex ??= nodeIndex;
        });
        return;
      }

      block.targets.forEach((target, targetIndex) => {
        const nodeId = `section:${section.id}:target:${blockIndex}:${targetIndex}`;
        const nodeIndex = pushNode({
          id: nodeId,
          kind: "target",
          sectionId: section.id,
          line: toTargetLine(target),
          target,
          anchorRole: "target",
        });
        anchorNodeIndex ??= nodeIndex;
        targetNodeIds.push(nodeId);
        targetNodes.push({
          targetId: nodeId,
          nodeId,
          sectionId: section.id,
          target,
          location: { kind: "line", nodeId },
        });
      });
    });

    sections.push({
      id: section.id,
      kind: section.kind,
      title: section.title,
      startNodeIndex,
      endNodeIndex: Math.max(startNodeIndex, nodes.length - 1),
      targetNodeIds,
    });

    if (anchorNodeIndex != null) {
      sectionAnchors.push({
        sectionId: section.id,
        nodeIndex: anchorNodeIndex,
      });
    }
  }

  return {
    recordKey: document.recordKey,
    title: document.title,
    nodes,
    sections,
    sectionAnchors,
    targetNodes,
  };
}

export function renderPageDocumentModel(
  model: PageDocumentModel,
  options: {
    activeSectionId?: string | null;
    selectedTargetId?: string | null;
  } = {},
): DerivedTagTerminalLine[] {
  return model.nodes.map((node) => {
    const selectedTargetId = options.selectedTargetId ?? null;
    if (node.inlineTargets && node.inlineTargets.length > 0) {
      const inlineTargetsBySegmentIndex = new Map(
        node.inlineTargets.map((target) => [target.segmentIndex, target]),
      );
      return {
        ...node.line,
        segments: node.line.segments?.map((segment, segmentIndex) => {
          const inlineTarget = inlineTargetsBySegmentIndex.get(segmentIndex);
          if (!inlineTarget) {
            return segment;
          }
          return {
            ...segment,
            tone: inlineTarget.targetId === selectedTargetId ? "selected" : "accent",
          };
        }),
      };
    }

    if (selectedTargetId && node.target && node.id === selectedTargetId) {
      return {
        ...node.line,
        tone: "selected",
      };
    }

    if (node.target) {
      return {
        ...node.line,
        tone: "accent",
      };
    }

    if (options.activeSectionId && node.kind === "sectionHeading" && node.sectionId === options.activeSectionId) {
      return {
        ...node.line,
        tone: "accent",
      };
    }

    return node.line;
  });
}
