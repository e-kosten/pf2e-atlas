import { Button, Popover, theme } from "antd";
import { X } from "lucide-react";
import { useId, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { restoreOverlayFocus } from "./overlays/restoreOverlayFocus";

/** An explanation of readable content; children must not contain other controls. */
export function ValueExplanation({
  children,
  label,
  content,
  title,
}: {
  children: ReactNode;
  label: string;
  content: ReactNode;
  title?: ReactNode;
}) {
  const { token } = theme.useToken();
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const popupId = useId();
  const focusAtOpen = useRef<Element | null>(null);
  const close = () => {
    restoreOverlayFocus(trigger.current, popup.current);
    setOpen(false);
  };
  const dismissEscape = (event: React.KeyboardEvent) => {
    if (event.key !== "Escape" || !open) return;
    event.preventDefault();
    event.stopPropagation();
    close();
  };
  return (
    <Popover
      open={open}
      trigger="click"
      fresh
      destroyOnHidden
      onOpenChange={(next) => {
        if (next) {
          focusAtOpen.current = trigger.current?.ownerDocument.activeElement ?? null;
          setOpen(true);
        } else close();
      }}
      afterOpenChange={(next) => {
        // WebKit may leave focus on the previous control after a pointer click.
        // Respect any deliberate focus move made while the opening motion runs.
        if (next && open && document.activeElement === focusAtOpen.current)
          popup.current?.focus({ preventScroll: true });
      }}
      content={
        // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- The focusable nonmodal dialog consumes Escape before its surrounding Drawer.
        <div
          ref={popup}
          id={popupId}
          className="value-explanation"
          role="dialog"
          aria-label={label}
          tabIndex={-1}
          onKeyDown={dismissEscape}
        >
          <div>
            {title && <strong className="value-explanation__title">{title}</strong>}
            {content}
          </div>
          <Button
            type="text"
            size="small"
            aria-label="Close explanation"
            icon={<X size={14} aria-hidden="true" />}
            onClick={close}
          />
        </div>
      }
    >
      <Button
        ref={trigger}
        type="text"
        size="small"
        className="explained-value"
        aria-haspopup="dialog"
        aria-labelledby={`${popupId}-context ${popupId}-value`}
        aria-expanded={open}
        aria-controls={open ? popupId : undefined}
        onKeyDown={dismissEscape}
        style={{ "--explanation-hover": token.colorLinkHover } as CSSProperties}
        styles={{
          root: {
            display: "inline",
            height: "auto",
            maxWidth: "100%",
            padding: 0,
            borderWidth: 0,
            color: "inherit",
            background: "transparent",
            font: "inherit",
            whiteSpace: "normal",
            textAlign: "start",
          },
        }}
      >
        <span id={`${popupId}-context`} className="sr-only">
          {label}:
        </span>
        <span id={`${popupId}-value`} className="explained-value__label">
          {children}
        </span>
      </Button>
    </Popover>
  );
}
