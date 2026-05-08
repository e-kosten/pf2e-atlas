import {
  buildAllOfFilter,
  type SearchFilterNode,
} from "../../domain/search-request-types.js";
import { createEmptyFilterExplorerComposeDraft, isFilterExplorerScalarTarget } from "./compose-state.js";
import { createFilterExplorerBrowserSnapshot } from "./controller-state.js";
import type { FilterExplorerKeyContext } from "./controller-types.js";
import { describeFilterExplorerHostNode } from "./host-adapter.js";
import { SEARCH_FILTER_OPERATOR_VOCABULARY } from "../../domain/search-filter-operators.js";
import {
  FILTER_EXPLORER_VOCABULARY,
  type FilterExplorerActivationStyle,
  type FilterExplorerComposeTarget,
  type FilterExplorerInspectAndOpenMode,
  type FilterExplorerInspectResult,
  type FilterExplorerSelectTargetOutcome,
  type FilterExplorerLaunchIntent,
  type FilterExplorerNode,
  type FilterExplorerOptions,
  type FilterExplorerQueryOpenIntent,
  type FilterExplorerQueryTarget,
  type FilterExplorerScalarClause,
} from "./types.js";
import { SEARCH_REQUEST_VOCABULARY } from "../../domain/search-request-types.js";

export function resolveFilterExplorerLaunchIntent(
  mode: FilterExplorerInspectAndOpenMode,
  query: FilterExplorerQueryTarget,
): FilterExplorerLaunchIntent {
  if (query.request.mode !== "browse") {
    return FILTER_EXPLORER_VOCABULARY.LAUNCH.INTENT.EDITOR;
  }

  return mode.defaultListRecordLaunchIntent ?? FILTER_EXPLORER_VOCABULARY.LAUNCH.INTENT.RESULTS;
}

export function buildFilterExplorerInspectResult(
  mode: FilterExplorerInspectAndOpenMode,
  node: FilterExplorerNode | undefined,
  target?: FilterExplorerComposeTarget,
): FilterExplorerInspectResult | undefined {
  if (!node?.query) {
    return undefined;
  }

  return {
    node,
    query: node.query,
    target,
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
): SearchFilterNode | null {
  if (!isFilterExplorerScalarTarget(target) || target.valueType !== "number") {
    return null;
  }

  const metricTarget = parseInspectScalarTargetKey(target.key);
  if (!metricTarget) {
    return null;
  }

  if (clause.operator === "between") {
    return {
      kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF,
      children: [
        {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC,
          metric: metricTarget.metric,
          op: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.GTE,
          value: clause.min,
        },
        {
          kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC,
          metric: metricTarget.metric,
          op: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.LTE,
          value: clause.max,
        },
      ],
    };
  }

  const operator =
    clause.operator === "eq"
      ? SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.EQ
      : clause.operator === "notEq"
        ? SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY.NOT_EQ
        : clause.operator === "gt"
          ? SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.GT
        : clause.operator === "gte"
          ? SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.GTE
          : clause.operator === "lt"
            ? SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.LT
          : SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND.LTE;

  return {
    kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC,
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
      : clause.operator === "notEq"
        ? "!="
        : clause.operator === "gt"
          ? ">"
        : clause.operator === "gte"
          ? ">="
          : clause.operator === "lt"
            ? "<"
          : "<=";
  return `${operator} ${clause.value}`;
}

export function buildCompiledFilterExplorerInspectResult(
  result: FilterExplorerInspectResult,
  clause: FilterExplorerScalarClause,
): FilterExplorerInspectResult | null {
  const request = result.query.request;
  if (request.mode !== "browse" || !isFilterExplorerScalarTarget(result.target)) {
    return null;
  }

  const metadataFilter = buildInspectScalarPredicate(result.target, clause);
  if (!metadataFilter) {
    return null;
  }

  return {
    ...result,
    query: {
      ...result.query,
      label: `Browse records where ${result.target.subjectLabel} ${formatInspectScalarClauseSummary(clause)}`,
      request: {
        ...request,
        filter: buildAllOfFilter([request.filter, metadataFilter]),
      },
    },
    launchIntent: FILTER_EXPLORER_VOCABULARY.LAUNCH.INTENT.RESULTS,
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

function resolveFilterExplorerInspectActivationStyle(
  options: Pick<FilterExplorerOptions, "host">,
  result: FilterExplorerInspectResult,
): FilterExplorerActivationStyle {
  return describeFilterExplorerHostNode({
    host: options.host,
    node: result.node,
    target: result.target,
    isFocused: true,
  })?.activationStyle ??
    (result.target?.kind === "scalar"
      ? FILTER_EXPLORER_VOCABULARY.ACTIVATION_STYLE.EDIT
      : FILTER_EXPLORER_VOCABULARY.ACTIVATION_STYLE.OPEN);
}

function buildFilterExplorerSelectTargetOutcome(
  options: Pick<FilterExplorerOptions, "host">,
  result: FilterExplorerInspectResult,
  launchIntent: FilterExplorerLaunchIntent,
): FilterExplorerSelectTargetOutcome {
  return {
    kind: "selectTarget",
    activationStyle: resolveFilterExplorerInspectActivationStyle(options, result),
    result: launchIntent === result.launchIntent ? result : { ...result, launchIntent },
    queryIntent: buildFilterExplorerQueryOpenIntent(result.query, launchIntent),
  };
}

function openFilterExplorerInspectResultDirect(
  options: FilterExplorerOptions,
  keyContext: FilterExplorerKeyContext,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== FILTER_EXPLORER_VOCABULARY.MODE_KIND.INSPECT_AND_OPEN || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  options.onOutcome(buildFilterExplorerSelectTargetOutcome(options, result, result.launchIntent), snapshot);
  return true;
}

export function openFilterExplorerInspectResult(args: {
  options: FilterExplorerOptions;
  keyContext: FilterExplorerKeyContext;
  result: FilterExplorerInspectResult | undefined;
}): boolean {
  const { keyContext, options, result } = args;

  if (
    options.mode.kind === FILTER_EXPLORER_VOCABULARY.MODE_KIND.INSPECT_AND_OPEN &&
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

  if (options.mode.kind !== FILTER_EXPLORER_VOCABULARY.MODE_KIND.INSPECT_AND_OPEN || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  options.onOutcome(
    buildFilterExplorerSelectTargetOutcome(options, result, FILTER_EXPLORER_VOCABULARY.LAUNCH.INTENT.EDITOR),
    snapshot,
  );
  return true;
}

export function shouldOpenImmediateFilterExplorerInspectResult(
  node: FilterExplorerNode | undefined,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  return Boolean(result && result.query.request.mode === "browse" && node?.kind !== "record");
}
