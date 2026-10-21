import { Button, Input, Tag, Typography, type InputRef } from "antd";
import { useLayoutEffect, useRef, useState } from "react";

export function SearchEntry({
  query,
  onSearch,
  disabled,
  label = "Search records",
  similar,
}: {
  query: string;
  onSearch: (query: string) => void;
  disabled?: boolean;
  label?: string;
  similar?: { title: string; onClear: () => void };
}) {
  const [entry, setEntry] = useState<{ baseline: string; value: string } | null>(null);
  const input = useRef<InputRef>(null);
  const focusAfterClear = useRef(false);
  const draft = entry?.baseline === query ? entry.value : query;
  useLayoutEffect(() => {
    if (!similar && focusAfterClear.current) {
      focusAfterClear.current = false;
      input.current?.focus({ preventScroll: true });
    }
  }, [similar]);
  if (similar)
    return (
      <div
        className="reference-search-entry reference-search-entry--similar"
        role="group"
        aria-label="Similar search"
      >
        <Tag>Similar to</Tag>
        <Typography.Text className="reference-search-entry__title">
          {similar.title}
        </Typography.Text>
        <Button
          onClick={() => {
            focusAfterClear.current = true;
            setEntry(null);
            similar.onClear();
          }}
          aria-label="Clear similar criterion"
        >
          Clear
        </Button>
      </div>
    );
  return (
    <Input.Search
      ref={input}
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
