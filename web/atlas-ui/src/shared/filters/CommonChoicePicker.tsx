import {
  CloseOutlined,
  DownOutlined,
  MinusOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import {
  Alert,
  Button,
  Drawer,
  Grid,
  Input,
  Popover,
  Space,
  Spin,
  Typography,
  theme,
} from "antd";
import type { InputRef } from "antd";
import { useEffect, useId, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
  QueryPredicate,
} from "../../generated/atlas";
import {
  changeChoiceValue,
  choiceClauses,
  choiceValueState,
  conditionValues,
  type ChoiceState,
} from "./filterControls";
import { useFilterVocabulary, VOCABULARY_PAGE_SIZE } from "./useFilterDiscovery";

/** One shared interaction for common categorical fields in search and saved lists. */
export function CommonChoicePicker({
  field,
  clause,
  scope,
  onChange,
}: {
  field: FilterEditorFieldView;
  clause: QueryPredicate | null;
  scope: FilterVocabularyScope;
  onChange: (next: QueryPredicate | null) => void;
}) {
  const selected = choiceClauses(clause).flatMap(conditionValues);
  const { token } = theme.useToken();
  const [open, setOpen] = useState(false);
  const [sessionValues, setSessionValues] = useState<string[]>([]);
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const input = useRef<InputRef>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const initialFocusPending = useRef(false);
  const menuId = useId();
  const screens = Grid.useBreakpoint();
  const narrow = screens.xs === true && !screens.sm;
  const closedChoices = field.choices.length > 0;
  const query = useFilterVocabulary(
    scope,
    field.definition.id,
    search,
    offset,
    selected,
    open && !closedChoices,
  );
  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setSearch(text);
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [text]);

  const choices = closedChoices
    ? field.choices.map((choice) => ({
        value: String(choice.value),
        label: choice.label,
      }))
    : (query.data?.values.options ?? []).map((option) => ({
        value: String(option.value),
        label: String(option.value),
      }));
  // Retain initial selections and every edited value through this open session.
  // Suggestions always keep server/catalog order, irrespective of selection.
  const options = [
    ...choices,
    ...[...new Set([...sessionValues, ...selected])]
      .filter((value) => !choices.some((choice) => choice.value === value))
      .map((value) => ({ value, label: value })),
  ];
  const rows = closedChoices
    ? options.filter((option) =>
        option.label.toLowerCase().includes(text.trim().toLowerCase()),
      )
    : options;
  function setMenuOpen(next: boolean) {
    initialFocusPending.current = next;
    if (next) setSessionValues(selected);
    else {
      setSessionValues([]);
      setText("");
      setSearch("");
      setOffset(0);
    }
    setOpen(next);
  }
  function cancelInitialFocus() {
    initialFocusPending.current = false;
  }
  function change(value: string, state: ChoiceState) {
    if (open)
      setSessionValues((current) =>
        current.includes(value) ? current : [...current, value],
      );
    onChange(changeChoiceValue(field.definition, clause, value, state));
  }
  function closeOnEscape(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && !narrow) {
      event.stopPropagation();
      setMenuOpen(false);
      trigger.current?.focus();
    }
  }
  const menu = (
    <section
      id={menuId}
      role={narrow ? "group" : "dialog"}
      aria-label={field.label + " choices"}
      className="common-choice-menu"
      onFocusCapture={cancelInitialFocus}
      onPointerDownCapture={cancelInitialFocus}
    >
      <Input
        ref={input}
        aria-label={"Search " + field.label.toLowerCase() + " choices"}
        name={field.definition.id + "-choices"}
        placeholder={"Search " + field.label.toLowerCase() + "…"}
        autoComplete="off"
        value={text}
        allowClear
        onChange={(event) => {
          setText(event.target.value);
          setOffset(0);
        }}
        onKeyDown={closeOnEscape}
      />
      <div
        className="common-choice-options"
        aria-busy={!closedChoices && query.isFetching}
      >
        {rows.map((option) => {
          const state = choiceValueState(clause, option.value);
          const next: ChoiceState =
            state === "neutral"
              ? "include"
              : state === "include"
                ? "exclude"
                : "neutral";
          return (
            <Button
              key={option.value}
              className="common-choice-row"
              block
              color={
                state === "exclude"
                  ? "danger"
                  : state === "include"
                    ? "primary"
                    : "default"
              }
              variant="text"
              styles={{
                root: {
                  background:
                    state === "include"
                      ? token.colorPrimaryBg
                      : state === "exclude"
                        ? token.colorErrorBg
                        : undefined,
                },
              }}
              aria-label={`${field.label}: ${option.label}, ${state === "include" ? "included" : state === "exclude" ? "excluded" : "neutral"}; activate to ${next === "neutral" ? "clear" : next}`}
              onClick={() => change(option.value, next)}
              onKeyDown={closeOnEscape}
            >
              <span className="common-choice-marker" aria-hidden="true">
                {state === "include" ? (
                  <PlusOutlined />
                ) : state === "exclude" ? (
                  <MinusOutlined />
                ) : null}
              </span>
              <span className="common-choice-label">{option.label}</span>
            </Button>
          );
        })}
        {!rows.length && !query.isFetching && !query.error && (
          <Typography.Text type="secondary" role="status">
            No matching values
          </Typography.Text>
        )}
      </div>
      {!closedChoices && (
        <div className="common-choice-footer">
          {query.isFetching && <Spin size="small" />}
          {query.data && (
            <Typography.Text type="secondary" role="status">
              {new Intl.NumberFormat().format(query.data.values.total_values)} values
              {scope.kind === "saved_list" ? " in this list" : ""}
            </Typography.Text>
          )}
          <Space size="small">
            <Button
              size="small"
              aria-label="Previous values"
              disabled={offset === 0 || query.isFetching}
              onClick={() => setOffset(Math.max(0, offset - VOCABULARY_PAGE_SIZE))}
              onKeyDown={closeOnEscape}
            >
              Previous
            </Button>
            <Button
              size="small"
              aria-label="More values"
              disabled={
                BigInt(offset + VOCABULARY_PAGE_SIZE) >=
                  (query.data?.values.total_values ?? 0n) || query.isFetching
              }
              onClick={() => setOffset(offset + VOCABULARY_PAGE_SIZE)}
              onKeyDown={closeOnEscape}
            >
              Next
            </Button>
          </Space>
        </div>
      )}
      {query.error && !closedChoices && (
        <Alert
          type="error"
          title="Values could not be loaded"
          description={query.error.message}
          action={
            <Button
              size="small"
              onClick={() => void query.refetch()}
              onKeyDown={closeOnEscape}
            >
              Retry values
            </Button>
          }
        />
      )}
    </section>
  );
  const button = (
    <Button
      ref={trigger}
      className="common-choice-trigger"
      block
      aria-label={"Edit " + field.label + " filter"}
      aria-haspopup="dialog"
      aria-expanded={open}
      aria-controls={open ? menuId : undefined}
      icon={<DownOutlined aria-hidden="true" />}
      iconPlacement="end"
      onClick={narrow ? () => setMenuOpen(true) : undefined}
      onKeyDown={closeOnEscape}
    >
      {selected.length ? "Edit choices" : "Any " + field.label.toLowerCase()}
    </Button>
  );
  return (
    <div className="common-choice-picker">
      {narrow ? (
        button
      ) : (
        <Popover
          content={menu}
          open={open}
          onOpenChange={setMenuOpen}
          afterOpenChange={(visible) => {
            if (visible && open && !narrow && initialFocusPending.current) {
              initialFocusPending.current = false;
              input.current?.focus();
            }
          }}
          placement="bottomLeft"
          trigger="click"
          arrow={false}
          destroyOnHidden
        >
          {button}
        </Popover>
      )}
      {narrow && (
        <Drawer
          title={field.label}
          open={open}
          placement="bottom"
          size="min(80vh, 540px)"
          onClose={() => setMenuOpen(false)}
          afterOpenChange={(visible) => {
            if (!visible) trigger.current?.focus();
          }}
        >
          {open && menu}
        </Drawer>
      )}
      {selected.length > 0 && (
        <div
          className="common-choice-selections"
          aria-label={"Selected " + field.label.toLowerCase()}
        >
          {selected.map((value) => {
            const state = choiceValueState(clause, value);
            const label =
              choices.find((choice) => choice.value === value)?.label ?? value;
            return (
              <Button
                key={value}
                size="small"
                color={state === "exclude" ? "danger" : "primary"}
                variant="filled"
                aria-label={`Remove ${state === "exclude" ? "excluded" : "included"} ${label} from ${field.label}`}
                onClick={() => change(value, "neutral")}
              >
                <span aria-hidden="true">
                  {state === "exclude" ? <MinusOutlined /> : <PlusOutlined />}
                </span>
                {label}
                <CloseOutlined aria-hidden="true" />
              </Button>
            );
          })}
        </div>
      )}
    </div>
  );
}
