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
  "are",
  "around",
  "as",
  "because",
  "be",
  "become",
  "becomes",
  "before",
  "being",
  "between",
  "beyond",
  "both",
  "by",
  "can",
  "creature",
  "creatures",
  "during",
  "each",
  "deal",
  "deals",
  "for",
  "from",
  "gain",
  "gains",
  "had",
  "has",
  "have",
  "helps",
  "if",
  "in",
  "into",
  "is",
  "it",
  "its",
  "just",
  "like",
  "make",
  "makes",
  "many",
  "more",
  "most",
  "of",
  "often",
  "on",
  "once",
  "own",
  "other",
  "or",
  "over",
  "per",
  "s",
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
  "up",
  "when",
  "where",
  "which",
  "while",
  "with",
  "within",
  "without",
  "you",
  "your",
]);

const DISCOVERY_NOISE_TOKENS = new Set([
  "activate",
  "activated",
  "activation",
  "action",
  "actions",
  "cast",
  "check",
  "checks",
  "command",
  "compendium",
  "critical",
  "critically",
  "effect",
  "failure",
  "frequency",
  "item",
  "items",
  "pf2e",
  "requirement",
  "requirements",
  "success",
  "trigger",
  "triggers",
  "uuid",
]);

const DICE_SENTINEL = "zzzdiceplaceholderzzz";
const RANGE_SENTINEL = "zzzrangeplaceholderzzz";
const NUMBER_SENTINEL = "zzznumberplaceholderzzz";

const FOUNDRY_INLINE_TAG_WITH_LABEL_PATTERN = /@[a-z]+(?:\.[a-z]+)?\[[^\]]+\]\{([^}]*)\}/gi;
const FOUNDRY_INLINE_TAG_PATTERN = /@[a-z]+(?:\.[a-z]+)?\[[^\]]+\]/gi;
const FOUNDRY_INLINE_ROLL_WITH_LABEL_PATTERN = /\[\[[^\]]+\]\]\{([^}]*)\}/g;
const FOUNDRY_INLINE_ROLL_PATTERN = /\[\[[^\]]+\]\]/g;
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

function stripFoundryInlineMarkup(value: string): string {
  return value
    .replace(FOUNDRY_INLINE_TAG_WITH_LABEL_PATTERN, " $1 ")
    .replace(FOUNDRY_INLINE_TAG_PATTERN, " ")
    .replace(FOUNDRY_INLINE_ROLL_WITH_LABEL_PATTERN, " $1 ")
    .replace(FOUNDRY_INLINE_ROLL_PATTERN, " ");
}

export function normalizeDiscoveryText(value: string): string {
  const staged = stripFoundryInlineMarkup(value)
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

export function isDiscoveryPlaceholder(value: string): boolean {
  return value === "{{number}}" || value === "{{range}}" || value === "{{dice}}";
}

export function isDiscoveryNoiseToken(value: string): boolean {
  return isDiscoveryPlaceholder(value) ||
    /^[a-z]$/.test(value) ||
    DISCOVERY_STOPWORDS.has(value) ||
    DISCOVERY_NOISE_TOKENS.has(value);
}

export function isDiscoveryNoisePhrase(value: string): boolean {
  const tokens = normalizeDiscoveryText(value).split(" ").filter(Boolean);
  return tokens.some((token) => isDiscoveryPlaceholder(token) || DISCOVERY_NOISE_TOKENS.has(token));
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
