import {
  CloseOutlined,
  DownOutlined,
  MinusOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import { Button, Drawer, Grid, Input, Popover, Typography, theme } from "antd";
import type { InputRef } from "antd";
import { useId, useRef, useState, type ReactNode } from "react";
import type { KeyboardEvent } from "react";
export type ChoiceState = "neutral" | "include" | "exclude";
export type ChoiceOption = { value: string; label: string };

/** Shared include/exclude interaction; callers own vocabulary and filter semantics. */
export function ChoicePicker({
  label,
  id,
  choices,
  selected,
  stateOf,
  onChange,
  open,
  onOpenChange,
  text,
  onTextChange,
  filterChoices = true,
  loading = false,
  children,
  emptyMessage = "No matching values",
  labelForValue = (value) => value,
  counts,
}: {
  label: string;
  id: string;
  choices: ChoiceOption[];
  selected: string[];
  stateOf: (value: string) => ChoiceState;
  onChange: (value: string, state: ChoiceState) => void;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  text: string;
  onTextChange: (text: string) => void;
  filterChoices?: boolean;
  loading?: boolean;
  children?: ReactNode;
  emptyMessage?: ReactNode;
  labelForValue?: (value: string) => string;
  counts?: ReadonlyMap<string, bigint>;
}) {
  const { token } = theme.useToken();
  const [sessionValues, setSessionValues] = useState<string[]>([]);
  const input = useRef<InputRef>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const initialFocusPending = useRef(false);
  const menuId = useId();
  const screens = Grid.useBreakpoint();
  const narrow = screens.xs === true && !screens.sm;
  // Retain initial selections and every edited value through this open session.
  // Suggestions always keep server/catalog order, irrespective of selection.
  const options = [
    ...choices,
    ...[...new Set([...sessionValues, ...selected])]
      .filter((value) => !choices.some((choice) => choice.value === value))
      .map((value) => ({ value, label: labelForValue(value) })),
  ];
  const rows = filterChoices
    ? options.filter((option) =>
        option.label.toLowerCase().includes(text.trim().toLowerCase()),
      )
    : options;
  function setMenuOpen(next: boolean) {
    initialFocusPending.current = next;
    if (next) setSessionValues(selected);
    else {
      setSessionValues([]);
      onTextChange("");
    }
    onOpenChange(next);
  }
  function cancelInitialFocus() {
    initialFocusPending.current = false;
  }
  function change(value: string, state: ChoiceState) {
    if (open)
      setSessionValues((current) =>
        current.includes(value) ? current : [...current, value],
      );
    onChange(value, state);
  }
  function closeOnEscape(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && !narrow) {
      event.stopPropagation();
      setMenuOpen(false);
      trigger.current?.focus();
    }
  }
  const menuContent = (
    <>
      <Input
        ref={input}
        aria-label={"Search " + label.toLowerCase() + " choices"}
        name={id + "-choices"}
        placeholder={"Search " + label.toLowerCase() + "…"}
        autoComplete="off"
        value={text}
        allowClear
        onChange={(event) => {
          onTextChange(event.target.value);
        }}
        onKeyDown={closeOnEscape}
      />
      <div className="common-choice-options" aria-busy={loading}>
        {rows.map((option) => {
          const state = stateOf(option.value);
          const unavailable = counts?.get(option.value) === 0n;
          const next: ChoiceState =
            state === "neutral"
              ? unavailable
                ? "exclude"
                : "include"
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
                  color:
                    state === "include"
                      ? "var(--filter-include-text)"
                      : state === "exclude"
                        ? "var(--filter-exclude-text)"
                        : unavailable
                          ? token.colorTextDisabled
                          : undefined,
                  background:
                    state === "include"
                      ? "var(--filter-include-bg)"
                      : state === "exclude"
                        ? "var(--filter-exclude-bg)"
                        : undefined,
                },
              }}
              aria-label={`${label}: ${option.label}, ${state === "include" ? "included" : state === "exclude" ? "excluded" : "neutral"}; activate to ${next === "neutral" ? "clear" : next}`}
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
              {counts?.has(option.value) && (
                <Typography.Text
                  type="secondary"
                  aria-label={`${counts.get(option.value)} matching records`}
                >
                  {new Intl.NumberFormat().format(counts.get(option.value)!)}
                </Typography.Text>
              )}
            </Button>
          );
        })}
        {!rows.length && !loading && emptyMessage && (
          <Typography.Text type="secondary" role="status">
            {emptyMessage}
          </Typography.Text>
        )}
      </div>
      {children}
    </>
  );
  const menuProps = {
    id: menuId,
    "aria-label": label + " choices",
    className: "common-choice-menu",
    onFocusCapture: cancelInitialFocus,
    onPointerDownCapture: cancelInitialFocus,
  };
  // The mobile Drawer owns Escape; the desktop dialog delegates it from all
  // descendants, including the caller's vocabulary paging and retry actions.
  const menu = narrow ? (
    <section {...menuProps} role="group">
      {menuContent}
    </section>
  ) : (
    <section {...menuProps} role="dialog" onKeyDownCapture={closeOnEscape}>
      {menuContent}
    </section>
  );
  const button = (
    <Button
      ref={trigger}
      className="common-choice-trigger"
      block
      aria-label={"Edit " + label + " filter"}
      aria-haspopup="dialog"
      aria-expanded={open}
      aria-controls={open ? menuId : undefined}
      icon={<DownOutlined aria-hidden="true" />}
      iconPlacement="end"
      onClick={narrow ? () => setMenuOpen(true) : undefined}
      onKeyDown={closeOnEscape}
    >
      {selected.length ? label : "Any " + label.toLowerCase()}
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
          placement="bottom"
          trigger="click"
          arrow={false}
          destroyOnHidden
        >
          {button}
        </Popover>
      )}
      {narrow && (
        <Drawer
          title={label}
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
          aria-label={"Selected " + label.toLowerCase()}
        >
          {selected.map((value) => {
            const state = stateOf(value);
            const valueLabel =
              choices.find((choice) => choice.value === value)?.label ??
              labelForValue(value);
            return (
              <Button
                key={value}
                size="small"
                color="default"
                variant="filled"
                styles={{
                  root: {
                    color:
                      state === "exclude"
                        ? "var(--filter-exclude-text)"
                        : "var(--filter-include-text)",
                    background:
                      state === "exclude"
                        ? "var(--filter-exclude-bg)"
                        : "var(--filter-include-bg)",
                    borderColor:
                      state === "exclude"
                        ? "var(--filter-exclude-border)"
                        : "var(--filter-include-border)",
                  },
                }}
                aria-label={`Remove ${state === "exclude" ? "excluded" : "included"} ${valueLabel} from ${label}`}
                onClick={() => change(value, "neutral")}
              >
                <span aria-hidden="true">
                  {state === "exclude" ? <MinusOutlined /> : <PlusOutlined />}
                </span>
                {valueLabel}
                <CloseOutlined aria-hidden="true" />
              </Button>
            );
          })}
        </div>
      )}
    </div>
  );
}
