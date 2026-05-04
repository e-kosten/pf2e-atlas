import type { DatabaseSync } from "node:sqlite";

import type { RankingConfigStore } from "../search/ranking-config.js";
import type {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  RuleGraphCollectionResult,
} from "../domain/rule-types.js";
import type { PageReferenceCollectionResult } from "../domain/page-relations-types.js";
import type { NormalizedRecord, PackInfo } from "../domain/record-types.js";
import type { RecordKey } from "../domain/record-types.js";
import type { SearchRequest } from "../domain/search-request-types.js";
import type {
  FilterValueQuery,
  FilterValueResult,
  LookupOptions,
  LookupQuery,
  LookupResult,
  SearchCountResult,
  SearchResult,
  SearchWindowPage,
} from "../domain/search-types.js";
import { Pf2eRecordCatalog } from "./backend/record-catalog.js";
import { Pf2ePageRelationsBackendService } from "./backend/page-relations-service.js";
import { Pf2eRuleGraphBackendService } from "./backend/rule-graph-service.js";
import { loadPf2eDataRuntime, rebuildPf2eDataRuntime } from "./backend/load-runtime.js";
import { Pf2eSearchBackendService } from "./backend/search-service.js";
import type { Pf2eLoadedDataRuntime } from "./backend/types.js";
import type { Pf2eDataServiceLoadOptions } from "./backend/types.js";
import type { SearchCategorySummaryResult, SearchSemanticsBootstrapSummaryResult } from "./vocabulary.js";

type LoadOptions = Pf2eDataServiceLoadOptions;

export class Pf2eDataService {
  readonly packs: PackInfo[];
  readonly warnings: string[];

  private readonly db: DatabaseSync;
  private readonly recordCount: number;
  private readonly rankingConfigStore: RankingConfigStore | null;
  private readonly catalog: Pf2eRecordCatalog;
  private readonly searchService: Pf2eSearchBackendService;
  private readonly ruleGraphService: Pf2eRuleGraphBackendService;
  private readonly pageRelationsService: Pf2ePageRelationsBackendService;

  private constructor({
    db,
    packs,
    warnings,
    recordCount,
    embeddingProvider,
    rankingConfigStore,
  }: Pf2eLoadedDataRuntime) {
    this.db = db;
    this.packs = packs;
    this.warnings = warnings;
    this.recordCount = recordCount;
    this.rankingConfigStore = rankingConfigStore;
    this.catalog = new Pf2eRecordCatalog(db, packs, rankingConfigStore);
    this.searchService = new Pf2eSearchBackendService(db, this.catalog, embeddingProvider, rankingConfigStore);
    this.ruleGraphService = new Pf2eRuleGraphBackendService(db, this.catalog, this.searchService);
    this.pageRelationsService = new Pf2ePageRelationsBackendService(db, this.catalog);
  }

  static async load(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    return new Pf2eDataService(await loadPf2eDataRuntime(rootPath, manifestPath, options));
  }

  static async rebuildIndex(
    rootPath: string,
    manifestPath: string,
    options: LoadOptions = {},
  ): Promise<Pf2eDataService> {
    return new Pf2eDataService(await rebuildPf2eDataRuntime(rootPath, manifestPath, options));
  }

  getStats(): { packCount: number; recordCount: number } {
    return {
      packCount: this.packs.length,
      recordCount: this.recordCount,
    };
  }

  getSearchCategorySummary(): SearchCategorySummaryResult {
    return this.catalog.getSearchCategorySummary();
  }

  getSearchSemanticsBootstrapSummary(
    options: { traitLimitPerCategory?: number } = {},
  ): SearchSemanticsBootstrapSummaryResult {
    return this.catalog.getSearchSemanticsBootstrapSummary(options);
  }

  getSearchVocabulary(options: { traitLimitPerCategory?: number } = {}) {
    return this.catalog.getSearchVocabulary(options);
  }

  getRankingConfigStatus() {
    return this.catalog.getRankingConfigStatus();
  }

  getPack(packValue: string): PackInfo | undefined {
    return this.catalog.getPack(packValue);
  }

  listPacks(): PackInfo[] {
    return this.catalog.listPacks();
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    return this.searchService.listFilterValues(query);
  }

  async discoverFilterValues(query: FilterValueQuery, request: Readonly<SearchRequest>): Promise<FilterValueResult> {
    return this.searchService.discoverFilterValues(query, request);
  }

  close(): void {
    this.rankingConfigStore?.close();
    this.db.close();
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    return this.catalog.getRecord(recordKeyOrPack, maybeId);
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    return this.catalog.getRecordsByKeys(recordKeys);
  }

  lookupMany(queries: LookupQuery[], options: { coreOnly?: boolean } = {}): LookupResult[] {
    return this.searchService.lookupMany(queries, options);
  }

  getRuleGraph(
    recordKeys: string[],
    options: {
      coreOnly?: boolean;
      includeOutgoing?: boolean;
      includeBacklinks?: boolean;
      maxOutgoingPerPrimary?: number;
      maxBacklinksPerPrimary?: number;
    } = {},
  ): RuleGraphCollectionResult {
    return this.ruleGraphService.getRuleGraph(recordKeys, options);
  }

  collectRuleQuestionContext(input: CollectRuleQuestionContextInput): CollectRuleQuestionContextResult {
    return this.ruleGraphService.collectRuleQuestionContext(input);
  }

  getReferenceEdges(
    recordKeys: readonly RecordKey[],
    options: {
      includeOutgoing?: boolean;
      includeIncoming?: boolean;
    } = {},
  ): PageReferenceCollectionResult {
    return this.pageRelationsService.getReferenceEdges(recordKeys, options);
  }

  listRecords(request: SearchRequest): SearchResult {
    return this.searchService.listRecords(request);
  }

  async countRecords(
    request: SearchRequest,
    options: { lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    return this.searchService.countRecords(request, options);
  }

  async openSearchWindow(request: SearchRequest): Promise<SearchWindowPage> {
    return this.searchService.openSearchWindow(request);
  }

  readSearchWindowPage(windowId: string, offset: number, limit: number): SearchWindowPage {
    return this.searchService.readSearchWindowPage(windowId, offset, limit);
  }

  closeSearchWindow(windowId: string): void {
    this.searchService.closeSearchWindow(windowId);
  }

  async search(request: SearchRequest): Promise<SearchResult> {
    return this.searchService.search(request);
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[]; matchType: LookupResult["matchType"] } {
    return this.searchService.lookup(name, options);
  }
}
