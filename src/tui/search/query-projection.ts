import type { SearchFilterNode } from "../../domain/search-request-types.js";
import type { Pf2eTerminalSearchQuery } from "./service.js";

export type Pf2eTerminalSearchQueryBase =
  | Omit<Extract<Pf2eTerminalSearchQuery, { mode: "browse" }>, "filter">
  | Omit<Extract<Pf2eTerminalSearchQuery, { mode: "search" }>, "filter">
  | Omit<Extract<Pf2eTerminalSearchQuery, { mode: "lookup" }>, "filter">;

export function stripSearchQueryFilter(query: Pf2eTerminalSearchQuery): Pf2eTerminalSearchQueryBase {
  const { filter: _filter, ...baseQuery } = query;
  return baseQuery;
}

export function projectSearchQueryFilter(
  baseQuery: Pf2eTerminalSearchQueryBase,
  filter: SearchFilterNode | undefined,
): Pf2eTerminalSearchQuery {
  return filter ? { ...baseQuery, filter } : { ...baseQuery };
}
