import type { SearchPageView } from "../generated/atlas";
import { toBigInt, type Wire } from "./wire";

export function convertSearchPage(page: Wire<SearchPageView>): SearchPageView {
  return { ...page, total: toBigInt(page.total) };
}
