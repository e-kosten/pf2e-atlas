import { Button, Popover, Tag } from "antd";
import { useLayoutEffect, useRef, useState, type CSSProperties } from "react";

// Ant Tag transitions all properties by default. Layout-only measurement nodes
// must hide immediately when removed from flow, preserving native colors/state.
const measuredStyle: CSSProperties = { transitionProperty: "none" };
const hiddenMeasuredStyle: CSSProperties = {
  ...measuredStyle,
  position: "absolute",
  left: 0,
  top: 0,
  visibility: "hidden",
};

/** Whole-chip metadata with one measured owner for index and detail density. */
export function ListTagSummary({
  tags,
  label = "List tags",
  lines = 1,
}: {
  tags: string[];
  label?: string;
  lines?: 1 | 2;
}) {
  const summary = useRef<HTMLDivElement>(null);
  const action = useRef<HTMLButtonElement>(null);
  const [visible, setVisible] = useState(tags.length);
  const [open, setOpen] = useState(false);
  useLayoutEffect(() => {
    const node = summary.current;
    if (!node) return;
    const chips = Array.from(node.querySelectorAll<HTMLElement>("[data-list-tag]"));
    const measure = () => {
      const available = node.getBoundingClientRect().width;
      if (!available) return; // Hidden panes have no usable width yet.
      const gap = Number.parseFloat(window.getComputedStyle(node).columnGap) || 0;
      const widths = chips.map((chip) => chip.getBoundingClientRect().width);
      if (fitsRows(widths, available, gap, lines)) {
        setVisible(tags.length);
        setOpen(false);
        return;
      }
      const controlWidth = action.current?.getBoundingClientRect().width ?? 0;
      for (let count = tags.length - 1; count >= 0; count--) {
        if (
          fitsRows([...widths.slice(0, count), controlWidth], available, gap, lines)
        ) {
          setVisible(count);
          return;
        }
      }
      setVisible(0);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(node);
    for (const chip of chips) observer.observe(chip);
    if (action.current) observer.observe(action.current);
    return () => observer.disconnect();
  }, [tags, lines, visible]);
  if (!tags.length) return null;
  const hidden = tags.length - visible;
  return (
    <div className="list-tag-summary" aria-label={label} ref={summary}>
      {tags.map((tag, index) => (
        <Tag
          key={tag}
          data-list-tag
          aria-hidden={index >= visible || undefined}
          style={index < visible ? measuredStyle : hiddenMeasuredStyle}
        >
          {tag}
        </Tag>
      ))}
      <Popover
        open={hidden > 0 && open}
        onOpenChange={setOpen}
        trigger="click"
        placement="bottom"
        content={
          <div
            className="list-tag-summary__all"
            aria-label={`All ${label.toLowerCase()}`}
            onClickCapture={(event) => event.stopPropagation()}
          >
            {tags.map((tag) => (
              <Tag key={tag}>{tag}</Tag>
            ))}
          </div>
        }
      >
        <Button
          ref={action}
          size="small"
          type="link"
          aria-hidden={!hidden || undefined}
          tabIndex={hidden ? undefined : -1}
          style={hidden ? measuredStyle : hiddenMeasuredStyle}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.stopPropagation();
              setOpen(false);
            }
          }}
        >
          +{hidden || tags.length} more
        </Button>
      </Popover>
    </div>
  );
}

function fitsRows(
  widths: number[],
  available: number,
  gap: number,
  lines: number,
): boolean {
  let row = 1;
  let occupied = 0;
  for (const width of widths) {
    if (width > available + 0.5) return false;
    if (occupied > 0 && occupied + gap + width > available + 0.5) {
      row++;
      occupied = width;
    } else occupied += (occupied > 0 ? gap : 0) + width;
    if (row > lines) return false;
  }
  return true;
}
