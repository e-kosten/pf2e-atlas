import React from "react";

import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { normalizeSearchCategory } from "../../domain/categories.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type { EntityPageDocument, EntityPageTarget } from "../../app/ontology/entity-page.js";
import { findSearchScopeFilter } from "../../domain/search-request-types.js";
import { usePf2eTerminalAppServices } from "../app-service-context.js";
import {
  createInspectFilterExplorerHostAdapter,
} from "../filter-explorer/host-adapter.js";
import {
  createFilterExplorerDiscoveryState,
  createFilterExplorerNumericScalarEditHandler,
  createFilterExplorerOutcomeHandler,
} from "../filter-explorer/host-helpers.js";
import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import type {
  FilterExplorerModeSwitchOption,
  FilterExplorerOptions,
  FilterExplorerSelectTargetOutcome,
} from "../filter-explorer/types.js";
import type {
  FilterExplorerComposeTarget,
  FilterExplorerDiscoveryState,
} from "../filter-explorer/types.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { buildPageDocumentModel } from "../page-document/model.js";
import type { RouteTransitionStatus } from "../route-transition-status.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";

export type OntologyInspectExplorerSnapshot = NonNullable<FilterExplorerOptions["initialSnapshot"]>;
export type OntologyInspectRouteData = {
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  snapshot?: OntologyInspectExplorerSnapshot;
};

const SEARCH_DISCOVERY_MODE_OPTIONS: readonly FilterExplorerModeSwitchOption<SearchFilterDiscoveryMode>[] = [
  {
    value: "matching",
    label: "Matching Counts",
    description: "Show values and counts from the current matching query context.",
  },
  {
    value: "catalog",
    label: "Catalog Counts",
    description: "Show values and counts from the wider applicability slice only.",
  },
];

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

export function OntologyInspectScreen({
  routeData,
  onExit,
  onSelectTarget,
  onActivatePageTarget,
  transitionStatus,
}: {
  routeData: OntologyInspectRouteData;
  onExit: () => void;
  onSelectTarget?: (outcome: FilterExplorerSelectTargetOutcome, snapshot: OntologyInspectExplorerSnapshot) => void;
  onActivatePageTarget?: (target: EntityPageTarget) => boolean | void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const { user } = usePf2eTerminalAppServices();
  const { snapshot: initialSnapshot } = routeData;
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = routeData.initialDiscoveryMode ?? "matching";
  const [model, setModel] = React.useState(routeData.model);
  const [discoveryMode, setDiscoveryMode] = React.useState<SearchFilterDiscoveryMode>(initialDiscoveryMode);
  const [previewDocumentsByRecordKey, setPreviewDocumentsByRecordKey] = React.useState<Map<string, EntityPageDocument>>(
    () => new Map(),
  );
  const refreshRequestIdRef = React.useRef(0);

  React.useEffect(() => {
    setModel(routeData.model);
    setDiscoveryMode(initialDiscoveryMode);
    setPreviewDocumentsByRecordKey(new Map());
  }, [initialDiscoveryMode, routeData.model]);

  const resolveInspectTarget = React.useCallback((node: OntologyDomainModel["rootNodes"][number] | undefined) => {
    return node ? buildOntologyInspectScalarTarget(node) : undefined;
  }, []);
  const onEditScalarTarget = React.useMemo(
    () => createFilterExplorerNumericScalarEditHandler(prompts, terminal),
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
  const discovery = React.useMemo<FilterExplorerDiscoveryState<SearchFilterDiscoveryMode> | undefined>(
    () =>
      createFilterExplorerDiscoveryState({
        mode: discoveryMode,
        modes: SEARCH_DISCOVERY_MODE_OPTIONS,
        onModeChange: onDiscoveryModeChange,
        enabled: Boolean(routeData.loadModelForDiscoveryMode),
      }),
    [discoveryMode, onDiscoveryModeChange, routeData.loadModelForDiscoveryMode],
  );

  const handleOutcome = React.useMemo<FilterExplorerOptions["onOutcome"]>(
    () =>
      createFilterExplorerOutcomeHandler({
        onBack: onExit,
        onExitRoot: onExit,
        onCancel: onExit,
        onSelectTarget,
      }),
    [onExit, onSelectTarget],
  );
  const resolvePageDocument = React.useCallback(
    (node: OntologyDomainModel["rootNodes"][number] | undefined) => {
      if (node?.kind !== "record" || !node.id) {
        return null;
      }

      const document =
        previewDocumentsByRecordKey.get(node.id) ??
        user.entityPages.buildDocumentByRecordKey(node.id, { recordTargetAction: "preview" });
      return document ? buildPageDocumentModel(document) : null;
    },
    [previewDocumentsByRecordKey, user.entityPages],
  );
  const host = React.useMemo(
    () =>
      createInspectFilterExplorerHostAdapter({
        resolveTarget: resolveInspectTarget,
        resolvePageDocument,
        activatePageTarget: ({ target, controller }) => {
          if (target.kind === "record" && target.action === "preview") {
            const previewDocument = user.entityPages.buildDocumentByRecordKey(target.recordKey, {
              recordTargetAction: "preview",
            });
            if (!previewDocument) {
              return false;
            }

            const sourceRecordKey = controller.browser.currentNode?.kind === "record"
              ? controller.browser.currentNode.id
              : null;
            if (!sourceRecordKey) {
              return false;
            }

            setPreviewDocumentsByRecordKey((current) => {
              const next = new Map(current);
              next.set(sourceRecordKey, previewDocument);
              return next;
            });
            return true;
          }

          return onActivatePageTarget?.(target);
        },
        onEditScalarTarget,
      }),
    [onActivatePageTarget, onEditScalarTarget, resolveInspectTarget, resolvePageDocument, user.entityPages],
  );

  return (
    <FilterExplorerScreen
      title={model.label}
      model={model}
      host={host}
      initialSnapshot={initialSnapshot}
      onOutcome={handleOutcome}
      discovery={discovery}
      transitionStatus={transitionStatus}
      mode={{
        kind: "inspect-and-open",
        onEditScalarTarget,
      }}
    />
  );
}
