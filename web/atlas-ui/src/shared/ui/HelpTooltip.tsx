import { Button, Popover, Typography } from "antd";
import { CircleHelp } from "lucide-react";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";

/** Clickable, keyboard-accessible help containing readable, noninteractive content. */
export function HelpTooltip({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const id = useId();
  useEffect(() => {
    if (!open) return;
    const owner = trigger.current?.ownerDocument;
    const dismiss = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      // Focus first: the focus trigger may reopen help; dismissal must win.
      trigger.current?.focus({ preventScroll: true });
      setOpen(false);
    };
    owner?.addEventListener("keydown", dismiss, true);
    return () => owner?.removeEventListener("keydown", dismiss, true);
  }, [open]);
  return (
    <Popover
      trigger={["hover", "focus", "click"]}
      placement="bottom"
      open={open}
      fresh
      destroyOnHidden
      onOpenChange={setOpen}
      styles={{ root: { maxWidth: "calc(100vw - 32px)" } }}
      content={
        <Typography.Paragraph
          id={id}
          style={{ margin: 0, maxWidth: "min(26rem, calc(100vw - 64px))" }}
        >
          {children}
        </Typography.Paragraph>
      }
    >
      <Button
        ref={trigger}
        type="text"
        size="small"
        icon={<CircleHelp size={16} aria-hidden="true" />}
        aria-label={label}
        aria-expanded={open}
        aria-controls={open ? id : undefined}
        onClick={(event) => {
          event.stopPropagation();
          trigger.current?.focus({ preventScroll: true });
          // Pointer activation focuses before clicking. Keep that focus-opened
          // explanation visible instead of accepting the click trigger's toggle.
          setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.stopPropagation();
          }
        }}
      />
    </Popover>
  );
}
