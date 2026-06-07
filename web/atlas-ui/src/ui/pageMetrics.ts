import type { SearchPageView } from "../generated/atlas";

export function totalPages(page: SearchPageView): bigint {
  if (page.size <= 0 || page.total === 0n) {
    return 0n;
  }
  const size = BigInt(page.size);
  return (page.total + size - 1n) / size;
}

export function pagePositionLabel(page: SearchPageView): string {
  const pages = totalPages(page);
  return pages === 0n
    ? "0 / 0"
    : `${page.number.toLocaleString()} / ${pages.toLocaleString()}`;
}
