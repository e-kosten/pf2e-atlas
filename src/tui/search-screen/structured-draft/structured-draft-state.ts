import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import { getSearchFilterNodeAtPath, isSearchFilterBooleanGroup } from "../../search/query-core.js";
import {
  clampStructuredDraftSelection,
  type SearchStructuredDraftAnchor,
  type SearchStructuredDraftEntry,
  type SearchStructuredDraftEntryKind,
} from "../../search/structured-draft-session.js";

export type StructuredDraftResumeTarget =
  | { kind: "root" }
  | { kind: "group"; groupPath: number[] }
  | { kind: "node"; path: number[] };

export type SearchStructuredDraftState = {
  anchor: SearchStructuredDraftAnchor;
  resumeTarget: StructuredDraftResumeTarget;
  moveSourcePath: number[] | null;
  selectedIndex: number;
};

export function createStructuredDraftRootResumeTarget(): StructuredDraftResumeTarget {
  return { kind: "root" };
}

export function createStructuredDraftGroupResumeTarget(groupPath: number[]): StructuredDraftResumeTarget {
  return { kind: "group", groupPath: [...groupPath] };
}

export function createStructuredDraftNodeResumeTarget(path: number[]): StructuredDraftResumeTarget {
  return { kind: "node", path: [...path] };
}

export function createStructuredDraftResumeTargetForNodePath(
  filter: SearchFilterNode | undefined,
  path: number[],
): StructuredDraftResumeTarget {
  const node = getSearchFilterNodeAtPath(filter, path);
  if (node && isSearchFilterBooleanGroup(node)) {
    return createStructuredDraftGroupResumeTarget(path);
  }

  return createStructuredDraftNodeResumeTarget(path);
}

export function createStructuredDraftResumeTargetForContainingGroup(
  filter: SearchFilterNode | undefined,
  path: number[],
): StructuredDraftResumeTarget {
  const groupPath = findStructuredDraftCanonicalGroupPath(filter, path);
  if (groupPath) {
    return createStructuredDraftGroupResumeTarget(groupPath);
  }

  return createStructuredDraftRootResumeTarget();
}

export function findStructuredDraftCanonicalGroupPath(
  filter: SearchFilterNode | undefined,
  path: readonly number[],
): number[] | null {
  for (let depth = path.length; depth >= 0; depth -= 1) {
    const candidatePath = path.slice(0, depth);
    const node = getSearchFilterNodeAtPath(filter, candidatePath);
    if (node && isSearchFilterBooleanGroup(node)) {
      return candidatePath;
    }
  }

  return null;
}

export function canonicalizeStructuredDraftResumeTarget(
  filter: SearchFilterNode | undefined,
  target: StructuredDraftResumeTarget | null | undefined,
): StructuredDraftResumeTarget {
  if (!target || target.kind === "root") {
    return createStructuredDraftRootResumeTarget();
  }

  if (target.kind === "node") {
    return getSearchFilterNodeAtPath(filter, target.path)
      ? createStructuredDraftNodeResumeTarget(target.path)
      : createStructuredDraftResumeTargetForContainingGroup(filter, target.path);
  }

  const groupPath = findStructuredDraftCanonicalGroupPath(filter, target.groupPath);
  return groupPath ? createStructuredDraftGroupResumeTarget(groupPath) : createStructuredDraftRootResumeTarget();
}

export function createStructuredDraftResumeTargetForEntryContext(
  filter: SearchFilterNode | undefined,
  entry: SearchStructuredDraftEntry | null | undefined,
): StructuredDraftResumeTarget {
  if (!entry) {
    return createStructuredDraftRootResumeTarget();
  }

  switch (entry.kind) {
    case "queryFieldBucket":
      return createStructuredDraftGroupResumeTarget(entry.groupPath ?? []);
    case "queryInsertionSlot":
      return createStructuredDraftGroupResumeTarget(entry.insertionPath ?? []);
    case "queryTreeRoot":
      return createStructuredDraftGroupResumeTarget(entry.treePath ?? []);
    case "queryNode":
      return createStructuredDraftResumeTargetForContainingGroup(filter, entry.treePath ?? []);
  }
}

export function getStructuredDraftResumeFocusPath(target: StructuredDraftResumeTarget | null): number[] | null {
  if (!target) {
    return null;
  }

  switch (target.kind) {
    case "root":
      return [];
    case "group":
      return [...target.groupPath];
    case "node":
      return [...target.path];
  }
}

function pathsMatch(path: number[] | undefined, targetPath: number[]): boolean {
  return Boolean(path) && JSON.stringify(path) === JSON.stringify(targetPath);
}

function getStructuredDraftAnchorKind(anchor: SearchStructuredDraftAnchor): SearchStructuredDraftEntryKind {
  if (anchor.kind === "addQueryPart") {
    return "queryInsertionSlot";
  }

  if (anchor.kind === "queryTreeRoot") {
    return "queryTreeRoot";
  }

  return "queryNode";
}

export function getStructuredDraftSelectionIndex(
  anchor: SearchStructuredDraftAnchor,
  entries: SearchStructuredDraftEntry[],
): number {
  const preferredKind = getStructuredDraftAnchorKind(anchor);

  if (anchor.kind === "queryNode") {
    const matchIndex = entries.findIndex(
      (entry) => entry.kind === "queryNode" && JSON.stringify(entry.treePath ?? []) === JSON.stringify(anchor.path),
    );
    return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : 0, entries.length);
  }

  if (anchor.kind === "addQueryPart") {
    const matchIndex = entries.findIndex(
      (entry) => entry.kind === preferredKind && (entry.insertionPath?.length ?? 0) === 0,
    );
    return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : entries.length - 1, entries.length);
  }

  const matchIndex = entries.findIndex((entry) => entry.kind === preferredKind);
  return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : 0, entries.length);
}

export function getStructuredDraftSelectionIndexForResumeTarget(
  entries: SearchStructuredDraftEntry[],
  target: StructuredDraftResumeTarget | null,
  fallbackIndex = 0,
): number {
  const focusPath = getStructuredDraftResumeFocusPath(target);
  if (!focusPath) {
    return clampStructuredDraftSelection(fallbackIndex, entries.length);
  }

  const exactNodeIndex = entries.findIndex(
    (entry) => entry.kind === "queryNode" && pathsMatch(entry.treePath, focusPath),
  );
  if (exactNodeIndex >= 0 && target?.kind === "node") {
    return clampStructuredDraftSelection(exactNodeIndex, entries.length);
  }

  const rootGroupIndex = entries.findIndex(
    (entry) => entry.kind === "queryTreeRoot" && pathsMatch(entry.treePath, focusPath),
  );
  if (rootGroupIndex >= 0) {
    return clampStructuredDraftSelection(rootGroupIndex, entries.length);
  }

  for (let depth = focusPath.length - 1; depth >= 0; depth -= 1) {
    const parentPath = focusPath.slice(0, depth);
    const ancestorRootIndex = entries.findIndex(
      (entry) => entry.kind === "queryTreeRoot" && pathsMatch(entry.treePath, parentPath),
    );
    if (ancestorRootIndex >= 0 && target?.kind === "group") {
      return clampStructuredDraftSelection(ancestorRootIndex, entries.length);
    }

    const insertionIndex = entries.findIndex(
      (entry) => entry.kind === "queryInsertionSlot" && pathsMatch(entry.insertionPath, parentPath),
    );
    if (insertionIndex >= 0) {
      return clampStructuredDraftSelection(insertionIndex, entries.length);
    }

    const ancestorNodeIndex = entries.findIndex(
      (entry) => entry.kind === "queryNode" && pathsMatch(entry.treePath, parentPath),
    );
    if (ancestorNodeIndex >= 0 && target?.kind === "node") {
      return clampStructuredDraftSelection(ancestorNodeIndex, entries.length);
    }

    if (ancestorRootIndex >= 0) {
      return clampStructuredDraftSelection(ancestorRootIndex, entries.length);
    }
  }

  const rootInsertionIndex = entries.findIndex(
    (entry) => entry.kind === "queryInsertionSlot" && (entry.insertionPath?.length ?? -1) === 0,
  );
  return clampStructuredDraftSelection(rootInsertionIndex >= 0 ? rootInsertionIndex : fallbackIndex, entries.length);
}
