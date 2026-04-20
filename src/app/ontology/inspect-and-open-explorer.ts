import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../domain/categories.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { Pf2eDataService } from "../../data/service.js";
import type { OntologyDomainModel, OntologyNode, OntologyNodeQuery } from "../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import type { Pf2eApplicationOntologyService } from "../ontology-service.js";
import {
  buildNormalizedRecordNode,
  cloneOntologyNode,
} from "./node-helpers.js";

type OntologyExplorerDataService = Pick<Pf2eDataService, "listRecords">;
type OntologyExplorerOntologyService = Pick<Pf2eApplicationOntologyService, "loadSearchSemanticsDomain">;

function buildOntologyQueryRecordChildren(
  dataService: OntologyExplorerDataService,
  query: OntologyNodeQuery | undefined,
): readonly OntologyNode[] {
  if (!query || query.kind !== "listRecords") {
    return [];
  }

  return dataService.listRecords(query.filters).records.map(buildNormalizedRecordNode);
}

function parseMetricInspectScope(node: OntologyNode): {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  metricField: "actorMetrics" | "itemMetrics";
  metricKey: string;
} | null {
  if (node.kind !== "metric") {
    return null;
  }

  const match = node.id.match(/^(.*):(actorMetrics|itemMetrics):([^:]+)$/);
  if (!match) {
    return null;
  }

  const scopeSegments = match[1]!.split(":");
  if (scopeSegments.length < 1 || scopeSegments.length > 2) {
    return null;
  }

  const category = normalizeSearchCategory(scopeSegments[0]) ?? null;
  const subcategory =
    scopeSegments.length === 2 ? (normalizeSearchSubcategory(scopeSegments[1]) ?? null) : null;
  if (!category) {
    return null;
  }

  return {
    category,
    subcategory,
    metricField: match[2] as "actorMetrics" | "itemMetrics",
    metricKey: match[3]!,
  };
}

function buildInspectMetricQuery(node: OntologyNode): OntologyNodeQuery | undefined {
  const scope = parseMetricInspectScope(node);
  if (!scope) {
    return undefined;
  }

  const valueType =
    scope.metricField === "actorMetrics"
      ? inferActorMetricValueType(scope.metricKey)
      : inferItemMetricValueType(scope.metricKey);
  if (valueType !== "number") {
    return undefined;
  }

  return {
    kind: "listRecords",
    label: `Browse records with the ${scope.metricKey} metric`,
    filters: {
      category: scope.category,
      subcategory: scope.subcategory ?? undefined,
      metadata:
        scope.metricField === "actorMetrics"
          ? {
              field: "actorMetricCompare",
              leftMetric: scope.metricKey,
              op: ">=",
              rightMetric: scope.metricKey,
            }
          : {
              field: "itemMetricCompare",
              leftMetric: scope.metricKey,
              op: ">=",
              rightMetric: scope.metricKey,
            },
      limit: 20,
    },
  };
}

function decorateNodeForInspectAndOpen(
  node: OntologyNode,
  dataService: OntologyExplorerDataService,
): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const children = cloned.children?.map((child) => decorateNodeForInspectAndOpen(child, dataService));

  if (children) {
    return {
      ...cloned,
      children,
    };
  }

  if (cloned.loadChildren) {
    return {
      ...cloned,
      loadChildren: () => cloned.loadChildren!().map((child) => decorateNodeForInspectAndOpen(child, dataService)),
    };
  }

  const metricInspectQuery = buildInspectMetricQuery(cloned);
  if (metricInspectQuery) {
    return {
      ...cloned,
      query: metricInspectQuery,
      loadChildren: () => buildOntologyQueryRecordChildren(dataService, metricInspectQuery),
    };
  }

  if (cloned.query?.kind === "listRecords") {
    return {
      ...cloned,
      loadChildren: () => buildOntologyQueryRecordChildren(dataService, cloned.query),
    };
  }

  return cloned;
}

export function buildInspectAndOpenOntologyExplorerModel(
  ontology: OntologyExplorerOntologyService,
  dataService: OntologyExplorerDataService,
): OntologyDomainModel {
  const domain = ontology.loadSearchSemanticsDomain();
  return {
    ...domain,
    description: `${domain.description} Inspect matching records inline and open search results when needed.`,
    rootNodes: domain.rootNodes.map((node) => decorateNodeForInspectAndOpen(node, dataService)),
  };
}
