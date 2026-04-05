import { normalizeText } from "../utils.js";

const DISCOVERY_STOPWORDS = new Set([
  "a",
  "about",
  "after",
  "also",
  "among",
  "and",
  "an",
  "another",
  "around",
  "because",
  "become",
  "becomes",
  "before",
  "being",
  "between",
  "beyond",
  "both",
  "can",
  "creature",
  "creatures",
  "during",
  "each",
  "from",
  "gain",
  "gains",
  "have",
  "helps",
  "into",
  "just",
  "like",
  "make",
  "makes",
  "many",
  "more",
  "most",
  "often",
  "other",
  "over",
  "such",
  "than",
  "that",
  "the",
  "their",
  "them",
  "they",
  "this",
  "those",
  "through",
  "to",
  "toward",
  "under",
  "until",
  "when",
  "where",
  "which",
  "while",
  "with",
  "within",
  "without",
  "your",
]);

const DICE_SENTINEL = "zzzdiceplaceholderzzz";
const RANGE_SENTINEL = "zzzrangeplaceholderzzz";
const NUMBER_SENTINEL = "zzznumberplaceholderzzz";

const DICE_EXPRESSION_PATTERN = /\b\d+d\d+(?:\s*[+\-]\s*\d+)?\b/gi;
const MEASUREMENT_PATTERN = /\b\d+\s*(?:-\s*)?(?:foot|feet|ft|mile|miles|meter|meters|yard|yards)\b/gi;
const STANDALONE_NUMBER_PATTERN = /\b\d+\b/g;

export type DiscoveryPhraseOccurrence = {
  normalized: string;
  raw: string;
};

function restorePlaceholders(value: string): string {
  return value
    .replace(new RegExp(DICE_SENTINEL, "g"), "{{dice}}")
    .replace(new RegExp(RANGE_SENTINEL, "g"), "{{range}}")
    .replace(new RegExp(NUMBER_SENTINEL, "g"), "{{number}}");
}

export function normalizeDiscoveryText(value: string): string {
  const staged = value
    .toLowerCase()
    .replace(/&nbsp;/g, " ")
    .replace(DICE_EXPRESSION_PATTERN, ` ${DICE_SENTINEL} `)
    .replace(MEASUREMENT_PATTERN, ` ${RANGE_SENTINEL} `)
    .replace(STANDALONE_NUMBER_PATTERN, ` ${NUMBER_SENTINEL} `);

  const normalized = staged
    .replace(/[^a-z0-9{}]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");

  return restorePlaceholders(normalized);
}

export function tokenizeDiscoveryText(
  value: string,
  options: { filterStopwords?: boolean } = {},
): string[] {
  const tokens = normalizeDiscoveryText(value).split(" ").filter(Boolean);
  if (!options.filterStopwords) {
    return tokens;
  }

  return tokens.filter((token) => !DISCOVERY_STOPWORDS.has(token));
}

export function normalizeDiscoveryFeature(value: string): string {
  return normalizeText(value);
}

export function extractDiscoveryNgrams(
  value: string,
  n: number,
  options: { filterStopwords?: boolean } = {},
): DiscoveryPhraseOccurrence[] {
  const rawTokens = normalizeDiscoveryText(value).split(" ").filter(Boolean);
  const filteredTokens = options.filterStopwords
    ? rawTokens.filter((token) => !DISCOVERY_STOPWORDS.has(token))
    : rawTokens;
  if (n <= 0 || filteredTokens.length < n) {
    return [];
  }

  const occurrences: DiscoveryPhraseOccurrence[] = [];
  for (let index = 0; index <= filteredTokens.length - n; index += 1) {
    const slice = filteredTokens.slice(index, index + n);
    const normalized = slice.join(" ");
    if (!normalized) {
      continue;
    }

    occurrences.push({
      normalized,
      raw: normalized,
    });
  }

  return occurrences;
}

export function isDiscoveryStopword(value: string): boolean {
  return DISCOVERY_STOPWORDS.has(value);
}
