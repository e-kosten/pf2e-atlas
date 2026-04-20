import type { DatabaseSync } from "node:sqlite";

import {
  collectRuleQuestionContext as collectRuleQuestionContextRuntime,
  getRuleGraph as getRuleGraphRuntime,
} from "../rule-runtime.js";
import { fetchReferenceEdgeRows } from "../record-queries.js";
import type { CollectRuleQuestionContextInput, CollectRuleQuestionContextResult, RuleGraphCollectionResult } from "../../types.js";
import type { Pf2eRecordCatalog } from "./record-catalog.js";
import type { Pf2eSearchBackendService } from "./search-service.js";

export class Pf2eRuleGraphBackendService {
  constructor(
    private readonly db: DatabaseSync,
    private readonly catalog: Pick<Pf2eRecordCatalog, "getRecordsByKeys">,
    private readonly searchService: Pick<Pf2eSearchBackendService, "lookupMany">,
  ) {}

  getRuleGraph(
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
  ): RuleGraphCollectionResult {
    return getRuleGraphRuntime(
      recordKeys,
      {
        coreOnly,
        includeOutgoing,
        includeBacklinks,
        maxOutgoingPerPrimary,
        maxBacklinksPerPrimary,
      },
      {
        fetchReferenceEdgeRows: (direction, keys, options) => fetchReferenceEdgeRows(this.db, direction, keys, options),
        getRecordsByKeys: (keys) => this.catalog.getRecordsByKeys(keys),
        lookupMany: (queries, options) => this.searchService.lookupMany(queries, options),
      },
    );
  }

  collectRuleQuestionContext(input: CollectRuleQuestionContextInput): CollectRuleQuestionContextResult {
    return collectRuleQuestionContextRuntime(input, {
      fetchReferenceEdgeRows: (direction, keys, options) => fetchReferenceEdgeRows(this.db, direction, keys, options),
      getRecordsByKeys: (keys) => this.catalog.getRecordsByKeys(keys),
      lookupMany: (queries, options) => this.searchService.lookupMany(queries, options),
    });
  }
}
