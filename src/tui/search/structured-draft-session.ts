export type SearchStructuredDraftAnchor =
  | { kind: "addQueryPart" }
  | { kind: "queryTreeRoot" }
  | {
      kind: "queryNode";
      path: number[];
    };

export type SearchStructuredDraftEntryKind =
  | "queryTreeRoot"
  | "queryNode"
  | "queryFieldBucket"
  | "queryInsertionSlot";

export type SearchStructuredDraftEntry = {
  kind: SearchStructuredDraftEntryKind;
  key: string;
  label: string;
  value?: string;
  description: string;
  disabled?: boolean;
  disabledReason?: string;
  treePath?: number[];
  groupPath?: number[];
  field?: string;
  fieldOperator?: "include" | "exclude";
  memberPaths?: number[][];
  fieldMemberPaths?: number[][];
  insertionPath?: number[];
  indent?: number;
  menuLabel?: string;
};

export function clampStructuredDraftSelection(index: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(index, itemCount - 1));
}
