import type {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  RuleGraphCollectionResult,
  RuleGraphResult,
  RuleReferenceEdge,
} from "../types.js";
import {
  backlinkTypeRank,
  edgeRowToReferenceEdge,
  extractQuestionRuleNames,
  ReferenceEdgeRow,
  sourceCategoryRank,
} from "./rows.js";

type FetchReferenceEdgeRows = (
  direction: RuleReferenceEdge["direction"],
  recordKeys: string[],
  options?: { coreOnly?: boolean },
) => ReferenceEdgeRow[];

type RuleRuntimeDependencies = {
  fetchReferenceEdgeRows: FetchReferenceEdgeRows;
  getRecordsByKeys: (recordKeys: string[]) => NormalizedRecord[];
  lookupMany: (queries: LookupQuery[], options?: { coreOnly?: boolean }) => LookupResult[];
};

function collectDirectionGraph(
  direction: RuleReferenceEdge["direction"],
  recordKeys: string[],
  {
    coreOnly = false,
    maxPerPrimary = 4,
  }: { coreOnly?: boolean; maxPerPrimary?: number } = {},
  deps: RuleRuntimeDependencies,
): RuleGraphResult {
  if (recordKeys.length === 0) {
    return { records: [], edges: [] };
  }

  const rows = deps.fetchReferenceEdgeRows(direction, recordKeys, { coreOnly });
  const grouped = new Map<string, ReferenceEdgeRow[]>();
  for (const row of rows) {
    const groupKey = direction === "outgoing" ? row.fromRecordKey : row.toRecordKey;
    const bucket = grouped.get(groupKey) ?? [];
    bucket.push(row);
    grouped.set(groupKey, bucket);
  }

  const keptRows: ReferenceEdgeRow[] = [];
  for (const primaryKey of recordKeys) {
    const bucket = grouped.get(primaryKey) ?? [];
    bucket.sort((left, right) => {
      const leftTypeRank =
        left.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(left.fromRecordType);
      const rightTypeRank =
        right.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(right.fromRecordType);
      const leftLabel = left.displayText ?? (direction === "outgoing" ? left.toRecordKey : left.fromRecordKey);
      const rightLabel = right.displayText ?? (direction === "outgoing" ? right.toRecordKey : right.fromRecordKey);
      return (
        sourceCategoryRank(left.fromSourceCategory) - sourceCategoryRank(right.fromSourceCategory) ||
        leftTypeRank - rightTypeRank ||
        leftLabel.localeCompare(rightLabel) ||
        left.referenceText.localeCompare(right.referenceText)
      );
    });
    keptRows.push(...bucket.slice(0, Math.max(1, maxPerPrimary)));
  }

  const relatedRecordKeys = [
    ...new Set(keptRows.map((row) => direction === "outgoing" ? row.toRecordKey : row.fromRecordKey)),
  ];
  return {
    records: deps.getRecordsByKeys(relatedRecordKeys),
    edges: keptRows.map((row) => edgeRowToReferenceEdge(row, direction)),
  };
}

export function getRuleGraph(
  recordKeys: string[],
  {
    coreOnly,
    includeOutgoing,
    includeBacklinks,
    maxOutgoingPerPrimary,
    maxBacklinksPerPrimary,
  }: {
    coreOnly?: boolean;
    includeOutgoing?: boolean;
    includeBacklinks?: boolean;
    maxOutgoingPerPrimary?: number;
    maxBacklinksPerPrimary?: number;
  } = {},
  deps: RuleRuntimeDependencies,
): RuleGraphCollectionResult {
  const uniqueRecordKeys = [...new Set(recordKeys)];
  const directionsSpecified = includeOutgoing !== undefined || includeBacklinks !== undefined;
  const shouldIncludeOutgoing = directionsSpecified ? includeOutgoing === true : true;
  const shouldIncludeBacklinks = directionsSpecified ? includeBacklinks === true : false;
  const emptyGraph: RuleGraphResult = { records: [], edges: [] };
  const outgoing = shouldIncludeOutgoing
    ? collectDirectionGraph("outgoing", uniqueRecordKeys, {
        coreOnly,
        maxPerPrimary: maxOutgoingPerPrimary,
      }, deps)
    : emptyGraph;
  const backlinks = shouldIncludeBacklinks
    ? collectDirectionGraph("backlink", uniqueRecordKeys, {
        coreOnly,
        maxPerPrimary: maxBacklinksPerPrimary,
      }, deps)
    : emptyGraph;

  return {
    outgoing,
    backlinks,
    edges: [...outgoing.edges, ...backlinks.edges],
  };
}

export function collectRuleQuestionContext(
  input: CollectRuleQuestionContextInput,
  deps: RuleRuntimeDependencies,
): CollectRuleQuestionContextResult {
  const explicitRules = (input.rules ?? []).map((rule) => rule.trim()).filter((rule) => rule.length > 0);
  const derivedRules = explicitRules.length > 0
    ? explicitRules
    : input.question
      ? extractQuestionRuleNames(input.question)
      : [];
  const primary = deps.lookupMany(derivedRules.map((name) => ({ name })), { coreOnly: input.coreOnly });
  const primaryKeys = primary
    .map((result) => result.match?.recordKey ?? null)
    .filter((recordKey): recordKey is string => Boolean(recordKey));
  const graph = getRuleGraph(primaryKeys, {
    coreOnly: input.coreOnly,
    includeOutgoing: true,
    includeBacklinks: input.includeBacklinks,
    maxOutgoingPerPrimary: input.maxOutgoingPerPrimary ?? 4,
    maxBacklinksPerPrimary: input.maxBacklinksPerPrimary ?? 4,
  }, deps);

  return {
    primary,
    ...graph,
  };
}
