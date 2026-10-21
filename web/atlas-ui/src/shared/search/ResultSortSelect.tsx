import { Select } from "antd";
import type { ResultSort } from "../../generated/atlas";
import {
  effectiveResultSort,
  INVALID_RESULT_SORT,
  UNKNOWN_RESULT_SORT,
  type SearchFormState,
} from "../filters/searchState";

const ATTRIBUTE_OPTIONS: { value: ResultSort; label: string }[] = [
  { value: "name_asc", label: "Name A–Z" },
  { value: "name_desc", label: "Name Z–A" },
  { value: "level_asc", label: "Level / Rank low to high" },
  { value: "level_desc", label: "Level / Rank high to low" },
  { value: "unit_price_asc", label: "Unit price low to high" },
  { value: "unit_price_desc", label: "Unit price high to low" },
];
const COMPACT_LABELS: Partial<Record<ResultSort, string>> = {
  level_asc: "Level / Rank: Low",
  level_desc: "Level / Rank: High",
  unit_price_asc: "Unit price: Low",
  unit_price_desc: "Unit price: High",
};

export function ResultSortSelect({
  search,
  setSearch,
  surface = "search",
  disabled = false,
}: {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  surface?: "search" | "saved_list";
  disabled?: boolean;
}) {
  const options = [
    ...(surface === "saved_list"
      ? [{ value: "saved_order" as const, label: "Saved order" }]
      : search.query.trim() || search.similarSeed
        ? [{ value: "relevance" as const, label: "Relevance" }]
        : []),
    ...ATTRIBUTE_OPTIONS,
  ];
  return (
    <Select<ResultSort>
      aria-label="Result order"
      size="small"
      style={{ width: 184 }}
      popupMatchSelectWidth={false}
      value={effectiveResultSort(search, surface)}
      disabled={disabled}
      options={options}
      labelRender={({ value, label }) => COMPACT_LABELS[value as ResultSort] ?? label}
      onSelect={(sort) =>
        setSearch({
          ...search,
          sort,
          filterError:
            search.filterError === UNKNOWN_RESULT_SORT ||
            search.filterError === INVALID_RESULT_SORT
              ? null
              : search.filterError,
        })
      }
    />
  );
}
