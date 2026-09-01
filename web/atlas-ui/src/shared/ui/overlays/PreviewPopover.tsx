import { Popover } from "antd";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import type React from "react";

export type PreviewPopoverTrigger = (open: boolean) => React.ReactElement;

const PreviewPopoverCloseContext = createContext<(() => void) | null>(null);

export function usePreviewPopoverClose() {
  const close = useContext(PreviewPopoverCloseContext);
  if (!close) {
    throw new Error(
      "usePreviewPopoverClose must be used inside PreviewPopover actions",
    );
  }
  return close;
}

export function PreviewPopover({
  actions,
  ariaLabel,
  children,
  content,
  onOpenChange,
  title,
}: {
  actions?: React.ReactNode;
  ariaLabel: string;
  children: PreviewPopoverTrigger;
  content: React.ReactNode;
  onOpenChange?: (open: boolean) => void;
  title: React.ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const triggerElement = useRef<HTMLElement | null>(null);
  const restoreFocusTimer = useRef<number | null>(null);

  const restoreTriggerFocus = useCallback(() => {
    if (restoreFocusTimer.current !== null) {
      window.clearTimeout(restoreFocusTimer.current);
    }
    restoreFocusTimer.current = window.setTimeout(() => {
      triggerElement.current?.focus();
      restoreFocusTimer.current = null;
    });
  }, []);

  const changeOpen = useCallback(
    (nextOpen: boolean, restoreFocus = false) => {
      setOpen(nextOpen);
      onOpenChange?.(nextOpen);
      if (!nextOpen && restoreFocus) {
        restoreTriggerFocus();
      }
    },
    [onOpenChange, restoreTriggerFocus],
  );

  const close = useCallback(() => changeOpen(false, true), [changeOpen]);

  useEffect(() => {
    if (!open) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      close();
    };
    document.addEventListener("keydown", closeOnEscape, true);
    return () => document.removeEventListener("keydown", closeOnEscape, true);
  }, [close, open]);

  useEffect(
    () => () => {
      if (restoreFocusTimer.current !== null) {
        window.clearTimeout(restoreFocusTimer.current);
      }
    },
    [],
  );

  const rememberTrigger = (event: React.SyntheticEvent<HTMLElement>) => {
    if (event.target instanceof HTMLElement) {
      triggerElement.current = event.target;
    }
  };

  return (
    <Popover
      arrow
      autoAdjustOverflow
      classNames={{ root: "preview-popover" }}
      content={
        <section
          aria-label={ariaLabel}
          className="preview-popover__content"
          role="dialog"
        >
          {content}
        </section>
      }
      destroyOnHidden
      onOpenChange={(nextOpen) => changeOpen(nextOpen)}
      open={open}
      placement="rightTop"
      title={
        <PreviewPopoverCloseContext.Provider value={close}>
          <header className="preview-popover__header">
            <span>{title}</span>
            {actions}
          </header>
        </PreviewPopoverCloseContext.Provider>
      }
      trigger="click"
    >
      <span
        className="preview-popover__trigger"
        onClickCapture={(event) => {
          event.preventDefault();
          rememberTrigger(event);
        }}
        onFocusCapture={rememberTrigger}
        onPointerDownCapture={rememberTrigger}
      >
        {children(open)}
      </span>
    </Popover>
  );
}
