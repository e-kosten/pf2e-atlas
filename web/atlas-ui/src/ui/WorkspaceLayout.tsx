import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

type PaneKey = "filter" | "results" | "detail";

type PaneState = {
  filter: number;
  results: number;
  detail: number;
};

type WorkspaceLayoutProps = {
  variant: "ant" | "mantine";
  filter: React.ReactNode;
  filterHeaderActions?: React.ReactNode;
  results: React.ReactNode;
  resultsHeaderActions?: React.ReactNode;
  detail: React.ReactNode;
  detailHeaderActions?: React.ReactNode;
  selectedRecordKey: string | null;
};

const COLLAPSED_WIDTH = 44;
const MIN_WIDTHS: PaneState = {
  filter: 240,
  results: 0,
  detail: 320,
};
const DEFAULT_WIDTHS: PaneState = {
  filter: 300,
  results: 560,
  detail: 420,
};

export function WorkspaceLayout({
  variant,
  filter,
  filterHeaderActions,
  results,
  resultsHeaderActions,
  detail,
  detailHeaderActions,
  selectedRecordKey,
}: WorkspaceLayoutProps) {
  const [widths, setWidths] = useState(DEFAULT_WIDTHS);
  const [collapsed, setCollapsed] = useState<Record<PaneKey, boolean>>({
    filter: false,
    results: false,
    detail: false,
  });
  const dragState = useRef<{
    handle: "filter-results" | "results-detail";
    startX: number;
    startWidths: PaneState;
  } | null>(null);

  useEffect(() => {
    if (!selectedRecordKey) {
      return;
    }
    setCollapsed((current) =>
      current.detail ? { ...current, detail: false } : current,
    );
  }, [selectedRecordKey]);

  const gridTemplateColumns = useMemo(
    () =>
      [
        paneColumn("filter", collapsed, widths),
        "var(--panel-gap)",
        paneColumn("results", collapsed, widths),
        "var(--panel-gap)",
        paneColumn("detail", collapsed, widths),
      ]
        .map((value) => (typeof value === "number" ? `${value}px` : value))
        .join(" "),
    [collapsed, widths],
  );

  function togglePane(pane: PaneKey) {
    setCollapsed((current) => ({ ...current, [pane]: !current[pane] }));
  }

  function beginResize(
    handle: "filter-results" | "results-detail",
    event: React.PointerEvent<HTMLDivElement>,
  ) {
    event.currentTarget.setPointerCapture(event.pointerId);
    dragState.current = {
      handle,
      startX: event.clientX,
      startWidths: widths,
    };
  }

  function resize(event: React.PointerEvent<HTMLDivElement>) {
    const drag = dragState.current;
    if (!drag) {
      return;
    }
    const delta = event.clientX - drag.startX;
    setWidths((current) => {
      if (drag.handle === "filter-results") {
        return {
          ...current,
          filter: clamp(drag.startWidths.filter + delta, MIN_WIDTHS.filter),
        };
      }
      return {
        ...current,
        detail: clamp(drag.startWidths.detail - delta, MIN_WIDTHS.detail),
      };
    });
  }

  function endResize(event: React.PointerEvent<HTMLDivElement>) {
    event.currentTarget.releasePointerCapture(event.pointerId);
    dragState.current = null;
  }

  return (
    <main
      className={`workspace-grid workspace-grid--${variant}`}
      style={{ gridTemplateColumns }}
    >
      <WorkspacePane
        collapsed={collapsed.filter}
        headerActions={filterHeaderActions}
        label="Filters"
        onToggle={() => togglePane("filter")}
      >
        {filter}
      </WorkspacePane>
      <ResizeHandle
        disabled={collapsed.filter && collapsed.results}
        label="Resize filters"
        onPointerDown={(event) => beginResize("filter-results", event)}
        onPointerMove={resize}
        onPointerUp={endResize}
      />
      <WorkspacePane
        collapsed={collapsed.results}
        headerActions={resultsHeaderActions}
        label="Results"
        onToggle={() => togglePane("results")}
      >
        {results}
      </WorkspacePane>
      <ResizeHandle
        disabled={collapsed.results && collapsed.detail}
        label="Resize results"
        onPointerDown={(event) => beginResize("results-detail", event)}
        onPointerMove={resize}
        onPointerUp={endResize}
      />
      <WorkspacePane
        collapsed={collapsed.detail}
        headerActions={detailHeaderActions}
        label="Detail"
        onToggle={() => togglePane("detail")}
      >
        {detail}
      </WorkspacePane>
    </main>
  );
}

function paneColumn(
  pane: PaneKey,
  collapsed: Record<PaneKey, boolean>,
  widths: PaneState,
): number | string {
  if (collapsed[pane]) {
    return COLLAPSED_WIDTH;
  }
  if (pane === "results") {
    return "minmax(0, 1fr)";
  }
  if (pane === "detail" && collapsed.results) {
    return "minmax(0, 1fr)";
  }
  if (pane === "filter" && collapsed.results && collapsed.detail) {
    return "minmax(0, 1fr)";
  }
  return `minmax(0, ${widths[pane]}px)`;
}

function WorkspacePane({
  children,
  collapsed,
  headerActions,
  label,
  onToggle,
}: {
  children: React.ReactNode;
  collapsed: boolean;
  headerActions?: React.ReactNode;
  label: string;
  onToggle: () => void;
}) {
  return (
    <div
      className={
        collapsed ? "workspace-pane workspace-pane--collapsed" : "workspace-pane"
      }
    >
      <div className="pane-header">
        <span>{label}</span>
        <div className="pane-header__actions">
          {!collapsed && headerActions}
          <button
            aria-label={collapsed ? `Expand ${label}` : `Collapse ${label}`}
            className="pane-toggle"
            onClick={onToggle}
            title={collapsed ? `Expand ${label}` : `Collapse ${label}`}
            type="button"
          >
            {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          </button>
        </div>
      </div>
      {collapsed ? (
        <button
          aria-label={`Expand ${label}`}
          className="pane-rail"
          onClick={onToggle}
          title={`Expand ${label}`}
          type="button"
        >
          <span>{label}</span>
        </button>
      ) : (
        children
      )}
    </div>
  );
}

function ResizeHandle({
  disabled,
  label,
  onPointerDown,
  onPointerMove,
  onPointerUp,
}: {
  disabled: boolean;
  label: string;
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
      onPointerDown={disabled ? undefined : onPointerDown}
      onPointerMove={disabled ? undefined : onPointerMove}
      onPointerUp={disabled ? undefined : onPointerUp}
      role="separator"
    />
  );
}

function clamp(value: number, min: number): number {
  return Math.max(min, Math.round(value));
}
