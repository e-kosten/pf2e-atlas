import type { MetadataFilterNode } from "../../domain/metadata-filter-types.js";
import {
  metadataFilterNodeToSearchRequestParts,
} from "../../domain/search-request-types.js";
import { createEmptyFilterExplorerComposeDraft, isFilterExplorerScalarTarget } from "./compose-state.js";
import { createFilterExplorerBrowserSnapshot } from "./controller-state.js";
import type { FilterExplorerKeyContext } from "./controller-types.js";
import {
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerInspectAndOpenMode,
  type FilterExplorerInspectResult,
  type FilterExplorerLaunchIntent,
  type FilterExplorerNode,
  type FilterExplorerOptions,
  type FilterExplorerQueryOpenIntent,
  type FilterExplorerQueryTarget,
  type FilterExplorerScalarClause,
} from "./types.js";

export function resolveFilterExplorerLaunchIntent(
  mode: FilterExplorerInspectAndOpenMode,
  query: FilterExplorerQueryTarget,
): FilterExplorerLaunchIntent {
  if (query.request.intent !== "browse") {
    return FILTER_EXPLORER_LAUNCH_INTENT.EDITOR;
  }

  return mode.defaultListRecordLaunchIntent ?? FILTER_EXPLORER_LAUNCH_INTENT.RESULTS;
}

export function buildFilterExplorerInspectResult(
  mode: FilterExplorerInspectAndOpenMode,
  node: FilterExplorerNode | undefined,
): FilterExplorerInspectResult | undefined {
  if (!node?.query) {
    return undefined;
  }

  return {
    node,
    query: node.query,
    target: mode.resolveInspectTarget?.(node),
    launchIntent: resolveFilterExplorerLaunchIntent(mode, node.query),
  };
}

function parseInspectScalarTargetKey(key: string): { field: "actorMetric" | "itemMetric"; metric: string } | null {
  const match = key.match(/^(actorMetric|itemMetric):(.+)$/);
  if (!match) {
    return null;
  }

  return {
    field: match[1] as "actorMetric" | "itemMetric",
    metric: match[2]!,
  };
}

function buildInspectScalarPredicate(
  target: Exclude<FilterExplorerInspectResult["target"], undefined>,
  clause: FilterExplorerScalarClause,
): MetadataFilterNode | null {
  if (!isFilterExplorerScalarTarget(target) || target.valueType !== "number") {
    return null;
  }

  const metricTarget = parseInspectScalarTargetKey(target.key);
  if (!metricTarget) {
    return null;
  }

  if (clause.operator === "between") {
    return {
      and: [
        {
          field: metricTarget.field,
          metric: metricTarget.metric,
          op: ">=",
          value: clause.min,
        },
        {
          field: metricTarget.field,
          metric: metricTarget.metric,
          op: "<=",
          value: clause.max,
        },
      ],
    };
  }

  const operator =
    clause.operator === "eq"
      ? "=="
      : clause.operator === "neq"
        ? "!="
        : clause.operator === "gte"
          ? ">="
          : "<=";

  return {
    field: metricTarget.field,
    metric: metricTarget.metric,
    op: operator,
    value: clause.value as number,
  };
}

function formatInspectScalarClauseSummary(clause: FilterExplorerScalarClause): string {
  if (clause.operator === "between") {
    return `between ${clause.min} and ${clause.max}`;
  }

  const operator =
    clause.operator === "eq"
      ? "="
      : clause.operator === "neq"
        ? "!="
        : clause.operator === "gte"
          ? ">="
          : "<=";
  return `${operator} ${clause.value}`;
}

export function buildCompiledFilterExplorerInspectResult(
  result: FilterExplorerInspectResult,
  clause: FilterExplorerScalarClause,
): FilterExplorerInspectResult | null {
  const request = result.query.request;
  if (request.intent !== "browse" || !isFilterExplorerScalarTarget(result.target)) {
    return null;
  }

  const metadata = buildInspectScalarPredicate(result.target, clause);
  if (!metadata) {
    return null;
  }

  return {
    ...result,
    query: {
      ...result.query,
      label: `Browse records where ${result.target.subjectLabel} ${formatInspectScalarClauseSummary(clause)}`,
      request: {
        ...request,
        parts: [
          ...(request.parts ?? []).filter(
            (part) => part.kind !== "metadataPredicate" && part.kind !== "metadataGroup" && part.kind !== "metadataNot",
          ),
          ...metadataFilterNodeToSearchRequestParts(metadata),
        ],
      },
    },
    launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.RESULTS,
  };
}

function buildFilterExplorerQueryOpenIntent(
  query: FilterExplorerQueryTarget,
  launchIntent: FilterExplorerLaunchIntent,
): FilterExplorerQueryOpenIntent {
  return {
    query,
    launchIntent,
  };
}

function openFilterExplorerInspectResultDirect(
  options: FilterExplorerOptions,
  keyContext: FilterExplorerKeyContext,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(result, snapshot);
    return true;
  }
  if (options.mode.onOpenQueryIntent) {
    options.mode.onOpenQueryIntent(buildFilterExplorerQueryOpenIntent(result.query, result.launchIntent), snapshot);
    return true;
  }
  return false;
}

export function openFilterExplorerInspectResult(args: {
  options: FilterExplorerOptions;
  keyContext: FilterExplorerKeyContext;
  result: FilterExplorerInspectResult | undefined;
}): boolean {
  const { keyContext, options, result } = args;

  if (
    options.mode.kind === "inspect-and-open" &&
    result &&
    isFilterExplorerScalarTarget(result.target) &&
    options.mode.onEditScalarTarget
  ) {
    void Promise.resolve(
      options.mode.onEditScalarTarget({
        target: result.target,
        draft: createEmptyFilterExplorerComposeDraft(),
      }),
    ).then((nextClause) => {
      if (nextClause === undefined || nextClause === null) {
        return;
      }

      const compiledResult = buildCompiledFilterExplorerInspectResult(result, nextClause);
      if (!compiledResult) {
        return;
      }

      openFilterExplorerInspectResultDirect(options, keyContext, compiledResult);
    });
    return true;
  }

  return openFilterExplorerInspectResultDirect(options, keyContext, result);
}

export function openFilterExplorerInspectQuery(args: {
  options: FilterExplorerOptions;
  keyContext: FilterExplorerKeyContext;
  result: FilterExplorerInspectResult | undefined;
}): boolean {
  const { keyContext, options, result } = args;

  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  if (options.mode.onOpenQueryIntent) {
    options.mode.onOpenQueryIntent(
      buildFilterExplorerQueryOpenIntent(result.query, FILTER_EXPLORER_LAUNCH_INTENT.EDITOR),
      snapshot,
    );
    return true;
  }
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(
      {
        ...result,
        launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
      },
      snapshot,
    );
    return true;
  }
  return false;
}

export function shouldOpenImmediateFilterExplorerInspectResult(
  node: FilterExplorerNode | undefined,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  return Boolean(result && result.query.request.intent === "browse" && node?.kind !== "record");
}
