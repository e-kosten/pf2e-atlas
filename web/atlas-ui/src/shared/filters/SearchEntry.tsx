import { Input } from "antd";
import { useState } from "react";

export function SearchEntry({
  query,
  onSearch,
  disabled,
  label = "Search records",
}: {
  query: string;
  onSearch: (query: string) => void;
  disabled?: boolean;
  label?: string;
}) {
  const [entry, setEntry] = useState<{ baseline: string; value: string } | null>(null);
  const draft = entry?.baseline === query ? entry.value : query;
  return (
    <Input.Search
      className="reference-search-entry"
      aria-label={label}
      placeholder="Name or description…"
      autoComplete="off"
      disabled={disabled}
      value={draft}
      allowClear
      // Opening the narrow reader moves focus to Results; Enter must not activate it.
      onPressEnter={(event) => event.preventDefault()}
      onChange={(event) => {
        const value = event.target.value;
        setEntry({ baseline: query, value });
        if (!value) onSearch("");
      }}
      onSearch={(value) => {
        setEntry(null);
        onSearch(value);
      }}
    />
  );
}
