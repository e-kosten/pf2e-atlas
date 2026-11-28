import {
  Tooltip,
  Button,
  Input,
  Popover,
  Space,
  Tag,
  theme,
  type InputRef,
} from "antd";
import { SearchOutlined } from "@ant-design/icons";
import type { SimilaritySeed } from "../../generated/atlas";
import { SimilaritySeedLink } from "../search/SimilaritySeedLink";
import { CloseCircleFilled } from "@ant-design/icons";
import scrollIntoView from "scroll-into-view-if-needed";
import {
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ComponentProps,
  type RefObject,
} from "react";
import {
  useNameSuggestions,
  type NameSuggestionContext,
} from "../search/useNameSuggestions";
import { SearchNameLabel } from "../search/SearchNameLabel";

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
  errorId,
  nameSuggestions,
}: {
  query: string;
  onSearch: (query: string, submission: SearchSubmission) => void;
  acceptedSubmission?: SearchSubmission;
  disabled?: boolean;
  label?: string;
  placeholder?: string;
  similar?: SimilarSearchCriterion;
  inputRef?: RefObject<InputRef | null>;
  errorId?: string;
  nameSuggestions?: NameSuggestionContext;
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
  const inputProps: SearchTextInputProps = {
    ref: input,
    "aria-label": label,
    "aria-invalid": errorId && value === query ? true : undefined,
    "aria-describedby": value === query ? errorId : undefined,
    placeholder,
    disabled,
    value,
    onClear: clear,
    onChange: (event) => {
      const next = event.target.value;
      if (next) setEntry({ ...entry, draft: { value: next } });
      else submit("");
    },
    onSearch: submit,
  };
  return nameSuggestions ? (
    <SuggestedSearchInput
      input={inputProps}
      draft={entry.draft?.submission ? "" : (entry.draft?.value ?? "")}
      context={nameSuggestions}
    />
  ) : (
    <SearchTextInput {...inputProps} />
  );
}

type SearchTextInputProps = ComponentProps<typeof Input.Search> & {
  onClear: () => void;
};

function SearchTextInput({ onClear, ...input }: SearchTextInputProps) {
  const { token } = theme.useToken();
  return (
    <Input.Search
      {...input}
      className="reference-search-entry"
      autoComplete="off"
      suffix={
        <Tooltip title="Clear search" trigger={["hover", "focus"]}>
          <Button
            type="text"
            disabled={input.disabled}
            aria-label="Clear search"
            aria-hidden={!input.value}
            tabIndex={input.value ? 0 : -1}
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
                visibility: input.value ? "visible" : "hidden",
              },
            }}
            onMouseDown={(event) => event.preventDefault()}
            onClick={onClear}
          />
        </Tooltip>
      }
      // Opening the narrow reader moves focus to Results; Enter must not activate it.
      onPressEnter={(event) => event.preventDefault()}
    />
  );
}

function SuggestedSearchInput({
  input,
  draft,
  context,
}: {
  input: SearchTextInputProps;
  draft: string;
  context: NameSuggestionContext;
}) {
  const { token } = theme.useToken();
  const suggestionId = useId();
  const [suggesting, setSuggesting] = useState(false);
  const [composing, setComposing] = useState(false);
  const [highlighted, setHighlighted] = useState<string | null>(null);
  const suggestions = useNameSuggestions(
    context,
    suggesting && !composing && !input.disabled ? draft : "",
  );
  const rows = suggestions?.rows ?? [];
  const activeOption = rows.findIndex((row) => row.record.record_key === highlighted);
  function dismissSuggestions() {
    setSuggesting(false);
    setHighlighted(null);
  }
  function openSuggestion(key: string) {
    if (!suggestions) return;
    dismissSuggestions();
    context.onSelect(key, suggestions.artifact_generation);
  }
  const searchInput = (
    <SearchTextInput
      {...input}
      role="combobox"
      aria-autocomplete="list"
      aria-expanded={rows.length > 0}
      aria-controls={rows.length > 0 ? suggestionId : undefined}
      aria-activedescendant={
        activeOption >= 0 ? `${suggestionId}-${activeOption}` : undefined
      }
      onBlur={dismissSuggestions}
      onCompositionStart={() => setComposing(true)}
      onCompositionEnd={() => setComposing(false)}
      onKeyDown={(event) => {
        if (composing || event.nativeEvent.isComposing) return;
        if (event.key === "Escape") {
          dismissSuggestions();
          if (rows.length) {
            event.preventDefault();
            event.stopPropagation();
          }
        } else if (
          rows.length &&
          (event.key === "ArrowDown" || event.key === "ArrowUp")
        ) {
          event.preventDefault();
          const direction = event.key === "ArrowDown" ? 1 : -1;
          const index =
            activeOption < 0
              ? direction === 1
                ? 0
                : rows.length - 1
              : (activeOption + direction + rows.length) % rows.length;
          setHighlighted(rows[index].record.record_key);
          const option = event.currentTarget.ownerDocument.getElementById(
            `${suggestionId}-${index}`,
          );
          const list = option?.parentElement;
          if (option && list)
            scrollIntoView(option, {
              boundary: list,
              block: "nearest",
              scrollMode: "if-needed",
            });
        }
      }}
      onChange={(event) => {
        setSuggesting(Boolean(event.target.value));
        setHighlighted(null);
        input.onChange?.(event);
      }}
      onSearch={(next, event, info) => {
        if (event?.type === "keydown" && activeOption >= 0) {
          openSuggestion(rows[activeOption].record.record_key);
          return;
        }
        dismissSuggestions();
        input.onSearch?.(next, event, info);
      }}
    />
  );
  return (
    <Popover
      open={rows.length > 0}
      trigger={[]}
      placement="bottomLeft"
      align={{ overflow: { shiftX: true, adjustY: true } }}
      arrow={false}
      styles={{ container: { padding: token.paddingXXS } }}
      content={
        <div
          className="search-name-suggestions"
          id={suggestionId}
          role="listbox"
          aria-label="Name suggestions"
        >
          {rows.map(({ record }, index) => (
            <Button
              color="default"
              variant="text"
              block
              className="search-name-suggestion"
              id={`${suggestionId}-${index}`}
              key={record.record_key}
              role="option"
              tabIndex={-1}
              aria-selected={activeOption === index}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => openSuggestion(record.record_key)}
            >
              <SearchNameLabel record={record} />
            </Button>
          ))}
        </div>
      }
    >
      <div className="reference-search-entry">{searchInput}</div>
    </Popover>
  );
}
