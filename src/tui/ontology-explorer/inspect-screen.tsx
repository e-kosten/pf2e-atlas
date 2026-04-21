import React from "react";

import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { normalizeSearchCategory } from "../../domain/categories.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import type { OntologyDomainModel, OntologyNodeQuery } from "../../domain/ontology-types.js";
import {
  FilterExplorerScreen,
  type FilterExplorerLaunchIntent,
  type FilterExplorerOptions,
} from "../filter-explorer/index.js";
import type {
  FilterExplorerComposeTarget,
  FilterExplorerScalarClause,
  FilterExplorerScalarEditRequest,
} from "../filter-explorer/types.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { RouteTransitionStatus } from "../route-transition-status.js";
import {
  promptNumericScalarClause,
  type NumericScalarClauseDraft,
} from "../search-screen/scalar-editor.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";

export type OntologyInspectExplorerSnapshot = NonNullable<FilterExplorerOptions["initialSnapshot"]>;
export type OntologyInspectRouteData = {
  model: OntologyDomainModel;
  snapshot?: OntologyInspectExplorerSnapshot;
};

function buildOntologyInspectScalarTarget(node: OntologyDomainModel["rootNodes"][number]): FilterExplorerComposeTarget | undefined {
  if (node.kind !== "metric" || node.query?.kind !== "listRecords") {
    return undefined;
  }

  const match = node.id.match(/^(.*):(actorMetrics|itemMetrics):([^:]+)$/);
  if (!match) {
    return undefined;
  }

  const metricField = match[2] as "actorMetrics" | "itemMetrics";
  const metricKey = match[3]!;
  const valueType =
    metricField === "actorMetrics" ? inferActorMetricValueType(metricKey) : inferItemMetricValueType(metricKey);
  if (valueType !== "number") {
    return undefined;
  }

  const fieldLabel = getMetricDiscoveryGroupLabel(normalizeSearchCategory(node.query.filters.category) ?? null, metricField);

  return {
    kind: "scalar",
    key: `${metricField === "actorMetrics" ? "actorMetric" : "itemMetric"}:${metricKey}`,
    fieldLabel,
    subjectLabel: node.label,
    valueType,
    editorLabel: `${fieldLabel} / ${node.label}`,
  };
}

function toNumericScalarDraft(
  clause: FilterExplorerScalarClause | undefined,
): NumericScalarClauseDraft | null {
  if (!clause) {
    return null;
  }

  if (clause.operator === "between") {
    return { op: "between", min: clause.min, max: clause.max };
  }

  if (typeof clause.value !== "number") {
    return null;
  }

  return { op: clause.operator, value: clause.value };
}

function toFilterExplorerScalarClause(
  clause: NumericScalarClauseDraft | null | undefined,
): FilterExplorerScalarClause | null | undefined {
  if (clause === undefined) {
    return undefined;
  }

  if (clause === null) {
    return null;
  }

  return clause.op === "between"
    ? { operator: "between", min: clause.min, max: clause.max }
    : { operator: clause.op, value: clause.value };
}

export function OntologyInspectScreen({
  routeData,
  onExit,
  onOpenQuery,
  transitionStatus,
}: {
  routeData: OntologyInspectRouteData;
  onExit: () => void;
  onOpenQuery?: (
    query: OntologyNodeQuery,
    snapshot: OntologyInspectExplorerSnapshot,
    launchIntent: FilterExplorerLaunchIntent,
  ) => void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const { model, snapshot: initialSnapshot } = routeData;
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const resolveInspectTarget = React.useCallback((node: OntologyDomainModel["rootNodes"][number] | undefined) => {
    return node ? buildOntologyInspectScalarTarget(node) : undefined;
  }, []);
  const onEditScalarTarget = React.useCallback(
    async ({ target, currentClause }: FilterExplorerScalarEditRequest) => {
      if (target.kind !== "scalar" || target.valueType !== "number") {
        return undefined;
      }

      const nextClause = await promptNumericScalarClause(prompts, terminal, {
        title: target.editorLabel ?? `${target.fieldLabel} / ${target.subjectLabel}`,
        currentClause: toNumericScalarDraft(currentClause),
      });

      return toFilterExplorerScalarClause(nextClause);
    },
    [prompts, terminal],
  );

  return (
    <FilterExplorerScreen
      title={model.label}
      model={model}
      initialSnapshot={initialSnapshot}
      onExit={onExit}
      transitionStatus={transitionStatus}
      mode={{
        kind: "inspect-and-open",
        resolveInspectTarget,
        onEditScalarTarget,
        onOpenQuery,
      }}
    />
  );
}
