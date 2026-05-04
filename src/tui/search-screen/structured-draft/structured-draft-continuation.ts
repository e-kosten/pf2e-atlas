import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type {
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  buildSearchFilterExplorerComposeDraft,
  type SearchFilterExplorerFieldState,
} from "../filter-explorer-field-state.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

export type StructuredDraftExplorerContinuationChange = {
  result: Pf2eTerminalFilterExplorerInsertionResult;
  query: Pf2eTerminalSearchQuery;
  fieldState: SearchFilterExplorerFieldState;
};

export type StructuredDraftExplorerContinuationResult =
  | { kind: "resumeHost"; change?: StructuredDraftExplorerContinuationChange }
  | { kind: "cancel"; change?: StructuredDraftExplorerContinuationChange }
  | { kind: "notOpened" };

export async function runStructuredDraftExplorerContinuation({
  currentNode,
  fieldOption,
  initialFieldState,
  onHostChange,
  openFilterExplorer,
  preservedMetadata,
  query,
  user,
}: {
  currentNode: MetadataFilterNode | null;
  fieldOption: Pf2eTerminalQueryFieldOption;
  initialFieldState?: SearchFilterExplorerFieldState;
  onHostChange?: (change: StructuredDraftExplorerContinuationChange) => void;
  openFilterExplorer: OpenSearchFilterExplorer;
  preservedMetadata?: MetadataFilterNode | null;
  query: Pf2eTerminalSearchQuery;
  user: SearchWorkspaceUser;
}): Promise<StructuredDraftExplorerContinuationResult> {
  if (fieldOption.editor !== "sharedExplorer") {
    return { kind: "notOpened" };
  }

  return new Promise<StructuredDraftExplorerContinuationResult>((resolve, reject) => {
    let settled = false;
    let currentChange: StructuredDraftExplorerContinuationChange | undefined;
    const resolvedPreservedMetadata =
      preservedMetadata ??
      user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).preservedMetadata;

    const buildChange = (
      nextQuery: Pf2eTerminalSearchQuery,
      nextFieldState: SearchFilterExplorerFieldState,
    ): StructuredDraftExplorerContinuationChange => ({
      result: user.search.buildFilterExplorerInsertionResult(
        buildSearchFilterExplorerComposeDraft(nextFieldState),
        {
          preservedMetadata: resolvedPreservedMetadata,
          preferReplace: currentNode !== null,
        },
      ),
      query: nextQuery,
      fieldState: nextFieldState,
    });

    const finish = (result: StructuredDraftExplorerContinuationResult): void => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(result);
    };

    void openFilterExplorer({
      queryOverride: query,
      initialFieldState,
      preservedMetadata: resolvedPreservedMetadata,
      fieldOptions: [fieldOption],
      onQueryChange: (nextQuery, nextFieldState) => {
        currentChange = buildChange(nextQuery, nextFieldState);
        onHostChange?.(currentChange);
      },
      onBack: (nextQuery, nextFieldState) => {
        currentChange = buildChange(nextQuery, nextFieldState);
        finish({ kind: "resumeHost", change: currentChange });
      },
      onExitRoot: (nextQuery, nextFieldState) => {
        currentChange = buildChange(nextQuery, nextFieldState);
        finish({ kind: "resumeHost", change: currentChange });
      },
      onCancel: (nextQuery, nextFieldState) => {
        const change = currentChange ?? buildChange(nextQuery, nextFieldState);
        finish({ kind: "cancel", change });
      },
      singleFieldBehavior: "directValues",
    })
      .then((opened) => {
        if (!opened) {
          finish({ kind: "notOpened" });
        }
      })
      .catch((error) => {
        reject(error instanceof Error ? error : new Error(String(error)));
      });
  });
}
