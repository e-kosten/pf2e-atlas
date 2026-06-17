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
  labels?: Partial<Record<PaneKey, string>>;
  sizing?: "results-focus" | "detail-focus";
  widthSpecs?: Partial<PaneState>;
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
  labels = {},
  sizing = "results-focus",
  selectedRecordKey,
  widthSpecs = {},
}: WorkspaceLayoutProps) {
  const effectiveWidthSpecs = {
    filter: widthSpecs.filter ?? WIDTH_SPECS.filter,
    results: widthSpecs.results ?? WIDTH_SPECS.results,
    detail: widthSpecs.detail ?? WIDTH_SPECS.detail,
  };
  const paneLabels = {
    filter: labels.filter ?? "Filters",
    results: labels.results ?? "Results",
    detail: labels.detail ?? "Detail",
  };
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
      widthSpecs={effectiveWidthSpecs}
      items={(widths) => [
        {
          kind: "pane",
          key: "filter",
          column: paneColumn("filter", effectiveCollapsed, widths, sizing),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.filter}
              headerActions={filterHeaderActions}
              label={paneLabels.filter}
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
          label: `Resize ${paneLabels.filter.toLowerCase()}`,
          resizePane: "filter",
          deltaMultiplier: 1,
        },
        {
          kind: "pane",
          key: "results",
          column: paneColumn("results", effectiveCollapsed, widths, sizing),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.results}
              headerActions={resultsHeaderActions}
              label={paneLabels.results}
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
          label: `Resize ${paneLabels.results.toLowerCase()}`,
          resizePane: sizing === "detail-focus" ? "results" : "detail",
          deltaMultiplier: sizing === "detail-focus" ? 1 : -1,
        },
        {
          kind: "pane",
          key: "detail",
          column: paneColumn("detail", effectiveCollapsed, widths, sizing),
          content: (
            <WorkspacePane
              collapsed={effectiveCollapsed.detail}
              headerActions={detailHeaderActions}
              label={paneLabels.detail}
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
  sizing: "results-focus" | "detail-focus",
): string {
  if (collapsed[pane]) {
    return `${COLLAPSED_WIDTH}px`;
  }
  if (sizing === "results-focus" && pane === "results") {
    return "minmax(0, 1fr)";
  }
  if (sizing === "detail-focus" && pane === "detail") {
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
