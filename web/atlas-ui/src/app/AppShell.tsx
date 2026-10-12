import { Activity, Moon, RefreshCw, Sun } from "lucide-react";
import { useRef, useState } from "react";
import { Alert, Button, Menu, Tag, Typography } from "antd";
import type { MenuProps } from "antd";
import type {
  ColorSchemePreference,
  ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { pagePositionLabel } from "../features/search/pageMetrics";
import {
  encountersPath,
  listsPath,
  searchPath,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "./routes";
import type { SearchWorkspaceState } from "../features/search/useSearchWorkspace";

type AppShellProps = {
  activeView: AtlasRoute["kind"];
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
  onNavigateLists: () => void;
  onNavigateEncounters: () => void;
  onNavigateSearch: () => void;
  resolvedColorScheme: ResolvedColorScheme;
  workspace: SearchWorkspaceState;
  children: React.ReactNode;
};

export function AppShell({
  activeView,
  colorScheme,
  onColorSchemeChange,
  onNavigateLists,
  onNavigateEncounters,
  onNavigateSearch,
  resolvedColorScheme,
  workspace,
  children,
}: AppShellProps) {
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const readiness = workspace.readiness.data;
  const status = readiness?.status ?? "blocked";
  const nextColorScheme = cycleColorScheme(colorScheme);
  const colorSchemeTitle = `Color: ${colorScheme}. Click for ${nextColorScheme}.`;
  const colorSchemeIcon =
    colorScheme === "system" ? (
      <SystemThemeIcon />
    ) : colorScheme === "light" ? (
      <Sun size={18} />
    ) : (
      <Moon size={18} />
    );
  const selectedView = topLevelView(activeView);
  const destinations = [
    { key: "search", label: "Search", href: searchPath(), navigate: onNavigateSearch },
    {
      key: "encounters",
      label: "Encounters",
      href: encountersPath(),
      navigate: onNavigateEncounters,
    },
    { key: "lists", label: "Lists", href: listsPath(), navigate: onNavigateLists },
  ];
  const navItems: MenuProps["items"] = destinations.map((destination) => ({
    key: destination.key,
    label: (
      <a
        href={destination.href}
        aria-current={selectedView === destination.key ? "page" : undefined}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          destination.navigate();
        }}
      >
        {destination.label}
      </a>
    ),
  }));
  const onNavClick: MenuProps["onClick"] = ({ key, domEvent }) => {
    // Ant also lets keyboard users activate the focused menu item itself.
    // Anchor activation is handled above so modified clicks stay native.
    if (
      domEvent.type !== "keydown" ||
      (domEvent.target instanceof Element && domEvent.target.closest("a")) ||
      domEvent.metaKey ||
      domEvent.ctrlKey ||
      domEvent.altKey ||
      domEvent.shiftKey
    )
      return;
    destinations.find((destination) => destination.key === key)?.navigate();
  };

  return (
    <div
      className={diagnosticsOpen ? "atlas-app atlas-app--diagnostics" : "atlas-app"}
      data-theme={resolvedColorScheme}
    >
      <Typography.Link
        className="skip-link"
        href="#atlas-main"
        onClick={() => contentRef.current?.focus()}
      >
        Skip to main content
      </Typography.Link>
      <header className="topbar">
        <div className="topbar__identity">
          <h1>PF2e Atlas</h1>
          <p>{readiness?.message ?? "Waiting for the local Atlas service."}</p>
        </div>
        <Menu
          aria-label="Atlas views"
          className="topbar__nav"
          disabledOverflow
          items={navItems}
          mode="horizontal"
          onClick={onNavClick}
          selectedKeys={[selectedView]}
        />
        <div className="topbar__actions">
          <Tag
            className="status-pill"
            color={status === "ready" ? "success" : "warning"}
          >
            {status}
          </Tag>
          <Button
            aria-label="Diagnostics"
            aria-pressed={diagnosticsOpen}
            icon={<Activity size={18} />}
            onClick={() => setDiagnosticsOpen(!diagnosticsOpen)}
            title="Diagnostics"
          />
          <Button
            aria-label="Refresh"
            icon={<RefreshCw size={18} />}
            onClick={workspace.refresh}
            title="Refresh"
          />
          <Button
            aria-label={colorSchemeTitle}
            icon={colorSchemeIcon}
            onClick={() => onColorSchemeChange(nextColorScheme)}
            title={colorSchemeTitle}
          />
        </div>
      </header>
      {workspace.errorMessage && (
        <Alert
          className="error-banner"
          type="error"
          showIcon
          title={workspace.errorMessage}
          action={
            workspace.resultsPaused ? (
              <Button
                onClick={() => void workspace.refreshResults()}
                loading={workspace.resultsRecovering}
              >
                Refresh results
              </Button>
            ) : undefined
          }
        />
      )}
      {diagnosticsOpen && <DiagnosticsPanel workspace={workspace} />}
      <div
        className="atlas-app__content"
        id="atlas-main"
        ref={contentRef}
        tabIndex={-1}
      >
        {children}
      </div>
    </div>
  );
}

function topLevelView(activeView: AtlasRoute["kind"]): string {
  if (activeView === "encounter" || activeView === "encounterEdit") {
    return "encounters";
  }
  if (activeView === "list" || activeView === "listEdit") {
    return "lists";
  }
  return activeView;
}

function DiagnosticsPanel({ workspace }: { workspace: SearchWorkspaceState }) {
  const { diagnostics, resultPage } = workspace;
  return (
    <section className="diagnostics-panel" aria-label="Diagnostics">
      <DiagnosticItem
        label="Readiness"
        value={workspace.readiness.data?.status ?? "loading"}
      />
      <DiagnosticItem
        label="Search"
        value={diagnostics.searchDebouncing ? "debouncing" : "settled"}
      />
      <DiagnosticItem
        label="Result request"
        value={
          diagnostics.resultRequest
            ? `${diagnostics.resultRequest.kind} ${diagnostics.resultRequest.durationMs}ms`
            : "pending"
        }
      />
      <DiagnosticItem
        label="Records"
        value={
          workspace.resultsRefreshing
            ? "updating"
            : resultPage
              ? resultPage.page.total.toLocaleString()
              : "pending"
        }
      />
      <DiagnosticItem
        label="Page"
        value={
          workspace.resultsRefreshing
            ? `loading ${workspace.pageNumber.toLocaleString()}`
            : resultPage
              ? pagePositionLabel(resultPage.page)
              : `${workspace.pageNumber}`
        }
      />
      <DiagnosticItem
        label="Detail request"
        value={
          diagnostics.detailRequest
            ? `${diagnostics.detailRequest.durationMs}ms`
            : workspace.selectedRecordKey
              ? "pending"
              : "none"
        }
      />
    </section>
  );
}

function DiagnosticItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="diagnostics-panel__item">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SystemThemeIcon() {
  return (
    <svg
      aria-hidden="true"
      className="theme-icon"
      fill="none"
      height="18"
      viewBox="0 0 24 24"
      width="18"
    >
      <circle cx="12" cy="12" r="9" stroke="currentColor" strokeWidth="2" />
      <path d="M12 3a9 9 0 0 0 0 18z" fill="currentColor" />
    </svg>
  );
}

function cycleColorScheme(colorScheme: ColorSchemePreference): ColorSchemePreference {
  if (colorScheme === "system") {
    return "light";
  }
  if (colorScheme === "light") {
    return "dark";
  }
  return "system";
}
