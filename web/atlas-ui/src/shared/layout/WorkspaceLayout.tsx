import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { Button, Drawer, Space } from "antd";
import { useEffect, useMemo, useState } from "react";
import { PaneIconButton } from "../ui/actions/PaneAction";
import { PaneFrame, ResizablePaneGroup } from "./PaneLayout";
import type { ResizablePaneItem } from "./PaneLayout";

type PaneKey = "filter" | "results" | "detail";

type PaneState = {
  filter: { defaultWidth: number; minWidth: number };
  results: { defaultWidth: number; minWidth: number };
  detail: { defaultWidth: number; minWidth: number };
};

type WorkspaceLayoutProps = {
  filter: React.ReactNode;
  responsiveSearch?: boolean;
  searchControls?: React.ReactNode;
  activeSearchFilter?: React.ReactNode;
  filterHeaderActions?: React.ReactNode;
  results: React.ReactNode;
  resultsHeaderActions?: React.ReactNode;
  detail?: React.ReactNode;
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
  responsiveSearch = false,
  searchControls,
  activeSearchFilter,
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
  const [viewportWidth, setViewportWidth] = useState(() => window.innerWidth);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [resultsForRecord, setResultsForRecord] = useState<string | null>(null);
  useEffect(() => {
    if (!responsiveSearch) return;
    const resize = () => setViewportWidth(window.innerWidth);
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [responsiveSearch]);
  const compact = responsiveSearch && viewportWidth <= 1100;
  const narrow = compact && viewportWidth <= 760;
  const showDetail =
    Boolean(selectedRecordKey) && resultsForRecord !== selectedRecordKey;
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

  const hasDetail = detail !== undefined;

  const panes = (
    <ResizablePaneGroup
      className={`workspace-grid${compact ? " workspace-grid--compact" : ""}${narrow ? (showDetail ? " workspace-grid--detail" : " workspace-grid--results") : ""}`}
      widthSpecs={effectiveWidthSpecs}
      items={(widths) => {
        const items: ResizablePaneItem<PaneKey>[] = [
          {
            kind: "pane" as const,
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
            kind: "handle" as const,
            key: "filter-results",
            disabled: effectiveCollapsed.filter && effectiveCollapsed.results,
            label: `Resize ${paneLabels.filter.toLowerCase()}`,
            resizePane: "filter",
            deltaMultiplier: 1,
          },
          {
            kind: "pane" as const,
            key: "results",
            column: hasDetail
              ? paneColumn("results", effectiveCollapsed, widths, sizing)
              : twoPaneResultsColumn(effectiveCollapsed),
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
        ];
        if (hasDetail) {
          items.push(
            {
              kind: "handle" as const,
              key: "results-detail",
              disabled: effectiveCollapsed.results && effectiveCollapsed.detail,
              label: `Resize ${paneLabels.results.toLowerCase()}`,
              resizePane: sizing === "detail-focus" ? "results" : "detail",
              deltaMultiplier: sizing === "detail-focus" ? 1 : -1,
            },
            {
              kind: "pane" as const,
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
          );
        }
        return compact
          ? items
              .filter(
                (entry) => entry.key !== "filter" && entry.key !== "filter-results",
              )
              .map((entry) =>
                entry.kind === "pane" ? { ...entry, column: "minmax(0, 1fr)" } : entry,
              )
          : items;
      }}
    />
  );
  if (!responsiveSearch) return panes;

  return (
    <div
      className={
        compact ? "search-workspace search-workspace--compact" : "search-workspace"
      }
    >
      {compact || activeSearchFilter ? (
        <div className="search-workspace__toolbar">
          {compact ? searchControls : null}
          {activeSearchFilter}
          {compact ? (
            <Space wrap>
              <Button onClick={() => setFiltersOpen(true)}>Filters</Button>
              {narrow && selectedRecordKey ? (
                <Button
                  onClick={() =>
                    setResultsForRecord(showDetail ? selectedRecordKey : null)
                  }
                >
                  {showDetail ? "Back to results" : "Show selected record"}
                </Button>
              ) : null}
            </Space>
          ) : null}
        </div>
      ) : null}
      <Drawer
        title="Search filters"
        open={compact && filtersOpen}
        onClose={() => setFiltersOpen(false)}
        width={Math.min(400, viewportWidth)}
      >
        {compact ? filter : null}
      </Drawer>
      {panes}
    </div>
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

function twoPaneResultsColumn(collapsed: Record<PaneKey, boolean>): string {
  if (collapsed.results) {
    return `${COLLAPSED_WIDTH}px`;
  }
  if (collapsed.filter) {
    return "minmax(0, 1fr)";
  }
  return "minmax(0, 1fr)";
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
          <PaneIconButton
            icon={
              collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />
            }
            label={collapsed ? `Expand ${label}` : `Collapse ${label}`}
            onClick={onToggle}
          />
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
