import type { SearchPageView } from "../generated/atlas";
import { pagePositionLabel, totalPages } from "./pageMetrics";

describe("pageMetrics", () => {
  it("derives total pages from total records and page size", () => {
    expect(totalPages(page({ total: 14_736n, size: 25 }))).toBe(590n);
    expect(pagePositionLabel(page({ number: 13, total: 14_736n, size: 25 }))).toBe(
      "13 / 590",
    );
  });

  it("handles exact and empty result sets", () => {
    expect(totalPages(page({ total: 50n, size: 25 }))).toBe(2n);
    expect(pagePositionLabel(page({ total: 0n, size: 25 }))).toBe("0 / 0");
  });
});

function page(overrides: Partial<SearchPageView>): SearchPageView {
  return {
    number: 1,
    size: 25,
    count: 25,
    total: 100n,
    has_more: true,
    ...overrides,
  };
}
