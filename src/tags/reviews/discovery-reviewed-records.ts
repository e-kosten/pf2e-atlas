import { SearchCategory, SearchSubcategory } from "../../domain/derived-tag-types.js";
import { uniqueSorted } from "../../shared/utils.js";
import { normalizeDerivedTag } from "../runtime/matcher/shared.js";

// Durable reviewed discovery state lives with other editorial review inputs.
export const REVIEWED_DISCOVERY_REASONS = [
  "not_family_salient",
  "insufficient_evidence",
  "mixed_family_cues",
  "manual_lore_only",
] as const;

export type ReviewedDiscoveryReason = (typeof REVIEWED_DISCOVERY_REASONS)[number];

export type ReviewedDiscoveryRegistryRecord = {
  recordKey: string;
  pack?: string;
  name?: string;
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

export type ReviewedDiscoveryRegistry = Partial<
  Record<
    SearchCategory,
    Partial<Record<string, Partial<Record<ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[]>>>>
  >
>;

type ReviewedDiscoveryRecordReference = Pick<ReviewedDiscoveryRegistryRecord, "recordKey" | "pack" | "name">;

function reviewedRecordRefs(records: ReviewedDiscoveryRecordReference[]): ReviewedDiscoveryRegistryRecord[] {
  return records.map((record) => {
    const pack = record.pack ?? record.recordKey.split(":")[0] ?? "";
    if (!pack) {
      throw new Error(`Reviewed discovery record "${record.recordKey}" is missing a pack.`);
    }
    if (!record.recordKey.startsWith(`${pack}:`)) {
      throw new Error(`Reviewed discovery record "${record.recordKey}" does not match pack "${pack}".`);
    }
    if (!record.name?.trim()) {
      throw new Error(`Reviewed discovery record "${record.recordKey}" is missing a name.`);
    }

    return {
      recordKey: record.recordKey,
      pack,
      name: record.name.trim(),
    };
  });
}

// Curated during family-gap passes to suppress repeatedly reviewed negatives from the default queue.
export const REVIEWED_DISCOVERY_RECORDS: ReviewedDiscoveryRegistry = {
  creature: {
    setting: {
      // First 250 uncovered creature/setting records by pack+name, reviewed 2026-04-16.
      // Records handled by new rules or exact seeds in the same pass are intentionally excluded.
      not_family_salient: reviewedRecordRefs([
        {
          recordKey: "abomination-vaults-bestiary:3d3NAcPfvn07mcGN",
          pack: "abomination-vaults-bestiary",
          name: "Afflicted Irnakurse",
        },
        {
          recordKey: "abomination-vaults-bestiary:tOL4rWj2oYWZ4ow2",
          pack: "abomination-vaults-bestiary",
          name: "Aller Rosk",
        },
        {
          recordKey: "abomination-vaults-bestiary:saEUzIgUtV2AzKhl",
          pack: "abomination-vaults-bestiary",
          name: "Augrael",
        },
        {
          recordKey: "abomination-vaults-bestiary:DDJGNAh3rfyIupAb",
          pack: "abomination-vaults-bestiary",
          name: "Belcorra Haruvex",
        },
        {
          recordKey: "abomination-vaults-bestiary:3ry9WSvMMXHUe3kE",
          pack: "abomination-vaults-bestiary",
          name: "Beluthus",
        },
        {
          recordKey: "abomination-vaults-bestiary:rketcmqDQJbFFYfq",
          pack: "abomination-vaults-bestiary",
          name: "Bone Gladiator",
        },
        {
          recordKey: "abomination-vaults-bestiary:xAfkUwJYq5JLmSrW",
          pack: "abomination-vaults-bestiary",
          name: "Boss Skrawng",
        },
        {
          recordKey: "abomination-vaults-bestiary:ChRgdkplhO1D81Lg",
          pack: "abomination-vaults-bestiary",
          name: "Bright Walker",
        },
        {
          recordKey: "abomination-vaults-bestiary:oXnpdJVN6NIE58W3",
          pack: "abomination-vaults-bestiary",
          name: "Caliddo Haruvex",
        },
        {
          recordKey: "abomination-vaults-bestiary:lMCEVxKkQ7XK6Nid",
          pack: "abomination-vaults-bestiary",
          name: "Canker Cultist",
        },
        {
          recordKey: "abomination-vaults-bestiary:tYzzLLUv9WBhHhQY",
          pack: "abomination-vaults-bestiary",
          name: "Carman Rajani",
        },
        {
          recordKey: "abomination-vaults-bestiary:x0NDgH3EMLTLh02r",
          pack: "abomination-vaults-bestiary",
          name: "Chafkhem",
        },
        {
          recordKey: "abomination-vaults-bestiary:3H1rBpUQwTcNd6xZ",
          pack: "abomination-vaults-bestiary",
          name: "Chandriu Invisar",
        },
        {
          recordKey: "abomination-vaults-bestiary:HBRz8BVLVN9u9Odp",
          pack: "abomination-vaults-bestiary",
          name: "Corpselight",
        },
        {
          recordKey: "abomination-vaults-bestiary:T6vOuhM1KV5Fr75F",
          pack: "abomination-vaults-bestiary",
          name: "Gibtanius",
        },
        {
          recordKey: "abomination-vaults-bestiary:BJYrYqkV7PkXgSfk",
          pack: "abomination-vaults-bestiary",
          name: "Gibtas Bounder",
        },
        {
          recordKey: "abomination-vaults-bestiary:3F3fPq5hFbej40T2",
          pack: "abomination-vaults-bestiary",
          name: "Gibtas Spawn Swarm",
        },
        {
          recordKey: "abomination-vaults-bestiary:jE8BEe6pcnGraw2p",
          pack: "abomination-vaults-bestiary",
          name: "Jafaki",
        },
        {
          recordKey: "abomination-vaults-bestiary:hia81Ut7fEREbhkq",
          pack: "abomination-vaults-bestiary",
          name: "Jarelle Kaldrian",
        },
        {
          recordKey: "abomination-vaults-bestiary:EN3mp0sVObP8ou3p",
          pack: "abomination-vaults-bestiary",
          name: "Jaul Mezmin",
        },
        {
          recordKey: "abomination-vaults-bestiary:xj1Qn0VA4H4aKSjW",
          pack: "abomination-vaults-bestiary",
          name: "Jaul's Wolf",
        },
        {
          recordKey: "abomination-vaults-bestiary:dWOK0nzGWyc5NkNz",
          pack: "abomination-vaults-bestiary",
          name: "Lady's Whisper",
        },
        {
          recordKey: "abomination-vaults-bestiary:kzX588Hjb3w4QPOj",
          pack: "abomination-vaults-bestiary",
          name: "Mister Beak",
        },
        {
          recordKey: "abomination-vaults-bestiary:mlifDVJJWwjFtUxv",
          pack: "abomination-vaults-bestiary",
          name: "Murmur",
        },
        {
          recordKey: "abomination-vaults-bestiary:w2N0foudBFcRCaHK",
          pack: "abomination-vaults-bestiary",
          name: "Nhakazarin",
        },
        {
          recordKey: "abomination-vaults-bestiary:TiAzR8SnYwhACWbj",
          pack: "abomination-vaults-bestiary",
          name: "Observation Deck Seugathi Researcher",
        },
        {
          recordKey: "abomination-vaults-bestiary:R0EEgMDKcynpAWoa",
          pack: "abomination-vaults-bestiary",
          name: "Otari Ilvashti",
        },
        {
          recordKey: "abomination-vaults-bestiary:BOaM3pAuWl06Q6IZ",
          pack: "abomination-vaults-bestiary",
          name: "Poisoning Room Specter",
        },
        {
          recordKey: "abomination-vaults-bestiary:WR07Z6MjvebSHzI7",
          pack: "abomination-vaults-bestiary",
          name: "Ryta",
        },
        {
          recordKey: "abomination-vaults-bestiary:3vn9W5SThovdsEnY",
          pack: "abomination-vaults-bestiary",
          name: "Sacuishu",
        },
        {
          recordKey: "abomination-vaults-bestiary:qOkxxiM4tNf96CHQ",
          pack: "abomination-vaults-bestiary",
          name: "Seugathi Guard",
        },
        {
          recordKey: "abomination-vaults-bestiary:vS1YISLmSnkNotkL",
          pack: "abomination-vaults-bestiary",
          name: "Seugathi Reality Warper",
        },
        {
          recordKey: "abomination-vaults-bestiary:cMpgGvq1fGxh8wI0",
          pack: "abomination-vaults-bestiary",
          name: "Seugathi Researcher",
        },
        {
          recordKey: "abomination-vaults-bestiary:v9B0hB5sm4YZxebY",
          pack: "abomination-vaults-bestiary",
          name: "Seugathi Servant",
        },
        {
          recordKey: "abomination-vaults-bestiary:qXT1SQDtGqMkVl7Q",
          pack: "abomination-vaults-bestiary",
          name: "Shanrigol Heap",
        },
        {
          recordKey: "abomination-vaults-bestiary:ZAXuvUW6kl6v3SuW",
          pack: "abomination-vaults-bestiary",
          name: "Volluk Azrinae",
        },
        {
          recordKey: "abomination-vaults-bestiary:ZDYMKYZVyR8Fqakp",
          pack: "abomination-vaults-bestiary",
          name: "Wrin Sivinxi",
        },
        {
          recordKey: "age-of-ashes-bestiary:xrxjjjRMKzwsYbGm",
          pack: "age-of-ashes-bestiary",
          name: "Accursed Forge-Spurned",
        },
        { recordKey: "age-of-ashes-bestiary:YV3wA2Kgjp74l7YJ", pack: "age-of-ashes-bestiary", name: "Alak Stagram" },
        {
          recordKey: "age-of-ashes-bestiary:yrU0xi4eKBmcGudo",
          pack: "age-of-ashes-bestiary",
          name: "Animated Dragonstorm",
        },
        {
          recordKey: "age-of-ashes-bestiary:sQZDwS08l6Agsryq",
          pack: "age-of-ashes-bestiary",
          name: "Barushak Il-Varashma",
        },
        { recordKey: "age-of-ashes-bestiary:jO2fGZayk4R1AzYK", pack: "age-of-ashes-bestiary", name: "Bida" },
        { recordKey: "age-of-ashes-bestiary:91BSCAJA1Oto2ctf", pack: "age-of-ashes-bestiary", name: "Blood Boar" },
        {
          recordKey: "age-of-ashes-bestiary:4vEQ8zRXC51Qo4Mv",
          pack: "age-of-ashes-bestiary",
          name: "Bloody Blade Mercenary",
        },
        { recordKey: "age-of-ashes-bestiary:aaDiR0EIWRQx8wdy", pack: "age-of-ashes-bestiary", name: "Calmont" },
        {
          recordKey: "age-of-ashes-bestiary:53lk7ek73j65A09B",
          pack: "age-of-ashes-bestiary",
          name: "Candlaron's Echo",
        },
        { recordKey: "age-of-ashes-bestiary:yLhO5vUrPQF42Lh8", pack: "age-of-ashes-bestiary", name: "Charau-ka" },
        {
          recordKey: "age-of-ashes-bestiary:T3UVfAfcvAMe9rih",
          pack: "age-of-ashes-bestiary",
          name: "Charau-ka Dragon Priest",
        },
        { recordKey: "age-of-ashes-bestiary:w1Pqv0YmG4MevpQc", pack: "age-of-ashes-bestiary", name: "Corrupt Guard" },
        { recordKey: "age-of-ashes-bestiary:L4K4V09tQVXj7ZiI", pack: "age-of-ashes-bestiary", name: "Dmiri Yoltosha" },
        { recordKey: "age-of-ashes-bestiary:MOyYEls8wNGHoe8F", pack: "age-of-ashes-bestiary", name: "Doorwarden" },
        {
          recordKey: "age-of-ashes-bestiary:scPhFbBqlmD2fQHa",
          pack: "age-of-ashes-bestiary",
          name: "Dragonscarred Dead",
        },
        { recordKey: "age-of-ashes-bestiary:NwNu2yvMnbvPvpY8", pack: "age-of-ashes-bestiary", name: "Ekujae Guardian" },
        {
          recordKey: "age-of-ashes-bestiary:jZc4PrsX3HCJnkXx",
          pack: "age-of-ashes-bestiary",
          name: "Emaliza Zandivar",
        },
        { recordKey: "age-of-ashes-bestiary:dG5DBgrxlaimsWOS", pack: "age-of-ashes-bestiary", name: "Falrok" },
        { recordKey: "age-of-ashes-bestiary:tr3qlFJVHqloE9zI", pack: "age-of-ashes-bestiary", name: "Forge-Spurned" },
        {
          recordKey: "age-of-ashes-bestiary:UQDxkvqXKg03ESTQ",
          pack: "age-of-ashes-bestiary",
          name: "Gerhard Pendergrast",
        },
        { recordKey: "age-of-ashes-bestiary:OJH8y8LqmgbkN8ce", pack: "age-of-ashes-bestiary", name: "Ghastly Bear" },
        { recordKey: "age-of-ashes-bestiary:xXtGS9uBa9z43X7y", pack: "age-of-ashes-bestiary", name: "Graveshell" },
        { recordKey: "age-of-ashes-bestiary:dlW3UaXVpnzjd6xe", pack: "age-of-ashes-bestiary", name: "Heuberk Thropp" },
        { recordKey: "age-of-ashes-bestiary:MWQmOXxGcbaMsXDD", pack: "age-of-ashes-bestiary", name: "Hezle" },
        { recordKey: "age-of-ashes-bestiary:Ev930dfPpwCR8Zju", pack: "age-of-ashes-bestiary", name: "Ilgreth" },
        {
          recordKey: "age-of-ashes-bestiary:XHJC4G6bfPiVXGKE",
          pack: "age-of-ashes-bestiary",
          name: "Ilssrah Embermead",
        },
        { recordKey: "age-of-ashes-bestiary:AtiN3EsRpHn5qbuv", pack: "age-of-ashes-bestiary", name: "Immortal Ichor" },
        { recordKey: "age-of-ashes-bestiary:lcYcBIFmfKFt8Hcs", pack: "age-of-ashes-bestiary", name: "Ingnovim Tluss" },
        {
          recordKey: "age-of-ashes-bestiary:Ig1joUmSHSNL6QVU",
          pack: "age-of-ashes-bestiary",
          name: "Ingnovim's Assistant",
        },
        { recordKey: "age-of-ashes-bestiary:30FX1WFr0RkGeueX", pack: "age-of-ashes-bestiary", name: "Inizra Arumelo" },
        { recordKey: "age-of-ashes-bestiary:lRqptquQcM6ZcQ4O", pack: "age-of-ashes-bestiary", name: "Ishti" },
        { recordKey: "age-of-ashes-bestiary:mY494hn9sKVD9q8C", pack: "age-of-ashes-bestiary", name: "Jaggaki" },
        { recordKey: "age-of-ashes-bestiary:kciXaXw31gHA3gZl", pack: "age-of-ashes-bestiary", name: "Jahsi" },
        { recordKey: "age-of-ashes-bestiary:qemmmfu7exswGMGZ", pack: "age-of-ashes-bestiary", name: "Kelda Halrig" },
        { recordKey: "age-of-ashes-bestiary:C4PD4p4I9byZ8yp6", pack: "age-of-ashes-bestiary", name: "King Harral" },
        { recordKey: "age-of-ashes-bestiary:9oNwQuwKGUuG9G9g", pack: "age-of-ashes-bestiary", name: "Laslunn" },
        {
          recordKey: "age-of-ashes-bestiary:L7IJO5z82nEN9IjM",
          pack: "age-of-ashes-bestiary",
          name: "Lazurite-Infused Stone Golem",
        },
        {
          recordKey: "age-of-ashes-bestiary:N8vOjTD2SqS2b9Sy",
          pack: "age-of-ashes-bestiary",
          name: "Lesser Manifestation Of Dahak",
        },
        { recordKey: "age-of-ashes-bestiary:SVLUUPOXDINbKyFL", pack: "age-of-ashes-bestiary", name: "Malarunk" },
        {
          recordKey: "age-of-ashes-bestiary:sY47PB9b7nCJjgRq",
          pack: "age-of-ashes-bestiary",
          name: "Manifestation Of Dahak",
        },
        { recordKey: "age-of-ashes-bestiary:QKkvnlqrhgLHuP1t", pack: "age-of-ashes-bestiary", name: "Mialari Docur" },
        { recordKey: "age-of-ashes-bestiary:ZtSb3mHZ5sD2uqHd", pack: "age-of-ashes-bestiary", name: "Mud Spider" },
        { recordKey: "age-of-ashes-bestiary:NK49bh04355Lgz5r", pack: "age-of-ashes-bestiary", name: "Nketiah" },
        { recordKey: "age-of-ashes-bestiary:tGyPrTGSpndXKU88", pack: "age-of-ashes-bestiary", name: "Nolly Peltry" },
        { recordKey: "age-of-ashes-bestiary:Afq4Osh3W1k9Bcsh", pack: "age-of-ashes-bestiary", name: "Promise Guard" },
        { recordKey: "age-of-ashes-bestiary:60aeSJvu09ZM3SPx", pack: "age-of-ashes-bestiary", name: "Racharak" },
        {
          recordKey: "age-of-ashes-bestiary:z4rlpEsE2KgzpOCc",
          pack: "age-of-ashes-bestiary",
          name: "Remnant of Barzillai",
        },
        { recordKey: "age-of-ashes-bestiary:r02b4f1XNq7OihhD", pack: "age-of-ashes-bestiary", name: "Renali" },
        {
          recordKey: "age-of-ashes-bestiary:kLX36WXp6rjTt71z",
          pack: "age-of-ashes-bestiary",
          name: "Rinnarv Bontimar",
        },
        { recordKey: "age-of-ashes-bestiary:qouYGPM8mE4KUCTe", pack: "age-of-ashes-bestiary", name: "Rusty Mae" },
        {
          recordKey: "age-of-ashes-bestiary:U2QGjhcg5QFFkHwv",
          pack: "age-of-ashes-bestiary",
          name: "Saggorak Poltergeist",
        },
        {
          recordKey: "age-of-ashes-bestiary:mYJ6NNthl702Pz2s",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Agent",
        },
        {
          recordKey: "age-of-ashes-bestiary:SVh7cPwyGgmZORVz",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Boss",
        },
        {
          recordKey: "age-of-ashes-bestiary:Lx5JiVOGWnzzjCrW",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Bruiser",
        },
        {
          recordKey: "age-of-ashes-bestiary:egTpOr4Wc0L5e0iY",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Enforcer",
        },
        {
          recordKey: "age-of-ashes-bestiary:gF8Qy1k8gPBEUBAd",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Mage",
        },
        {
          recordKey: "age-of-ashes-bestiary:nQ2eBOpK71I8D2JC",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Poisoner",
        },
        {
          recordKey: "age-of-ashes-bestiary:1lkay2gwgEquq0NF",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Sneak",
        },
        {
          recordKey: "age-of-ashes-bestiary:7WNFNE3SVMRGbBDf",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Sniper",
        },
        {
          recordKey: "age-of-ashes-bestiary:NwLrwNqwedh95iry",
          pack: "age-of-ashes-bestiary",
          name: "Scarlet Triad Thug",
        },
        {
          recordKey: "age-of-ashes-bestiary:Aii18rhNPMLW4Pxh",
          pack: "age-of-ashes-bestiary",
          name: "Skeletal Hellknight",
        },
        { recordKey: "age-of-ashes-bestiary:t5mX2rkcKzcOzJXQ", pack: "age-of-ashes-bestiary", name: "Spawn of Dahak" },
        { recordKey: "age-of-ashes-bestiary:Fe1lYhCUY4UO4Plw", pack: "age-of-ashes-bestiary", name: "Talamira" },
        {
          recordKey: "age-of-ashes-bestiary:5r2S6FeE6D1Oh66T",
          pack: "age-of-ashes-bestiary",
          name: "Tarrasque, The Armageddon Engine",
        },
        { recordKey: "age-of-ashes-bestiary:X7zSx8LFh2ZDnOYy", pack: "age-of-ashes-bestiary", name: "Teyam Ishtori" },
        { recordKey: "age-of-ashes-bestiary:Oqj8XIWBb29NZ8QX", pack: "age-of-ashes-bestiary", name: "Tixitog" },
        { recordKey: "age-of-ashes-bestiary:aiNE06bxKD6jfoLd", pack: "age-of-ashes-bestiary", name: "Uri Zandivar" },
        { recordKey: "age-of-ashes-bestiary:Ejxngh2tHFseZQHW", pack: "age-of-ashes-bestiary", name: "Vaklish" },
        { recordKey: "age-of-ashes-bestiary:dTikLHqGfiSYemuZ", pack: "age-of-ashes-bestiary", name: "Veshumirix" },
        { recordKey: "age-of-ashes-bestiary:UD7EwTQG2Sbl4d8R", pack: "age-of-ashes-bestiary", name: "Voz Lirayne" },
        {
          recordKey: "age-of-ashes-bestiary:1tDEWL9mAIXvTYik",
          pack: "age-of-ashes-bestiary",
          name: "Warbal Bumblebrasher",
        },
        { recordKey: "age-of-ashes-bestiary:H7NO4Q7ctHnfGKeJ", pack: "age-of-ashes-bestiary", name: "Weathered Wail" },
        { recordKey: "age-of-ashes-bestiary:JD6kdfZveObBe1mR", pack: "age-of-ashes-bestiary", name: "Xotanispawn" },
        { recordKey: "age-of-ashes-bestiary:YIXAcvSyI1C94r9l", pack: "age-of-ashes-bestiary", name: "Zephyr Guard" },
        { recordKey: "age-of-ashes-bestiary:XNso2IMnhnfHcMCn", pack: "age-of-ashes-bestiary", name: "Zuferian" },
        {
          recordKey: "agents-of-edgewatch-bestiary:RtWlzHaOrfFdJyJY",
          pack: "agents-of-edgewatch-bestiary",
          name: "Alchemical Horror",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:phkvSUK6WXxgJOoC",
          pack: "agents-of-edgewatch-bestiary",
          name: "Alchemist Aspirant",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:gsn4NsJLwZCQUwgf",
          pack: "agents-of-edgewatch-bestiary",
          name: "Almiraj",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:1G4OdEHRPF8GMHK8",
          pack: "agents-of-edgewatch-bestiary",
          name: "Amateur Chemist",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:MONwgTbrcZFzr6vC",
          pack: "agents-of-edgewatch-bestiary",
          name: "Antaro Boldblade",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:azyIfDNNW44jY8YX",
          pack: "agents-of-edgewatch-bestiary",
          name: "Barrel Launcher",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:4ffoPNBKdEwBmYgL",
          pack: "agents-of-edgewatch-bestiary",
          name: "Battle Leader Rekarek",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:fvOjtzuRNpmpEHXA",
          pack: "agents-of-edgewatch-bestiary",
          name: "Binumir",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:7LN8clGKJWTpxISR",
          pack: "agents-of-edgewatch-bestiary",
          name: "Blackfingers Acolyte",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:y0bIU9FCWHOJxUzG",
          pack: "agents-of-edgewatch-bestiary",
          name: "Bloody Barber Goon",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:TIaZIUb9Mq9B4Mf2",
          pack: "agents-of-edgewatch-bestiary",
          name: "Bloody Berleth",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:sUrJ7jxzBiJTbwVo",
          pack: "agents-of-edgewatch-bestiary",
          name: "Blune Bandersworth",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:qY1NrXKmL0y18qoz",
          pack: "agents-of-edgewatch-bestiary",
          name: "Bolar Of Stonemoor",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:5u7luLMeRNJ4en65",
          pack: "agents-of-edgewatch-bestiary",
          name: "Bone Skipper Swarm",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:a3pTQfLzsJThQvI9",
          pack: "agents-of-edgewatch-bestiary",
          name: "Calennia",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:MQnhyCM9LInNYtl0",
          pack: "agents-of-edgewatch-bestiary",
          name: "Carvey",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:0ti3f4fdcB5D2bLB",
          pack: "agents-of-edgewatch-bestiary",
          name: "Casino Bouncer",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:9vt9Dr2a8MkkD83z",
          pack: "agents-of-edgewatch-bestiary",
          name: "Chalky",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:EGQgqBfV80ll3pcf",
          pack: "agents-of-edgewatch-bestiary",
          name: "Chaos Gulgamodh",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:wsVW8MdOTeGgGM59",
          pack: "agents-of-edgewatch-bestiary",
          name: "Child Of Venom",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:HifZEgdCuZearOG2",
          pack: "agents-of-edgewatch-bestiary",
          name: "Clockwork Amalgam",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:fV5VIoXMtixmI3Wc",
          pack: "agents-of-edgewatch-bestiary",
          name: "Clockwork Assassin",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:D36nL0YjCVHfjBNw",
          pack: "agents-of-edgewatch-bestiary",
          name: "Clockwork Chopper",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:w2J6GpuMYM24U4sb",
          pack: "agents-of-edgewatch-bestiary",
          name: "Clockwork Injector",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Wb4Md6byPhBWe56J",
          pack: "agents-of-edgewatch-bestiary",
          name: "Cobbleswarm (AoE)",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:JuCLvgvxYbSRXqON",
          pack: "agents-of-edgewatch-bestiary",
          name: "Copper Hand Illusionist",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:9M1YJxTXqya55HDx",
          pack: "agents-of-edgewatch-bestiary",
          name: "Copper Hand Rogue",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:JdMqHbaTwtOHVE7Y",
          pack: "agents-of-edgewatch-bestiary",
          name: "Diobel Sweeper Chemist",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:xkkj0TW6BKNT3Bg4",
          pack: "agents-of-edgewatch-bestiary",
          name: "Diobel Sweeper Tough",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:ZL2qLXwomKfBB8Eu",
          pack: "agents-of-edgewatch-bestiary",
          name: "Dreadsong Dancer",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:1eMDYXXf2leLTYHV",
          pack: "agents-of-edgewatch-bestiary",
          name: "Eberark",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:sn9Pjkr2jlMEqc3E",
          pack: "agents-of-edgewatch-bestiary",
          name: "Eunice",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:SgkV5RtcK72d0HwI",
          pack: "agents-of-edgewatch-bestiary",
          name: "Excorion",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:1UtsScuNjmgwMZRn",
          pack: "agents-of-edgewatch-bestiary",
          name: "Excorion Paragon",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:rM6ix6XTroJod3Vr",
          pack: "agents-of-edgewatch-bestiary",
          name: "Fayati Alummur",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:HF0ymYmKC6KydPQ1",
          pack: "agents-of-edgewatch-bestiary",
          name: "Franca Laurentz",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8AdsRaXftoR1beTk",
          pack: "agents-of-edgewatch-bestiary",
          name: "Frefferth",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:eqwAdGsAk5JZKxUY",
          pack: "agents-of-edgewatch-bestiary",
          name: "Gage Carlyle",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8GQ7dq7s9CetOlkg",
          pack: "agents-of-edgewatch-bestiary",
          name: "Gang Tough",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:pfHOwcITyC4gdCVu",
          pack: "agents-of-edgewatch-bestiary",
          name: "Garrote Master Assassin",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Xi53GFvTgBApltjp",
          pack: "agents-of-edgewatch-bestiary",
          name: "Giant Bone Skipper",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Uhi3wX4KveuMSARt",
          pack: "agents-of-edgewatch-bestiary",
          name: "Giant Joro Spider",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:JnNPdvOXtkGQHyfQ",
          pack: "agents-of-edgewatch-bestiary",
          name: "Gloaming Will-o'-Wisp",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:H4bY8v6e3drOIoUe",
          pack: "agents-of-edgewatch-bestiary",
          name: "Grabble Forden",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:rGTq4qItRB5H7nEk",
          pack: "agents-of-edgewatch-bestiary",
          name: "Graveknight Of Kharnas",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:pE5GxoB1FtXBqnF7",
          pack: "agents-of-edgewatch-bestiary",
          name: "Gref",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Bv1s6xJ55HS3Gxgs",
          pack: "agents-of-edgewatch-bestiary",
          name: "Grick",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:r3j5KEvULFP3fZS7",
          pack: "agents-of-edgewatch-bestiary",
          name: "Grimwold",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:gd0sVQtCHbhP8iHI",
          pack: "agents-of-edgewatch-bestiary",
          name: "Grospek Lavarsus",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:181ucNY1zpp2Lz3x",
          pack: "agents-of-edgewatch-bestiary",
          name: "Grunka",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:jJ1UTNLiRCoN1O3i",
          pack: "agents-of-edgewatch-bestiary",
          name: "Hendrid Pratchett",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:LAamprMlzk7k5auj",
          pack: "agents-of-edgewatch-bestiary",
          name: "Hestriviniaas",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:vwxNCuBksHYU2Dwf",
          pack: "agents-of-edgewatch-bestiary",
          name: "Hundun Chaos Mage",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:cLlOUUpCIAQwUuOP",
          pack: "agents-of-edgewatch-bestiary",
          name: "Il'setsya Wyrmtouched",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:pK1tlsgkmzkaaCe5",
          pack: "agents-of-edgewatch-bestiary",
          name: "Iroran Skeleton",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:M8ONVV7yl4uu0zcz",
          pack: "agents-of-edgewatch-bestiary",
          name: "Ixusoth",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:gIcReNQOZceZBBlw",
          pack: "agents-of-edgewatch-bestiary",
          name: "Jonis Flakfatter",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:E7NEf3kmsY3YjRrz",
          pack: "agents-of-edgewatch-bestiary",
          name: "Kapral",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:6c5CnrSxMYEgP6Fz",
          pack: "agents-of-edgewatch-bestiary",
          name: "Kekker",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:yR6p0KVvZ3tPflRt",
          pack: "agents-of-edgewatch-bestiary",
          name: "Kemeneles",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:3JEiwAFwkEjCDEYa",
          pack: "agents-of-edgewatch-bestiary",
          name: "Kolo Harvan",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:E279VPhAy1a4ihqI",
          pack: "agents-of-edgewatch-bestiary",
          name: "Living Mural",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:B4kIfCimsz8wfc0k",
          pack: "agents-of-edgewatch-bestiary",
          name: "Lord Guirden",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:lzmGdArS3kjJOqT6",
          pack: "agents-of-edgewatch-bestiary",
          name: "Lyrma Swampwalker",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:t9m4ikMZsDwo9TQ1",
          pack: "agents-of-edgewatch-bestiary",
          name: "Maurrisa Jonne",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Sq0Kb92nGkqj19Xx",
          pack: "agents-of-edgewatch-bestiary",
          name: "Miogimo",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:GWO6vweLGT2J6q62",
          pack: "agents-of-edgewatch-bestiary",
          name: "Miriel Grayleaf",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:ZY3q7AV1qbwWwNl2",
          pack: "agents-of-edgewatch-bestiary",
          name: "Mobana",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:07AGJt4ZRjwH85Xp",
          pack: "agents-of-edgewatch-bestiary",
          name: "Mother Venom",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Wk2T0Wr8Sebo4br5",
          pack: "agents-of-edgewatch-bestiary",
          name: "Mr. Snips",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:vPMmTtvl5UPOcCoa",
          pack: "agents-of-edgewatch-bestiary",
          name: "Myrna Rath",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:WDTdWiC9Rdl6rqh8",
          pack: "agents-of-edgewatch-bestiary",
          name: "Myrucarx",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:zj2sCM8tQSMG9Qm6",
          pack: "agents-of-edgewatch-bestiary",
          name: "Najra Lizard",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:RyXA4wOGY8lenKVw",
          pack: "agents-of-edgewatch-bestiary",
          name: "Nenchuuj",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:cQMM2Ld0IBM9GcDo",
          pack: "agents-of-edgewatch-bestiary",
          name: "Norgorberite Poisoner",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:1bPc2rjR4MghbMwD",
          pack: "agents-of-edgewatch-bestiary",
          name: "Obrousian",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:ySpOZlKUbcxWhKQ6",
          pack: "agents-of-edgewatch-bestiary",
          name: "Ofalth Zombie",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:oWASKud0jwlGSfJg",
          pack: "agents-of-edgewatch-bestiary",
          name: "Pelmo",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:3XbjmNeUtPLzxDge",
          pack: "agents-of-edgewatch-bestiary",
          name: "Penqual",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8HTPdiH6yEk0jlNF",
          pack: "agents-of-edgewatch-bestiary",
          name: "Pickled Punk",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:S9JJsUNSeeoIClON",
          pack: "agents-of-edgewatch-bestiary",
          name: "Poison Eater",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:LkaB8RH73DY4TO9V",
          pack: "agents-of-edgewatch-bestiary",
          name: "Priest of Blackfingers",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:MqfZxoxFwzqAXhTP",
          pack: "agents-of-edgewatch-bestiary",
          name: "Prospecti Statue",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:2uxm1SxZXaG0ynCp",
          pack: "agents-of-edgewatch-bestiary",
          name: "Pulping Golem",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:JZUYQzQtzIwOGYvd",
          pack: "agents-of-edgewatch-bestiary",
          name: "Ralso",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:aDgVmO3afhIgXQSN",
          pack: "agents-of-edgewatch-bestiary",
          name: "Ravenile Rager",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:BuPf7xtqfwCjNOQv",
          pack: "agents-of-edgewatch-bestiary",
          name: "Reginald Vancaskerkin",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:kcvU9CatCyBUJRr2",
          pack: "agents-of-edgewatch-bestiary",
          name: "Rhevanna",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:4TB1eo3O22khMyDU",
          pack: "agents-of-edgewatch-bestiary",
          name: "Sad Liza",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:CLntGVs7cAIL9Trk",
          pack: "agents-of-edgewatch-bestiary",
          name: "Scathka",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:BCMkxoQB4xK4BniG",
          pack: "agents-of-edgewatch-bestiary",
          name: "Secret-Keeper",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:1xGAktpj6N2Ugh0r",
          pack: "agents-of-edgewatch-bestiary",
          name: "Shatterling",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:DKReNCapWWubM3pm",
          pack: "agents-of-edgewatch-bestiary",
          name: "Shikwashim Mercenary",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:eQc0ADMuHl1JzL8z",
          pack: "agents-of-edgewatch-bestiary",
          name: "Shredskin",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:oTa25rAxytm03T3X",
          pack: "agents-of-edgewatch-bestiary",
          name: "Shristi Melipdra",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:XDt87cqF85zWnlC8",
          pack: "agents-of-edgewatch-bestiary",
          name: "Siege Shard",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:nTn2szBbqQNdXhOr",
          pack: "agents-of-edgewatch-bestiary",
          name: "Skebs",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8PRqUfkHLvu9ufGL",
          pack: "agents-of-edgewatch-bestiary",
          name: "Skinsaw Murderer",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:y1tw2ohNagqQJ6RV",
          pack: "agents-of-edgewatch-bestiary",
          name: "Skinsaw Seamer",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:AYNIAAxV7TbIKPI4",
          pack: "agents-of-edgewatch-bestiary",
          name: "Skitterstitch",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:xN3FDmrCKWW0psBu",
          pack: "agents-of-edgewatch-bestiary",
          name: "Sleepless Sun Veteran",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:P9Wg0sGcNkemOvm3",
          pack: "agents-of-edgewatch-bestiary",
          name: "Slithering Rift",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb",
          pack: "agents-of-edgewatch-bestiary",
          name: "Starwatch Commando",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:ai7Q9vBHHAGj7uFE",
          pack: "agents-of-edgewatch-bestiary",
          name: "Svartalfar Killer",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:zrh3MrS68H2gPlVs",
          pack: "agents-of-edgewatch-bestiary",
          name: "Tenome",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8jjEOs99Bmo1v0Qc",
          pack: "agents-of-edgewatch-bestiary",
          name: "Teraphant",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:WplBGSeB9pK9AULX",
          pack: "agents-of-edgewatch-bestiary",
          name: "The Rabbit Prince",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:BoFg19e3N8WiNa3Z",
          pack: "agents-of-edgewatch-bestiary",
          name: "The Stabbing Beast",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:4BXo2a305RHmspMX",
          pack: "agents-of-edgewatch-bestiary",
          name: "Twisted Jack",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Rinxhe1cRXKEsXuW",
          pack: "agents-of-edgewatch-bestiary",
          name: "Tyrroicese",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:uFU6dQfcNeKq68YT",
          pack: "agents-of-edgewatch-bestiary",
          name: "Vargouille",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:tm8vQ7gdAe9zVdDg",
          pack: "agents-of-edgewatch-bestiary",
          name: "Vaultbreaker Ooze",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:Fmsw7P5CF3uHtD5W",
          pack: "agents-of-edgewatch-bestiary",
          name: "Veksciralenix",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:GYhV5eYNDO1Llbv2",
          pack: "agents-of-edgewatch-bestiary",
          name: "Venom Mage",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:5pBr5aWUb7yGCqN7",
          pack: "agents-of-edgewatch-bestiary",
          name: "Washboard Dog Tough",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:syUXLdUsEDYgni5R",
          pack: "agents-of-edgewatch-bestiary",
          name: "Wrent Dicaspiron",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:8eBXEszbl4gOHGdU",
          pack: "agents-of-edgewatch-bestiary",
          name: "Wynsal Starborn",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:JmHyHwaPMNKXzBts",
          pack: "agents-of-edgewatch-bestiary",
          name: "Zealborn",
        },
        {
          recordKey: "agents-of-edgewatch-bestiary:PEjcy9CxelKC3Kp6",
          pack: "agents-of-edgewatch-bestiary",
          name: "Zrukbat",
        },
        { recordKey: "battlecry-bestiary:HIwXNbXV2sfSiYf4", pack: "battlecry-bestiary", name: "Animated Army" },
        {
          recordKey: "battlecry-bestiary:tKPZ1iDZukJDvzAK",
          pack: "battlecry-bestiary",
          name: "Apprentice Magician Clique",
        },
        { recordKey: "battlecry-bestiary:mKQXrEpheSCjgJt8", pack: "battlecry-bestiary", name: "Dezullon Thicket" },
        { recordKey: "battlecry-bestiary:V2vFgzymDp2wKRwh", pack: "battlecry-bestiary", name: "Druid Circle" },
        { recordKey: "battlecry-bestiary:xEWRFIUNr52EmwVM", pack: "battlecry-bestiary", name: "Fleshwarp Amalgam" },
        { recordKey: "battlecry-bestiary:pY9fEPDxG925iivp", pack: "battlecry-bestiary", name: "Giant Ant Army" },
        // Second 250 uncovered creature/setting records by pack+name after the 2026-04-16 first-pass exclusions.
        // Records handled by troop battlefield coverage, direct seeds, or new setting tags in the same pass are intentionally excluded.
        { recordKey: "blog-bestiary:bupIhMxa1zvWI91a", pack: "blog-bestiary", name: "Dust Bunny Swarm" },
        { recordKey: "blog-bestiary:TejC7NPsqRPHwOe3", pack: "blog-bestiary", name: "Edolpho Phinelli" },
        { recordKey: "blog-bestiary:GQE978SzTLS0Sa8B", pack: "blog-bestiary", name: "Morlibint" },
        { recordKey: "blog-bestiary:FdsQgecBdj4h5qca", pack: "blog-bestiary", name: "Rt5rrmn" },
        { recordKey: "blog-bestiary:mngmrx4PtFoFMp5I", pack: "blog-bestiary", name: "Urok" },
        { recordKey: "blog-bestiary:Nd54wB2wWnpPbHCc", pack: "blog-bestiary", name: "Whispering Way Medusa" },
        { recordKey: "blog-bestiary:zvxYdvU6TAxoFMRu", pack: "blog-bestiary", name: "Wordsmith" },
        { recordKey: "blood-lords-bestiary:zEPIkUw9ZlXi5Jzb", pack: "blood-lords-bestiary", name: "Afziaka Brute" },
        { recordKey: "blood-lords-bestiary:OgaEYZJaxGXPnhcz", pack: "blood-lords-bestiary", name: "Afziaka Stalker" },
        { recordKey: "blood-lords-bestiary:9yd6jVPcLusk4BX1", pack: "blood-lords-bestiary", name: "Ancient Skaveling" },
        {
          recordKey: "blood-lords-bestiary:DIDEV6UK5S5jQunK",
          pack: "blood-lords-bestiary",
          name: "Animated Fireplace",
        },
        {
          recordKey: "blood-lords-bestiary:btuLRbWKuZK7ozti",
          pack: "blood-lords-bestiary",
          name: "Arghun the Annihilator",
        },
        { recordKey: "blood-lords-bestiary:PCGYst9QySIxsjDK", pack: "blood-lords-bestiary", name: "Chattering Jaws" },
        { recordKey: "blood-lords-bestiary:PFAxHAnorkDmvi26", pack: "blood-lords-bestiary", name: "Clockwork Rifler" },
        { recordKey: "blood-lords-bestiary:4Z704FJ69DWl71vP", pack: "blood-lords-bestiary", name: "Creeping Crone" },
        { recordKey: "blood-lords-bestiary:LRHYWNJg7231HUOc", pack: "blood-lords-bestiary", name: "Dirge Piper" },
        {
          recordKey: "blood-lords-bestiary:AvF72rXp6bcQHPdo",
          pack: "blood-lords-bestiary",
          name: "Facetbound Cascader",
        },
        {
          recordKey: "blood-lords-bestiary:A3joovUPMBOVDpVz",
          pack: "blood-lords-bestiary",
          name: "Facetbound Nullifier",
        },
        { recordKey: "blood-lords-bestiary:6I4ZtQJPT0ToITvq", pack: "blood-lords-bestiary", name: "Floating Femur" },
        { recordKey: "blood-lords-bestiary:p6wud30R7C5OAFGz", pack: "blood-lords-bestiary", name: "Ghast Outlaw" },
        { recordKey: "blood-lords-bestiary:HNcBJFUhdLfOZxPS", pack: "blood-lords-bestiary", name: "Ghiasi" },
        { recordKey: "blood-lords-bestiary:7Ah4NAOuhPqMQ5e7", pack: "blood-lords-bestiary", name: "Ghiasi's Double" },
        { recordKey: "blood-lords-bestiary:0PJ75DaIe8x5ynp1", pack: "blood-lords-bestiary", name: "Ghoul Antipaladin" },
        { recordKey: "blood-lords-bestiary:0a2d830uaGnBN2OS", pack: "blood-lords-bestiary", name: "Ghoul Razorclaw" },
        {
          recordKey: "blood-lords-bestiary:pg4YAN1whVZsnLKf",
          pack: "blood-lords-bestiary",
          name: 'Grace "The Rhino" Owano',
        },
        { recordKey: "blood-lords-bestiary:aZMbCNaUnmqIVoyA", pack: "blood-lords-bestiary", name: "Granite Vulture" },
        { recordKey: "blood-lords-bestiary:7BqiEDwmEPDwGMAn", pack: "blood-lords-bestiary", name: "Harmony In Agony" },
        { recordKey: "blood-lords-bestiary:3t57lJmLz6qQPqoD", pack: "blood-lords-bestiary", name: "Hyrune Loxenna" },
        {
          recordKey: "blood-lords-bestiary:3Yi3PBesesksFZi6",
          pack: "blood-lords-bestiary",
          name: "Intellect Assemblage",
        },
        { recordKey: "blood-lords-bestiary:zrMBLTlxPCIwtwGo", pack: "blood-lords-bestiary", name: "Iron Taviah" },
        { recordKey: "blood-lords-bestiary:oVqkX1LugEW4qoXN", pack: "blood-lords-bestiary", name: "Kelganth" },
        { recordKey: "blood-lords-bestiary:X9Sz3ttieDhYSlvB", pack: "blood-lords-bestiary", name: "Kemnebi" },
        {
          recordKey: "blood-lords-bestiary:5pfU6ZAyAWq3VEdq",
          pack: "blood-lords-bestiary",
          name: "Kepgeda the Hag-Nailed",
        },
        { recordKey: "blood-lords-bestiary:cUUyIZ3Y8VAbfI9P", pack: "blood-lords-bestiary", name: "Kerinza" },
        { recordKey: "blood-lords-bestiary:bZPLaHBF66XuRQlv", pack: "blood-lords-bestiary", name: "Lasheeli" },
        { recordKey: "blood-lords-bestiary:Kq2dxvP4wqJJVzBr", pack: "blood-lords-bestiary", name: "Meat Guardian" },
        { recordKey: "blood-lords-bestiary:PTAKDBMDrm1JJTYC", pack: "blood-lords-bestiary", name: "Mithral Golem" },
        { recordKey: "blood-lords-bestiary:SSFzb5JBZDG6sr9B", pack: "blood-lords-bestiary", name: "Opkherab" },
        {
          recordKey: "blood-lords-bestiary:t92NDPmfXnOke88G",
          pack: "blood-lords-bestiary",
          name: "Pesgahi the Poisoner",
        },
        {
          recordKey: "blood-lords-bestiary:nX7wdbO4lvdGlFPM",
          pack: "blood-lords-bestiary",
          name: "Phalanx of Phalanges",
        },
        {
          recordKey: "blood-lords-bestiary:yydyiiBoUGrzPu2v",
          pack: "blood-lords-bestiary",
          name: "Pokmit Bloody-Pike",
        },
        { recordKey: "blood-lords-bestiary:624fuRv6nLi31THS", pack: "blood-lords-bestiary", name: "Prachalla" },
        { recordKey: "blood-lords-bestiary:FVBJjBbvITVpHiec", pack: "blood-lords-bestiary", name: "Prince Doriel" },
        { recordKey: "blood-lords-bestiary:1YdRAByWMiyUIXwt", pack: "blood-lords-bestiary", name: "Princess Kerinza" },
        { recordKey: "blood-lords-bestiary:0iZHpYIbxquRDd8E", pack: "blood-lords-bestiary", name: "Pyrogeist" },
        { recordKey: "blood-lords-bestiary:MKYtgU2G42jk2Dwo", pack: "blood-lords-bestiary", name: "Restored Doll" },
        { recordKey: "blood-lords-bestiary:1EOyUGMELVUIxFeB", pack: "blood-lords-bestiary", name: "Rotbomber" },
        { recordKey: "blood-lords-bestiary:xtDVxT49QGCo78yp", pack: "blood-lords-bestiary", name: "Rumin Purgo" },
        {
          recordKey: "blood-lords-bestiary:CVBM8oY76Yw4IxN5",
          pack: "blood-lords-bestiary",
          name: "Scrabbling Ribcage",
        },
        { recordKey: "blood-lords-bestiary:DInkix6cy2NgjpJd", pack: "blood-lords-bestiary", name: "Shabti Slayer" },
        { recordKey: "blood-lords-bestiary:ZDrt7yX9seyY3lCC", pack: "blood-lords-bestiary", name: "Shabti Votary" },
        { recordKey: "blood-lords-bestiary:ppFL3vBBYT4EXQom", pack: "blood-lords-bestiary", name: "Shadow Leydroth" },
        {
          recordKey: "blood-lords-bestiary:yfyh7TkvcQuyWyie",
          pack: "blood-lords-bestiary",
          name: "Shadowbound Monk Statue",
        },
        { recordKey: "blood-lords-bestiary:NoOG4QIbIEBWKk1I", pack: "blood-lords-bestiary", name: "Soul Slime" },
        { recordKey: "blood-lords-bestiary:2p9BaMPkI0HiTWyC", pack: "blood-lords-bestiary", name: "Sulvik" },
        {
          recordKey: "blood-lords-bestiary:GVIv3WxwbwztNmvD",
          pack: "blood-lords-bestiary",
          name: "Teaching Assistant",
        },
        { recordKey: "blood-lords-bestiary:yQHyvoLOi6HxUTfe", pack: "blood-lords-bestiary", name: "Umbraex" },
        { recordKey: "blood-lords-bestiary:pqlqDWVOtPxJ2oRn", pack: "blood-lords-bestiary", name: "Urbulinex" },
        { recordKey: "blood-lords-bestiary:syFR3SChA5KCx8OA", pack: "blood-lords-bestiary", name: "Vampire Guardian" },
        { recordKey: "blood-lords-bestiary:RIKDtmwVQQeJUZKg", pack: "blood-lords-bestiary", name: "Vampire Taviah" },
        {
          recordKey: "blood-lords-bestiary:epfj3ZAwLHMlYlF3",
          pack: "blood-lords-bestiary",
          name: "Virulak Necromancer",
        },
        { recordKey: "blood-lords-bestiary:CWg8NX6XHv2sEVlg", pack: "blood-lords-bestiary", name: "Weeping Jack" },
        { recordKey: "blood-lords-bestiary:pHiRAXi3YxFhBeLi", pack: "blood-lords-bestiary", name: "Yshula" },
        { recordKey: "blood-lords-bestiary:e5iMWebwtUQrOtPB", pack: "blood-lords-bestiary", name: "Yulthruk" },
        { recordKey: "blood-lords-bestiary:noCFIy7gHIaaQEdJ", pack: "blood-lords-bestiary", name: "Yurgak" },
        {
          recordKey: "book-of-the-dead-bestiary:NcqruxA82SFvTnD1",
          pack: "book-of-the-dead-bestiary",
          name: "Child of Urgathoa",
        },
        {
          recordKey: "book-of-the-dead-bestiary:ishwgxZAlNJxNwGE",
          pack: "book-of-the-dead-bestiary",
          name: "Daqqanoenyent",
        },
        {
          recordKey: "book-of-the-dead-bestiary:27NxweHr9rB1hDCn",
          pack: "book-of-the-dead-bestiary",
          name: "Deathless Acolyte Of Urgathoa",
        },
        {
          recordKey: "book-of-the-dead-bestiary:h0OtQsMR4OqnYatd",
          pack: "book-of-the-dead-bestiary",
          name: "Deathless Hierophant Of Urgathoa",
        },
        {
          recordKey: "book-of-the-dead-bestiary:sucEX2JFrVevTNjU",
          pack: "book-of-the-dead-bestiary",
          name: "Drake Skeleton",
        },
        { recordKey: "book-of-the-dead-bestiary:zqftTUpxqkdLx2IY", pack: "book-of-the-dead-bestiary", name: "Ecorche" },
        {
          recordKey: "book-of-the-dead-bestiary:KhHVStbsPSuPElFI",
          pack: "book-of-the-dead-bestiary",
          name: "Excorion",
        },
        {
          recordKey: "book-of-the-dead-bestiary:A5GOni2mXNhVsdRg",
          pack: "book-of-the-dead-bestiary",
          name: "Faithless Ecclesiarch",
        },
        {
          recordKey: "book-of-the-dead-bestiary:JazJz2crkoG9koQR",
          pack: "book-of-the-dead-bestiary",
          name: "Fallen Champion",
        },
        {
          recordKey: "book-of-the-dead-bestiary:EDhme1IKgO34NDrt",
          pack: "book-of-the-dead-bestiary",
          name: "Festering Gnasher",
        },
        {
          recordKey: "book-of-the-dead-bestiary:jIypNMJE7rVYpItG",
          pack: "book-of-the-dead-bestiary",
          name: "Fiddling Bones",
        },
        {
          recordKey: "book-of-the-dead-bestiary:hDQmEtisrgRmufUW",
          pack: "book-of-the-dead-bestiary",
          name: "Fluxwraith",
        },
        { recordKey: "book-of-the-dead-bestiary:iEQOUQk1wVHFsajW", pack: "book-of-the-dead-bestiary", name: "Geist" },
        {
          recordKey: "book-of-the-dead-bestiary:bckYmdaq03CUDdc5",
          pack: "book-of-the-dead-bestiary",
          name: "Gholdako",
        },
        {
          recordKey: "book-of-the-dead-bestiary:c6qRZuHQ7RHJEAtj",
          pack: "book-of-the-dead-bestiary",
          name: "Grappling Spirit",
        },
        {
          recordKey: "book-of-the-dead-bestiary:Wv9iS7JacYjymqlY",
          pack: "book-of-the-dead-bestiary",
          name: "Harlo Krant",
        },
        {
          recordKey: "book-of-the-dead-bestiary:wGEZUOgoJNkgXx9Z",
          pack: "book-of-the-dead-bestiary",
          name: "Horde Lich",
        },
        {
          recordKey: "book-of-the-dead-bestiary:7JWjoMGf7f7PZgSD",
          pack: "book-of-the-dead-bestiary",
          name: "Ice Mummy",
        },
        {
          recordKey: "book-of-the-dead-bestiary:zOEuwWcvD7sQA1kc",
          pack: "book-of-the-dead-bestiary",
          name: "Ichor Slinger",
        },
        {
          recordKey: "book-of-the-dead-bestiary:dlizffh8cFUFOhUf",
          pack: "book-of-the-dead-bestiary",
          name: "Jitterbone Contortionist",
        },
        { recordKey: "book-of-the-dead-bestiary:uqY9TkQTO5n9rCLZ", pack: "book-of-the-dead-bestiary", name: "Llorona" },
        { recordKey: "book-of-the-dead-bestiary:mL4l2kwPPSMKwzQo", pack: "book-of-the-dead-bestiary", name: "Onryo" },
        {
          recordKey: "book-of-the-dead-bestiary:GcHzyaMYK5QeKUyM",
          pack: "book-of-the-dead-bestiary",
          name: "Pale Stranger",
        },
        { recordKey: "book-of-the-dead-bestiary:M59XiYnJ4Z3bSwCC", pack: "book-of-the-dead-bestiary", name: "Polong" },
        {
          recordKey: "book-of-the-dead-bestiary:e49MDE5dJQ1XFq3O",
          pack: "book-of-the-dead-bestiary",
          name: "Predatory Rabbit",
        },
        {
          recordKey: "book-of-the-dead-bestiary:OsIjZpdtKmKrrBID",
          pack: "book-of-the-dead-bestiary",
          name: "Priest Of Kabriri",
        },
        {
          recordKey: "book-of-the-dead-bestiary:j88xR2MqqZrmF5Wz",
          pack: "book-of-the-dead-bestiary",
          name: "Raw Nerve",
        },
        {
          recordKey: "book-of-the-dead-bestiary:cEJh8SY1YDULyhI9",
          pack: "book-of-the-dead-bestiary",
          name: "Relictner Eroder",
        },
        {
          recordKey: "book-of-the-dead-bestiary:zPQZLswJISQrvPb7",
          pack: "book-of-the-dead-bestiary",
          name: "Runecarved Lich",
        },
        {
          recordKey: "book-of-the-dead-bestiary:z3BpUMkUFoXqOFgf",
          pack: "book-of-the-dead-bestiary",
          name: "Scorned Hound",
        },
        {
          recordKey: "book-of-the-dead-bestiary:pDA6tCwQ3pE6ji4U",
          pack: "book-of-the-dead-bestiary",
          name: "Shadern Immolator",
        },
        {
          recordKey: "book-of-the-dead-bestiary:eEE3lRhXCjGzlRLJ",
          pack: "book-of-the-dead-bestiary",
          name: "Shredskin",
        },
        { recordKey: "book-of-the-dead-bestiary:hhoSyH9QthtvFptC", pack: "book-of-the-dead-bestiary", name: "Siabrae" },
        {
          recordKey: "book-of-the-dead-bestiary:Cl1SZNiURw65rz4p",
          pack: "book-of-the-dead-bestiary",
          name: "Silent Stalker",
        },
        {
          recordKey: "book-of-the-dead-bestiary:ipVQuGff2OeTVwFK",
          pack: "book-of-the-dead-bestiary",
          name: "Skeletal Mage",
        },
        {
          recordKey: "book-of-the-dead-bestiary:dKkHFA4aBgk82QJO",
          pack: "book-of-the-dead-bestiary",
          name: "Skeletal Soldier",
        },
        {
          recordKey: "book-of-the-dead-bestiary:uURI8Netd0ytD9vc",
          pack: "book-of-the-dead-bestiary",
          name: "Sluagh Reaper",
        },
        {
          recordKey: "book-of-the-dead-bestiary:Ur3dzfmvtN7lyPNG",
          pack: "book-of-the-dead-bestiary",
          name: "Taunting Skull",
        },
        {
          recordKey: "book-of-the-dead-bestiary:0PrrwvV1936eSCQy",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Burning)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:AXp5EzgP7p8FrYpN",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Crushing)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:f2dAjBXK55w6rnsh",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Dislocation)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:dcLHA7tihCF2Mraj",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Drowning)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:VN2Vz1dxA9ti66bC",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Impalement)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:hjBcN7ulP6FSK7ie",
          pack: "book-of-the-dead-bestiary",
          name: "Tormented (Starvation)",
        },
        {
          recordKey: "book-of-the-dead-bestiary:LWLUaSHl8YCZdDMH",
          pack: "book-of-the-dead-bestiary",
          name: "Vetalarana Emergent",
        },
        {
          recordKey: "book-of-the-dead-bestiary:Nzf3AfA46cBiWCwN",
          pack: "book-of-the-dead-bestiary",
          name: "Vetalarana Manipulator",
        },
        {
          recordKey: "book-of-the-dead-bestiary:kY8MSttryLjbI5wN",
          pack: "book-of-the-dead-bestiary",
          name: "Wight Commander",
        },
        {
          recordKey: "book-of-the-dead-bestiary:z72hPIIjIiLIxnor",
          pack: "book-of-the-dead-bestiary",
          name: "Withered",
        },
        {
          recordKey: "book-of-the-dead-bestiary:8o27tdIZFp0eTs5N",
          pack: "book-of-the-dead-bestiary",
          name: "Zombie Lord",
        },
        {
          recordKey: "book-of-the-dead-bestiary:nbVljZhCnWgGxA18",
          pack: "book-of-the-dead-bestiary",
          name: "Zombie Owlbear",
        },
        {
          recordKey: "book-of-the-dead-bestiary:ZqZlji7aCGCGATMP",
          pack: "book-of-the-dead-bestiary",
          name: "Zombie Snake",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:ElTYWqGvup7okvC5",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Brumesgarth",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:U6kC8ohP723gydUI",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Divine Warden of Iomedae",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:mGTK0uSZiQY0vsCf",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Garrholdion",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:aEBup6SntDIOlTtY",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Lady Siccale",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:OlBw2JqWcb3aBfKf",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Mirmicette",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:hXEFhS8iL5d6MqdD",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Moldering Steed",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:M2Co6b3pKO3TOWQj",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Omelia",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:ZcrucwbmzI5J5jvk",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Radiant Veranallia",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:Kqd7MVuRRYvvZQ0g",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Seldeg Bhedlis",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:RiB74rq2dsKATghi",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Splinter Officer",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:OZRvOYnkvEVlTx90",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Stelemora",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:Wem1kZUef1ZlG79G",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Ashen Guardian",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:x5UScdVKs3e9rsgb",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Dark Talon Kobold",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:4kAU6bQdEZYCqEh7",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Dismemberment Table",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:BWOdyTjHnV153RDK",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Drazmorg The Damned",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:NGofilVufDzya0uJ",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Edgrin Galesong",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:XH1Wqnxy8Cq9Z5Cn",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Grimbal",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:bwJ79rNvV7Elia5Q",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Gurtlekep",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:gGtA1Dhiai4jOTte",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Hymmir Urath",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:dcuPAoJUTKSSFx93",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Jekkajak",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:QbV3wJXAyLz4Drad",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Kapmek",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:s2XZc2b1FaZQDnhN",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Kerrdremak",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:qIOi43L0KcLoqCu6",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Kieragan Skross",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:xDF9jeMsYjrRvuE5",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "King Merlokrep",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:QhJH546TLP0aQKiR",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Lekmek",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:LYSdv8QlMgV0U5Zq",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Narlynark",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:2D2sECCnlFT8X8OQ",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Tallow Guardian",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:IYrz3P4rsRr0TnmK",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "The Disciples",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:WukFzZnikG2vg57F",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Ulizmila's Cauldron",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:IZ34VEavH7xRaaeI",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Ygrik",
        },
        { recordKey: "curtain-call-bestiary:SGWcXt1bqMh09qab", pack: "curtain-call-bestiary", name: "Animated Boiler" },
        {
          recordKey: "curtain-call-bestiary:8L1GpUj3eJgaeFUk",
          pack: "curtain-call-bestiary",
          name: "Augusta Wormwood",
        },
        { recordKey: "curtain-call-bestiary:yNkLAeHXYRtrPdz5", pack: "curtain-call-bestiary", name: "Azarpal" },
        { recordKey: "curtain-call-bestiary:VOtMQkKzvL7nMGcX", pack: "curtain-call-bestiary", name: "Blackfingers" },
        {
          recordKey: "curtain-call-bestiary:ar9K6cpiG6G3v2wZ",
          pack: "curtain-call-bestiary",
          name: "Delaraius Solzakarr",
        },
        { recordKey: "curtain-call-bestiary:RB1DZPTjSC0mnw95", pack: "curtain-call-bestiary", name: "Echopsyvne" },
        // Third 250 uncovered creature/setting records by pack+name after the 2026-04-16 second-pass exclusions.
        // Records handled by new Tian Xia coverage, swamp and fortress rule extensions, or direct seeds in the same pass are intentionally excluded.
        { recordKey: "curtain-call-bestiary:NmzB37eKJpmflJIl", pack: "curtain-call-bestiary", name: "Egarhowl" },
        { recordKey: "curtain-call-bestiary:LIxnoH1iC1vEP5xq", pack: "curtain-call-bestiary", name: "Fallenta" },
        { recordKey: "curtain-call-bestiary:m3VQLbUy5rKWJOyA", pack: "curtain-call-bestiary", name: "Father Skinsaw" },
        { recordKey: "curtain-call-bestiary:N5sypiEac4BKyeQb", pack: "curtain-call-bestiary", name: "Fenton Vilorgo" },
        {
          recordKey: "curtain-call-bestiary:ZaT4rd3bTIfECkCd",
          pack: "curtain-call-bestiary",
          name: "Fenton's Faithful",
        },
        { recordKey: "curtain-call-bestiary:ylAEACovrsq8XaOa", pack: "curtain-call-bestiary", name: "Graverobber" },
        { recordKey: "curtain-call-bestiary:1eX4Csnv3psAsfLf", pack: "curtain-call-bestiary", name: "Gray Master" },
        {
          recordKey: "curtain-call-bestiary:BgPshRvqaBvy4ulr",
          pack: "curtain-call-bestiary",
          name: "Karumzek Priest (Dispel Magic)",
        },
        {
          recordKey: "curtain-call-bestiary:9qFTRSaEYnrYDVHH",
          pack: "curtain-call-bestiary",
          name: "Karumzek Priest (Divine Immolation)",
        },
        {
          recordKey: "curtain-call-bestiary:REOHdavtL7k20KQc",
          pack: "curtain-call-bestiary",
          name: "Karumzek Priest (Eclipse Burst)",
        },
        { recordKey: "curtain-call-bestiary:jyKGV1HeXcLF1E5f", pack: "curtain-call-bestiary", name: "Kimingio" },
        {
          recordKey: "curtain-call-bestiary:AOCjBM0drUBjGUIE",
          pack: "curtain-call-bestiary",
          name: "Masque Mannequin",
        },
        { recordKey: "curtain-call-bestiary:d4uqDeZ03CKl43zF", pack: "curtain-call-bestiary", name: "Nemesis" },
        { recordKey: "curtain-call-bestiary:XlEHPOxIIvne6oxe", pack: "curtain-call-bestiary", name: "Niallana" },
        {
          recordKey: "curtain-call-bestiary:iaQrKM85V4hWWmxw",
          pack: "curtain-call-bestiary",
          name: "Operatic Stone Bulwark",
        },
        { recordKey: "curtain-call-bestiary:NpIoefm1VZTTnR8h", pack: "curtain-call-bestiary", name: "Oriole" },
        {
          recordKey: "curtain-call-bestiary:8VjUGaYgDutmaYjk",
          pack: "curtain-call-bestiary",
          name: "Prince of Propaganda",
        },
        { recordKey: "curtain-call-bestiary:RPE2rcJvkcGwM5r6", pack: "curtain-call-bestiary", name: "Razorbones" },
        {
          recordKey: "curtain-call-bestiary:HiJBqmjbRp7MmPUW",
          pack: "curtain-call-bestiary",
          name: "Reaper of Reputation",
        },
        { recordKey: "curtain-call-bestiary:RUuUJtm8jAJ66z0Q", pack: "curtain-call-bestiary", name: "Risen Nemesis" },
        { recordKey: "curtain-call-bestiary:h2ibApkyVasD995i", pack: "curtain-call-bestiary", name: "Skurg" },
        {
          recordKey: "curtain-call-bestiary:yBd3PYNJoUTrJVgM",
          pack: "curtain-call-bestiary",
          name: "The Final Herald",
        },
        { recordKey: "curtain-call-bestiary:OG0soHK46kY5I5QH", pack: "curtain-call-bestiary", name: "Vorens" },
        { recordKey: "curtain-call-bestiary:DBr25IAKQHwz69YD", pack: "curtain-call-bestiary", name: "Waxen Effigy" },
        { recordKey: "dawn-of-the-frogs:X5uEvNj26g4RBPqw", pack: "dawn-of-the-frogs", name: "Leather Cap" },
        { recordKey: "dawn-of-the-frogs:uA7Ptt5Q7TjoqzNo", pack: "dawn-of-the-frogs", name: "Young Monitor Lizard" },
        {
          recordKey: "extinction-curse-bestiary:dP6sDHTZrDcV2I9w",
          pack: "extinction-curse-bestiary",
          name: "Abberton Ruffian",
        },
        {
          recordKey: "extinction-curse-bestiary:beb9LqlOBFseROnY",
          pack: "extinction-curse-bestiary",
          name: "Adrivallo",
        },
        {
          recordKey: "extinction-curse-bestiary:pOlP9ZR0eFS57c27",
          pack: "extinction-curse-bestiary",
          name: "Aives The Smoke Dragon",
        },
        {
          recordKey: "extinction-curse-bestiary:oYstle5FAR800UQT",
          pack: "extinction-curse-bestiary",
          name: "Andera Paldreen",
        },
        {
          recordKey: "extinction-curse-bestiary:otSfn7djr37Bdejj",
          pack: "extinction-curse-bestiary",
          name: "Arskuva the Gnasher",
        },
        {
          recordKey: "extinction-curse-bestiary:HEeRO5IF4lAGfDqE",
          pack: "extinction-curse-bestiary",
          name: "Barking Stag",
        },
        {
          recordKey: "extinction-curse-bestiary:88VcnxbKPF2QkPiF",
          pack: "extinction-curse-bestiary",
          name: "Bitter Truth Bandit",
        },
        {
          recordKey: "extinction-curse-bestiary:HFAhDmrkxg6YLhdF",
          pack: "extinction-curse-bestiary",
          name: "Blood Wolf",
        },
        { recordKey: "extinction-curse-bestiary:u6b7tlXDXMxmkdLO", pack: "extinction-curse-bestiary", name: "Bogey" },
        {
          recordKey: "extinction-curse-bestiary:SAqbWugp2NcdEtrr",
          pack: "extinction-curse-bestiary",
          name: "Bone Croupier",
        },
        { recordKey: "extinction-curse-bestiary:jCcA2ca8VnnDtVU9", pack: "extinction-curse-bestiary", name: "Bugaboo" },
        {
          recordKey: "extinction-curse-bestiary:7p4RWS26W5k6vCkH",
          pack: "extinction-curse-bestiary",
          name: "Cat Sith",
        },
        {
          recordKey: "extinction-curse-bestiary:7YJHi9niIKpFXXrf",
          pack: "extinction-curse-bestiary",
          name: "Celestial Menagerie Bruiser",
        },
        {
          recordKey: "extinction-curse-bestiary:WOeSP1KLzo0qTpZu",
          pack: "extinction-curse-bestiary",
          name: "Chimpanzee Visitant",
        },
        {
          recordKey: "extinction-curse-bestiary:jwcMb71QRhMw94Id",
          pack: "extinction-curse-bestiary",
          name: "Convergent Kendley Nathrael",
        },
        {
          recordKey: "extinction-curse-bestiary:DXNDZNHSZxlNXJnk",
          pack: "extinction-curse-bestiary",
          name: "Convergent Soldier",
        },
        {
          recordKey: "extinction-curse-bestiary:EBDDeBHGGZ8xvIM6",
          pack: "extinction-curse-bestiary",
          name: "Corrosive Lizard",
        },
        {
          recordKey: "extinction-curse-bestiary:wi94WddixQEID9Jl",
          pack: "extinction-curse-bestiary",
          name: "Corrupted Priest",
        },
        {
          recordKey: "extinction-curse-bestiary:Q3Z0rIINoWYxVSrS",
          pack: "extinction-curse-bestiary",
          name: "Corrupted Retainer",
        },
        {
          recordKey: "extinction-curse-bestiary:29NHB8DNNAbEk5Va",
          pack: "extinction-curse-bestiary",
          name: "Counteflora",
        },
        {
          recordKey: "extinction-curse-bestiary:NBCqA1NLRlhMuqGl",
          pack: "extinction-curse-bestiary",
          name: "Daring Danika",
        },
        {
          recordKey: "extinction-curse-bestiary:bJUlb2DxT1xyaYAp",
          pack: "extinction-curse-bestiary",
          name: "Darklands Alchemical Golem",
        },
        {
          recordKey: "extinction-curse-bestiary:hz2uoBS8DFOWshko",
          pack: "extinction-curse-bestiary",
          name: "Darricus Stallit",
        },
        {
          recordKey: "extinction-curse-bestiary:GuJJJzmjLKbkZUur",
          pack: "extinction-curse-bestiary",
          name: "Deghuun (Child of Mhar)",
        },
        {
          recordKey: "extinction-curse-bestiary:qKHkamxIPbqxEiwp",
          pack: "extinction-curse-bestiary",
          name: "Delamar Gianvin",
        },
        {
          recordKey: "extinction-curse-bestiary:yqH59ltUd0f3kLLL",
          pack: "extinction-curse-bestiary",
          name: "Drow Bodyguard Golem",
        },
        {
          recordKey: "extinction-curse-bestiary:s6Yfy1AgPp4ky7QI",
          pack: "extinction-curse-bestiary",
          name: "Drunken Brawler",
        },
        {
          recordKey: "extinction-curse-bestiary:NgmbMRekIHiA44lg",
          pack: "extinction-curse-bestiary",
          name: "Dyzallin Shraen",
        },
        {
          recordKey: "extinction-curse-bestiary:BsnU2Hf4a3MuXVPn",
          pack: "extinction-curse-bestiary",
          name: "Dyzallin's Golem",
        },
        {
          recordKey: "extinction-curse-bestiary:R3SbxfWLp8xUvXUP",
          pack: "extinction-curse-bestiary",
          name: "Elysian Sheep",
        },
        {
          recordKey: "extinction-curse-bestiary:i3kQzeKcSoxkNYJb",
          pack: "extinction-curse-bestiary",
          name: "Evora Yarket",
        },
        {
          recordKey: "extinction-curse-bestiary:sQvm52N0E5ulBaaq",
          pack: "extinction-curse-bestiary",
          name: "Faceless Butcher",
        },
        {
          recordKey: "extinction-curse-bestiary:wS2SN0dnQMybLzSA",
          pack: "extinction-curse-bestiary",
          name: "Flea Swarm",
        },
        {
          recordKey: "extinction-curse-bestiary:gN8VuDZ8b9dp0Ep0",
          pack: "extinction-curse-bestiary",
          name: "Giant Flea",
        },
        {
          recordKey: "extinction-curse-bestiary:XyHk8ChyuOkJuhVZ",
          pack: "extinction-curse-bestiary",
          name: "Guardian of the Faithful",
        },
        {
          recordKey: "extinction-curse-bestiary:IVpSJcoRLGgUfqW7",
          pack: "extinction-curse-bestiary",
          name: "Harrow Doll",
        },
        {
          recordKey: "extinction-curse-bestiary:QFScy7QFB7PIeTEa",
          pack: "extinction-curse-bestiary",
          name: "Helg Eats-The-Eaters",
        },
        {
          recordKey: "extinction-curse-bestiary:1vOre5O8t3pQPUp6",
          pack: "extinction-curse-bestiary",
          name: "Herecite of Zevgavizeb",
        },
        {
          recordKey: "extinction-curse-bestiary:YZcw33uhsPHWKcHM",
          pack: "extinction-curse-bestiary",
          name: "Hollow Hush",
        },
        { recordKey: "extinction-curse-bestiary:L25SQceNMS8IstYI", pack: "extinction-curse-bestiary", name: "Horba" },
        {
          recordKey: "extinction-curse-bestiary:x2XcPDPLeCAXITlZ",
          pack: "extinction-curse-bestiary",
          name: "Iridescent Elephant",
        },
        {
          recordKey: "extinction-curse-bestiary:16cZVZxsXVRHfuuQ",
          pack: "extinction-curse-bestiary",
          name: "Jellico Bounce-Bounce",
        },
        {
          recordKey: "extinction-curse-bestiary:a2FCggU8UCQl6RDx",
          pack: "extinction-curse-bestiary",
          name: "Juvenile Boar",
        },
        {
          recordKey: "extinction-curse-bestiary:b5creqUAlBl0Tmmc",
          pack: "extinction-curse-bestiary",
          name: "Leandrus",
        },
        {
          recordKey: "extinction-curse-bestiary:lh3pcyJlUUtNpWcI",
          pack: "extinction-curse-bestiary",
          name: "Ledorick Banyan",
        },
        {
          recordKey: "extinction-curse-bestiary:xGTl3DVCD0etE6MU",
          pack: "extinction-curse-bestiary",
          name: "Ledorick Banyan (Possessed)",
        },
        {
          recordKey: "extinction-curse-bestiary:HbxPY2GSxhRu4rVi",
          pack: "extinction-curse-bestiary",
          name: "Lion Visitant",
        },
        {
          recordKey: "extinction-curse-bestiary:RIZDryL3Wnk6ucks",
          pack: "extinction-curse-bestiary",
          name: "Luminous Ooze",
        },
        {
          recordKey: "extinction-curse-bestiary:wBlnbLj8FfgxArQm",
          pack: "extinction-curse-bestiary",
          name: "Lyrt Cozurn",
        },
        {
          recordKey: "extinction-curse-bestiary:3Fxih1eU4IXABxpy",
          pack: "extinction-curse-bestiary",
          name: "Mechanical Carny",
        },
        {
          recordKey: "extinction-curse-bestiary:QMFlW9qrUKWBOF1Q",
          pack: "extinction-curse-bestiary",
          name: "Mistress Dusklight",
        },
        {
          recordKey: "extinction-curse-bestiary:jaUUH2i5UQZYQqab",
          pack: "extinction-curse-bestiary",
          name: "Muse Phantom",
        },
        {
          recordKey: "extinction-curse-bestiary:IacftcXDDMBNcKbY",
          pack: "extinction-curse-bestiary",
          name: "Nemmia Bramblecloak",
        },
        {
          recordKey: "extinction-curse-bestiary:2Ey2VZ3aQlN4FGHJ",
          pack: "extinction-curse-bestiary",
          name: "Pin Tingwheely",
        },
        {
          recordKey: "extinction-curse-bestiary:KXNSBeUWLxMfd2Zg",
          pack: "extinction-curse-bestiary",
          name: "Pinacosaurus",
        },
        {
          recordKey: "extinction-curse-bestiary:4QgC23j1wzfaCecR",
          pack: "extinction-curse-bestiary",
          name: "Pruana Two-punch",
        },
        {
          recordKey: "extinction-curse-bestiary:gZBayD2gJu7iZrud",
          pack: "extinction-curse-bestiary",
          name: "Ruanna Nyamma",
        },
        {
          recordKey: "extinction-curse-bestiary:S5pcyiSXhMKrMjcC",
          pack: "extinction-curse-bestiary",
          name: "Shraen Graveknight",
        },
        { recordKey: "extinction-curse-bestiary:qCTU0ywtCOgiUM0q", pack: "extinction-curse-bestiary", name: "Skarja" },
        {
          recordKey: "extinction-curse-bestiary:rxGByemLAAeM4h29",
          pack: "extinction-curse-bestiary",
          name: "Smoldering Leopard",
        },
        {
          recordKey: "extinction-curse-bestiary:f4H9d0b1vJvxeFqs",
          pack: "extinction-curse-bestiary",
          name: "Sodden Sentinel",
        },
        {
          recordKey: "extinction-curse-bestiary:HHy2GazURA6Cx1ee",
          pack: "extinction-curse-bestiary",
          name: "Starved Staff",
        },
        {
          recordKey: "extinction-curse-bestiary:tPvr9zvUfktMIvYU",
          pack: "extinction-curse-bestiary",
          name: "Stirvyn Banyan",
        },
        {
          recordKey: "extinction-curse-bestiary:f6uVOvKEkojOf9Ab",
          pack: "extinction-curse-bestiary",
          name: "Swardlands Delinquent",
        },
        {
          recordKey: "extinction-curse-bestiary:BORxkpaFBSCyB1f1",
          pack: "extinction-curse-bestiary",
          name: "Tallow Ooze",
        },
        {
          recordKey: "extinction-curse-bestiary:CNO54boXvXg7xSP6",
          pack: "extinction-curse-bestiary",
          name: "Tanessa Fleer",
        },
        {
          recordKey: "extinction-curse-bestiary:j31HXlZiUqQrAHSB",
          pack: "extinction-curse-bestiary",
          name: "Tashlock Banyan",
        },
        {
          recordKey: "extinction-curse-bestiary:oJJspO9P2vDdtMYd",
          pack: "extinction-curse-bestiary",
          name: "The Vanish Man",
        },
        {
          recordKey: "extinction-curse-bestiary:UKBR2GdXIdg66Nqm",
          pack: "extinction-curse-bestiary",
          name: "Ulthadar",
        },
        {
          recordKey: "extinction-curse-bestiary:JvSCGGnexk7CmVke",
          pack: "extinction-curse-bestiary",
          name: "Viktor Volkano",
        },
        { recordKey: "extinction-curse-bestiary:lM9j6lc5MkBlGfzD", pack: "extinction-curse-bestiary", name: "Violet" },
        {
          recordKey: "extinction-curse-bestiary:QcJtBai5JViNFqUC",
          pack: "extinction-curse-bestiary",
          name: "Viskithrel",
        },
        {
          recordKey: "extinction-curse-bestiary:2XRzA5GZeDk88Y2z",
          pack: "extinction-curse-bestiary",
          name: "War Sauropelta",
        },
        { recordKey: "extinction-curse-bestiary:jxGA1C8xX0WkJGI4", pack: "extinction-curse-bestiary", name: "Yaganty" },
        {
          recordKey: "extinction-curse-bestiary:G2ICeUU6br5Xem3P",
          pack: "extinction-curse-bestiary",
          name: "Zinogyvaz",
        },
        {
          recordKey: "extinction-curse-bestiary:TdqGpBasgDOUqmNp",
          pack: "extinction-curse-bestiary",
          name: "Zuipnyrn",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:yFfT9yPbUzldlhxN",
          pack: "fall-of-plaguestone-bestiary",
          name: "Alchemical Drudge",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:JtBjUnYz859krAVv",
          pack: "fall-of-plaguestone-bestiary",
          name: "Bee Swarm",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:b2PXIdbqdzlkGkeX",
          pack: "fall-of-plaguestone-bestiary",
          name: "Blood Ooze",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:C7Wb70xpG2PvTslg",
          pack: "fall-of-plaguestone-bestiary",
          name: "Bloodlash Bush",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:PFBclNHc78MqYjBB",
          pack: "fall-of-plaguestone-bestiary",
          name: "Caustic Wolf",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:9dUSCjehW90dZalE",
          pack: "fall-of-plaguestone-bestiary",
          name: "Fiery Leopard",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:1Nr9cgWHMn8KtiXe",
          pack: "fall-of-plaguestone-bestiary",
          name: "Giant Lightning Serpent",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:mTVzzdrinwHbsvqu",
          pack: "fall-of-plaguestone-bestiary",
          name: "Graytusk",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:H4yGpC6VuQpszlUl",
          pack: "fall-of-plaguestone-bestiary",
          name: "Hallod",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:EhtODTbFdElXSyXX",
          pack: "fall-of-plaguestone-bestiary",
          name: "Icy Rat",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:3h9BvfzrIU6fNxch",
          pack: "fall-of-plaguestone-bestiary",
          name: "Lord Nar",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:kZI5UjgXcJjrOIR8",
          pack: "fall-of-plaguestone-bestiary",
          name: "Mangy Wolf",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:WV8sdgARGVUb39WM",
          pack: "fall-of-plaguestone-bestiary",
          name: "Mutant Wolf",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:X0AJV2Zfj2d6rab5",
          pack: "fall-of-plaguestone-bestiary",
          name: "Orc Alchemist",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:23kCcF391RtapbfJ",
          pack: "fall-of-plaguestone-bestiary",
          name: "Stone Horse",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:lPHwAyFAeAhHLNnU",
          pack: "fall-of-plaguestone-bestiary",
          name: "The Amalgam",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:IU4Gx9vHKMT2KYDq",
          pack: "fall-of-plaguestone-bestiary",
          name: "The Behemoth",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:h9b23NJBKSACAkWa",
          pack: "fall-of-plaguestone-bestiary",
          name: "The Sculptor",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:Er9kW7JtUX4zf15Q",
          pack: "fall-of-plaguestone-bestiary",
          name: "Vilree",
        },
        {
          recordKey: "fall-of-plaguestone-bestiary:wBCpq9pX4NZFXE0T",
          pack: "fall-of-plaguestone-bestiary",
          name: "Vine Lasher",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:f0nafAQA1tSXLqwL",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Abbot Tsujon",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:AxVNn9nyobosLEAq",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Agile Warrior",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:fOdNBYgjTveLzpbd",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Agile Warrior (Nightmares)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:xiQIw7iFjOizo3Fm",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Akila Stormheel",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:EItI5u34FIIuZM9W",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Angoyang",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:YHSV9DfaqoaSggLi",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Archery Specialist",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:5cMbN10QKGS06Zhy",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Arms of Balance (Jivati Rovat)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:xBSLIoVsZ6I4LOZc",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Arms of Balance (Pravan Majinapti)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:2ZnZ0d61CQHMHtHN",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Arms of Balance (Ranya Shibhatesh)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:65LJVPu9DFkF0YNX",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Arms of Balance (Usvani)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:FwIJpQn1FIpGgvIY",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Artus Rodrivan",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:RKkIczqYUJkij0Oh",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Blue Viper (Level 14)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:SF6OlCfNiHvtJQjw",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Blue Viper (Level 16)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:se8SrwtuaamXIEo5",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Blue Viper (Level 20)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:80tgHHqfxA5lUfjS",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Bul-Gae",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:jnN7fYYndv3oobJJ",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Butterfly Blade Warrior",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:UbXvVA3ZdSN6pj1j",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Cloudsplitter",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:TEHxsglxWi1i3jSN",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Elder Cauthooj",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:lqHn5wSzxvdllGgH",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Flying Mountain Kaminari",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:TqKFvTmJnhdclCgt",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ghost Monk",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:tu64qeuTiIGHtvgR",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Brartork)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:CKS4OsoyI2QI8RUn",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Han)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:N9Qp7x6n6ighaXOY",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Jun)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:snrGRRYXD8mUG4Y1",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Krankkiss)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:2ryxyUYlf8lY6o1D",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Mingyu)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:JFa9WD0nXOTgFGha",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Numoriz)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:wFswIWjySnPqVNVT",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Paunnima)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:pi3njPvz8AsOrVGZ",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Golarion's Finest (Rajna)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:nejHZ9ccKcDXZWQM",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Gomwai",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:T5KMiSE8W1R8ChbV",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Grave Spinosaurus",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:zFye7hJzzwgajovA",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Halspin the Stung",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:Lk4LPjGsidzuD6Vy",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Huldrin Skolsdottir",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:gGDWBzon2T2mrSqf",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Hummingbird",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:ofYZYIZTkakVj2R0",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ji-yook (Gumiho Form)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:ghAlcSfpO03JLvwG",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ji-yook (Level 13)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:Q6gjfL2rha7tRS0B",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ji-yook (Level 9)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:HGtyyMucLkYlqZ8i",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Jin-hae",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:PhFDdGFXaaiLe4k4",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Joon-seo",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:i3dC41mOjfoBxQTk",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Juspix Rammel",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:Lsa0Q8aU8aPWjFYv",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Kannitri",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:o8UMCrKWf89xNvSE",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ki Adept",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:haUIlixea4iwy0GZ",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ki Adept (Ahmoza Twins)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:XiO3tJlYmuZwOBaA",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Koto Zekora",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:qV1bsHLfgI8xh0Nr",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Lantondo",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:7MiWOVxjocxNr0dh",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Laruhao",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:blm15iXMR3Lbwdom",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Mafika Ayuwari",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:AYQEkPFyTuGlxNg3",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Mage of Many Styles",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:sXtwJyM7sWWDQDOU",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Muckish Creep",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:IGEht7ysUSZndtgp",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Old Man Statue",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:QhFUhlZl1bvUrtPs",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rai Sho Postulant",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:XN2uksGMgndh1jSP",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ran-to (Level 14)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:9WcpNMJJqF3cLTqT",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ran-to (Level 16)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:qW8stNdPqQwibYQz",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Ran-to (Level 20)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:9ksOukuecb43zqOd",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Razu",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:k5UjbiYr87NVK7qp",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rivka (Cimurlian)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:zpXQHbe037haBVmD",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rivka (Igroon)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:56U7JoNKbci67vRE",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rivka (Kujiba)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:MdkbAYfKgIvtTxBO",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rivka (Mogaru)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:EYr9GBleLFezHqBk",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Rivka (Yorak)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:YHbBs2Jq6ukngOra",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Sanzuwu",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:cg1kQPO3FBSCFDVt",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Shadow Yai",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:eVKjtiQtDaJpk9Ra",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Sixth Pillar Student",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:TjmD89wG7qQ0tRXj",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Speakers to the Wind (Gnoll Cascade Bearer)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:4RaeyGxw5EgagDZE",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Surjit Hamelan",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:6wRicVhIx07i94s3",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Taiga Yai",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:Y1Px08o90xG5dWTq",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Takatorra (Daitengu Form)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:1y2FBsV4g80KNac1",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Takatorra (Level 13)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:J7s3MTb3eZP6u3LT",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Takatorra (Level 9)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:526cj9mVkwQ9gDn6",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Tino (Oni Form)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:TTEcqxuMqzIdkX6q",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Troff Frostknuckles",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:l2oapyRhfCFRsCHX",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Umbasi",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:H5koEzKL8qtfX4fK",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Urnak Lostwind",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:gSlkZ86P3QrbM874",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Weapon Master",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:FWfKJ7e8UgbBXTkU",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Weapon Master (Under the Pale Sun Dervishes)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:wfp1cV5xCdpElkoU",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Yabin (White Serpent Form)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:7qyR096MhufccSiM",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Yabin the Just (Level 13)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:u3vdt2J02rw8RyRq",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Yabin the Just (Level 9)",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:zmE4irZj6a2WpB6f",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Yarrika Mulandez",
        },
        {
          recordKey: "fists-of-the-ruby-phoenix-bestiary:tfBczpCPNVnRE8pJ",
          pack: "fists-of-the-ruby-phoenix-bestiary",
          name: "Yoh Souran",
        },
        { recordKey: "gatewalkers-bestiary:kgW2ycNTgQsMmkTC", pack: "gatewalkers-bestiary", name: "Ainamuuren" },
        { recordKey: "gatewalkers-bestiary:JrMv2lQ1hRTpMumk", pack: "gatewalkers-bestiary", name: "Alkoasha" },
        { recordKey: "gatewalkers-bestiary:bnKUnG9XzsKzIUxK", pack: "gatewalkers-bestiary", name: "Ancient Tupilaq" },
        {
          recordKey: "gatewalkers-bestiary:uTaGZT1vl7UG9tdf",
          pack: "gatewalkers-bestiary",
          name: "Apothecary's Cabinet",
        },
        { recordKey: "gatewalkers-bestiary:eRRWMgdYHwNIm6zp", pack: "gatewalkers-bestiary", name: "Aspect of Ruun" },
        { recordKey: "gatewalkers-bestiary:qsRrYCsKLiMwZ1I9", pack: "gatewalkers-bestiary", name: "Bolan Nogasso" },
        { recordKey: "gatewalkers-bestiary:u1VUKkC4mBDhi7YZ", pack: "gatewalkers-bestiary", name: "Creeping Evil" },
        { recordKey: "gatewalkers-bestiary:bz5OjD8rYMm1M4EC", pack: "gatewalkers-bestiary", name: "Delphon" },
        { recordKey: "gatewalkers-bestiary:Ol3XPxYXHmSHYlk8", pack: "gatewalkers-bestiary", name: "Deniigi" },
        { recordKey: "gatewalkers-bestiary:AeM32zT3kFOOAugS", pack: "gatewalkers-bestiary", name: "Equendia" },
        { recordKey: "gatewalkers-bestiary:MnOXwpGD3vJIFydk", pack: "gatewalkers-bestiary", name: "Etward Ritalson" },
        { recordKey: "gatewalkers-bestiary:rrMtTABR1587fJGU", pack: "gatewalkers-bestiary", name: "Ghodrak the Quick" },
        { recordKey: "gatewalkers-bestiary:16Ru7zfHAHD544xO", pack: "gatewalkers-bestiary", name: "Glimmervine" },
        { recordKey: "gatewalkers-bestiary:PAQHiQqCcfFo8koZ", pack: "gatewalkers-bestiary", name: "Ilakni" },
        { recordKey: "gatewalkers-bestiary:N659rGqG2uyI2gp5", pack: "gatewalkers-bestiary", name: "Ilverani Sentry" },
        { recordKey: "gatewalkers-bestiary:cFXimh6v7f02kWri", pack: "gatewalkers-bestiary", name: "Innumma" },
        { recordKey: "gatewalkers-bestiary:58GbVCENX813DbHw", pack: "gatewalkers-bestiary", name: "Jarvs" },
        { recordKey: "gatewalkers-bestiary:ZzVs5T27LnLhRZ5u", pack: "gatewalkers-bestiary", name: "Kaneepo the Slim" },
      ]),
      insufficient_evidence: reviewedRecordRefs([
        { recordKey: "blog-bestiary:8Cbj8BtyRDerg4Lf", pack: "blog-bestiary", name: "Blarghest" },
        { recordKey: "blog-bestiary:DqDwGNZzUrtlKJQ6", pack: "blog-bestiary", name: "Chea" },
        { recordKey: "blog-bestiary:tKrAtcACOtDnHLG0", pack: "blog-bestiary", name: "Duhgik" },
        { recordKey: "blog-bestiary:ssaJVgjmjvk9ezoe", pack: "blog-bestiary", name: "Eleukas" },
        { recordKey: "blog-bestiary:erbFDwCcn1qsJnxS", pack: "blog-bestiary", name: "Floolf" },
        { recordKey: "blog-bestiary:uPeDOylL6zsZKSIs", pack: "blog-bestiary", name: "Goblin Zombie" },
        { recordKey: "blog-bestiary:M6MAHeD7U52inXED", pack: "blog-bestiary", name: "Gristleburst" },
        { recordKey: "blog-bestiary:bCyzCto3GIqvWBoM", pack: "blog-bestiary", name: "Hellknight Centaur" },
        { recordKey: "blog-bestiary:oF1SkWMwJpWtKIoq", pack: "blog-bestiary", name: "Kobold Tunnelrunner" },
        { recordKey: "blog-bestiary:L9Gg91HVIzYANuXK", pack: "blog-bestiary", name: "Lisavet" },
        { recordKey: "blog-bestiary:CHLE0BV8wQJY4ksk", pack: "blog-bestiary", name: "Mari Lwyd" },
        { recordKey: "blog-bestiary:4BzHtRTPQ5mZj9Mr", pack: "blog-bestiary", name: "Nonsequitaur" },
        { recordKey: "blog-bestiary:MO2QmDcf117b8FQk", pack: "blog-bestiary", name: "Nosferotter" },
        { recordKey: "blog-bestiary:xkMQlY10mUIIkrfa", pack: "blog-bestiary", name: "Ogre Hurler" },
        { recordKey: "blog-bestiary:JhC9sqhFy88lX4Xu", pack: "blog-bestiary", name: "Pr'rall" },
        { recordKey: "blog-bestiary:h3qZZZ5PCbY87HZ4", pack: "blog-bestiary", name: "Psstpsstmitl" },
        { recordKey: "blog-bestiary:8xPlavExNidQSFyb", pack: "blog-bestiary", name: "Wendlyn" },
        { recordKey: "blog-bestiary:d0nNIzVZe1f81aAg", pack: "blog-bestiary", name: "Weredigo" },
        { recordKey: "blog-bestiary:YuSniRhf05jBHwPy", pack: "blog-bestiary", name: "Zhang Yong" },
        { recordKey: "blood-lords-bestiary:iVzVgK6igRZ8hkCu", pack: "blood-lords-bestiary", name: "Animated Tea Cart" },
        {
          recordKey: "blood-lords-bestiary:nzY0BPNKt4NF1Nhq",
          pack: "blood-lords-bestiary",
          name: "Bellator Mortus Soldier",
        },
        { recordKey: "blood-lords-bestiary:N387myolfXJxWYvF", pack: "blood-lords-bestiary", name: "Bone Shard Tough" },
        {
          recordKey: "blood-lords-bestiary:pJ9Pfo3DCsV97GYO",
          pack: "blood-lords-bestiary",
          name: "Crooked Coffin Brewer",
        },
        { recordKey: "blood-lords-bestiary:mqHULEf9atfBObra", pack: "blood-lords-bestiary", name: "Dead Faine" },
        {
          recordKey: "blood-lords-bestiary:zw1uj88nmUrCf7iS",
          pack: "blood-lords-bestiary",
          name: "Ectoplasmic Amalgam",
        },
        { recordKey: "blood-lords-bestiary:dmUBlUsO6AoxjgxL", pack: "blood-lords-bestiary", name: "Firebrand Bastion" },
        { recordKey: "blood-lords-bestiary:bblPYqxgXeRmR4Bi", pack: "blood-lords-bestiary", name: "Ghiono" },
        { recordKey: "blood-lords-bestiary:CMQu4nHDLH7tt6yu", pack: "blood-lords-bestiary", name: "Ghoul Gnawer" },
        { recordKey: "blood-lords-bestiary:duqhXs5SXmvSC1uI", pack: "blood-lords-bestiary", name: "Hollow Husk" },
        { recordKey: "blood-lords-bestiary:Q7Mfkogemi5D9LJh", pack: "blood-lords-bestiary", name: "Hungering Growth" },
        { recordKey: "blood-lords-bestiary:NiDaVEpAOgbuwnau", pack: "blood-lords-bestiary", name: "Kemnebi's Puppet" },
        {
          recordKey: "blood-lords-bestiary:nmv3qAignhfl1in2",
          pack: "blood-lords-bestiary",
          name: "Mosghuta, Boss Cow",
        },
        { recordKey: "blood-lords-bestiary:JiT7B2Y7yYG8G0ZL", pack: "blood-lords-bestiary", name: "Nwanyian Archer" },
        { recordKey: "blood-lords-bestiary:A8jGRgn97x59cJn8", pack: "blood-lords-bestiary", name: "Nwanyian Defender" },
        {
          recordKey: "blood-lords-bestiary:1Hc1BLc9LIXLuBdg",
          pack: "blood-lords-bestiary",
          name: "Rival Corpsekiller",
        },
        { recordKey: "blood-lords-bestiary:1McAtOIMRIVOa0lf", pack: "blood-lords-bestiary", name: "Rival Necromancer" },
        { recordKey: "blood-lords-bestiary:dUl7C2wT4mT3mAGH", pack: "blood-lords-bestiary", name: "Ruby" },
        {
          recordKey: "blood-lords-bestiary:OhJ4yePPE1zQ8JNL",
          pack: "blood-lords-bestiary",
          name: "Sahreg the Dirge Screamer",
        },
        { recordKey: "blood-lords-bestiary:tu9Y0LJ6ghOpkJTX", pack: "blood-lords-bestiary", name: "Seldeg Bheldis" },
        { recordKey: "blood-lords-bestiary:6DRyViLYSsw2sYBy", pack: "blood-lords-bestiary", name: "Seldeg's Steed" },
        { recordKey: "blood-lords-bestiary:l5jzqFVpGEg6Mj3e", pack: "blood-lords-bestiary", name: "Skeletal Knight" },
        {
          recordKey: "blood-lords-bestiary:S1rfj73uV1ZUBCJ3",
          pack: "blood-lords-bestiary",
          name: "Skeleton Rival Corpsekiller",
        },
        {
          recordKey: "blood-lords-bestiary:X6bu6VmaXgtQMBTY",
          pack: "blood-lords-bestiary",
          name: "Skeleton Rival Necromancer",
        },
        {
          recordKey: "blood-lords-bestiary:RpFpcfMqD6wqIHfE",
          pack: "blood-lords-bestiary",
          name: "Vampire Rival Necromancer",
        },
        {
          recordKey: "blood-lords-bestiary:KJm77VlSLPNpukcI",
          pack: "blood-lords-bestiary",
          name: "Vice-Chancellor Vikroti Stroh",
        },
        { recordKey: "blood-lords-bestiary:woGNg9IcqOtEy9hb", pack: "blood-lords-bestiary", name: "Zombie Horse" },
        { recordKey: "blood-lords-bestiary:LeJeFOI8P0L7pxxS", pack: "blood-lords-bestiary", name: "Zombie Hound" },
        {
          recordKey: "blood-lords-bestiary:HKH4GURuq59l0ag4",
          pack: "blood-lords-bestiary",
          name: "Zombie Rival Necromancer",
        },
        { recordKey: "blood-lords-bestiary:pQJbY3PTeHNYbkho", pack: "blood-lords-bestiary", name: "Zuntishan Guard" },
        {
          recordKey: "claws-of-the-tyrant-bestiary:IOzVkKmHQvHdmO90",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Divine Warden of Arazni",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:fF04YIUUzwQlK1TW",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Ossuary Warden",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:uhTMcy0blrxpP6aZ",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Priest of Iomedae",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:RwNVqGmL6VjxhO82",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Skeletal Rat Swarm",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:zqnIJ602BTZSBFaL",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Undead Murder",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:sWoLuDXZoGO9Ld4n",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Zombie Bear",
        },
        {
          recordKey: "claws-of-the-tyrant-bestiary:NzhVuwCmMTPUQPAB",
          pack: "claws-of-the-tyrant-bestiary",
          name: "Zombie Desecrator",
        },
        {
          recordKey: "crown-of-the-kobold-king-bestiary:btIGNqj5SqOAKPbS",
          pack: "crown-of-the-kobold-king-bestiary",
          name: "Foolish Hunter",
        },
        {
          recordKey: "curtain-call-bestiary:A1MdmLZnbanM5CZi",
          pack: "curtain-call-bestiary",
          name: "Anguish Siktempora",
        },
        { recordKey: "curtain-call-bestiary:Mmg3SDdcpcgX311I", pack: "curtain-call-bestiary", name: "Skinsaw Trophy" },
        {
          recordKey: "gatewalkers-bestiary:Yy2DDehhj47BMWgP",
          pack: "gatewalkers-bestiary",
          name: "Blackfrost Guecubu",
        },
      ]),
      mixed_family_cues: reviewedRecordRefs([
        { recordKey: "book-of-the-dead-bestiary:j777BjOqZff6S1v9", pack: "book-of-the-dead-bestiary", name: "Bhuta" },
        {
          recordKey: "book-of-the-dead-bestiary:vaVIJeQnKFUeaC8K",
          pack: "book-of-the-dead-bestiary",
          name: "Combusted",
        },
        {
          recordKey: "book-of-the-dead-bestiary:XrSjNpkJkBO6ysRG",
          pack: "book-of-the-dead-bestiary",
          name: "Gashadokuro",
        },
        {
          recordKey: "book-of-the-dead-bestiary:laYufBuih3cT95j4",
          pack: "book-of-the-dead-bestiary",
          name: "Hollow Serpent",
        },
        {
          recordKey: "book-of-the-dead-bestiary:3pQvzrGhxO3bUwYT",
          pack: "book-of-the-dead-bestiary",
          name: "Iruxi Ossature",
        },
        { recordKey: "curtain-call-bestiary:58QVgNXmRsI7p8xq", pack: "curtain-call-bestiary", name: "Hellshadow" },
        {
          recordKey: "curtain-call-bestiary:WS3KRvmJMbftLbdV",
          pack: "curtain-call-bestiary",
          name: "Mask of Blackfingers",
        },
        {
          recordKey: "curtain-call-bestiary:NWgVQNeDHnjSxOiK",
          pack: "curtain-call-bestiary",
          name: "Mask of Father Skinsaw",
        },
        {
          recordKey: "curtain-call-bestiary:6s7h25mjQAsVEkcY",
          pack: "curtain-call-bestiary",
          name: "Mask of Gray Master",
        },
        {
          recordKey: "curtain-call-bestiary:AIcxTXQjR0Y70oNE",
          pack: "curtain-call-bestiary",
          name: "Mask of the Reaper",
        },
        { recordKey: "curtain-call-bestiary:7vC2VnYOgqpkYeNi", pack: "curtain-call-bestiary", name: "Zimiezek" },
      ]),
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

  for (const [category, families] of Object.entries(registry) as Array<
    [SearchCategory, ReviewedDiscoveryRegistry[SearchCategory]]
  >) {
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

      for (const [reason, records] of Object.entries(reasonBuckets ?? {}) as Array<
        [ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[] | undefined]
      >) {
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
            pack: record.pack,
            name: record.name,
            subcategory: record.subcategory ?? null,
            note: record.note,
          });
        }
      }
    }
  }

  return entries
    .slice()
    .sort(
      (left, right) =>
        left.category.localeCompare(right.category) ||
        left.family.localeCompare(right.family) ||
        left.reason.localeCompare(right.reason) ||
        left.recordKey.localeCompare(right.recordKey),
    );
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
  const entries = getReviewedDiscoveryEntries(
    {
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      reason: reviewReason ?? undefined,
    },
    registry,
  );

  return {
    mode: reviewReason ? "filtered" : options.includeReviewed ? "included" : "excluded",
    reviewReason,
    entries,
    recordKeys: uniqueSorted(entries.map((entry) => entry.recordKey)),
    reasonCounts: getReviewedDiscoveryReasonCounts(
      {
        category: options.category,
        subcategory: options.subcategory,
        family: options.family,
        reason: reviewReason ?? undefined,
      },
      registry,
    ),
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
