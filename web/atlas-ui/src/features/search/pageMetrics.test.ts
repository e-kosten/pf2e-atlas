import type { SearchPageView } from "../../generated/atlas";
import { pagePositionLabel, totalPages } from "./pageMetrics";

describe("pageMetrics", () => {
  it("derives total pages from total records and page size", () => {
    expect(totalPages(page({ total: 14_736, size: 25 }))).toBe(590n);
    expect(pagePositionLabel(page({ number: 13, total: 14_736, size: 25 }))).toBe(
      "13 / 590",
    );
  });

  it("calculates exactly when the intermediate sum exceeds the safe integer range", () => {
    expect(totalPages(page({ total: Number.MAX_SAFE_INTEGER, size: 2 }))).toBe(
      4_503_599_627_370_496n,
    );
  });

  it("handles exact and empty result sets", () => {
    expect(totalPages(page({ total: 50, size: 25 }))).toBe(2n);
    expect(pagePositionLabel(page({ total: 0, size: 25 }))).toBe("0 / 0");
  });
});

function page(overrides: Partial<SearchPageView>): SearchPageView {
  return {
    number: 1,
    size: 25,
    count: 25,
    total: 100,
    has_more: true,
    next_page: 2,
    ...overrides,
  };
}
