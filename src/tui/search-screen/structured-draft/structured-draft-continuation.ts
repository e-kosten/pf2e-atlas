import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../search/service.js";
import type { FilterExplorerComposeTarget, FilterExplorerSelectTargetOutcome } from "../../filter-explorer/types.js";
import type { SearchFilterExplorerFieldState } from "../filter-explorer-field-state.js";
import type { OpenSearchFilterExplorer, SearchWorkspaceUser } from "../workspace/workspace-action-types.js";

export type StructuredDraftHostMutation =
  | { kind: "replaceNode"; node: SearchFilterNode | null }
  | { kind: "appendNodes"; nodes: SearchFilterNode[] }
  | {
      kind: "replaceGroupedField";
      field: Pf2eTerminalQueryFieldOption["value"];
      fieldOption: Pf2eTerminalQueryFieldOption;
      fieldState: SearchFilterExplorerFieldState;
    };

export type StructuredDraftPromptFlowResult<T> =
  | { kind: "apply"; value: T }
  | { kind: "back" }
  | { kind: "cancel" };

export function structuredDraftPromptApply<T>(value: T): StructuredDraftPromptFlowResult<T> {
  return { kind: "apply", value };
}

export function structuredDraftPromptBack<T = never>(): StructuredDraftPromptFlowResult<T> {
  return { kind: "back" };
}

export function structuredDraftPromptCancel<T = never>(): StructuredDraftPromptFlowResult<T> {
  return { kind: "cancel" };
}

export function buildStructuredDraftGroupedFieldMutation({
  fieldOption,
  fieldState,
}: {
  fieldOption: Pf2eTerminalQueryFieldOption;
  fieldState: SearchFilterExplorerFieldState;
}): StructuredDraftHostMutation {
  return {
    kind: "replaceGroupedField",
    field: fieldOption.value,
    fieldOption,
    fieldState,
  };
}

export type StructuredDraftContinuationChange = {
  mutation: StructuredDraftHostMutation;
  query: Pf2eTerminalSearchQuery;
  fieldState: SearchFilterExplorerFieldState;
};

export type StructuredDraftExplorerContinuationResult =
  | { kind: "resumeHost"; change?: StructuredDraftContinuationChange }
  | {
      kind: "selectTarget";
      outcome: FilterExplorerSelectTargetOutcome;
      query: Pf2eTerminalSearchQuery;
      fieldState: SearchFilterExplorerFieldState;
      discoveryMode: SearchFilterDiscoveryMode;
      change: StructuredDraftContinuationChange;
    }
  | { kind: "cancel"; change?: StructuredDraftContinuationChange }
  | { kind: "notOpened" };

export type StructuredDraftExplorerChildSurfaceResult =
  | { kind: "resumeHost"; query: Pf2eTerminalSearchQuery; fieldState: SearchFilterExplorerFieldState }
  | {
      kind: "selectTarget";
      outcome: FilterExplorerSelectTargetOutcome;
      query: Pf2eTerminalSearchQuery;
      fieldState: SearchFilterExplorerFieldState;
      discoveryMode: SearchFilterDiscoveryMode;
    }
  | { kind: "cancel"; query: Pf2eTerminalSearchQuery; fieldState: SearchFilterExplorerFieldState }
  | { kind: "notOpened" };

export async function runStructuredDraftExplorerContinuation({
  fieldOption,
  buildHostMutation,
  initialDiscoveryMode,
  initialFieldState,
  onHostChange,
  openFilterExplorer,
  preservedMetadata,
  query,
  resolveSelectionTarget,
  singleFieldBehavior = "directValues",
  title,
  user,
}: {
  fieldOption: Pf2eTerminalQueryFieldOption;
  buildHostMutation: (fieldState: SearchFilterExplorerFieldState) => StructuredDraftHostMutation;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  initialFieldState?: SearchFilterExplorerFieldState;
  onHostChange?: (change: StructuredDraftContinuationChange) => void;
  openFilterExplorer: OpenSearchFilterExplorer;
  preservedMetadata?: MetadataFilterNode | null;
  query: Pf2eTerminalSearchQuery;
  resolveSelectionTarget?: (
    node: import("../../../domain/ontology-types.js").OntologyNode | undefined,
  ) => FilterExplorerComposeTarget | undefined;
  singleFieldBehavior?: "list" | "directValues";
  title?: string;
  user: SearchWorkspaceUser;
}): Promise<StructuredDraftExplorerContinuationResult> {
  if (fieldOption.editor !== "sharedExplorer") {
    return { kind: "notOpened" };
  }

  return new Promise<StructuredDraftExplorerContinuationResult>((resolve, reject) => {
    let settled = false;
    let currentChange: StructuredDraftContinuationChange | undefined;
    const resolvedPreservedMetadata =
      preservedMetadata ?? user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).preservedMetadata;

    const buildChange = (
      nextQuery: Pf2eTerminalSearchQuery,
      nextFieldState: SearchFilterExplorerFieldState,
    ): StructuredDraftContinuationChange => ({
      mutation: buildHostMutation(nextFieldState),
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
      title,
      queryOverride: query,
      initialDiscoveryMode,
      initialFieldState,
      preservedMetadata: resolvedPreservedMetadata,
      fieldOptions: [fieldOption],
      resolveSelectionTarget,
      onEvent: (event) => {
        if (event.kind === "change") {
          currentChange = buildChange(event.query, event.fieldState);
          onHostChange?.(currentChange);
          return;
        }
        if (event.kind === "back" || event.kind === "exitRoot") {
          currentChange = buildChange(event.query, event.fieldState);
          finish({ kind: "resumeHost", change: currentChange });
          return;
        }
        if (event.kind === "cancel") {
          const change = currentChange ?? buildChange(event.query, event.fieldState);
          finish({ kind: "cancel", change });
          return;
        }
        currentChange = buildChange(event.query, event.fieldState);
        if (resolveSelectionTarget) {
          finish({
            kind: "selectTarget",
            outcome: event.outcome,
            query: event.query,
            fieldState: event.fieldState,
            discoveryMode: event.discoveryMode,
            change: currentChange,
          });
        }
      },
      singleFieldBehavior,
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

export async function runStructuredDraftExplorerChildSurface({
  fieldOption,
  initialDiscoveryMode,
  initialFieldState,
  openFilterExplorer,
  preservedMetadata,
  query,
  resolveSelectionTarget,
  singleFieldBehavior = "directValues",
  title,
  user,
}: {
  fieldOption: Pf2eTerminalQueryFieldOption;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  initialFieldState?: SearchFilterExplorerFieldState;
  openFilterExplorer: OpenSearchFilterExplorer;
  preservedMetadata?: MetadataFilterNode | null;
  query: Pf2eTerminalSearchQuery;
  resolveSelectionTarget?: (
    node: import("../../../domain/ontology-types.js").OntologyNode | undefined,
  ) => FilterExplorerComposeTarget | undefined;
  singleFieldBehavior?: "list" | "directValues";
  title?: string;
  user: SearchWorkspaceUser;
}): Promise<StructuredDraftExplorerChildSurfaceResult> {
  if (fieldOption.editor !== "sharedExplorer") {
    return { kind: "notOpened" };
  }

  return new Promise<StructuredDraftExplorerChildSurfaceResult>((resolve, reject) => {
    let settled = false;
    const resolvedPreservedMetadata =
      preservedMetadata ?? user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).preservedMetadata;

    const finish = (result: StructuredDraftExplorerChildSurfaceResult): void => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(result);
    };

    void openFilterExplorer({
      title,
      queryOverride: query,
      initialDiscoveryMode,
      initialFieldState,
      preservedMetadata: resolvedPreservedMetadata,
      fieldOptions: [fieldOption],
      resolveSelectionTarget,
      onEvent: (event) => {
        if (event.kind === "change") {
          return;
        }
        if (event.kind === "back" || event.kind === "exitRoot") {
          finish({ kind: "resumeHost", query: event.query, fieldState: event.fieldState });
          return;
        }
        if (event.kind === "cancel") {
          finish({ kind: "cancel", query: event.query, fieldState: event.fieldState });
          return;
        }
        if (resolveSelectionTarget) {
          finish({
            kind: "selectTarget",
            outcome: event.outcome,
            query: event.query,
            fieldState: event.fieldState,
            discoveryMode: event.discoveryMode,
          });
        }
      },
      singleFieldBehavior,
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
