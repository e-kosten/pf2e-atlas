import { Tooltip, Button, Input, Space, Tag, theme, type InputRef } from "antd";
import { SearchOutlined } from "@ant-design/icons";
import type { SimilaritySeed } from "../../generated/atlas";
import { SimilaritySeedLink } from "../search/SimilaritySeedLink";
import { CloseCircleFilled } from "@ant-design/icons";
import { useLayoutEffect, useRef, useState, type RefObject } from "react";

import type { SearchSubmission } from "./filterControls";

export type SimilarSearchCriterion = {
  title: string;
  seed: SimilaritySeed;
  onClear: () => void;
  onSearch: () => void;
};

export function SearchEntry({
  query,
  onSearch,
  acceptedSubmission,
  disabled,
  label = "Search records",
  placeholder = "Name or description…",
  similar,
  inputRef,
}: {
  query: string;
  onSearch: (query: string, submission: SearchSubmission) => void;
  acceptedSubmission?: SearchSubmission;
  disabled?: boolean;
  label?: string;
  placeholder?: string;
  similar?: SimilarSearchCriterion;
  inputRef?: RefObject<InputRef | null>;
}) {
  const { token } = theme.useToken();
  const [field] = useState(() => ({}));
  const [entry, setEntry] = useState<{
    query: string;
    acknowledgement?: SearchSubmission;
    draft?: { value: string; submission?: SearchSubmission };
  }>({ query, acknowledgement: acceptedSubmission });
  if (entry.query !== query || entry.acknowledgement !== acceptedSubmission) {
    const ownCommit =
      acceptedSubmission?.field === field &&
      entry.acknowledgement !== acceptedSubmission;
    const keepDraft = ownCommit
      ? entry.draft?.submission !== acceptedSubmission
      : entry.query === query;
    setEntry({
      query,
      acknowledgement: acceptedSubmission,
      draft: keepDraft ? entry.draft : undefined,
    });
  }
  const localInput = useRef<InputRef>(null);
  const input = inputRef ?? localInput;
  const focusAfterClear = useRef(false);
  const value = entry.draft?.value ?? query;
  useLayoutEffect(() => {
    if (!similar && focusAfterClear.current) {
      focusAfterClear.current = false;
      input.current?.focus({ preventScroll: true });
    }
  }, [input, similar]);
  function submit(value: string) {
    const submission = { field };
    setEntry({ ...entry, draft: { value, submission } });
    onSearch(value, submission);
  }
  function clear() {
    if (similar) {
      setEntry({ ...entry, draft: { value: "" } });
      focusAfterClear.current = true;
      similar.onClear();
    } else {
      submit("");
      input.current?.focus({ preventScroll: true });
    }
  }
  if (similar)
    return (
      <Space.Compact className="reference-search-entry--similar">
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
          <span className="reference-search-entry__source" title={similar.title}>
            <SimilaritySeedLink
              seed={similar.seed}
              title={similar.title}
              disabled={disabled}
            />
          </span>
          <Tooltip title="Clear similar criterion" trigger={["hover", "focus"]}>
            <Button
              type="text"
              disabled={disabled}
              size="small"
              aria-label="Clear similar criterion"
              icon={<CloseCircleFilled aria-hidden="true" />}
              onClick={clear}
            />
          </Tooltip>
        </div>
        <Tooltip title="Run similar search" trigger={["hover", "focus"]}>
          <Button
            disabled={disabled}
            aria-label="Run similar search"
            icon={<SearchOutlined />}
            onClick={similar.onSearch}
          />
        </Tooltip>
      </Space.Compact>
    );
  return (
    <Input.Search
      ref={input}
      className="reference-search-entry"
      aria-label={label}
      placeholder={placeholder}
      autoComplete="off"
      disabled={disabled}
      value={value}
      suffix={
        <Tooltip title="Clear search" trigger={["hover", "focus"]}>
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
        </Tooltip>
      }
      // Opening the narrow reader moves focus to Results; Enter must not activate it.
      onPressEnter={(event) => event.preventDefault()}
      onChange={(event) => {
        const next = event.target.value;
        if (next) setEntry({ ...entry, draft: { value: next } });
        else submit("");
      }}
      onSearch={submit}
    />
  );
}
