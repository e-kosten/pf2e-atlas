import { describe, expect, it } from "vitest";

import { analyzeNormalizedTextTokens } from "../../src/tags/runtime/wink-nlp-adapter.js";
import { normalizeText } from "../../src/shared/utils.js";

function analyze(value: string) {
  const normalized = normalizeText(value);
  const tokens = normalized.length > 0 ? normalized.split(" ") : [];
  return analyzeNormalizedTextTokens(normalized, tokens);
}

describe("wink nlp adapter", () => {
  it("returns analyzed tokens when wink tokenization stays aligned", () => {
    const analyzed = analyze("The guardian stands within the keep walls.");

    expect(analyzed.map((token) => token.token)).toEqual([
      "the",
      "guardian",
      "stands",
      "within",
      "the",
      "keep",
      "walls",
    ]);
    expect(analyzed.find((token) => token.token === "keep")?.lemma).toBe("keep");
    expect(analyzed.find((token) => token.token === "keep")?.pos).toBe("NOUN");
  });

  it("fails closed when caller-provided tokens no longer match wink tokenization", () => {
    const analyzed = analyzeNormalizedTextTokens("the sentry cant keep watch", ["the", "sentry", "keep", "watch"]);

    expect(analyzed.map((token) => token.token)).toEqual(["the", "sentry", "keep", "watch"]);
    expect(analyzed.every((token) => token.pos === null && token.lemma === null)).toBe(true);
  });
});
