import React from "react";

import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { normalizeSearchCategory } from "../../domain/categories.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import { findSearchScopeFilter } from "../../domain/search-request-types.js";
import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import type { FilterExplorerOptions, FilterExplorerQueryOpenIntent } from "../filter-explorer/types.js";
import type {
  FilterExplorerComposeTarget,
  FilterExplorerDiscoveryState,
  FilterExplorerScalarClause,
  FilterExplorerScalarEditRequest,
} from "../filter-explorer/types.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { RouteTransitionStatus } from "../route-transition-status.js";
import { promptNumericScalarClause, type NumericScalarClauseDraft } from "../filter-explorer/scalar-editor.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";

export type OntologyInspectExplorerSnapshot = NonNullable<FilterExplorerOptions["initialSnapshot"]>;
export type OntologyInspectRouteData = {
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  snapshot?: OntologyInspectExplorerSnapshot;
};

function buildOntologyInspectScalarTarget(
  node: OntologyDomainModel["rootNodes"][number],
): FilterExplorerComposeTarget | undefined {
  const request = node.query?.request ?? null;
  if (node.kind !== "metric" || request?.mode !== "browse") {
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

  const scope = findSearchScopeFilter(request.filter);
  const fieldLabel = getMetricDiscoveryGroupLabel(
    normalizeSearchCategory(scope?.category ?? "") ?? null,
    metricField,
  );

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
  onOpenQueryIntent,
  transitionStatus,
}: {
  routeData: OntologyInspectRouteData;
  onExit: () => void;
  onOpenQueryIntent?: (intent: FilterExplorerQueryOpenIntent, snapshot: OntologyInspectExplorerSnapshot) => void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const { snapshot: initialSnapshot } = routeData;
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = routeData.initialDiscoveryMode ?? "matching";
  const [model, setModel] = React.useState(routeData.model);
  const [discoveryMode, setDiscoveryMode] = React.useState<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const refreshRequestIdRef = React.useRef(0);

  React.useEffect(() => {
    setModel(routeData.model);
    setDiscoveryMode(initialDiscoveryMode);
  }, [initialDiscoveryMode, routeData.model]);

  const resolveInspectTarget = React.useCallback((node: OntologyDomainModel["rootNodes"][number] | undefined) => {
    return node ? buildOntologyInspectScalarTarget(node) : undefined;
  }, []);
  const onEditScalarTarget = React.useCallback(
    async ({ target, currentClause }: FilterExplorerScalarEditRequest) => {
      if (target.valueType !== "number") {
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
  const onDiscoveryModeChange = React.useCallback(
    (nextMode: SearchFilterDiscoveryMode) => {
      if (nextMode === discoveryMode) {
        return;
      }

      if (!routeData.loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        return;
      }

      const requestId = refreshRequestIdRef.current + 1;
      refreshRequestIdRef.current = requestId;
      void routeData
        .loadModelForDiscoveryMode(nextMode)
        .then((nextModel) => {
          if (refreshRequestIdRef.current !== requestId) {
            return;
          }
          setModel(nextModel);
          setDiscoveryMode(nextMode);
        })
        .catch((error) => {
          if (refreshRequestIdRef.current !== requestId) {
            return;
          }
          void terminal.pauseForAnyKey(`Could not refresh explorer data.\n\n${(error as Error).message}`);
        });
    },
    [discoveryMode, routeData, terminal],
  );
  const discovery = React.useMemo<FilterExplorerDiscoveryState>(
    () => ({
      mode: discoveryMode,
      onModeChange: onDiscoveryModeChange,
    }),
    [discoveryMode, onDiscoveryModeChange],
  );

  return (
    <FilterExplorerScreen
      title={model.label}
      model={model}
      initialSnapshot={initialSnapshot}
      onExit={onExit}
      discovery={discovery}
      transitionStatus={transitionStatus}
      mode={{
        kind: "inspect-and-open",
        resolveInspectTarget,
        onEditScalarTarget,
        onOpenQueryIntent,
      }}
    />
  );
}
