import { Button, Input, Tag, theme, type InputRef } from "antd";
import { CloseCircleFilled } from "@ant-design/icons";
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
  similar?: { title: string; onClear: () => void; onSearch: () => void };
}) {
  const { token } = theme.useToken();
  const [entry, setEntry] = useState<{ baseline: string; value: string } | null>(null);
  const input = useRef<InputRef>(null);
  const focusAfterClear = useRef(false);
  const draft = entry?.baseline === query ? entry.value : query;
  const value = similar?.title ?? draft;
  useLayoutEffect(() => {
    if (!similar && focusAfterClear.current) {
      focusAfterClear.current = false;
      input.current?.focus({ preventScroll: true });
    }
  }, [similar]);
  function clear() {
    setEntry({ baseline: query, value: "" });
    if (similar) {
      focusAfterClear.current = true;
      similar.onClear();
    } else {
      onSearch("");
      input.current?.focus({ preventScroll: true });
    }
  }
  return (
    <Input.Search
      ref={input}
      className="reference-search-entry"
      aria-label={similar ? "Similar search" : label}
      placeholder="Name or description…"
      autoComplete="off"
      disabled={disabled}
      readOnly={Boolean(similar)}
      value={value}
      prefix={
        <span style={{ display: "inline-flex", alignItems: "center" }}>
          {similar ? <Tag>Similar to</Tag> : null}
        </span>
      }
      suffix={
        <Button
          type="text"
          disabled={disabled}
          aria-label={similar ? "Clear similar criterion" : "Clear search"}
          aria-hidden={!value}
          tabIndex={value ? 0 : -1}
          icon={<CloseCircleFilled aria-hidden="true" />}
          styles={{
            root: {
              width: token.controlHeightSM,
              minWidth: token.controlHeightSM,
              height: token.controlHeightSM,
              marginBlock: -2,
              padding: 0,
              fontSize: token.fontSizeSM,
              color: token.colorTextQuaternary,
              visibility: value ? "visible" : "hidden",
            },
          }}
          onMouseDown={(event) => event.preventDefault()}
          onClick={clear}
        />
      }
      // Chromium does not select all in a readonly search input by default.
      onKeyDown={(event) => {
        if (
          similar &&
          (event.metaKey || event.ctrlKey) &&
          !event.altKey &&
          event.key.toLowerCase() === "a"
        ) {
          event.preventDefault();
          event.currentTarget.select();
        }
      }}
      // Opening the narrow reader moves focus to Results; Enter must not activate it.
      onPressEnter={(event) => event.preventDefault()}
      onChange={(event) => {
        if (similar) return;
        const next = event.target.value;
        setEntry({ baseline: query, value: next });
        if (!next) onSearch("");
      }}
      onSearch={(next) => {
        if (similar) {
          similar.onSearch();
          return;
        }
        setEntry(null);
        onSearch(next);
      }}
    />
  );
}
