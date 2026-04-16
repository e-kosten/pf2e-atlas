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

export type ReviewedDiscoveryRegistry = Partial<Record<
  SearchCategory,
  Partial<Record<string, Partial<Record<ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[]>>>>
>>;

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
        { recordKey: "abomination-vaults-bestiary:3d3NAcPfvn07mcGN", pack: "abomination-vaults-bestiary", name: "Afflicted Irnakurse" },
        { recordKey: "abomination-vaults-bestiary:tOL4rWj2oYWZ4ow2", pack: "abomination-vaults-bestiary", name: "Aller Rosk" },
        { recordKey: "abomination-vaults-bestiary:saEUzIgUtV2AzKhl", pack: "abomination-vaults-bestiary", name: "Augrael" },
        { recordKey: "abomination-vaults-bestiary:DDJGNAh3rfyIupAb", pack: "abomination-vaults-bestiary", name: "Belcorra Haruvex" },
        { recordKey: "abomination-vaults-bestiary:3ry9WSvMMXHUe3kE", pack: "abomination-vaults-bestiary", name: "Beluthus" },
        { recordKey: "abomination-vaults-bestiary:rketcmqDQJbFFYfq", pack: "abomination-vaults-bestiary", name: "Bone Gladiator" },
        { recordKey: "abomination-vaults-bestiary:xAfkUwJYq5JLmSrW", pack: "abomination-vaults-bestiary", name: "Boss Skrawng" },
        { recordKey: "abomination-vaults-bestiary:ChRgdkplhO1D81Lg", pack: "abomination-vaults-bestiary", name: "Bright Walker" },
        { recordKey: "abomination-vaults-bestiary:oXnpdJVN6NIE58W3", pack: "abomination-vaults-bestiary", name: "Caliddo Haruvex" },
        { recordKey: "abomination-vaults-bestiary:lMCEVxKkQ7XK6Nid", pack: "abomination-vaults-bestiary", name: "Canker Cultist" },
        { recordKey: "abomination-vaults-bestiary:tYzzLLUv9WBhHhQY", pack: "abomination-vaults-bestiary", name: "Carman Rajani" },
        { recordKey: "abomination-vaults-bestiary:x0NDgH3EMLTLh02r", pack: "abomination-vaults-bestiary", name: "Chafkhem" },
        { recordKey: "abomination-vaults-bestiary:3H1rBpUQwTcNd6xZ", pack: "abomination-vaults-bestiary", name: "Chandriu Invisar" },
        { recordKey: "abomination-vaults-bestiary:HBRz8BVLVN9u9Odp", pack: "abomination-vaults-bestiary", name: "Corpselight" },
        { recordKey: "abomination-vaults-bestiary:T6vOuhM1KV5Fr75F", pack: "abomination-vaults-bestiary", name: "Gibtanius" },
        { recordKey: "abomination-vaults-bestiary:BJYrYqkV7PkXgSfk", pack: "abomination-vaults-bestiary", name: "Gibtas Bounder" },
        { recordKey: "abomination-vaults-bestiary:3F3fPq5hFbej40T2", pack: "abomination-vaults-bestiary", name: "Gibtas Spawn Swarm" },
        { recordKey: "abomination-vaults-bestiary:jE8BEe6pcnGraw2p", pack: "abomination-vaults-bestiary", name: "Jafaki" },
        { recordKey: "abomination-vaults-bestiary:hia81Ut7fEREbhkq", pack: "abomination-vaults-bestiary", name: "Jarelle Kaldrian" },
        { recordKey: "abomination-vaults-bestiary:EN3mp0sVObP8ou3p", pack: "abomination-vaults-bestiary", name: "Jaul Mezmin" },
        { recordKey: "abomination-vaults-bestiary:xj1Qn0VA4H4aKSjW", pack: "abomination-vaults-bestiary", name: "Jaul's Wolf" },
        { recordKey: "abomination-vaults-bestiary:dWOK0nzGWyc5NkNz", pack: "abomination-vaults-bestiary", name: "Lady's Whisper" },
        { recordKey: "abomination-vaults-bestiary:kzX588Hjb3w4QPOj", pack: "abomination-vaults-bestiary", name: "Mister Beak" },
        { recordKey: "abomination-vaults-bestiary:mlifDVJJWwjFtUxv", pack: "abomination-vaults-bestiary", name: "Murmur" },
        { recordKey: "abomination-vaults-bestiary:w2N0foudBFcRCaHK", pack: "abomination-vaults-bestiary", name: "Nhakazarin" },
        { recordKey: "abomination-vaults-bestiary:TiAzR8SnYwhACWbj", pack: "abomination-vaults-bestiary", name: "Observation Deck Seugathi Researcher" },
        { recordKey: "abomination-vaults-bestiary:R0EEgMDKcynpAWoa", pack: "abomination-vaults-bestiary", name: "Otari Ilvashti" },
        { recordKey: "abomination-vaults-bestiary:BOaM3pAuWl06Q6IZ", pack: "abomination-vaults-bestiary", name: "Poisoning Room Specter" },
        { recordKey: "abomination-vaults-bestiary:WR07Z6MjvebSHzI7", pack: "abomination-vaults-bestiary", name: "Ryta" },
        { recordKey: "abomination-vaults-bestiary:3vn9W5SThovdsEnY", pack: "abomination-vaults-bestiary", name: "Sacuishu" },
        { recordKey: "abomination-vaults-bestiary:qOkxxiM4tNf96CHQ", pack: "abomination-vaults-bestiary", name: "Seugathi Guard" },
        { recordKey: "abomination-vaults-bestiary:vS1YISLmSnkNotkL", pack: "abomination-vaults-bestiary", name: "Seugathi Reality Warper" },
        { recordKey: "abomination-vaults-bestiary:cMpgGvq1fGxh8wI0", pack: "abomination-vaults-bestiary", name: "Seugathi Researcher" },
        { recordKey: "abomination-vaults-bestiary:v9B0hB5sm4YZxebY", pack: "abomination-vaults-bestiary", name: "Seugathi Servant" },
        { recordKey: "abomination-vaults-bestiary:qXT1SQDtGqMkVl7Q", pack: "abomination-vaults-bestiary", name: "Shanrigol Heap" },
        { recordKey: "abomination-vaults-bestiary:ZAXuvUW6kl6v3SuW", pack: "abomination-vaults-bestiary", name: "Volluk Azrinae" },
        { recordKey: "abomination-vaults-bestiary:ZDYMKYZVyR8Fqakp", pack: "abomination-vaults-bestiary", name: "Wrin Sivinxi" },
        { recordKey: "age-of-ashes-bestiary:xrxjjjRMKzwsYbGm", pack: "age-of-ashes-bestiary", name: "Accursed Forge-Spurned" },
        { recordKey: "age-of-ashes-bestiary:YV3wA2Kgjp74l7YJ", pack: "age-of-ashes-bestiary", name: "Alak Stagram" },
        { recordKey: "age-of-ashes-bestiary:yrU0xi4eKBmcGudo", pack: "age-of-ashes-bestiary", name: "Animated Dragonstorm" },
        { recordKey: "age-of-ashes-bestiary:sQZDwS08l6Agsryq", pack: "age-of-ashes-bestiary", name: "Barushak Il-Varashma" },
        { recordKey: "age-of-ashes-bestiary:jO2fGZayk4R1AzYK", pack: "age-of-ashes-bestiary", name: "Bida" },
        { recordKey: "age-of-ashes-bestiary:91BSCAJA1Oto2ctf", pack: "age-of-ashes-bestiary", name: "Blood Boar" },
        { recordKey: "age-of-ashes-bestiary:4vEQ8zRXC51Qo4Mv", pack: "age-of-ashes-bestiary", name: "Bloody Blade Mercenary" },
        { recordKey: "age-of-ashes-bestiary:aaDiR0EIWRQx8wdy", pack: "age-of-ashes-bestiary", name: "Calmont" },
        { recordKey: "age-of-ashes-bestiary:53lk7ek73j65A09B", pack: "age-of-ashes-bestiary", name: "Candlaron's Echo" },
        { recordKey: "age-of-ashes-bestiary:yLhO5vUrPQF42Lh8", pack: "age-of-ashes-bestiary", name: "Charau-ka" },
        { recordKey: "age-of-ashes-bestiary:T3UVfAfcvAMe9rih", pack: "age-of-ashes-bestiary", name: "Charau-ka Dragon Priest" },
        { recordKey: "age-of-ashes-bestiary:w1Pqv0YmG4MevpQc", pack: "age-of-ashes-bestiary", name: "Corrupt Guard" },
        { recordKey: "age-of-ashes-bestiary:L4K4V09tQVXj7ZiI", pack: "age-of-ashes-bestiary", name: "Dmiri Yoltosha" },
        { recordKey: "age-of-ashes-bestiary:MOyYEls8wNGHoe8F", pack: "age-of-ashes-bestiary", name: "Doorwarden" },
        { recordKey: "age-of-ashes-bestiary:scPhFbBqlmD2fQHa", pack: "age-of-ashes-bestiary", name: "Dragonscarred Dead" },
        { recordKey: "age-of-ashes-bestiary:NwNu2yvMnbvPvpY8", pack: "age-of-ashes-bestiary", name: "Ekujae Guardian" },
        { recordKey: "age-of-ashes-bestiary:jZc4PrsX3HCJnkXx", pack: "age-of-ashes-bestiary", name: "Emaliza Zandivar" },
        { recordKey: "age-of-ashes-bestiary:dG5DBgrxlaimsWOS", pack: "age-of-ashes-bestiary", name: "Falrok" },
        { recordKey: "age-of-ashes-bestiary:tr3qlFJVHqloE9zI", pack: "age-of-ashes-bestiary", name: "Forge-Spurned" },
        { recordKey: "age-of-ashes-bestiary:UQDxkvqXKg03ESTQ", pack: "age-of-ashes-bestiary", name: "Gerhard Pendergrast" },
        { recordKey: "age-of-ashes-bestiary:OJH8y8LqmgbkN8ce", pack: "age-of-ashes-bestiary", name: "Ghastly Bear" },
        { recordKey: "age-of-ashes-bestiary:xXtGS9uBa9z43X7y", pack: "age-of-ashes-bestiary", name: "Graveshell" },
        { recordKey: "age-of-ashes-bestiary:dlW3UaXVpnzjd6xe", pack: "age-of-ashes-bestiary", name: "Heuberk Thropp" },
        { recordKey: "age-of-ashes-bestiary:MWQmOXxGcbaMsXDD", pack: "age-of-ashes-bestiary", name: "Hezle" },
        { recordKey: "age-of-ashes-bestiary:Ev930dfPpwCR8Zju", pack: "age-of-ashes-bestiary", name: "Ilgreth" },
        { recordKey: "age-of-ashes-bestiary:XHJC4G6bfPiVXGKE", pack: "age-of-ashes-bestiary", name: "Ilssrah Embermead" },
        { recordKey: "age-of-ashes-bestiary:AtiN3EsRpHn5qbuv", pack: "age-of-ashes-bestiary", name: "Immortal Ichor" },
        { recordKey: "age-of-ashes-bestiary:lcYcBIFmfKFt8Hcs", pack: "age-of-ashes-bestiary", name: "Ingnovim Tluss" },
        { recordKey: "age-of-ashes-bestiary:Ig1joUmSHSNL6QVU", pack: "age-of-ashes-bestiary", name: "Ingnovim's Assistant" },
        { recordKey: "age-of-ashes-bestiary:30FX1WFr0RkGeueX", pack: "age-of-ashes-bestiary", name: "Inizra Arumelo" },
        { recordKey: "age-of-ashes-bestiary:lRqptquQcM6ZcQ4O", pack: "age-of-ashes-bestiary", name: "Ishti" },
        { recordKey: "age-of-ashes-bestiary:mY494hn9sKVD9q8C", pack: "age-of-ashes-bestiary", name: "Jaggaki" },
        { recordKey: "age-of-ashes-bestiary:kciXaXw31gHA3gZl", pack: "age-of-ashes-bestiary", name: "Jahsi" },
        { recordKey: "age-of-ashes-bestiary:qemmmfu7exswGMGZ", pack: "age-of-ashes-bestiary", name: "Kelda Halrig" },
        { recordKey: "age-of-ashes-bestiary:C4PD4p4I9byZ8yp6", pack: "age-of-ashes-bestiary", name: "King Harral" },
        { recordKey: "age-of-ashes-bestiary:9oNwQuwKGUuG9G9g", pack: "age-of-ashes-bestiary", name: "Laslunn" },
        { recordKey: "age-of-ashes-bestiary:L7IJO5z82nEN9IjM", pack: "age-of-ashes-bestiary", name: "Lazurite-Infused Stone Golem" },
        { recordKey: "age-of-ashes-bestiary:N8vOjTD2SqS2b9Sy", pack: "age-of-ashes-bestiary", name: "Lesser Manifestation Of Dahak" },
        { recordKey: "age-of-ashes-bestiary:SVLUUPOXDINbKyFL", pack: "age-of-ashes-bestiary", name: "Malarunk" },
        { recordKey: "age-of-ashes-bestiary:sY47PB9b7nCJjgRq", pack: "age-of-ashes-bestiary", name: "Manifestation Of Dahak" },
        { recordKey: "age-of-ashes-bestiary:QKkvnlqrhgLHuP1t", pack: "age-of-ashes-bestiary", name: "Mialari Docur" },
        { recordKey: "age-of-ashes-bestiary:ZtSb3mHZ5sD2uqHd", pack: "age-of-ashes-bestiary", name: "Mud Spider" },
        { recordKey: "age-of-ashes-bestiary:NK49bh04355Lgz5r", pack: "age-of-ashes-bestiary", name: "Nketiah" },
        { recordKey: "age-of-ashes-bestiary:tGyPrTGSpndXKU88", pack: "age-of-ashes-bestiary", name: "Nolly Peltry" },
        { recordKey: "age-of-ashes-bestiary:Afq4Osh3W1k9Bcsh", pack: "age-of-ashes-bestiary", name: "Promise Guard" },
        { recordKey: "age-of-ashes-bestiary:60aeSJvu09ZM3SPx", pack: "age-of-ashes-bestiary", name: "Racharak" },
        { recordKey: "age-of-ashes-bestiary:z4rlpEsE2KgzpOCc", pack: "age-of-ashes-bestiary", name: "Remnant of Barzillai" },
        { recordKey: "age-of-ashes-bestiary:r02b4f1XNq7OihhD", pack: "age-of-ashes-bestiary", name: "Renali" },
        { recordKey: "age-of-ashes-bestiary:kLX36WXp6rjTt71z", pack: "age-of-ashes-bestiary", name: "Rinnarv Bontimar" },
        { recordKey: "age-of-ashes-bestiary:qouYGPM8mE4KUCTe", pack: "age-of-ashes-bestiary", name: "Rusty Mae" },
        { recordKey: "age-of-ashes-bestiary:U2QGjhcg5QFFkHwv", pack: "age-of-ashes-bestiary", name: "Saggorak Poltergeist" },
        { recordKey: "age-of-ashes-bestiary:mYJ6NNthl702Pz2s", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Agent" },
        { recordKey: "age-of-ashes-bestiary:SVh7cPwyGgmZORVz", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Boss" },
        { recordKey: "age-of-ashes-bestiary:Lx5JiVOGWnzzjCrW", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Bruiser" },
        { recordKey: "age-of-ashes-bestiary:egTpOr4Wc0L5e0iY", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Enforcer" },
        { recordKey: "age-of-ashes-bestiary:gF8Qy1k8gPBEUBAd", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Mage" },
        { recordKey: "age-of-ashes-bestiary:nQ2eBOpK71I8D2JC", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Poisoner" },
        { recordKey: "age-of-ashes-bestiary:1lkay2gwgEquq0NF", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Sneak" },
        { recordKey: "age-of-ashes-bestiary:7WNFNE3SVMRGbBDf", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Sniper" },
        { recordKey: "age-of-ashes-bestiary:NwLrwNqwedh95iry", pack: "age-of-ashes-bestiary", name: "Scarlet Triad Thug" },
        { recordKey: "age-of-ashes-bestiary:Aii18rhNPMLW4Pxh", pack: "age-of-ashes-bestiary", name: "Skeletal Hellknight" },
        { recordKey: "age-of-ashes-bestiary:t5mX2rkcKzcOzJXQ", pack: "age-of-ashes-bestiary", name: "Spawn of Dahak" },
        { recordKey: "age-of-ashes-bestiary:Fe1lYhCUY4UO4Plw", pack: "age-of-ashes-bestiary", name: "Talamira" },
        { recordKey: "age-of-ashes-bestiary:5r2S6FeE6D1Oh66T", pack: "age-of-ashes-bestiary", name: "Tarrasque, The Armageddon Engine" },
        { recordKey: "age-of-ashes-bestiary:X7zSx8LFh2ZDnOYy", pack: "age-of-ashes-bestiary", name: "Teyam Ishtori" },
        { recordKey: "age-of-ashes-bestiary:Oqj8XIWBb29NZ8QX", pack: "age-of-ashes-bestiary", name: "Tixitog" },
        { recordKey: "age-of-ashes-bestiary:aiNE06bxKD6jfoLd", pack: "age-of-ashes-bestiary", name: "Uri Zandivar" },
        { recordKey: "age-of-ashes-bestiary:Ejxngh2tHFseZQHW", pack: "age-of-ashes-bestiary", name: "Vaklish" },
        { recordKey: "age-of-ashes-bestiary:dTikLHqGfiSYemuZ", pack: "age-of-ashes-bestiary", name: "Veshumirix" },
        { recordKey: "age-of-ashes-bestiary:UD7EwTQG2Sbl4d8R", pack: "age-of-ashes-bestiary", name: "Voz Lirayne" },
        { recordKey: "age-of-ashes-bestiary:1tDEWL9mAIXvTYik", pack: "age-of-ashes-bestiary", name: "Warbal Bumblebrasher" },
        { recordKey: "age-of-ashes-bestiary:H7NO4Q7ctHnfGKeJ", pack: "age-of-ashes-bestiary", name: "Weathered Wail" },
        { recordKey: "age-of-ashes-bestiary:JD6kdfZveObBe1mR", pack: "age-of-ashes-bestiary", name: "Xotanispawn" },
        { recordKey: "age-of-ashes-bestiary:YIXAcvSyI1C94r9l", pack: "age-of-ashes-bestiary", name: "Zephyr Guard" },
        { recordKey: "age-of-ashes-bestiary:XNso2IMnhnfHcMCn", pack: "age-of-ashes-bestiary", name: "Zuferian" },
        { recordKey: "agents-of-edgewatch-bestiary:RtWlzHaOrfFdJyJY", pack: "agents-of-edgewatch-bestiary", name: "Alchemical Horror" },
        { recordKey: "agents-of-edgewatch-bestiary:phkvSUK6WXxgJOoC", pack: "agents-of-edgewatch-bestiary", name: "Alchemist Aspirant" },
        { recordKey: "agents-of-edgewatch-bestiary:gsn4NsJLwZCQUwgf", pack: "agents-of-edgewatch-bestiary", name: "Almiraj" },
        { recordKey: "agents-of-edgewatch-bestiary:1G4OdEHRPF8GMHK8", pack: "agents-of-edgewatch-bestiary", name: "Amateur Chemist" },
        { recordKey: "agents-of-edgewatch-bestiary:MONwgTbrcZFzr6vC", pack: "agents-of-edgewatch-bestiary", name: "Antaro Boldblade" },
        { recordKey: "agents-of-edgewatch-bestiary:azyIfDNNW44jY8YX", pack: "agents-of-edgewatch-bestiary", name: "Barrel Launcher" },
        { recordKey: "agents-of-edgewatch-bestiary:4ffoPNBKdEwBmYgL", pack: "agents-of-edgewatch-bestiary", name: "Battle Leader Rekarek" },
        { recordKey: "agents-of-edgewatch-bestiary:fvOjtzuRNpmpEHXA", pack: "agents-of-edgewatch-bestiary", name: "Binumir" },
        { recordKey: "agents-of-edgewatch-bestiary:7LN8clGKJWTpxISR", pack: "agents-of-edgewatch-bestiary", name: "Blackfingers Acolyte" },
        { recordKey: "agents-of-edgewatch-bestiary:y0bIU9FCWHOJxUzG", pack: "agents-of-edgewatch-bestiary", name: "Bloody Barber Goon" },
        { recordKey: "agents-of-edgewatch-bestiary:TIaZIUb9Mq9B4Mf2", pack: "agents-of-edgewatch-bestiary", name: "Bloody Berleth" },
        { recordKey: "agents-of-edgewatch-bestiary:sUrJ7jxzBiJTbwVo", pack: "agents-of-edgewatch-bestiary", name: "Blune Bandersworth" },
        { recordKey: "agents-of-edgewatch-bestiary:qY1NrXKmL0y18qoz", pack: "agents-of-edgewatch-bestiary", name: "Bolar Of Stonemoor" },
        { recordKey: "agents-of-edgewatch-bestiary:5u7luLMeRNJ4en65", pack: "agents-of-edgewatch-bestiary", name: "Bone Skipper Swarm" },
        { recordKey: "agents-of-edgewatch-bestiary:a3pTQfLzsJThQvI9", pack: "agents-of-edgewatch-bestiary", name: "Calennia" },
        { recordKey: "agents-of-edgewatch-bestiary:MQnhyCM9LInNYtl0", pack: "agents-of-edgewatch-bestiary", name: "Carvey" },
        { recordKey: "agents-of-edgewatch-bestiary:0ti3f4fdcB5D2bLB", pack: "agents-of-edgewatch-bestiary", name: "Casino Bouncer" },
        { recordKey: "agents-of-edgewatch-bestiary:9vt9Dr2a8MkkD83z", pack: "agents-of-edgewatch-bestiary", name: "Chalky" },
        { recordKey: "agents-of-edgewatch-bestiary:EGQgqBfV80ll3pcf", pack: "agents-of-edgewatch-bestiary", name: "Chaos Gulgamodh" },
        { recordKey: "agents-of-edgewatch-bestiary:wsVW8MdOTeGgGM59", pack: "agents-of-edgewatch-bestiary", name: "Child Of Venom" },
        { recordKey: "agents-of-edgewatch-bestiary:HifZEgdCuZearOG2", pack: "agents-of-edgewatch-bestiary", name: "Clockwork Amalgam" },
        { recordKey: "agents-of-edgewatch-bestiary:fV5VIoXMtixmI3Wc", pack: "agents-of-edgewatch-bestiary", name: "Clockwork Assassin" },
        { recordKey: "agents-of-edgewatch-bestiary:D36nL0YjCVHfjBNw", pack: "agents-of-edgewatch-bestiary", name: "Clockwork Chopper" },
        { recordKey: "agents-of-edgewatch-bestiary:w2J6GpuMYM24U4sb", pack: "agents-of-edgewatch-bestiary", name: "Clockwork Injector" },
        { recordKey: "agents-of-edgewatch-bestiary:Wb4Md6byPhBWe56J", pack: "agents-of-edgewatch-bestiary", name: "Cobbleswarm (AoE)" },
        { recordKey: "agents-of-edgewatch-bestiary:JuCLvgvxYbSRXqON", pack: "agents-of-edgewatch-bestiary", name: "Copper Hand Illusionist" },
        { recordKey: "agents-of-edgewatch-bestiary:9M1YJxTXqya55HDx", pack: "agents-of-edgewatch-bestiary", name: "Copper Hand Rogue" },
        { recordKey: "agents-of-edgewatch-bestiary:JdMqHbaTwtOHVE7Y", pack: "agents-of-edgewatch-bestiary", name: "Diobel Sweeper Chemist" },
        { recordKey: "agents-of-edgewatch-bestiary:xkkj0TW6BKNT3Bg4", pack: "agents-of-edgewatch-bestiary", name: "Diobel Sweeper Tough" },
        { recordKey: "agents-of-edgewatch-bestiary:ZL2qLXwomKfBB8Eu", pack: "agents-of-edgewatch-bestiary", name: "Dreadsong Dancer" },
        { recordKey: "agents-of-edgewatch-bestiary:1eMDYXXf2leLTYHV", pack: "agents-of-edgewatch-bestiary", name: "Eberark" },
        { recordKey: "agents-of-edgewatch-bestiary:sn9Pjkr2jlMEqc3E", pack: "agents-of-edgewatch-bestiary", name: "Eunice" },
        { recordKey: "agents-of-edgewatch-bestiary:SgkV5RtcK72d0HwI", pack: "agents-of-edgewatch-bestiary", name: "Excorion" },
        { recordKey: "agents-of-edgewatch-bestiary:1UtsScuNjmgwMZRn", pack: "agents-of-edgewatch-bestiary", name: "Excorion Paragon" },
        { recordKey: "agents-of-edgewatch-bestiary:rM6ix6XTroJod3Vr", pack: "agents-of-edgewatch-bestiary", name: "Fayati Alummur" },
        { recordKey: "agents-of-edgewatch-bestiary:HF0ymYmKC6KydPQ1", pack: "agents-of-edgewatch-bestiary", name: "Franca Laurentz" },
        { recordKey: "agents-of-edgewatch-bestiary:8AdsRaXftoR1beTk", pack: "agents-of-edgewatch-bestiary", name: "Frefferth" },
        { recordKey: "agents-of-edgewatch-bestiary:eqwAdGsAk5JZKxUY", pack: "agents-of-edgewatch-bestiary", name: "Gage Carlyle" },
        { recordKey: "agents-of-edgewatch-bestiary:8GQ7dq7s9CetOlkg", pack: "agents-of-edgewatch-bestiary", name: "Gang Tough" },
        { recordKey: "agents-of-edgewatch-bestiary:pfHOwcITyC4gdCVu", pack: "agents-of-edgewatch-bestiary", name: "Garrote Master Assassin" },
        { recordKey: "agents-of-edgewatch-bestiary:Xi53GFvTgBApltjp", pack: "agents-of-edgewatch-bestiary", name: "Giant Bone Skipper" },
        { recordKey: "agents-of-edgewatch-bestiary:Uhi3wX4KveuMSARt", pack: "agents-of-edgewatch-bestiary", name: "Giant Joro Spider" },
        { recordKey: "agents-of-edgewatch-bestiary:JnNPdvOXtkGQHyfQ", pack: "agents-of-edgewatch-bestiary", name: "Gloaming Will-o'-Wisp" },
        { recordKey: "agents-of-edgewatch-bestiary:H4bY8v6e3drOIoUe", pack: "agents-of-edgewatch-bestiary", name: "Grabble Forden" },
        { recordKey: "agents-of-edgewatch-bestiary:rGTq4qItRB5H7nEk", pack: "agents-of-edgewatch-bestiary", name: "Graveknight Of Kharnas" },
        { recordKey: "agents-of-edgewatch-bestiary:pE5GxoB1FtXBqnF7", pack: "agents-of-edgewatch-bestiary", name: "Gref" },
        { recordKey: "agents-of-edgewatch-bestiary:Bv1s6xJ55HS3Gxgs", pack: "agents-of-edgewatch-bestiary", name: "Grick" },
        { recordKey: "agents-of-edgewatch-bestiary:r3j5KEvULFP3fZS7", pack: "agents-of-edgewatch-bestiary", name: "Grimwold" },
        { recordKey: "agents-of-edgewatch-bestiary:gd0sVQtCHbhP8iHI", pack: "agents-of-edgewatch-bestiary", name: "Grospek Lavarsus" },
        { recordKey: "agents-of-edgewatch-bestiary:181ucNY1zpp2Lz3x", pack: "agents-of-edgewatch-bestiary", name: "Grunka" },
        { recordKey: "agents-of-edgewatch-bestiary:jJ1UTNLiRCoN1O3i", pack: "agents-of-edgewatch-bestiary", name: "Hendrid Pratchett" },
        { recordKey: "agents-of-edgewatch-bestiary:LAamprMlzk7k5auj", pack: "agents-of-edgewatch-bestiary", name: "Hestriviniaas" },
        { recordKey: "agents-of-edgewatch-bestiary:vwxNCuBksHYU2Dwf", pack: "agents-of-edgewatch-bestiary", name: "Hundun Chaos Mage" },
        { recordKey: "agents-of-edgewatch-bestiary:cLlOUUpCIAQwUuOP", pack: "agents-of-edgewatch-bestiary", name: "Il'setsya Wyrmtouched" },
        { recordKey: "agents-of-edgewatch-bestiary:pK1tlsgkmzkaaCe5", pack: "agents-of-edgewatch-bestiary", name: "Iroran Skeleton" },
        { recordKey: "agents-of-edgewatch-bestiary:M8ONVV7yl4uu0zcz", pack: "agents-of-edgewatch-bestiary", name: "Ixusoth" },
        { recordKey: "agents-of-edgewatch-bestiary:gIcReNQOZceZBBlw", pack: "agents-of-edgewatch-bestiary", name: "Jonis Flakfatter" },
        { recordKey: "agents-of-edgewatch-bestiary:E7NEf3kmsY3YjRrz", pack: "agents-of-edgewatch-bestiary", name: "Kapral" },
        { recordKey: "agents-of-edgewatch-bestiary:6c5CnrSxMYEgP6Fz", pack: "agents-of-edgewatch-bestiary", name: "Kekker" },
        { recordKey: "agents-of-edgewatch-bestiary:yR6p0KVvZ3tPflRt", pack: "agents-of-edgewatch-bestiary", name: "Kemeneles" },
        { recordKey: "agents-of-edgewatch-bestiary:3JEiwAFwkEjCDEYa", pack: "agents-of-edgewatch-bestiary", name: "Kolo Harvan" },
        { recordKey: "agents-of-edgewatch-bestiary:E279VPhAy1a4ihqI", pack: "agents-of-edgewatch-bestiary", name: "Living Mural" },
        { recordKey: "agents-of-edgewatch-bestiary:B4kIfCimsz8wfc0k", pack: "agents-of-edgewatch-bestiary", name: "Lord Guirden" },
        { recordKey: "agents-of-edgewatch-bestiary:lzmGdArS3kjJOqT6", pack: "agents-of-edgewatch-bestiary", name: "Lyrma Swampwalker" },
        { recordKey: "agents-of-edgewatch-bestiary:t9m4ikMZsDwo9TQ1", pack: "agents-of-edgewatch-bestiary", name: "Maurrisa Jonne" },
        { recordKey: "agents-of-edgewatch-bestiary:Sq0Kb92nGkqj19Xx", pack: "agents-of-edgewatch-bestiary", name: "Miogimo" },
        { recordKey: "agents-of-edgewatch-bestiary:GWO6vweLGT2J6q62", pack: "agents-of-edgewatch-bestiary", name: "Miriel Grayleaf" },
        { recordKey: "agents-of-edgewatch-bestiary:ZY3q7AV1qbwWwNl2", pack: "agents-of-edgewatch-bestiary", name: "Mobana" },
        { recordKey: "agents-of-edgewatch-bestiary:07AGJt4ZRjwH85Xp", pack: "agents-of-edgewatch-bestiary", name: "Mother Venom" },
        { recordKey: "agents-of-edgewatch-bestiary:Wk2T0Wr8Sebo4br5", pack: "agents-of-edgewatch-bestiary", name: "Mr. Snips" },
        { recordKey: "agents-of-edgewatch-bestiary:vPMmTtvl5UPOcCoa", pack: "agents-of-edgewatch-bestiary", name: "Myrna Rath" },
        { recordKey: "agents-of-edgewatch-bestiary:WDTdWiC9Rdl6rqh8", pack: "agents-of-edgewatch-bestiary", name: "Myrucarx" },
        { recordKey: "agents-of-edgewatch-bestiary:zj2sCM8tQSMG9Qm6", pack: "agents-of-edgewatch-bestiary", name: "Najra Lizard" },
        { recordKey: "agents-of-edgewatch-bestiary:RyXA4wOGY8lenKVw", pack: "agents-of-edgewatch-bestiary", name: "Nenchuuj" },
        { recordKey: "agents-of-edgewatch-bestiary:cQMM2Ld0IBM9GcDo", pack: "agents-of-edgewatch-bestiary", name: "Norgorberite Poisoner" },
        { recordKey: "agents-of-edgewatch-bestiary:1bPc2rjR4MghbMwD", pack: "agents-of-edgewatch-bestiary", name: "Obrousian" },
        { recordKey: "agents-of-edgewatch-bestiary:ySpOZlKUbcxWhKQ6", pack: "agents-of-edgewatch-bestiary", name: "Ofalth Zombie" },
        { recordKey: "agents-of-edgewatch-bestiary:oWASKud0jwlGSfJg", pack: "agents-of-edgewatch-bestiary", name: "Pelmo" },
        { recordKey: "agents-of-edgewatch-bestiary:3XbjmNeUtPLzxDge", pack: "agents-of-edgewatch-bestiary", name: "Penqual" },
        { recordKey: "agents-of-edgewatch-bestiary:8HTPdiH6yEk0jlNF", pack: "agents-of-edgewatch-bestiary", name: "Pickled Punk" },
        { recordKey: "agents-of-edgewatch-bestiary:S9JJsUNSeeoIClON", pack: "agents-of-edgewatch-bestiary", name: "Poison Eater" },
        { recordKey: "agents-of-edgewatch-bestiary:LkaB8RH73DY4TO9V", pack: "agents-of-edgewatch-bestiary", name: "Priest of Blackfingers" },
        { recordKey: "agents-of-edgewatch-bestiary:MqfZxoxFwzqAXhTP", pack: "agents-of-edgewatch-bestiary", name: "Prospecti Statue" },
        { recordKey: "agents-of-edgewatch-bestiary:2uxm1SxZXaG0ynCp", pack: "agents-of-edgewatch-bestiary", name: "Pulping Golem" },
        { recordKey: "agents-of-edgewatch-bestiary:JZUYQzQtzIwOGYvd", pack: "agents-of-edgewatch-bestiary", name: "Ralso" },
        { recordKey: "agents-of-edgewatch-bestiary:aDgVmO3afhIgXQSN", pack: "agents-of-edgewatch-bestiary", name: "Ravenile Rager" },
        { recordKey: "agents-of-edgewatch-bestiary:BuPf7xtqfwCjNOQv", pack: "agents-of-edgewatch-bestiary", name: "Reginald Vancaskerkin" },
        { recordKey: "agents-of-edgewatch-bestiary:kcvU9CatCyBUJRr2", pack: "agents-of-edgewatch-bestiary", name: "Rhevanna" },
        { recordKey: "agents-of-edgewatch-bestiary:4TB1eo3O22khMyDU", pack: "agents-of-edgewatch-bestiary", name: "Sad Liza" },
        { recordKey: "agents-of-edgewatch-bestiary:CLntGVs7cAIL9Trk", pack: "agents-of-edgewatch-bestiary", name: "Scathka" },
        { recordKey: "agents-of-edgewatch-bestiary:BCMkxoQB4xK4BniG", pack: "agents-of-edgewatch-bestiary", name: "Secret-Keeper" },
        { recordKey: "agents-of-edgewatch-bestiary:1xGAktpj6N2Ugh0r", pack: "agents-of-edgewatch-bestiary", name: "Shatterling" },
        { recordKey: "agents-of-edgewatch-bestiary:DKReNCapWWubM3pm", pack: "agents-of-edgewatch-bestiary", name: "Shikwashim Mercenary" },
        { recordKey: "agents-of-edgewatch-bestiary:eQc0ADMuHl1JzL8z", pack: "agents-of-edgewatch-bestiary", name: "Shredskin" },
        { recordKey: "agents-of-edgewatch-bestiary:oTa25rAxytm03T3X", pack: "agents-of-edgewatch-bestiary", name: "Shristi Melipdra" },
        { recordKey: "agents-of-edgewatch-bestiary:XDt87cqF85zWnlC8", pack: "agents-of-edgewatch-bestiary", name: "Siege Shard" },
        { recordKey: "agents-of-edgewatch-bestiary:nTn2szBbqQNdXhOr", pack: "agents-of-edgewatch-bestiary", name: "Skebs" },
        { recordKey: "agents-of-edgewatch-bestiary:8PRqUfkHLvu9ufGL", pack: "agents-of-edgewatch-bestiary", name: "Skinsaw Murderer" },
        { recordKey: "agents-of-edgewatch-bestiary:y1tw2ohNagqQJ6RV", pack: "agents-of-edgewatch-bestiary", name: "Skinsaw Seamer" },
        { recordKey: "agents-of-edgewatch-bestiary:AYNIAAxV7TbIKPI4", pack: "agents-of-edgewatch-bestiary", name: "Skitterstitch" },
        { recordKey: "agents-of-edgewatch-bestiary:xN3FDmrCKWW0psBu", pack: "agents-of-edgewatch-bestiary", name: "Sleepless Sun Veteran" },
        { recordKey: "agents-of-edgewatch-bestiary:P9Wg0sGcNkemOvm3", pack: "agents-of-edgewatch-bestiary", name: "Slithering Rift" },
        { recordKey: "agents-of-edgewatch-bestiary:rsKf8ixrl3yBq1gb", pack: "agents-of-edgewatch-bestiary", name: "Starwatch Commando" },
        { recordKey: "agents-of-edgewatch-bestiary:ai7Q9vBHHAGj7uFE", pack: "agents-of-edgewatch-bestiary", name: "Svartalfar Killer" },
        { recordKey: "agents-of-edgewatch-bestiary:zrh3MrS68H2gPlVs", pack: "agents-of-edgewatch-bestiary", name: "Tenome" },
        { recordKey: "agents-of-edgewatch-bestiary:8jjEOs99Bmo1v0Qc", pack: "agents-of-edgewatch-bestiary", name: "Teraphant" },
        { recordKey: "agents-of-edgewatch-bestiary:WplBGSeB9pK9AULX", pack: "agents-of-edgewatch-bestiary", name: "The Rabbit Prince" },
        { recordKey: "agents-of-edgewatch-bestiary:BoFg19e3N8WiNa3Z", pack: "agents-of-edgewatch-bestiary", name: "The Stabbing Beast" },
        { recordKey: "agents-of-edgewatch-bestiary:4BXo2a305RHmspMX", pack: "agents-of-edgewatch-bestiary", name: "Twisted Jack" },
        { recordKey: "agents-of-edgewatch-bestiary:Rinxhe1cRXKEsXuW", pack: "agents-of-edgewatch-bestiary", name: "Tyrroicese" },
        { recordKey: "agents-of-edgewatch-bestiary:uFU6dQfcNeKq68YT", pack: "agents-of-edgewatch-bestiary", name: "Vargouille" },
        { recordKey: "agents-of-edgewatch-bestiary:tm8vQ7gdAe9zVdDg", pack: "agents-of-edgewatch-bestiary", name: "Vaultbreaker Ooze" },
        { recordKey: "agents-of-edgewatch-bestiary:Fmsw7P5CF3uHtD5W", pack: "agents-of-edgewatch-bestiary", name: "Veksciralenix" },
        { recordKey: "agents-of-edgewatch-bestiary:GYhV5eYNDO1Llbv2", pack: "agents-of-edgewatch-bestiary", name: "Venom Mage" },
        { recordKey: "agents-of-edgewatch-bestiary:5pBr5aWUb7yGCqN7", pack: "agents-of-edgewatch-bestiary", name: "Washboard Dog Tough" },
        { recordKey: "agents-of-edgewatch-bestiary:syUXLdUsEDYgni5R", pack: "agents-of-edgewatch-bestiary", name: "Wrent Dicaspiron" },
        { recordKey: "agents-of-edgewatch-bestiary:8eBXEszbl4gOHGdU", pack: "agents-of-edgewatch-bestiary", name: "Wynsal Starborn" },
        { recordKey: "agents-of-edgewatch-bestiary:JmHyHwaPMNKXzBts", pack: "agents-of-edgewatch-bestiary", name: "Zealborn" },
        { recordKey: "agents-of-edgewatch-bestiary:PEjcy9CxelKC3Kp6", pack: "agents-of-edgewatch-bestiary", name: "Zrukbat" },
        { recordKey: "battlecry-bestiary:HIwXNbXV2sfSiYf4", pack: "battlecry-bestiary", name: "Animated Army" },
        { recordKey: "battlecry-bestiary:tKPZ1iDZukJDvzAK", pack: "battlecry-bestiary", name: "Apprentice Magician Clique" },
        { recordKey: "battlecry-bestiary:mKQXrEpheSCjgJt8", pack: "battlecry-bestiary", name: "Dezullon Thicket" },
        { recordKey: "battlecry-bestiary:V2vFgzymDp2wKRwh", pack: "battlecry-bestiary", name: "Druid Circle" },
        { recordKey: "battlecry-bestiary:xEWRFIUNr52EmwVM", pack: "battlecry-bestiary", name: "Fleshwarp Amalgam" },
        { recordKey: "battlecry-bestiary:pY9fEPDxG925iivp", pack: "battlecry-bestiary", name: "Giant Ant Army" },
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
