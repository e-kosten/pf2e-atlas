import React from "react";

import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import type { OntologyDomainId, OntologyDomainModel, OntologyNodeQuery } from "../../domain/ontology-types.js";
import { FilterExplorerScreen, type FilterExplorerOptions } from "../filter-explorer/index.js";
import type {
  FilterExplorerComposeTarget,
  FilterExplorerScalarClause,
  FilterExplorerScalarEditRequest,
} from "../filter-explorer/types.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import {
  promptNumericScalarClause,
  type NumericScalarClauseDraft,
} from "../search-screen/scalar-editor.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";

type OntologyInspectDomainSummary = {
  id: OntologyDomainId;
  label: string;
  description: string;
};

type OntologyInspectModelSource = {
  listDomains: () => readonly OntologyInspectDomainSummary[];
  loadDomain: (id: OntologyDomainId) => OntologyDomainModel;
};

export type OntologyInspectExplorerSnapshot = NonNullable<FilterExplorerOptions["initialSnapshot"]>;

function buildExplorerFilterText(...parts: Array<string | undefined>): string {
  return parts
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0)
    .join(" ")
    .toLowerCase();
}

function buildOntologyInspectDomainNode(
  domain: OntologyInspectDomainSummary,
  ontology: OntologyInspectModelSource,
) {
  return {
    id: `domain:${domain.id}`,
    kind: "domain" as const,
    label: domain.label,
    listLabel: domain.label,
    filterText: buildExplorerFilterText(domain.id, domain.label, domain.description),
    detailTitle: "Ontology Domain",
    detailLines: [
      { text: `Domain: ${domain.id}` },
      { text: "" },
      {
        text:
          domain.description || "Inspect the domain in the shared explorer and open search queries from matching entries.",
      },
    ],
    loadChildren: () => ontology.loadDomain(domain.id).rootNodes,
  };
}

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

  const fieldLabel = metricField === "actorMetrics" ? "Actor Metric" : "Item Metric";

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

export function buildOntologyInspectExplorerModel(ontology: OntologyInspectModelSource): OntologyDomainModel {
  const domains = ontology.listDomains();
  return {
    id: "searchSemantics",
    label: "Ontology Browser",
    description: "Browse ontology-backed domains and open shared browse/search queries from focused entries.",
    rootNodes: domains.map((domain) => buildOntologyInspectDomainNode(domain, ontology)),
  };
}

export function OntologyInspectScreen({
  initialSnapshot,
  model,
  onExit,
  onOpenQuery,
}: {
  initialSnapshot?: OntologyInspectExplorerSnapshot;
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyInspectExplorerSnapshot) => void;
}): React.JSX.Element {
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
      mode={{
        kind: "inspect-and-open",
        resolveInspectTarget,
        onEditScalarTarget,
        onOpenQuery,
      }}
    />
  );
}
