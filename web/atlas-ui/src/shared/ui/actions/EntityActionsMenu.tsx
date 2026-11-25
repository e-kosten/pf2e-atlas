import { Button, Dropdown, type ButtonProps, type MenuProps } from "antd";
import { ChevronDown, Ellipsis } from "lucide-react";
import { useLayoutEffect, useState, type KeyboardEvent, type RefObject } from "react";
import { useDesktopTab } from "../../../app/DesktopTabContext";

const buttonTypes = {
  overflow: "text",
  outlined: "default",
  primary: "primary",
} as const;

export function EntityActionsMenu({
  label,
  trigger,
  items,
  onAction,
  disabled,
  loading,
  size,
  appearance = "overflow",
}: {
  label: string;
  trigger: RefObject<HTMLButtonElement | null>;
  items: MenuProps["items"];
  onAction: (key: string, opener: HTMLButtonElement) => void;
  disabled?: boolean;
  loading?: boolean;
  size?: ButtonProps["size"];
  appearance?: keyof typeof buttonTypes;
}) {
  const tab = useDesktopTab();
  const activation = tab?.activation ?? 0;
  const [openedAt, setOpenedAt] = useState<number>();
  const [popup, setPopup] = useState<HTMLDivElement | null>(null);
  const open = tab?.active !== false && openedAt === activation;
  useLayoutEffect(() => {
    if (!popup || !open) return;
    function focusFirstCommand() {
      if (popup?.ownerDocument.activeElement !== trigger.current) return;
      popup
        ?.querySelector<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])')
        ?.focus({ preventScroll: true });
    }
    // Wait for the portaled menu to be laid out, including when it reopens
    // before Ant has finished releasing the previous popup.
    const observer = new ResizeObserver(focusFirstCommand);
    observer.observe(popup);
    focusFirstCommand();
    return () => observer.disconnect();
  }, [open, popup, trigger]);

  function dismissByKey(event: KeyboardEvent<HTMLElement>) {
    if (!open || (event.key !== "Escape" && event.key !== "Tab")) return;
    // A closing popup stays mounted during its exit. Move focus before Ant's
    // window listener or the browser can send it back into that popup.
    event.stopPropagation();
    if (event.key === "Escape") event.preventDefault();
    trigger.current?.focus({ preventScroll: true });
    setOpenedAt(undefined);
  }

  return (
    <Dropdown
      destroyOnHidden
      // Release the popup's Escape handler before a command opens another overlay.
      transitionName=""
      popupRender={(menu) => <div ref={setPopup}>{menu}</div>}
      open={open}
      disabled={disabled}
      placement="bottomRight"
      trigger={["click"]}
      onOpenChange={(visible) => setOpenedAt(visible ? activation : undefined)}
      menu={{
        "aria-label": label,
        items,
        onKeyDown: dismissByKey,
        onClick: ({ key, domEvent }) => {
          // Ant activates menu items on keydown. Cancel native activation before
          // focus moves to the button in a newly opened dialog.
          domEvent.preventDefault();
          const opener = trigger.current;
          if (!opener) return;
          setOpenedAt(undefined);
          opener.focus({ preventScroll: true });
          onAction(key, opener);
        },
      }}
    >
      <Button
        ref={trigger}
        size={size}
        type={buttonTypes[appearance]}
        disabled={disabled}
        aria-label={label}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-busy={loading}
        onKeyDown={dismissByKey}
        loading={loading ? { delay: 200 } : false}
        icon={
          appearance !== "primary" ? (
            <Ellipsis size={18} aria-hidden="true" />
          ) : undefined
        }
      >
        {appearance === "primary" && (
          <>
            {label} <ChevronDown size={14} aria-hidden="true" />
          </>
        )}
      </Button>
    </Dropdown>
  );
}
