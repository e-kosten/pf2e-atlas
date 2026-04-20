import { watch, FSWatcher } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";

import { pathExists } from "../shared/fs.js";
import type { RankingConfigStatus } from "../domain/search-types.js";

export interface RankingConfig {
  lexicalChannels: {
    fullTextSearch: number;
    descriptionText: number;
    themeName: number;
    themeTraits: number;
  };
  hybridFusion: {
    rrfK: number;
    balanced: {
      lexicalWeight: number;
      semanticWeight: number;
      lexicalTopK: number;
      semanticTopK: number;
    };
    concept: {
      lexicalWeight: number;
      semanticWeight: number;
      lexicalTopK: number;
      semanticTopK: number;
    };
  };
  packQuality: {
    macroPenalty: number;
    glossaryPenalty: number;
    effectPenalty: number;
    utilityPackBoost: number;
  };
  sourceQuality: {
    core: number;
    rules: number;
    adventure: number;
    unknown: number;
  };
  rarityPreference: {
    common: number;
    uncommon: number;
    rare: number;
    unique: number;
    themeQueryUniquePenalty: number;
  };
  sourcePenalty: {
    societyMetadataOnlyPenalty: number;
    scenarioScaleSuffixPenalty: number;
  };
}

type RankingConfigInput = {
  lexicalChannels?: Partial<RankingConfig["lexicalChannels"]>;
  hybridFusion?: {
    rrfK?: number;
    balanced?: Partial<RankingConfig["hybridFusion"]["balanced"]>;
    concept?: Partial<RankingConfig["hybridFusion"]["concept"]>;
  };
  packQuality?: Partial<RankingConfig["packQuality"]>;
  sourceQuality?: Partial<RankingConfig["sourceQuality"]>;
  rarityPreference?: Partial<RankingConfig["rarityPreference"]>;
  sourcePenalty?: Partial<RankingConfig["sourcePenalty"]>;
};

export const DEFAULT_RANKING_CONFIG: RankingConfig = {
  lexicalChannels: {
    fullTextSearch: 0.15,
    descriptionText: 0.2,
    themeName: 0.3,
    themeTraits: 0.35,
  },
  hybridFusion: {
    rrfK: 60,
    balanced: {
      lexicalWeight: 0.65,
      semanticWeight: 0.35,
      lexicalTopK: 150,
      semanticTopK: 80,
    },
    concept: {
      lexicalWeight: 0.3,
      semanticWeight: 0.7,
      lexicalTopK: 100,
      semanticTopK: 150,
    },
  },
  packQuality: {
    macroPenalty: -0.2,
    glossaryPenalty: -0.1,
    effectPenalty: -0.05,
    utilityPackBoost: 0.05,
  },
  sourceQuality: {
    core: 0.04,
    rules: 0.02,
    adventure: -0.01,
    unknown: 0,
  },
  rarityPreference: {
    common: 0.05,
    uncommon: 0.05,
    rare: 0.01,
    unique: -0.03,
    themeQueryUniquePenalty: -0.17,
  },
  sourcePenalty: {
    societyMetadataOnlyPenalty: -0.2,
    scenarioScaleSuffixPenalty: -0.1,
  },
};

function cloneDefaults(): RankingConfig {
  return {
    lexicalChannels: { ...DEFAULT_RANKING_CONFIG.lexicalChannels },
    hybridFusion: {
      rrfK: DEFAULT_RANKING_CONFIG.hybridFusion.rrfK,
      balanced: { ...DEFAULT_RANKING_CONFIG.hybridFusion.balanced },
      concept: { ...DEFAULT_RANKING_CONFIG.hybridFusion.concept },
    },
    packQuality: { ...DEFAULT_RANKING_CONFIG.packQuality },
    sourceQuality: { ...DEFAULT_RANKING_CONFIG.sourceQuality },
    rarityPreference: { ...DEFAULT_RANKING_CONFIG.rarityPreference },
    sourcePenalty: { ...DEFAULT_RANKING_CONFIG.sourcePenalty },
  };
}

function asObject(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object.`);
  }

  return value as Record<string, unknown>;
}

function readOptionalNumber(source: Record<string, unknown>, key: string, label: string): number | undefined {
  if (!(key in source)) {
    return undefined;
  }

  const value = source[key];
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${label}.${key} must be a finite number.`);
  }

  return value;
}

function applySection<T extends Record<string, number>>(current: T, override: unknown, label: string): T {
  if (override === undefined) {
    return current;
  }

  const source = asObject(override, label);
  const next = { ...current };
  for (const key of Object.keys(next)) {
    const value = readOptionalNumber(source, key, label);
    if (value !== undefined) {
      next[key as keyof T] = value as T[keyof T];
    }
  }

  return next;
}

function validateRankingConfig(config: RankingConfig): RankingConfig {
  const sections: Array<[string, Record<string, number>]> = [
    ["lexicalChannels", config.lexicalChannels],
    ["hybridFusion.balanced", config.hybridFusion.balanced],
    ["hybridFusion.concept", config.hybridFusion.concept],
    ["packQuality", config.packQuality],
    ["sourceQuality", config.sourceQuality],
    ["rarityPreference", config.rarityPreference],
    ["sourcePenalty", config.sourcePenalty],
  ];

  for (const [label, section] of sections) {
    for (const [key, value] of Object.entries(section)) {
      if (!Number.isFinite(value)) {
        throw new Error(`${label}.${key} must be a finite number.`);
      }
    }
  }

  for (const [key, value] of Object.entries(config.lexicalChannels)) {
    if (value < 0) {
      throw new Error(`lexicalChannels.${key} must be non-negative.`);
    }
  }

  if (config.lexicalChannels.descriptionText >= 1) {
    throw new Error("lexicalChannels.descriptionText must be less than 1.");
  }

  if (!Number.isInteger(config.hybridFusion.rrfK) || config.hybridFusion.rrfK <= 0) {
    throw new Error("hybridFusion.rrfK must be a positive integer.");
  }

  for (const profile of ["balanced", "concept"] as const) {
    const fusionProfile = config.hybridFusion[profile];
    if (fusionProfile.lexicalWeight < 0 || fusionProfile.semanticWeight < 0) {
      throw new Error(`hybridFusion.${profile} weights must be non-negative.`);
    }

    const weightTotal = fusionProfile.lexicalWeight + fusionProfile.semanticWeight;
    if (weightTotal <= 0) {
      throw new Error(`hybridFusion.${profile} must have a positive total weight.`);
    }

    fusionProfile.lexicalWeight /= weightTotal;
    fusionProfile.semanticWeight /= weightTotal;

    if (!Number.isInteger(fusionProfile.lexicalTopK) || fusionProfile.lexicalTopK <= 0) {
      throw new Error(`hybridFusion.${profile}.lexicalTopK must be a positive integer.`);
    }
    if (!Number.isInteger(fusionProfile.semanticTopK) || fusionProfile.semanticTopK <= 0) {
      throw new Error(`hybridFusion.${profile}.semanticTopK must be a positive integer.`);
    }
  }

  return config;
}

export function mergeRankingConfig(overrides: RankingConfigInput | null | undefined): RankingConfig {
  const merged = cloneDefaults();
  if (!overrides) {
    return validateRankingConfig(merged);
  }

  merged.lexicalChannels = applySection(merged.lexicalChannels, overrides.lexicalChannels, "lexicalChannels");
  if (overrides.hybridFusion !== undefined) {
    const fusion = asObject(overrides.hybridFusion, "hybridFusion");
    const rrfK = readOptionalNumber(fusion, "rrfK", "hybridFusion");
    if (rrfK !== undefined) {
      merged.hybridFusion.rrfK = rrfK;
    }
    merged.hybridFusion.balanced = applySection(merged.hybridFusion.balanced, fusion.balanced, "hybridFusion.balanced");
    merged.hybridFusion.concept = applySection(merged.hybridFusion.concept, fusion.concept, "hybridFusion.concept");
  }
  merged.packQuality = applySection(merged.packQuality, overrides.packQuality, "packQuality");
  merged.sourceQuality = applySection(merged.sourceQuality, overrides.sourceQuality, "sourceQuality");
  merged.rarityPreference = applySection(merged.rarityPreference, overrides.rarityPreference, "rarityPreference");
  merged.sourcePenalty = applySection(merged.sourcePenalty, overrides.sourcePenalty, "sourcePenalty");
  return validateRankingConfig(merged);
}

export class RankingConfigStore {
  private currentConfig: RankingConfig;
  private currentStatus: RankingConfigStatus;
  private readonly watchers: FSWatcher[] = [];
  private reloadTimer: NodeJS.Timeout | null = null;

  readonly warnings: string[] = [];

  private constructor(
    private readonly configPath: string,
    initialConfig: RankingConfig,
    initialStatus: RankingConfigStatus,
  ) {
    this.currentConfig = initialConfig;
    this.currentStatus = initialStatus;
  }

  static async create(configPath: string, options: { watch?: boolean } = {}): Promise<RankingConfigStore> {
    let initialConfig = cloneDefaults();
    let initialStatus: RankingConfigStatus = {
      path: configPath,
      source: "default",
      revision: 1,
      loadedAt: new Date().toISOString(),
      lastError: null,
    };
    const initialWarnings: string[] = [];
    try {
      const initialLoad = await RankingConfigStore.loadFromDisk(configPath, 1);
      initialConfig = initialLoad.config;
      initialStatus = initialLoad.status;
      if (initialLoad.warning) {
        initialWarnings.push(initialLoad.warning);
      }
    } catch (error) {
      const message = `Failed to load ranking config ${configPath}: ${(error as Error).message}`;
      initialStatus = { ...initialStatus, lastError: message };
      initialWarnings.push(message);
    }
    const store = new RankingConfigStore(configPath, initialConfig, initialStatus);
    for (const warning of initialWarnings) {
      store.warnings.push(warning);
      console.error(warning);
    }
    if (options.watch !== false) {
      store.startWatching();
    }
    return store;
  }

  getConfig(): RankingConfig {
    return this.currentConfig;
  }

  getStatus(): RankingConfigStatus {
    return { ...this.currentStatus };
  }

  async reload(): Promise<void> {
    const nextRevision = this.currentStatus.revision + 1;
    try {
      const next = await RankingConfigStore.loadFromDisk(this.configPath, nextRevision);
      this.currentConfig = next.config;
      this.currentStatus = next.status;
      if (next.warning) {
        this.currentStatus = { ...this.currentStatus, lastError: next.warning };
        console.error(next.warning);
      }
    } catch (error) {
      const message = `Failed to reload ranking config ${this.configPath}: ${(error as Error).message}`;
      this.currentStatus = { ...this.currentStatus, lastError: message };
      console.error(message);
    }
  }

  close(): void {
    if (this.reloadTimer) {
      clearTimeout(this.reloadTimer);
      this.reloadTimer = null;
    }

    for (const watcher of this.watchers) {
      watcher.close();
    }
    this.watchers.length = 0;
  }

  private startWatching(): void {
    const directory = path.dirname(this.configPath);
    try {
      const watcher = watch(directory, (_eventType, changedFile) => {
        if (!changedFile || changedFile.toString() !== path.basename(this.configPath)) {
          return;
        }

        if (this.reloadTimer) {
          clearTimeout(this.reloadTimer);
        }

        this.reloadTimer = setTimeout(() => {
          this.reloadTimer = null;
          void this.reload();
        }, 50);
      });
      this.watchers.push(watcher);
    } catch (error) {
      const message = `Failed to watch ranking config directory ${directory}: ${(error as Error).message}`;
      this.warnings.push(message);
      this.currentStatus = { ...this.currentStatus, lastError: message };
      console.error(message);
    }
  }

  private static async loadFromDisk(
    configPath: string,
    revision: number,
  ): Promise<{ config: RankingConfig; status: RankingConfigStatus; warning: string | null }> {
    const loadedAt = new Date().toISOString();
    if (!(await pathExists(configPath))) {
      return {
        config: cloneDefaults(),
        status: {
          path: configPath,
          source: "default",
          revision,
          loadedAt,
          lastError: null,
        },
        warning: null,
      };
    }

    const rawConfig = await readFile(configPath, "utf8");
    const parsed = asObject(JSON.parse(rawConfig), "rankingConfig") as RankingConfigInput;
    const merged = mergeRankingConfig(parsed);
    return {
      config: merged,
      status: {
        path: configPath,
        source: "file",
        revision,
        loadedAt,
        lastError: null,
      },
      warning: null,
    };
  }
}
