import winkNLP from "wink-nlp";
import model from "wink-eng-lite-web-model";

import { normalizeText } from "../../shared/utils.js";

export type { PartOfSpeech } from "wink-nlp";
import type { PartOfSpeech } from "wink-nlp";

const nlp = winkNLP(model);
const its = nlp.its;

export type AnalyzedToken = {
  token: string;
  pos: PartOfSpeech | null;
  lemma: string | null;
};

function createUnknownAnalyzedTokens(tokens: string[]): AnalyzedToken[] {
  return tokens.map((token) => ({
    token,
    pos: null,
    lemma: null,
  }));
}

export function analyzeNormalizedTextTokens(normalized: string, tokens: string[]): AnalyzedToken[] {
  if (normalized.length === 0 || tokens.length === 0) {
    return [];
  }

  const doc = nlp.readDoc(normalized);
  // wink-nlp token extraction expects the library-provided helpers directly.
  // eslint-disable-next-line @typescript-eslint/unbound-method
  const analyzedTokens = doc.tokens().out(its.normal);
  // eslint-disable-next-line @typescript-eslint/unbound-method
  const analyzedPos = doc.tokens().out(its.pos) as PartOfSpeech[];
  const analyzedLemmas = doc.tokens().out((index, rdd, _cache, addons) => its.lemma(index, rdd, addons));
  if (
    analyzedTokens.length !== tokens.length ||
    analyzedPos.length !== tokens.length ||
    analyzedLemmas.length !== tokens.length ||
    analyzedTokens.some((token, index) => token !== tokens[index])
  ) {
    return createUnknownAnalyzedTokens(tokens);
  }

  return tokens.map((token, index) => ({
    token,
    pos: analyzedPos[index] ?? null,
    lemma: normalizeText(analyzedLemmas[index] ?? "") || null,
  }));
}
