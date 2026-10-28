import { Button, Input, Space, Tag, theme, type InputRef } from "antd";
import { SearchOutlined } from "@ant-design/icons";
import type { SimilaritySeed } from "../../generated/atlas";
import { SimilaritySeedLink } from "../search/SimilaritySeedLink";
import { CloseCircleFilled } from "@ant-design/icons";
import { useLayoutEffect, useRef, useState } from "react";

export type SimilarSearchCriterion = {
  title: string;
  seed: SimilaritySeed;
  onClear: () => void;
  onSearch: () => void;
};

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
  similar?: SimilarSearchCriterion;
}) {
  const { token } = theme.useToken();
  const [entry, setEntry] = useState<{ baseline: string; value: string } | null>(null);
  const input = useRef<InputRef>(null);
  const focusAfterClear = useRef(false);
  const draft = entry?.baseline === query ? entry.value : query;
  const value = draft;
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
  if (similar)
    return (
      <Space.Compact className="reference-search-entry reference-search-entry--similar">
        <div
          className="reference-search-entry__criterion"
          style={{
            borderColor: token.colorBorder,
            borderRadius: `${token.borderRadius}px 0 0 ${token.borderRadius}px`,
            background: token.colorBgContainer,
            minHeight: token.controlHeight,
            paddingInline: token.paddingSM,
          }}
        >
          <Tag>Similar to</Tag>
          <span className="reference-search-entry__source">
            <SimilaritySeedLink
              seed={similar.seed}
              title={similar.title}
              disabled={disabled}
            />
          </span>
          <Button
            type="text"
            disabled={disabled}
            size="small"
            aria-label="Clear similar criterion"
            icon={<CloseCircleFilled aria-hidden="true" />}
            onClick={clear}
          />
        </div>
        <Button
          disabled={disabled}
          aria-label="Run similar search"
          icon={<SearchOutlined />}
          onClick={similar.onSearch}
        />
      </Space.Compact>
    );
  return (
    <Input.Search
      ref={input}
      className="reference-search-entry"
      aria-label={label}
      placeholder="Name or description…"
      autoComplete="off"
      disabled={disabled}
      value={value}
      suffix={
        <Button
          type="text"
          disabled={disabled}
          aria-label="Clear search"
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
      // Opening the narrow reader moves focus to Results; Enter must not activate it.
      onPressEnter={(event) => event.preventDefault()}
      onChange={(event) => {
        const next = event.target.value;
        setEntry({ baseline: query, value: next });
        if (!next) onSearch("");
      }}
      onSearch={(next) => {
        setEntry(null);
        onSearch(next);
      }}
    />
  );
}
