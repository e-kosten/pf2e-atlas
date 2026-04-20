import type { NormalizedRecord } from "../../domain/record-types.js";
import type { SearchSort, SearchWindow, SearchWindowPage } from "../../domain/search-types.js";

type RuntimeSearchWindow = {
  id: string;
  kind: "recordKeys";
  mode: SearchWindow["mode"];
  searchProfile: SearchWindow["searchProfile"];
  sort: SearchSort;
  sortSeed: number | null;
  total: number;
  orderedRecordKeys: string[];
};

const MAX_SEARCH_WINDOWS = 24;

export class Pf2eSearchWindowStore {
  private readonly searchWindows = new Map<string, RuntimeSearchWindow>();
  private searchWindowCounter = 0;

  constructor(private readonly getRecordsByKeys: (recordKeys: string[]) => NormalizedRecord[]) {}

  openWindow(window: Omit<RuntimeSearchWindow, "id">): RuntimeSearchWindow {
    const nextWindow = {
      ...window,
      id: this.createSearchWindowId(),
    };

    this.searchWindows.set(nextWindow.id, nextWindow);
    while (this.searchWindows.size > MAX_SEARCH_WINDOWS) {
      const oldestId = this.searchWindows.keys().next().value;
      if (!oldestId) {
        break;
      }
      this.searchWindows.delete(oldestId);
    }

    return nextWindow;
  }

  readWindowPage(windowId: string, offset: number, limit: number): SearchWindowPage {
    const window = this.searchWindows.get(windowId);
    if (!window) {
      throw new Error(`Search window "${windowId}" is no longer available.`);
    }

    const recordKeys = window.orderedRecordKeys.slice(offset, offset + limit);
    const records = this.getRecordsByKeys(recordKeys);
    const hasMore = offset + records.length < window.total;

    return {
      id: window.id,
      searchProfile: window.searchProfile,
      mode: window.mode,
      sort: window.sort,
      sortSeed: window.sortSeed,
      total: window.total,
      offset,
      limit,
      hasMore,
      nextOffset: hasMore ? offset + records.length : null,
      records,
    };
  }

  closeWindow(windowId: string): void {
    this.searchWindows.delete(windowId);
  }

  private createSearchWindowId(): string {
    this.searchWindowCounter += 1;
    return `search-window-${this.searchWindowCounter}`;
  }
}
