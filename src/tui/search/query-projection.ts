import type { SearchFilterNode } from "../../domain/search-request-types.js";
import type { Pf2eTerminalSearchQuery } from "./service.js";

export type Pf2eTerminalSearchQueryBase =
  Omit<Pf2eTerminalSearchQuery, "filter">;

export function stripSearchQueryFilter(query: Pf2eTerminalSearchQuery): Pf2eTerminalSearchQueryBase {
  const { filter: _filter, ...baseQuery } = query;
  return baseQuery;
}

export function projectSearchQueryFilter(
  baseQuery: Pf2eTerminalSearchQueryBase,
  filter: SearchFilterNode | undefined,
): Pf2eTerminalSearchQuery {
  return filter ? ({ ...baseQuery, filter } as Pf2eTerminalSearchQuery) : ({ ...baseQuery } as Pf2eTerminalSearchQuery);
}
