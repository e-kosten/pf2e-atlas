import type React from "react";
import { Fragment, useRef, useState } from "react";

type PaneWidthSpec = {
  defaultWidth: number;
  minWidth: number;
};

type PaneWidths<PaneId extends string> = Record<PaneId, number>;

type ResizablePaneItem<PaneId extends string> =
  | {
      kind: "pane";
      key: string;
      column: string;
      content: React.ReactNode;
    }
  | {
      kind: "handle";
      key: string;
      disabled?: boolean;
      label: string;
      resizePane: PaneId;
      deltaMultiplier: 1 | -1;
    };

export function ResizablePaneGroup<PaneId extends string>({
  className,
  items,
  widthSpecs,
}: {
  className: string;
  items: (widths: PaneWidths<PaneId>) => ResizablePaneItem<PaneId>[];
  widthSpecs: Record<PaneId, PaneWidthSpec>;
}) {
  const [widths, setWidths] = useState<PaneWidths<PaneId>>(
    () =>
      Object.fromEntries(
        (Object.keys(widthSpecs) as PaneId[]).map((key) => [
          key,
          widthSpecs[key].defaultWidth,
        ]),
      ) as PaneWidths<PaneId>,
  );
  const dragState = useRef<{
    deltaMultiplier: 1 | -1;
    resizePane: PaneId;
    startWidth: number;
    startX: number;
  } | null>(null);
  const resolvedItems = items(widths);
  const gridTemplateColumns = resolvedItems
    .map((item) => (item.kind === "pane" ? item.column : "var(--panel-gap)"))
    .join(" ");

  function beginResize(
    item: Extract<ResizablePaneItem<PaneId>, { kind: "handle" }>,
    event: React.PointerEvent<HTMLDivElement>,
  ) {
    const spec = widthSpecs[item.resizePane];
    event.currentTarget.setPointerCapture(event.pointerId);
    dragState.current = {
      deltaMultiplier: item.deltaMultiplier,
      resizePane: item.resizePane,
      startWidth: widths[item.resizePane] ?? spec.defaultWidth,
      startX: event.clientX,
    };
  }

  function resize(event: React.PointerEvent<HTMLDivElement>) {
    const drag = dragState.current;
    if (!drag) {
      return;
    }
    const spec = widthSpecs[drag.resizePane];
    const delta = (event.clientX - drag.startX) * drag.deltaMultiplier;
    setWidths((current) => ({
      ...current,
      [drag.resizePane]: clampPaneWidth(drag.startWidth + delta, spec.minWidth),
    }));
  }

  function endResize(event: React.PointerEvent<HTMLDivElement>, releaseCapture = true) {
    if (releaseCapture && event.currentTarget.hasPointerCapture?.(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    dragState.current = null;
  }

  return (
    <main className={className} style={{ gridTemplateColumns }}>
      {resolvedItems.map((item) =>
        item.kind === "pane" ? (
          <Fragment key={item.key}>{item.content}</Fragment>
        ) : (
          <ResizeHandle
            key={item.key}
            disabled={item.disabled ?? false}
            label={item.label}
            onPointerDown={(event) => beginResize(item, event)}
            onPointerCancel={endResize}
            onLostPointerCapture={(event) => endResize(event, false)}
            onPointerMove={resize}
            onPointerUp={endResize}
          />
        ),
      )}
    </main>
  );
}

export function PaneFrame({
  children,
  className,
  headerActions,
  label,
}: {
  children: React.ReactNode;
  className?: string;
  headerActions?: React.ReactNode;
  label: string;
}) {
  return (
    <section className={className ?? "workspace-pane"}>
      <div className="pane-header">
        <span>{label}</span>
        {headerActions && <div className="pane-header__actions">{headerActions}</div>}
      </div>
      {children}
    </section>
  );
}

function ResizeHandle({
  disabled,
  label,
  onLostPointerCapture,
  onPointerCancel,
  onPointerDown,
  onPointerMove,
  onPointerUp,
}: {
  disabled: boolean;
  label: string;
  onLostPointerCapture: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerCancel: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerDown: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerMove: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerUp: (event: React.PointerEvent<HTMLDivElement>) => void;
}) {
  return (
    <div
      aria-disabled={disabled}
      aria-label={label}
      aria-orientation="vertical"
      className="pane-resizer"
      onLostPointerCapture={disabled ? undefined : onLostPointerCapture}
      onPointerCancel={disabled ? undefined : onPointerCancel}
      onPointerDown={disabled ? undefined : onPointerDown}
      onPointerMove={disabled ? undefined : onPointerMove}
      onPointerUp={disabled ? undefined : onPointerUp}
      role="separator"
    />
  );
}

function clampPaneWidth(value: number, min: number): number {
  return Math.max(min, Math.round(value));
}
