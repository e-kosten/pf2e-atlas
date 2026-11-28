import { FilterEditorPopover } from "./FilterEditorPopover";
import { restoreOverlayFocus } from "../overlays/restoreOverlayFocus";
import { createPortal } from "react-dom";
import { AtlasDrawer as Drawer } from "../overlays/AtlasDrawer";
import {
  CheckOutlined,
  CloseCircleFilled,
  CloseOutlined,
  DownOutlined,
  MinusOutlined,
  PlusOutlined,
  RightOutlined,
} from "@ant-design/icons";
import { Button, Grid, Input, Tag, Typography, theme } from "antd";
import type { InputRef } from "antd";
import {
  Fragment,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { CSSProperties, KeyboardEvent, MouseEvent } from "react";
import { LoadingStatus } from "../LoadingStatus";

type ChoiceArrow = "ArrowDown" | "ArrowUp";

export type ChoiceState = "neutral" | "include" | "exclude";
export type ChoiceOption = {
  value: string;
  label: string;
  parent?: string;
  expanded?: boolean;
};
/** Shared include/exclude interaction; callers own vocabulary and filter semantics. */
export function ChoicePicker({
  label,
  inlineSelections = false,
  id,
  choices,
  visibleChoices,
  onToggleGroup,
  selected,
  applied = selected.length > 0,
  stateOf,
  onChange,
  open,
  onOpenChange,
  text,
  onTextChange,
  filterChoices = true,
  loading = false,
  loadingLabel,
  searchPending = false,
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
  inlineSelections?: boolean;
  id: string;
  choices: ChoiceOption[];
  visibleChoices?: ChoiceOption[];
  onToggleGroup?: (value: string) => void;
  selected: string[];
  applied?: boolean;
  stateOf: (value: string) => ChoiceState;
  onChange: (value: string, state: ChoiceState) => void | "close";
  open: boolean;
  onOpenChange: (open: boolean) => void;
  text: string;
  onTextChange: (text: string) => void;
  filterChoices?: boolean;
  loading?: boolean;
  loadingLabel?: string;
  searchPending?: boolean;
  children?: ReactNode;
  emptyMessage?: ReactNode;
  labelForValue?: (value: string) => string;
  counts?: ReadonlyMap<string, number>;
  valueEditing?: {
    triggerText: string;
    scalar?: boolean;
    enterValue?: string;
    manualValue?: string;
    onClear: () => void;
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
  const optionList = useRef<HTMLDivElement>(null);
  const pendingArrow = useRef<ChoiceArrow | null>(null);
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
  useLayoutEffect(() => {
    if (searchPending || !pendingArrow.current) return;
    const arrow = pendingArrow.current;
    pendingArrow.current = null;
    if (open && document.activeElement === input.current?.input) {
      const buttons =
        optionList.current?.querySelectorAll<HTMLButtonElement>(".common-choice-row");
      buttons?.[arrow === "ArrowDown" ? 0 : buttons.length - 1]?.focus();
    }
  }, [open, searchPending]);
  // Keep the server's search page authoritative for advanced operands.
  // Retained rows still support editing on the unsearched page.
  const retained =
    valueEditing && !filterChoices && text.trim()
      ? []
      : [...new Set([...sessionValues, ...selected])]
          .filter((value) => !choices.some((choice) => choice.value === value))
          .map((value) => ({ value, label: labelForValue(value) }));
  const options = [...choices, ...retained];
  const rows: ChoiceOption[] =
    visibleChoices ??
    (filterChoices
      ? options.filter((option) =>
          option.label.toLowerCase().includes(text.trim().toLowerCase()),
        )
      : options);
  function setMenuOpen(next: boolean) {
    if (!next && reading) {
      readOrigin.current = null;
      reading.onDismiss();
    }
    pendingArrow.current = null;
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
    if (event.key !== "Escape" || !open) return;
    if (reading) {
      event.stopPropagation();
      reading.onBack();
    } else if (!narrow) {
      event.stopPropagation();
      setMenuOpen(false);
      trigger.current?.focus();
    }
  }
  function moveChoiceFocus(event: KeyboardEvent<HTMLElement>) {
    const { key, target } = event;
    if (
      (key !== "ArrowDown" && key !== "ArrowUp") ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.shiftKey ||
      event.nativeEvent.isComposing ||
      event.keyCode === 229
    )
      return;

    const fromSearch = target === input.current?.input;
    const buttons = Array.from(
      optionList.current?.querySelectorAll<HTMLButtonElement>(".common-choice-row") ??
        [],
    );
    const index = buttons.findIndex((button) => button === target);
    if (!fromSearch && index < 0) return;

    event.preventDefault();
    event.stopPropagation();
    if (fromSearch) {
      // Results can still belong to the previous text during the debounce.
      if (searchPending) pendingArrow.current = key;
      else buttons[key === "ArrowDown" ? 0 : buttons.length - 1]?.focus();
    } else if (key === "ArrowUp" && index === 0) {
      input.current?.focus();
    } else {
      buttons[index + (key === "ArrowDown" ? 1 : -1)]?.focus();
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
        autoCorrect="off"
        value={text}
        allowClear
        onBlur={() => {
          pendingArrow.current = null;
        }}
        onChange={(event) => {
          pendingArrow.current = null;
          onTextChange(event.target.value);
        }}
        onPressEnter={(event) => {
          if (pendingArrow.current) {
            event.preventDefault();
            return;
          }
          if (!valueEditing?.enterValue || event.nativeEvent.isComposing) return;
          event.preventDefault();
          change(valueEditing.enterValue, "include");
        }}
        onKeyDown={closeOnEscape}
      />
      <div
        key={pageKey}
        ref={optionList}
        className="common-choice-options"
        aria-busy={loading}
      >
        {rows.map((option) => {
          const action = optionAction?.(option);
          const state = stateOf(option.value);
          const count = counts?.get(option.value);
          const dimmed = count === 0 && state === "neutral";
          const inherited = option.parent ? stateOf(option.parent) : "neutral";
          const inheritedSelection =
            !valueEditing && state === "neutral" && inherited !== "neutral";
          const applied = inheritedSelection ? inherited : state;
          const appliedLabel =
            applied === "neutral"
              ? "neutral"
              : applied === "include"
                ? "included"
                : "excluded";
          const stateLabel =
            inheritedSelection && option.parent
              ? `${appliedLabel} by ${labelForValue(option.parent)}`
              : appliedLabel;
          const next: ChoiceState = valueEditing
            ? state === "include"
              ? "neutral"
              : "include"
            : inheritedSelection && inherited === "include"
              ? "exclude"
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
                valueEditing
                  ? "default"
                  : state === "exclude"
                    ? "danger"
                    : state === "include"
                      ? "primary"
                      : "default"
              }
              variant="text"
              aria-pressed={valueEditing ? state === "include" : undefined}
              styles={{
                root: {
                  minWidth: 0,
                  flex: action ? 1 : undefined,
                  ...(valueEditing
                    ? {
                        minHeight: token.controlHeight,
                        containIntrinsicSize: `auto ${token.controlHeight}px`,
                        paddingInline: token.controlPaddingHorizontal,
                        borderRadius: token.borderRadiusSM,
                        fontWeight:
                          state === "include" ? token.fontWeightStrong : undefined,
                      }
                    : {}),
                  color: dimmed
                    ? token.colorTextDisabled
                    : valueEditing || state === "neutral"
                      ? undefined
                      : state === "include"
                        ? "var(--filter-include-text)"
                        : "var(--filter-exclude-text)",
                  backgroundColor:
                    state === "neutral"
                      ? undefined
                      : valueEditing
                        ? token.controlItemBgActive
                        : state === "include"
                          ? "var(--filter-include-bg)"
                          : "var(--filter-exclude-bg)",
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
                  : `${label}: ${option.label}, ${stateLabel}; activate to ${next === "neutral" ? "clear" : next}`
              }
              onClick={() => change(option.value, next)}
              onKeyDown={closeOnEscape}
            >
              <span className="common-choice-label">{option.label}</span>
              {count !== undefined && (
                <Typography.Text
                  type="secondary"
                  style={dimmed || valueEditing ? { color: "inherit" } : undefined}
                  aria-label={`${count} matching records`}
                >
                  {new Intl.NumberFormat().format(count)}
                </Typography.Text>
              )}
              {applied !== "neutral" && (
                <span className="common-choice-marker" aria-hidden="true">
                  {valueEditing ? (
                    <CheckOutlined style={{ color: token.colorPrimary }} />
                  ) : applied === "include" ? (
                    <PlusOutlined />
                  ) : (
                    <MinusOutlined />
                  )}
                </span>
              )}
            </Button>
          );
          return action || onToggleGroup ? (
            <div
              key={option.value}
              className="common-choice-option"
              data-child={!!option.parent || undefined}
            >
              {button}
              {option.expanded !== undefined && (
                <Button
                  type="text"
                  className="common-choice-disclosure"
                  icon={option.expanded ? <DownOutlined /> : <RightOutlined />}
                  aria-label={`${option.expanded ? "Collapse" : "Expand"} ${option.label}`}
                  aria-expanded={option.expanded}
                  onClick={() => onToggleGroup?.(option.value)}
                />
              )}
              {action && <span className="common-choice-option__action">{action}</span>}
            </div>
          ) : (
            <Fragment key={option.value}>{button}</Fragment>
          );
        })}
        {!rows.length && loading && loadingLabel && (
          <LoadingStatus label={loadingLabel} />
        )}
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
      onKeyDown={moveChoiceFocus}
      data-reference-picker={!!optionAction || undefined}
      data-reading={isReading || undefined}
    >
      <div
        inert={isReading || undefined}
        className="common-choice-view"
        data-hierarchical={!!onToggleGroup || undefined}
      >
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
  const openerProps = {
    ref: trigger,
    "aria-label": "Edit " + label + (valueEditing ? " values" : " filter"),
    "aria-haspopup": "dialog" as const,
    "aria-expanded": visible,
    "aria-controls": visible ? menuId : undefined,
    onClick: narrow ? () => setMenuOpen(true) : undefined,
    onKeyDown: valueEditing ? undefined : closeOnEscape,
  };
  const selectedLabel = (value: string) =>
    choices.find((choice) => choice.value === value)?.label ?? labelForValue(value);
  function removeSelection(event: MouseEvent<HTMLElement>, remove: () => void) {
    event.preventDefault();
    event.stopPropagation();
    restoreOverlayFocus(trigger.current, event.currentTarget);
    remove();
  }
  function removeValue(event: MouseEvent<HTMLElement>, value: string) {
    removeSelection(event, () => change(value, "neutral"));
  }
  const removeName = (value: string) =>
    `Remove ${inlineSelections ? (stateOf(value) === "exclude" ? "excluded" : "included") : "selected"} ${selectedLabel(value)} from ${label}`;
  const clearValue = valueEditing?.scalar ? selected[0] : undefined;
  const inField = Boolean(valueEditing) || inlineSelections;
  const triggerText = valueEditing?.triggerText ?? "Any";
  const hasClear = Boolean(valueEditing) && selected.length > 0;
  const showChips = inField && !valueEditing?.scalar && selected.length > 0;
  const anchor = inField ? (
    <div
      className="common-choice-operand"
      data-has-clear={hasClear || undefined}
      data-has-chips={showChips || undefined}
      onKeyDownCapture={closeOnEscape}
      style={
        {
          "--choice-field-border": token.colorBorder,
          "--choice-field-outline": token.controlOutline,
          "--choice-field-outline-width": token.controlOutlineWidth + "px",
          "--choice-field-placeholder": token.colorTextPlaceholder,
          "--choice-field-affix": token.colorTextQuaternary,
          "--choice-field-affix-hover": token.colorTextTertiary,
          "--choice-field-icon-size": token.fontSizeIcon + "px",
          "--choice-field-padding": token.controlPaddingHorizontal + "px",
          "--choice-field-motion": `${token.motionDurationSlow} ${token.motionEaseInOut}`,
        } as CSSProperties
      }
    >
      <button
        {...openerProps}
        type="button"
        className="common-choice-operand__opener"
        aria-describedby={menuId + "-selection"}
      >
        {!showChips && (
          <span
            className="common-choice-operand__value"
            title={clearValue !== undefined ? triggerText : undefined}
          >
            {triggerText}
          </span>
        )}
        <DownOutlined className="common-choice-operand__chevron" aria-hidden="true" />
      </button>
      <span id={menuId + "-selection"} className="sr-only">
        {clearValue !== undefined
          ? `Selected: ${selectedLabel(clearValue)}`
          : `${selected.length} selected`}
      </span>
      {showChips && (
        <div
          className="common-choice-operand__chips"
          aria-label={"Selected " + label.toLowerCase()}
        >
          {selected.map((value) => (
            <Tag
              key={value}
              className="common-choice-operand__chip"
              classNames={{
                close: "common-choice-operand__chip-close",
              }}
              variant="filled"
              styles={{
                root: {
                  background: token.colorFillSecondary,
                  marginInlineEnd: 0,
                  fontSize: token.fontSize,
                  borderRadius: token.borderRadiusSM,
                  paddingInlineStart: token.paddingXS,
                  paddingInlineEnd: 0,
                },
                close: { marginInlineStart: 0, fontSize: 10 },
              }}
              closeIcon={<CloseOutlined aria-label={removeName(value)} />}
              onClose={(event) => removeValue(event, value)}
            >
              {inlineSelections && (
                <span
                  className={
                    stateOf(value) === "exclude"
                      ? "applied-filter-chip__exclude"
                      : "applied-filter-chip__include"
                  }
                  aria-hidden="true"
                >
                  {stateOf(value) === "exclude" ? <MinusOutlined /> : <PlusOutlined />}
                </span>
              )}
              <span className="common-choice-operand__chip-label">
                {selectedLabel(value)}
              </span>
            </Tag>
          ))}
        </div>
      )}
      {hasClear && valueEditing && (
        <button
          type="button"
          className="common-choice-operand__clear"
          aria-label={
            clearValue === undefined
              ? "Clear all " + label + " values"
              : removeName(clearValue)
          }
          onClick={(event) => removeSelection(event, valueEditing.onClear)}
        >
          <CloseCircleFilled aria-hidden="true" />
        </button>
      )}
    </div>
  ) : (
    <Button
      {...openerProps}
      className="common-choice-trigger"
      color={applied ? "primary" : "default"}
      variant="outlined"
      aria-description={applied ? "Filter applied" : undefined}
      block
      icon={<DownOutlined aria-hidden="true" />}
      iconPlacement="end"
    >
      {label}
    </Button>
  );
  const selections =
    !inField && selected.length > 0 ? (
      <div
        className="common-choice-selections"
        aria-label={"Selected " + label.toLowerCase()}
      >
        {selected.map((value) => {
          const excluded = stateOf(value) === "exclude";
          const valueLabel = selectedLabel(value);
          return (
            <Button
              key={value}
              size="small"
              color="default"
              variant="filled"
              className="applied-filter-chip"
              aria-label={`Remove ${excluded ? "excluded" : "included"} ${valueLabel} from ${label}`}
              onClick={(event) => removeValue(event, value)}
            >
              <span
                className={
                  excluded
                    ? "applied-filter-chip__exclude"
                    : "applied-filter-chip__include"
                }
                aria-hidden="true"
              >
                {excluded ? <MinusOutlined /> : <PlusOutlined />}
              </span>
              {valueLabel}
              <CloseOutlined aria-hidden="true" />
            </Button>
          );
        })}
      </div>
    ) : null;
  return (
    <div className="common-choice-picker">
      {narrow ? (
        anchor
      ) : (
        <FilterEditorPopover
          content={menu}
          open={visible}
          onOpenChange={setMenuOpen}
          afterOpenChange={(shown) => {
            if (shown && open && initialFocusPending.current) {
              initialFocusPending.current = false;
              input.current?.focus();
            }
          }}
        >
          {anchor}
        </FilterEditorPopover>
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
      {selectionContainer === undefined
        ? selections
        : selectionContainer && createPortal(selections, selectionContainer)}
    </div>
  );
}
