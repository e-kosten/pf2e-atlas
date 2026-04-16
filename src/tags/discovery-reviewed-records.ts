import { SearchCategory, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import { normalizeDerivedTag } from "./shared.js";

export const REVIEWED_DISCOVERY_REASONS = [
  "not_family_salient",
  "insufficient_evidence",
  "mixed_family_cues",
  "manual_lore_only",
] as const;

export type ReviewedDiscoveryReason = typeof REVIEWED_DISCOVERY_REASONS[number];

export type ReviewedDiscoveryRegistryRecord = {
  recordKey: string;
  subcategory?: SearchSubcategory | null;
  note?: string;
};

export type ReviewedDiscoveryEntry = ReviewedDiscoveryRegistryRecord & {
  category: SearchCategory;
  family: string;
  reason: ReviewedDiscoveryReason;
};

export type ReviewedDiscoveryReasonCount = {
  reason: ReviewedDiscoveryReason;
  count: number;
};

export type ReviewedDiscoverySelectionMode = "excluded" | "included" | "filtered";

export type ReviewedDiscoverySelection = {
  mode: ReviewedDiscoverySelectionMode;
  reviewReason: ReviewedDiscoveryReason | null;
  entries: ReviewedDiscoveryEntry[];
  recordKeys: string[];
  reasonCounts: ReviewedDiscoveryReasonCount[];
};

export type ReviewedDiscoveryApplicationSummary = {
  mode: ReviewedDiscoverySelectionMode;
  reviewReason: ReviewedDiscoveryReason | null;
  scopedCount: number;
  appliedCount: number;
  reasonCounts: ReviewedDiscoveryReasonCount[];
};

export type ReviewedDiscoveryScope = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  reason?: ReviewedDiscoveryReason;
};

export type ReviewedDiscoverySelectionOptions = ReviewedDiscoveryScope & {
  includeReviewed?: boolean;
  reviewReason?: ReviewedDiscoveryReason;
};

export type ReviewedDiscoveryRegistry = Partial<Record<
  SearchCategory,
  Partial<Record<string, Partial<Record<ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[]>>>>
>>;

function reviewedRecordKeys(recordKeys: string[]): ReviewedDiscoveryRegistryRecord[] {
  return recordKeys.map((recordKey) => ({ recordKey }));
}

// Curated during family-gap passes to suppress repeatedly reviewed negatives from the default queue.
export const REVIEWED_DISCOVERY_RECORDS: ReviewedDiscoveryRegistry = {
  creature: {
    setting: {
      // First 250 uncovered creature/setting records by pack+name, reviewed 2026-04-16.
      // Records handled by new rules or exact seeds in the same pass are intentionally excluded.
      not_family_salient: reviewedRecordKeys([
        "abomination-vaults-bestiary:3d3NAcPfvn07mcGN",
        "abomination-vaults-bestiary:tOL4rWj2oYWZ4ow2",
        "abomination-vaults-bestiary:saEUzIgUtV2AzKhl",
        "abomination-vaults-bestiary:DDJGNAh3rfyIupAb",
        "abomination-vaults-bestiary:3ry9WSvMMXHUe3kE",
        "abomination-vaults-bestiary:rketcmqDQJbFFYfq",
        "abomination-vaults-bestiary:xAfkUwJYq5JLmSrW",
        "abomination-vaults-bestiary:ChRgdkplhO1D81Lg",
        "abomination-vaults-bestiary:oXnpdJVN6NIE58W3",
        "abomination-vaults-bestiary:lMCEVxKkQ7XK6Nid",
        "abomination-vaults-bestiary:tYzzLLUv9WBhHhQY",
        "abomination-vaults-bestiary:x0NDgH3EMLTLh02r",
        "abomination-vaults-bestiary:3H1rBpUQwTcNd6xZ",
        "abomination-vaults-bestiary:HBRz8BVLVN9u9Odp",
        "abomination-vaults-bestiary:T6vOuhM1KV5Fr75F",
        "abomination-vaults-bestiary:BJYrYqkV7PkXgSfk",
        "abomination-vaults-bestiary:3F3fPq5hFbej40T2",
        "abomination-vaults-bestiary:jE8BEe6pcnGraw2p",
        "abomination-vaults-bestiary:hia81Ut7fEREbhkq",
        "abomination-vaults-bestiary:EN3mp0sVObP8ou3p",
        "abomination-vaults-bestiary:xj1Qn0VA4H4aKSjW",
        "abomination-vaults-bestiary:dWOK0nzGWyc5NkNz",
        "abomination-vaults-bestiary:kzX588Hjb3w4QPOj",
        "abomination-vaults-bestiary:mlifDVJJWwjFtUxv",
        "abomination-vaults-bestiary:w2N0foudBFcRCaHK",
        "abomination-vaults-bestiary:TiAzR8SnYwhACWbj",
        "abomination-vaults-bestiary:R0EEgMDKcynpAWoa",
        "abomination-vaults-bestiary:BOaM3pAuWl06Q6IZ",
        "abomination-vaults-bestiary:WR07Z6MjvebSHzI7",
        "abomination-vaults-bestiary:3vn9W5SThovdsEnY",
        "abomination-vaults-bestiary:qOkxxiM4tNf96CHQ",
        "abomination-vaults-bestiary:vS1YISLmSnkNotkL",
        "abomination-vaults-bestiary:cMpgGvq1fGxh8wI0",
        "abomination-vaults-bestiary:v9B0hB5sm4YZxebY",
        "abomination-vaults-bestiary:qXT1SQDtGqMkVl7Q",
        "abomination-vaults-bestiary:ZAXuvUW6kl6v3SuW",
        "abomination-vaults-bestiary:ZDYMKYZVyR8Fqakp",
        "age-of-ashes-bestiary:xrxjjjRMKzwsYbGm",
        "age-of-ashes-bestiary:YV3wA2Kgjp74l7YJ",
        "age-of-ashes-bestiary:yrU0xi4eKBmcGudo",
        "age-of-ashes-bestiary:sQZDwS08l6Agsryq",
        "age-of-ashes-bestiary:jO2fGZayk4R1AzYK",
        "age-of-ashes-bestiary:91BSCAJA1Oto2ctf",
        "age-of-ashes-bestiary:4vEQ8zRXC51Qo4Mv",
        "age-of-ashes-bestiary:aaDiR0EIWRQx8wdy",
        "age-of-ashes-bestiary:53lk7ek73j65A09B",
        "age-of-ashes-bestiary:yLhO5vUrPQF42Lh8",
        "age-of-ashes-bestiary:T3UVfAfcvAMe9rih",
        "age-of-ashes-bestiary:w1Pqv0YmG4MevpQc",
        "age-of-ashes-bestiary:L4K4V09tQVXj7ZiI",
        "age-of-ashes-bestiary:MOyYEls8wNGHoe8F",
        "age-of-ashes-bestiary:scPhFbBqlmD2fQHa",
        "age-of-ashes-bestiary:NwNu2yvMnbvPvpY8",
        "age-of-ashes-bestiary:jZc4PrsX3HCJnkXx",
        "age-of-ashes-bestiary:dG5DBgrxlaimsWOS",
        "age-of-ashes-bestiary:tr3qlFJVHqloE9zI",
        "age-of-ashes-bestiary:UQDxkvqXKg03ESTQ",
        "age-of-ashes-bestiary:OJH8y8LqmgbkN8ce",
        "age-of-ashes-bestiary:xXtGS9uBa9z43X7y",
        "age-of-ashes-bestiary:dlW3UaXVpnzjd6xe",
        "age-of-ashes-bestiary:MWQmOXxGcbaMsXDD",
        "age-of-ashes-bestiary:Ev930dfPpwCR8Zju",
        "age-of-ashes-bestiary:XHJC4G6bfPiVXGKE",
        "age-of-ashes-bestiary:AtiN3EsRpHn5qbuv",
        "age-of-ashes-bestiary:lcYcBIFmfKFt8Hcs",
        "age-of-ashes-bestiary:Ig1joUmSHSNL6QVU",
        "age-of-ashes-bestiary:30FX1WFr0RkGeueX",
        "age-of-ashes-bestiary:lRqptquQcM6ZcQ4O",
        "age-of-ashes-bestiary:mY494hn9sKVD9q8C",
        "age-of-ashes-bestiary:kciXaXw31gHA3gZl",
        "age-of-ashes-bestiary:qemmmfu7exswGMGZ",
        "age-of-ashes-bestiary:C4PD4p4I9byZ8yp6",
        "age-of-ashes-bestiary:9oNwQuwKGUuG9G9g",
        "age-of-ashes-bestiary:L7IJO5z82nEN9IjM",
        "age-of-ashes-bestiary:N8vOjTD2SqS2b9Sy",
        "age-of-ashes-bestiary:SVLUUPOXDINbKyFL",
        "age-of-ashes-bestiary:sY47PB9b7nCJjgRq",
        "age-of-ashes-bestiary:QKkvnlqrhgLHuP1t",
        "age-of-ashes-bestiary:ZtSb3mHZ5sD2uqHd",
        "age-of-ashes-bestiary:NK49bh04355Lgz5r",
        "age-of-ashes-bestiary:tGyPrTGSpndXKU88",
        "age-of-ashes-bestiary:Afq4Osh3W1k9Bcsh",
        "age-of-ashes-bestiary:60aeSJvu09ZM3SPx",
        "age-of-ashes-bestiary:z4rlpEsE2KgzpOCc",
        "age-of-ashes-bestiary:r02b4f1XNq7OihhD",
        "age-of-ashes-bestiary:kLX36WXp6rjTt71z",
        "age-of-ashes-bestiary:qouYGPM8mE4KUCTe",
        "age-of-ashes-bestiary:U2QGjhcg5QFFkHwv",
        "age-of-ashes-bestiary:mYJ6NNthl702Pz2s",
        "age-of-ashes-bestiary:SVh7cPwyGgmZORVz",
        "age-of-ashes-bestiary:Lx5JiVOGWnzzjCrW",
        "age-of-ashes-bestiary:egTpOr4Wc0L5e0iY",
        "age-of-ashes-bestiary:gF8Qy1k8gPBEUBAd",
        "age-of-ashes-bestiary:nQ2eBOpK71I8D2JC",
        "age-of-ashes-bestiary:1lkay2gwgEquq0NF",
        "age-of-ashes-bestiary:7WNFNE3SVMRGbBDf",
        "age-of-ashes-bestiary:NwLrwNqwedh95iry",
        "age-of-ashes-bestiary:Aii18rhNPMLW4Pxh",
        "age-of-ashes-bestiary:t5mX2rkcKzcOzJXQ",
        "age-of-ashes-bestiary:Fe1lYhCUY4UO4Plw",
        "age-of-ashes-bestiary:5r2S6FeE6D1Oh66T",
        "age-of-ashes-bestiary:X7zSx8LFh2ZDnOYy",
        "age-of-ashes-bestiary:Oqj8XIWBb29NZ8QX",
        "age-of-ashes-bestiary:aiNE06bxKD6jfoLd",
        "age-of-ashes-bestiary:Ejxngh2tHFseZQHW",
        "age-of-ashes-bestiary:dTikLHqGfiSYemuZ",
        "age-of-ashes-bestiary:UD7EwTQG2Sbl4d8R",
        "age-of-ashes-bestiary:1tDEWL9mAIXvTYik",
        "age-of-ashes-bestiary:H7NO4Q7ctHnfGKeJ",
        "age-of-ashes-bestiary:JD6kdfZveObBe1mR",
        "age-of-ashes-bestiary:YIXAcvSyI1C94r9l",
        "age-of-ashes-bestiary:XNso2IMnhnfHcMCn",
        "agents-of-edgewatch-bestiary:RtWlzHaOrfFdJyJY",
        "agents-of-edgewatch-bestiary:phkvSUK6WXxgJOoC",
        "agents-of-edgewatch-bestiary:gsn4NsJLwZCQUwgf",
        "agents-of-edgewatch-bestiary:1G4OdEHRPF8GMHK8",
        "agents-of-edgewatch-bestiary:MONwgTbrcZFzr6vC",
        "agents-of-edgewatch-bestiary:azyIfDNNW44jY8YX",
        "agents-of-edgewatch-bestiary:4ffoPNBKdEwBmYgL",
        "agents-of-edgewatch-bestiary:fvOjtzuRNpmpEHXA",
        "agents-of-edgewatch-bestiary:7LN8clGKJWTpxISR",
        "agents-of-edgewatch-bestiary:y0bIU9FCWHOJxUzG",
        "agents-of-edgewatch-bestiary:TIaZIUb9Mq9B4Mf2",
        "agents-of-edgewatch-bestiary:sUrJ7jxzBiJTbwVo",
        "agents-of-edgewatch-bestiary:qY1NrXKmL0y18qoz",
        "agents-of-edgewatch-bestiary:5u7luLMeRNJ4en65",
        "agents-of-edgewatch-bestiary:a3pTQfLzsJThQvI9",
        "agents-of-edgewatch-bestiary:MQnhyCM9LInNYtl0",
        "agents-of-edgewatch-bestiary:0ti3f4fdcB5D2bLB",
        "agents-of-edgewatch-bestiary:9vt9Dr2a8MkkD83z",
        "agents-of-edgewatch-bestiary:EGQgqBfV80ll3pcf",
        "agents-of-edgewatch-bestiary:wsVW8MdOTeGgGM59",
        "agents-of-edgewatch-bestiary:HifZEgdCuZearOG2",
        "agents-of-edgewatch-bestiary:fV5VIoXMtixmI3Wc",
        "agents-of-edgewatch-bestiary:D36nL0YjCVHfjBNw",
        "agents-of-edgewatch-bestiary:w2J6GpuMYM24U4sb",
        "agents-of-edgewatch-bestiary:Wb4Md6byPhBWe56J",
        "agents-of-edgewatch-bestiary:JuCLvgvxYbSRXqON",
        "agents-of-edgewatch-bestiary:9M1YJxTXqya55HDx",
        "agents-of-edgewatch-bestiary:JdMqHbaTwtOHVE7Y",
        "agents-of-edgewatch-bestiary:xkkj0TW6BKNT3Bg4",
        "agents-of-edgewatch-bestiary:ZL2qLXwomKfBB8Eu",
        "agents-of-edgewatch-bestiary:1eMDYXXf2leLTYHV",
        "agents-of-edgewatch-bestiary:sn9Pjkr2jlMEqc3E",
        "agents-of-edgewatch-bestiary:SgkV5RtcK72d0HwI",
        "agents-of-edgewatch-bestiary:1UtsScuNjmgwMZRn",
        "agents-of-edgewatch-bestiary:rM6ix6XTroJod3Vr",
        "agents-of-edgewatch-bestiary:HF0ymYmKC6KydPQ1",
        "agents-of-edgewatch-bestiary:8AdsRaXftoR1beTk",
        "agents-of-edgewatch-bestiary:eqwAdGsAk5JZKxUY",
        "agents-of-edgewatch-bestiary:8GQ7dq7s9CetOlkg",
        "agents-of-edgewatch-bestiary:pfHOwcITyC4gdCVu",
        "agents-of-edgewatch-bestiary:Xi53GFvTgBApltjp",
        "agents-of-edgewatch-bestiary:Uhi3wX4KveuMSARt",
        "agents-of-edgewatch-bestiary:JnNPdvOXtkGQHyfQ",
        "agents-of-edgewatch-bestiary:H4bY8v6e3drOIoUe",
        "agents-of-edgewatch-bestiary:rGTq4qItRB5H7nEk",
        "agents-of-edgewatch-bestiary:pE5GxoB1FtXBqnF7",
        "agents-of-edgewatch-bestiary:Bv1s6xJ55HS3Gxgs",
        "agents-of-edgewatch-bestiary:r3j5KEvULFP3fZS7",
        "agents-of-edgewatch-bestiary:gd0sVQtCHbhP8iHI",
        "agents-of-edgewatch-bestiary:181ucNY1zpp2Lz3x",
        "agents-of-edgewatch-bestiary:jJ1UTNLiRCoN1O3i",
        "agents-of-edgewatch-bestiary:LAamprMlzk7k5auj",
        "agents-of-edgewatch-bestiary:vwxNCuBksHYU2Dwf",
        "agents-of-edgewatch-bestiary:cLlOUUpCIAQwUuOP",
        "agents-of-edgewatch-bestiary:pK1tlsgkmzkaaCe5",
        "agents-of-edgewatch-bestiary:M8ONVV7yl4uu0zcz",
        "agents-of-edgewatch-bestiary:gIcReNQOZceZBBlw",
        "agents-of-edgewatch-bestiary:E7NEf3kmsY3YjRrz",
        "agents-of-edgewatch-bestiary:6c5CnrSxMYEgP6Fz",
        "agents-of-edgewatch-bestiary:yR6p0KVvZ3tPflRt",
        "agents-of-edgewatch-bestiary:3JEiwAFwkEjCDEYa",
        "agents-of-edgewatch-bestiary:E279VPhAy1a4ihqI",
        "agents-of-edgewatch-bestiary:B4kIfCimsz8wfc0k",
        "agents-of-edgewatch-bestiary:lzmGdArS3kjJOqT6",
        "agents-of-edgewatch-bestiary:t9m4ikMZsDwo9TQ1",
        "agents-of-edgewatch-bestiary:Sq0Kb92nGkqj19Xx",
        "agents-of-edgewatch-bestiary:GWO6vweLGT2J6q62",
        "agents-of-edgewatch-bestiary:ZY3q7AV1qbwWwNl2",
        "agents-of-edgewatch-bestiary:07AGJt4ZRjwH85Xp",
        "agents-of-edgewatch-bestiary:Wk2T0Wr8Sebo4br5",
        "agents-of-edgewatch-bestiary:vPMmTtvl5UPOcCoa",
        "agents-of-edgewatch-bestiary:WDTdWiC9Rdl6rqh8",
        "agents-of-edgewatch-bestiary:zj2sCM8tQSMG9Qm6",
        "agents-of-edgewatch-bestiary:RyXA4wOGY8lenKVw",
        "agents-of-edgewatch-bestiary:cQMM2Ld0IBM9GcDo",
        "agents-of-edgewatch-bestiary:1bPc2rjR4MghbMwD",
        "agents-of-edgewatch-bestiary:ySpOZlKUbcxWhKQ6",
        "agents-of-edgewatch-bestiary:oWASKud0jwlGSfJg",
        "agents-of-edgewatch-bestiary:3XbjmNeUtPLzxDge",
        "agents-of-edgewatch-bestiary:8HTPdiH6yEk0jlNF",
        "agents-of-edgewatch-bestiary:S9JJsUNSeeoIClON",
        "agents-of-edgewatch-bestiary:LkaB8RH73DY4TO9V",
        "agents-of-edgewatch-bestiary:MqfZxoxFwzqAXhTP",
        "agents-of-edgewatch-bestiary:2uxm1SxZXaG0ynCp",
        "agents-of-edgewatch-bestiary:JZUYQzQtzIwOGYvd",
        "agents-of-edgewatch-bestiary:aDgVmO3afhIgXQSN",
        "agents-of-edgewatch-bestiary:BuPf7xtqfwCjNOQv",
        "agents-of-edgewatch-bestiary:kcvU9CatCyBUJRr2",
        "agents-of-edgewatch-bestiary:4TB1eo3O22khMyDU",
        "agents-of-edgewatch-bestiary:CLntGVs7cAIL9Trk",
        "agents-of-edgewatch-bestiary:BCMkxoQB4xK4BniG",
        "agents-of-edgewatch-bestiary:1xGAktpj6N2Ugh0r",
        "agents-of-edgewatch-bestiary:DKReNCapWWubM3pm",
        "agents-of-edgewatch-bestiary:eQc0ADMuHl1JzL8z",
        "agents-of-edgewatch-bestiary:oTa25rAxytm03T3X",
        "agents-of-edgewatch-bestiary:XDt87cqF85zWnlC8",
        "agents-of-edgewatch-bestiary:nTn2szBbqQNdXhOr",
        "agents-of-edgewatch-bestiary:8PRqUfkHLvu9ufGL",
        "agents-of-edgewatch-bestiary:y1tw2ohNagqQJ6RV",
        "agents-of-edgewatch-bestiary:AYNIAAxV7TbIKPI4",
        "agents-of-edgewatch-bestiary:xN3FDmrCKWW0psBu",
        "agents-of-edgewatch-bestiary:P9Wg0sGcNkemOvm3",
        "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
        "agents-of-edgewatch-bestiary:ai7Q9vBHHAGj7uFE",
        "agents-of-edgewatch-bestiary:zrh3MrS68H2gPlVs",
        "agents-of-edgewatch-bestiary:8jjEOs99Bmo1v0Qc",
        "agents-of-edgewatch-bestiary:WplBGSeB9pK9AULX",
        "agents-of-edgewatch-bestiary:BoFg19e3N8WiNa3Z",
        "agents-of-edgewatch-bestiary:4BXo2a305RHmspMX",
        "agents-of-edgewatch-bestiary:Rinxhe1cRXKEsXuW",
        "agents-of-edgewatch-bestiary:uFU6dQfcNeKq68YT",
        "agents-of-edgewatch-bestiary:tm8vQ7gdAe9zVdDg",
        "agents-of-edgewatch-bestiary:Fmsw7P5CF3uHtD5W",
        "agents-of-edgewatch-bestiary:GYhV5eYNDO1Llbv2",
        "agents-of-edgewatch-bestiary:5pBr5aWUb7yGCqN7",
        "agents-of-edgewatch-bestiary:syUXLdUsEDYgni5R",
        "agents-of-edgewatch-bestiary:8eBXEszbl4gOHGdU",
        "agents-of-edgewatch-bestiary:JmHyHwaPMNKXzBts",
        "agents-of-edgewatch-bestiary:PEjcy9CxelKC3Kp6",
        "battlecry-bestiary:HIwXNbXV2sfSiYf4",
        "battlecry-bestiary:tKPZ1iDZukJDvzAK",
        "battlecry-bestiary:mKQXrEpheSCjgJt8",
        "battlecry-bestiary:V2vFgzymDp2wKRwh",
        "battlecry-bestiary:xEWRFIUNr52EmwVM",
        "battlecry-bestiary:pY9fEPDxG925iivp",
      ]),
      insufficient_evidence: [],
      mixed_family_cues: [],
      manual_lore_only: [],
    },
  },
};

export function isReviewedDiscoveryReason(value: string): value is ReviewedDiscoveryReason {
  return REVIEWED_DISCOVERY_REASONS.includes(value as ReviewedDiscoveryReason);
}

export function getReviewedDiscoveryEntries(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoveryEntry[] {
  const requestedFamily = scope.family ? normalizeDerivedTag(scope.family) : undefined;
  const entries: ReviewedDiscoveryEntry[] = [];

  for (const [category, families] of Object.entries(registry) as Array<[SearchCategory, ReviewedDiscoveryRegistry[SearchCategory]]>) {
    if (!families) {
      continue;
    }
    if (scope.category && category !== scope.category) {
      continue;
    }

    for (const [family, reasonBuckets] of Object.entries(families)) {
      const normalizedFamily = normalizeDerivedTag(family);
      if (requestedFamily && normalizedFamily !== requestedFamily) {
        continue;
      }

      for (const [reason, records] of Object.entries(reasonBuckets ?? {}) as Array<[ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[] | undefined]>) {
        if (!records || records.length === 0) {
          continue;
        }
        if (scope.reason && reason !== scope.reason) {
          continue;
        }

        for (const record of records) {
          if (scope.subcategory !== undefined && (record.subcategory ?? null) !== scope.subcategory) {
            continue;
          }
          entries.push({
            category,
            family: normalizedFamily,
            reason,
            recordKey: record.recordKey,
            subcategory: record.subcategory ?? null,
            note: record.note,
          });
        }
      }
    }
  }

  return entries
    .slice()
    .sort((left, right) =>
      left.category.localeCompare(right.category) ||
      left.family.localeCompare(right.family) ||
      left.reason.localeCompare(right.reason) ||
      left.recordKey.localeCompare(right.recordKey));
}

export function getReviewedDiscoveryRecordKeys(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): string[] {
  return uniqueSorted(getReviewedDiscoveryEntries(scope, registry).map((entry) => entry.recordKey));
}

export function getReviewedDiscoveryReasonCounts(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoveryReasonCount[] {
  const entries = getReviewedDiscoveryEntries(scope, registry);
  const counts = new Map<ReviewedDiscoveryReason, Set<string>>();
  for (const entry of entries) {
    const bucket = counts.get(entry.reason) ?? new Set<string>();
    bucket.add(entry.recordKey);
    counts.set(entry.reason, bucket);
  }

  return [...counts.entries()]
    .map(([reason, recordKeys]) => ({
      reason,
      count: recordKeys.size,
    }))
    .sort((left, right) => right.count - left.count || left.reason.localeCompare(right.reason));
}

export function getReviewedDiscoverySelection(
  options: ReviewedDiscoverySelectionOptions,
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoverySelection | undefined {
  if (!options.family) {
    return undefined;
  }

  const reviewReason = options.reviewReason ?? null;
  const entries = getReviewedDiscoveryEntries({
    category: options.category,
    subcategory: options.subcategory,
    family: options.family,
    reason: reviewReason ?? undefined,
  }, registry);

  return {
    mode: reviewReason
      ? "filtered"
      : options.includeReviewed
        ? "included"
        : "excluded",
    reviewReason,
    entries,
    recordKeys: uniqueSorted(entries.map((entry) => entry.recordKey)),
    reasonCounts: getReviewedDiscoveryReasonCounts({
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      reason: reviewReason ?? undefined,
    }, registry),
  };
}

export function summarizeReviewedDiscoverySelection(
  selection: ReviewedDiscoverySelection,
  appliedCount = selection.recordKeys.length,
): ReviewedDiscoveryApplicationSummary {
  return {
    mode: selection.mode,
    reviewReason: selection.reviewReason,
    scopedCount: selection.recordKeys.length,
    appliedCount,
    reasonCounts: selection.reasonCounts,
  };
}
