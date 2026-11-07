import { restoreOverlayFocus } from "../overlays/restoreOverlayFocus";
import { createPortal } from "react-dom";
import { AtlasDrawer as Drawer } from "../overlays/AtlasDrawer";
import {
  CloseOutlined,
  DownOutlined,
  MinusOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import { Button, Grid, Input, Popover, Typography, theme } from "antd";
import type { InputRef } from "antd";
import {
  Fragment,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
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
  valueEditing,
  pageKey,
  optionAction,
  reading,
  selectionContainer,
}: {
  label: string;
  id: string;
  choices: ChoiceOption[];
  selected: string[];
  stateOf: (value: string) => ChoiceState;
  onChange: (value: string, state: ChoiceState) => void | "close";
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
  valueEditing?: {
    triggerText: string;
    scalar?: boolean;
    enterValue?: string;
    manualValue?: string;
  };
  pageKey?: string;
  optionAction?: (option: ChoiceOption) => ReactNode;
  reading?: {
    content: ReactNode;
    origin: HTMLElement | null;
    onBack: () => void;
    onDismiss: () => void;
  };
  selectionContainer?: HTMLElement | null;
}) {
  const { token } = theme.useToken();
  const [sessionValues, setSessionValues] = useState<string[]>([]);
  const input = useRef<InputRef>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const initialFocusPending = useRef(false);
  const menuId = useId();
  const screens = Grid.useBreakpoint();
  const narrow = screens.xs === true && !screens.sm;
  const visible = open;
  const readingView = useRef<HTMLDivElement>(null);
  const readOrigin = useRef<HTMLElement | null>(null);
  const isReading = !!reading;
  const origin = reading?.origin;
  useLayoutEffect(() => {
    if (isReading) {
      readOrigin.current = origin ?? null;
      readingView.current?.focus({ preventScroll: true });
    } else if (readOrigin.current) {
      const origin = readOrigin.current;
      readOrigin.current = null;
      const restore = () =>
        (origin.isConnected ? origin : input.current)?.focus({ preventScroll: true });
      restore();
      if (narrow && document.activeElement !== origin) {
        const frame = requestAnimationFrame(restore);
        return () => cancelAnimationFrame(frame);
      }
    }
  }, [isReading, origin, narrow]);
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
    if (!next && reading) {
      readOrigin.current = null;
      reading.onDismiss();
    }
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
    if (onChange(value, state) === "close" && open) {
      setMenuOpen(false);
      if (!narrow) trigger.current?.focus();
    }
  }
  function closeOnEscape(event: KeyboardEvent<HTMLElement>) {
    if (event.key !== "Escape") return;
    if (reading) {
      event.stopPropagation();
      reading.onBack();
    } else if (!narrow) {
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
        onPressEnter={(event) => {
          if (!valueEditing?.enterValue || event.nativeEvent.isComposing) return;
          event.preventDefault();
          change(valueEditing.enterValue, "include");
        }}
        onKeyDown={closeOnEscape}
      />
      <div key={pageKey} className="common-choice-options" aria-busy={loading}>
        {rows.map((option) => {
          const state = stateOf(option.value);
          const count = counts?.get(option.value);
          const dimmed = count === 0n && state === "neutral";
          const next: ChoiceState = valueEditing
            ? state === "include"
              ? "neutral"
              : "include"
            : state === "neutral"
              ? "include"
              : state === "include"
                ? "exclude"
                : "neutral";
          const button = (
            <Button
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
                  minWidth: 0,
                  flex: optionAction ? 1 : undefined,
                  color:
                    state === "include"
                      ? "var(--filter-include-text)"
                      : state === "exclude"
                        ? "var(--filter-exclude-text)"
                        : dimmed
                          ? token.colorTextDisabled
                          : undefined,
                  backgroundColor:
                    state === "include"
                      ? "var(--filter-include-bg)"
                      : state === "exclude"
                        ? "var(--filter-exclude-bg)"
                        : undefined,
                },
              }}
              aria-label={
                valueEditing
                  ? label +
                    ": " +
                    option.label +
                    (state === "include"
                      ? ", selected; activate to remove"
                      : ", activate to select")
                  : `${label}: ${option.label}, ${state === "include" ? "included" : state === "exclude" ? "excluded" : "neutral"}; activate to ${next === "neutral" ? "clear" : next}`
              }
              onClick={() => change(option.value, next)}
              onKeyDown={closeOnEscape}
            >
              <span className="common-choice-label">{option.label}</span>
              {count !== undefined && (
                <Typography.Text
                  type="secondary"
                  style={dimmed ? { color: "inherit" } : undefined}
                  aria-label={`${count} matching records`}
                >
                  {new Intl.NumberFormat().format(count)}
                </Typography.Text>
              )}
              {state !== "neutral" && (
                <span className="common-choice-marker" aria-hidden="true">
                  {state === "include" ? <PlusOutlined /> : <MinusOutlined />}
                </span>
              )}
            </Button>
          );
          return optionAction ? (
            <div
              key={option.value}
              style={{ display: "flex", alignItems: "center", minWidth: 0 }}
            >
              {button}
              <span style={{ flexShrink: 0 }}>{optionAction(option)}</span>
            </div>
          ) : (
            <Fragment key={option.value}>{button}</Fragment>
          );
        })}
        {!rows.length && !loading && emptyMessage && (
          <Typography.Text type="secondary" role="status">
            {emptyMessage}
          </Typography.Text>
        )}
      </div>
      {valueEditing?.manualValue && (
        <Button
          size="small"
          onClick={() => change(valueEditing.manualValue!, "include")}
        >
          Use exact value: {valueEditing.manualValue}
        </Button>
      )}
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
  const menu = (
    <section
      {...menuProps}
      role={narrow ? "group" : "dialog"}
      onKeyDownCapture={closeOnEscape}
      data-reference-picker={!!optionAction || undefined}
      data-reading={isReading || undefined}
    >
      <div inert={isReading || undefined} className="common-choice-view">
        {menuContent}
      </div>
      {reading && (
        <div
          className="common-choice-reading"
          ref={readingView}
          tabIndex={-1}
          aria-label="Option reference"
        >
          {reading.content}
        </div>
      )}
    </section>
  );
  const button = (
    <Button
      ref={trigger}
      className="common-choice-trigger"
      type={valueEditing ? "text" : "default"}
      block={!valueEditing}
      aria-label={"Edit " + label + (valueEditing ? " values" : " filter")}
      aria-haspopup="dialog"
      aria-expanded={visible}
      aria-controls={visible ? menuId : undefined}
      data-affix-only={
        (valueEditing && !valueEditing.scalar && selected.length > 0) || undefined
      }
      icon={<DownOutlined aria-hidden="true" />}
      iconPlacement="end"
      styles={valueEditing ? { root: { height: token.controlHeight - 2 } } : undefined}
      onClick={narrow ? () => setMenuOpen(true) : undefined}
      onKeyDown={closeOnEscape}
    >
      {valueEditing && !valueEditing.scalar && selected.length
        ? null
        : (valueEditing?.triggerText ?? label)}
    </Button>
  );
  const selections =
    selected.length > 0 ? (
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
              variant={valueEditing?.scalar ? "text" : "filled"}
              className={
                valueEditing?.scalar
                  ? "common-choice-clear"
                  : valueEditing
                    ? undefined
                    : "applied-filter-chip"
              }
              aria-label={
                "Remove " +
                (valueEditing
                  ? "selected"
                  : state === "exclude"
                    ? "excluded"
                    : "included") +
                " " +
                valueLabel +
                " from " +
                label
              }
              onClick={(event) => {
                event.stopPropagation();
                restoreOverlayFocus(trigger.current, event.currentTarget);
                change(value, "neutral");
              }}
            >
              {!valueEditing && (
                <span
                  className={
                    state === "exclude"
                      ? "applied-filter-chip__exclude"
                      : "applied-filter-chip__include"
                  }
                  aria-hidden="true"
                >
                  {state === "exclude" ? <MinusOutlined /> : <PlusOutlined />}
                </span>
              )}
              {valueEditing?.scalar ? null : valueLabel}
              <CloseOutlined aria-hidden="true" />
            </Button>
          );
        })}
      </div>
    ) : null;
  const anchor = valueEditing ? (
    <div
      className="common-choice-operand"
      data-operand={valueEditing.scalar ? "scalar" : "list"}
    >
      {valueEditing.scalar ? button : selections}
      {valueEditing.scalar ? selections : button}
    </div>
  ) : (
    button
  );
  return (
    <div className="common-choice-picker">
      {narrow ? (
        anchor
      ) : (
        <Popover
          content={menu}
          open={visible}
          onOpenChange={setMenuOpen}
          afterOpenChange={(shown) => {
            if (shown && open && initialFocusPending.current) {
              initialFocusPending.current = false;
              input.current?.focus();
            }
          }}
          placement="bottom"
          trigger="click"
          arrow={false}
          destroyOnHidden
        >
          {anchor}
        </Popover>
      )}
      {narrow && (
        <Drawer
          title={label}
          open={visible}
          placement="bottom"
          size="min(80vh, 540px)"
          onClose={() => setMenuOpen(false)}
          keyboard={!isReading}
          focusable={{ focusTriggerAfterClose: false }}
          afterOpenChange={(shown) => {
            if (!shown) trigger.current?.focus();
          }}
        >
          {open && menu}
        </Drawer>
      )}
      {!valueEditing &&
        (selectionContainer === undefined
          ? selections
          : selectionContainer && createPortal(selections, selectionContainer))}
    </div>
  );
}
