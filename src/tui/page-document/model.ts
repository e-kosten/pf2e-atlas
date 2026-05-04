import type {
  EntityPageDocument,
  EntityPageSection,
  EntityPageTarget,
} from "../../app/ontology/entity-page.js";
import type { RecordKey } from "../../domain/record-types.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";

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
  nodeId: string;
  sectionId: string;
  target: EntityPageTarget;
};

export type PageDocumentModel = {
  recordKey: RecordKey;
  title: string;
  nodes: PageDocumentNode[];
  sections: PageDocumentSectionModel[];
  sectionAnchors: PageDocumentSectionAnchor[];
  targetNodes: PageDocumentTargetNode[];
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

  if (document.aonLink) {
    const nodeId = "header:external:aon";
    const nodeIndex = pushNode({
      id: "header:external:aon",
      kind: "external",
      line: toTargetLine(document.aonLink),
      target: document.aonLink,
      anchorRole: "target",
    });
    sections.push({
      id: "header",
      kind: "identity",
      title: "Actions",
      startNodeIndex: nodeIndex,
      endNodeIndex: nodeIndex,
      targetNodeIds: [nodeId],
    });
    sectionAnchors.push({
      sectionId: "header",
      nodeIndex,
    });
    targetNodes.push({
      nodeId,
      sectionId: "header",
      target: document.aonLink,
    });
  }

  if (document.traits.length > 0) {
    const traitTargets = document.traitTargets ?? [];
    if (traitTargets.length > 0) {
      const startNodeIndex = nodes.length;
      const targetNodeIds: string[] = [];
      const headingIndex = pushNode({
        id: "header:traits:heading",
        kind: "sectionHeading",
        sectionId: "traits",
        line: { text: "Traits", tone: "section" },
        anchorRole: "sectionStart",
      });

      traitTargets.forEach((target, targetIndex) => {
        const nodeId = `header:traits:target:${targetIndex}`;
        pushNode({
          id: nodeId,
          kind: "target",
          sectionId: "traits",
          line: toTargetLine(target),
          target,
          anchorRole: "target",
        });
        targetNodeIds.push(nodeId);
        targetNodes.push({
          nodeId,
          sectionId: "traits",
          target,
        });
      });

      sections.push({
        id: "traits",
        kind: "traits",
        title: "Traits",
        startNodeIndex,
        endNodeIndex: Math.max(startNodeIndex, nodes.length - 1),
        targetNodeIds,
      });
      sectionAnchors.push({
        sectionId: "traits",
        nodeIndex: headingIndex,
      });
    } else {
      pushNode({
        id: "header:traits",
        kind: "traits",
        line: { text: `Traits: ${document.traits.join(", ")}`, indent: 2 },
        anchorRole: "content",
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
        const nodeIndex = pushNode({
          id: `section:${section.id}:text:${blockIndex}`,
          kind: "text",
          sectionId: section.id,
          line: { text: block.text, indent: 2 },
          anchorRole: "content",
        });
        anchorNodeIndex ??= nodeIndex;
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
          nodeId,
          sectionId: section.id,
          target,
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
    selectedTargetNodeId?: string | null;
  } = {},
): DerivedTagTerminalLine[] {
  return model.nodes.map((node) => {
    if (options.selectedTargetNodeId && node.id === options.selectedTargetNodeId) {
      return {
        ...node.line,
        tone: "selected",
      };
    }

    if (options.activeSectionId && node.kind === "sectionHeading" && node.sectionId === options.activeSectionId) {
      return {
        ...node.line,
        tone: "selected",
      };
    }

    return node.line;
  });
}
