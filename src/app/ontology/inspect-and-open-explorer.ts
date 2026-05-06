import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../domain/categories.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { Pf2eDataService } from "../../data/service.js";
import type {
  OntologyChildSource,
  OntologyDomainModel,
  OntologyNode,
  OntologyNodeQuery,
} from "../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { buildAllOfFilter, buildScopeFilter } from "../../domain/search-request-types.js";
import type { Pf2eApplicationOntologyService } from "../ontology-service.js";
import { buildNormalizedRecordNode, cloneOntologyNode } from "./node-helpers.js";

type OntologyExplorerDataService = Pick<Pf2eDataService, "listRecords">;
type OntologyExplorerOntologyService = Pick<Pf2eApplicationOntologyService, "loadSearchSemanticsDomain">;

function buildOntologyQueryRecordChildren(
  dataService: OntologyExplorerDataService,
  query: OntologyNodeQuery | undefined,
): readonly OntologyNode[] {
  if (!query) {
    return [];
  }

  const request = query.request;
  if (request.mode !== "browse") {
    return [];
  }

  return dataService.listRecords(request).records.map(buildNormalizedRecordNode);
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
  const subcategory = scopeSegments.length === 2 ? (normalizeSearchSubcategory(scopeSegments[1]) ?? null) : null;
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
    label: `Browse records with the ${scope.metricKey} metric`,
    request: {
      mode: "browse",
      filter: buildAllOfFilter([
        buildScopeFilter(scope.category, scope.subcategory),
        {
          kind: "metricCompare",
          leftMetric: scope.metricKey,
          op: "gte",
          rightMetric: scope.metricKey,
        },
      ]),
      limit: 20,
    },
  };
}

function decorateNodeForInspectAndOpen(node: OntologyNode, dataService: OntologyExplorerDataService): OntologyNode {
  const cloned = cloneOntologyNode(node);
  const decoratedChildSource = decorateChildSourceForInspectAndOpen(cloned.childSource, dataService);

  if (decoratedChildSource) {
    return {
      ...cloned,
      childSource: decoratedChildSource,
    };
  }

  const metricInspectQuery = buildInspectMetricQuery(cloned);
  if (metricInspectQuery) {
    return {
      ...cloned,
      query: metricInspectQuery,
      childSource: { kind: "lazy", load: () => Promise.resolve(buildOntologyQueryRecordChildren(dataService, metricInspectQuery)) },
    };
  }

  if (cloned.query && cloned.query.request.mode === "browse") {
    return {
      ...cloned,
      childSource: { kind: "lazy", load: () => Promise.resolve(buildOntologyQueryRecordChildren(dataService, cloned.query)) },
    };
  }

  return cloned;
}

function decorateChildSourceForInspectAndOpen(
  source: OntologyChildSource | undefined,
  dataService: OntologyExplorerDataService,
): OntologyChildSource | undefined {
  if (!source) {
    return undefined;
  }
  if (source.kind === "static") {
    return {
      kind: "static",
      children: source.children.map((child) => decorateNodeForInspectAndOpen(child, dataService)),
    };
  }
  if (source.kind === "sync") {
    return {
      kind: "sync",
      load: () => source.load().map((child) => decorateNodeForInspectAndOpen(child, dataService)),
    };
  }
  return {
    kind: "lazy",
    load: async () => (await source.load()).map((child) => decorateNodeForInspectAndOpen(child, dataService)),
  };
}

export function buildInspectAndOpenOntologyExplorerModel(
  ontology: OntologyExplorerOntologyService,
  dataService: OntologyExplorerDataService,
): Promise<OntologyDomainModel> {
  return ontology.loadSearchSemanticsDomain({ discoveryMode: "matching" }).then((domain) => ({
    ...domain,
    description: `${domain.description} Inspect matching records inline and open search results when needed.`,
    rootNodes: domain.rootNodes.map((node) => decorateNodeForInspectAndOpen(node, dataService)),
  }));
}
