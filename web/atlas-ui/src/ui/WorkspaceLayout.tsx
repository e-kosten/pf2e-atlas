import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useMemo, useState } from "react";
import { PaneFrame, ResizablePaneGroup } from "./PaneLayout";

type PaneKey = "filter" | "results" | "detail";

type PaneState = {
  filter: { defaultWidth: number; minWidth: number };
  results: { defaultWidth: number; minWidth: number };
  detail: { defaultWidth: number; minWidth: number };
};

type WorkspaceLayoutProps = {
  filter: React.ReactNode;
  filterHeaderActions?: React.ReactNode;
  results: React.ReactNode;
  resultsHeaderActions?: React.ReactNode;
  detail: React.ReactNode;
  detailHeaderActions?: React.ReactNode;
  selectedRecordKey: string | null;
};

const COLLAPSED_WIDTH = 44;
const WIDTH_SPECS: PaneState = {
  filter: { defaultWidth: 300, minWidth: 240 },
  results: { defaultWidth: 560, minWidth: 0 },
  detail: { defaultWidth: 420, minWidth: 320 },
};

export function WorkspaceLayout({
  filter,
  filterHeaderActions,
  results,
  resultsHeaderActions,
  detail,
  detailHeaderActions,
  selectedRecordKey,
}: WorkspaceLayoutProps) {
  const [collapsed, setCollapsed] = useState<Record<PaneKey, boolean>>({
    filter: false,
    results: false,
    detail: false,
  });
  const [detailCollapsedForRecordKey, setDetailCollapsedForRecordKey] = useState<
    string | null
  >(null);

  const effectiveCollapsed = useMemo(
    () => ({
      ...collapsed,
      detail:
        collapsed.detail &&
        detailCollapsedForRecordKey !== null &&
        detailCollapsedForRecordKey === selectedRecordKey,
    }),
    [collapsed, detailCollapsedForRecordKey, selectedRecordKey],
  );

  function togglePane(pane: PaneKey) {
    if (pane === "detail") {
      const nextDetailCollapsed = !effectiveCollapsed.detail;
      setDetailCollapsedForRecordKey(nextDetailCollapsed ? selectedRecordKey : null);
      setCollapsed((current) => ({ ...current, detail: nextDetailCollapsed }));
      return;
    }
    setCollapsed((current) => ({ ...current, [pane]: !current[pane] }));
  }

  return (
    <ResizablePaneGroup
      className="workspace-grid"
      widthSpecs={WIDTH_SPECS}
      items={(widths) => [
        {
          kind: "pane",
          key: "filter",
          column: paneColumn("filter", effectiveCollapsed, widths),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.filter}
              headerActions={filterHeaderActions}
              label="Filters"
              onToggle={() => togglePane("filter")}
            >
              {filter}
            </WorkspacePane>
          ),
        },
        {
          kind: "handle",
          key: "filter-results",
          disabled: effectiveCollapsed.filter && effectiveCollapsed.results,
          label: "Resize filters",
          resizePane: "filter",
          deltaMultiplier: 1,
        },
        {
          kind: "pane",
          key: "results",
          column: paneColumn("results", effectiveCollapsed, widths),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.results}
              headerActions={resultsHeaderActions}
              label="Results"
              onToggle={() => togglePane("results")}
            >
              {results}
            </WorkspacePane>
          ),
        },
        {
          kind: "handle",
          key: "results-detail",
          disabled: effectiveCollapsed.results && effectiveCollapsed.detail,
          label: "Resize results",
          resizePane: "detail",
          deltaMultiplier: -1,
        },
        {
          kind: "pane",
          key: "detail",
          column: paneColumn("detail", effectiveCollapsed, widths),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.detail}
              headerActions={detailHeaderActions}
              label="Detail"
              onToggle={() => togglePane("detail")}
            >
              {detail}
            </WorkspacePane>
          ),
        },
      ]}
    />
  );
}

function paneColumn(
  pane: PaneKey,
  collapsed: Record<PaneKey, boolean>,
  widths: Record<string, number>,
): string {
  if (collapsed[pane]) {
    return `${COLLAPSED_WIDTH}px`;
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
    <PaneFrame
      className={
        collapsed ? "workspace-pane workspace-pane--collapsed" : "workspace-pane"
      }
      headerActions={
        <>
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
        </>
      }
      label={label}
    >
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
    </PaneFrame>
  );
}
