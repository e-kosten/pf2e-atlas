import { Select, type RefSelectProps, type SelectProps } from "antd";
import type { DefaultOptionType } from "antd/es/select";
import { useLayoutEffect, useRef, useState } from "react";

const preferredWidth = 320;
const gutter = 8;

export function FilterFieldSelect<Value, Option extends DefaultOptionType>({
  onOpenChange,
  ...props
}: SelectProps<Value, Option>) {
  const select = useRef<RefSelectProps>(null);
  const [open, setOpen] = useState(false);
  const [popup, setPopup] = useState({
    width: preferredWidth,
    placement: "bottomLeft" as "bottomLeft" | "bottomRight",
    listHeight: 420,
  });

  const expanded = props.open ?? open;
  useLayoutEffect(() => {
    const trigger = select.current?.nativeElement;
    if (!expanded || !trigger) return;
    const dialog = trigger.closest<HTMLElement>('[role="dialog"]');
    const viewport = window.visualViewport;
    const measure = () => {
      const anchor = trigger.getBoundingClientRect();
      const frame = dialog?.getBoundingClientRect();
      const viewportLeft = viewport?.offsetLeft ?? 0;
      const viewportRight = viewportLeft + (viewport?.width ?? window.innerWidth);
      const left = Math.max(viewportLeft, frame?.left ?? viewportLeft) + gutter;
      const right = Math.min(viewportRight, frame?.right ?? viewportRight) - gutter;
      const rightward = right - anchor.left;
      const leftward = anchor.right - left;
      const alignRight = rightward < preferredWidth && leftward > rightward;
      const width = Math.max(
        1,
        Math.floor(Math.min(preferredWidth, alignRight ? leftward : rightward)),
      );
      const placement = alignRight ? "bottomRight" : "bottomLeft";
      const listHeight = Math.min(
        420,
        Math.max(144, Math.floor((viewport?.height ?? window.innerHeight) * 0.56)),
      );
      setPopup((previous) =>
        previous.width === width &&
        previous.placement === placement &&
        previous.listHeight === listHeight
          ? previous
          : { width, placement, listHeight },
      );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(trigger);
    if (dialog) observer.observe(dialog);
    window.addEventListener("resize", measure);
    viewport?.addEventListener("resize", measure);
    viewport?.addEventListener("scroll", measure);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", measure);
      viewport?.removeEventListener("resize", measure);
      viewport?.removeEventListener("scroll", measure);
    };
  }, [expanded]);

  return (
    <Select<Value, Option>
      {...props}
      ref={select}
      placement={popup.placement}
      popupMatchSelectWidth={popup.width}
      listHeight={popup.listHeight}
      onOpenChange={(next) => {
        setOpen(next);
        onOpenChange?.(next);
      }}
    />
  );
}
