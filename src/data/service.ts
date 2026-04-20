import type { DatabaseSync } from "node:sqlite";

import type { RankingConfigStore } from "../search/ranking-config.js";
import type {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  RuleGraphCollectionResult,
} from "../domain/rule-types.js";
import type { NormalizedRecord, PackInfo } from "../domain/record-types.js";
import type {
  FilterValueQuery,
  FilterValueResult,
  LookupOptions,
  LookupQuery,
  LookupResult,
  SearchCountResult,
  SearchFilters,
  SearchResult,
  SearchWindowPage,
} from "../domain/search-types.js";
import { Pf2eRecordCatalog } from "./backend/record-catalog.js";
import { Pf2eRuleGraphBackendService } from "./backend/rule-graph-service.js";
import { loadPf2eDataRuntime, rebuildPf2eDataRuntime } from "./backend/load-runtime.js";
import { Pf2eSearchBackendService } from "./backend/search-service.js";
import type { Pf2eLoadedDataRuntime } from "./backend/types.js";
import type { Pf2eDataServiceLoadOptions } from "./backend/types.js";

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

  listRecords(filters: SearchFilters): SearchResult {
    return this.searchService.listRecords(filters);
  }

  async countRecords(
    filters: SearchFilters,
    options: { mode?: "browse" | "search" | "lookup"; lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    return this.searchService.countRecords(filters, options);
  }

  async openSearchWindow(
    filters: SearchFilters,
    options: { mode?: "browse" | "search" | "lookup" } = {},
  ): Promise<SearchWindowPage> {
    return this.searchService.openSearchWindow(filters, options);
  }

  readSearchWindowPage(windowId: string, offset: number, limit: number): SearchWindowPage {
    return this.searchService.readSearchWindowPage(windowId, offset, limit);
  }

  closeSearchWindow(windowId: string): void {
    this.searchService.closeSearchWindow(windowId);
  }

  async search(filters: SearchFilters): Promise<SearchResult> {
    return this.searchService.search(filters);
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    return this.searchService.lookup(name, options);
  }
}
