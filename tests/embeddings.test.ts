import { describe, expect, it } from "vitest";

import { HashEmbeddingProvider } from "../src/embeddings.js";

describe("embedding providers", () => {
  it("returns stable ordered vectors from hash embedMany", async () => {
    const provider = new HashEmbeddingProvider(16);
    const texts = ["swallow spike", "ship captain", "masquerade scarf"];

    const singleVectors = await Promise.all(texts.map((text) => provider.embed(text)));
    const batchVectors = await provider.embedMany(texts);

    expect(batchVectors).toHaveLength(texts.length);
    expect(batchVectors.map((vector) => Array.from(vector))).toEqual(
      singleVectors.map((vector) => Array.from(vector)),
    );
  });
});
