import { Activity, Moon, RefreshCw, Sun } from "lucide-react";
import { useState } from "react";
import type {
  ColorSchemePreference,
  ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { pagePositionLabel } from "../features/search/pageMetrics";
import type { AtlasRoute } from "./routes";
import type { SearchWorkspaceState } from "../features/search/useSearchWorkspace";

type AppShellProps = {
  activeView: AtlasRoute["kind"];
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
  onNavigateLists: () => void;
  onNavigateEncounters: () => void;
  onNavigatePresentationMocks: () => void;
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
  onNavigatePresentationMocks,
  onNavigateSearch,
  resolvedColorScheme,
  workspace,
  children,
}: AppShellProps) {
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
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

  return (
    <div
      className={diagnosticsOpen ? "atlas-app atlas-app--diagnostics" : "atlas-app"}
      data-theme={resolvedColorScheme}
    >
      <header className="topbar">
        <div className="topbar__identity">
          <h1>PF2e Atlas</h1>
          <p>{readiness?.message ?? "Waiting for the local Atlas service."}</p>
        </div>
        <nav className="topbar__nav" aria-label="Atlas views">
          <button
            aria-current={activeView === "search" ? "page" : undefined}
            className="topbar__nav-item"
            onClick={onNavigateSearch}
            type="button"
          >
            Search
          </button>
          <button
            aria-current={activeView === "presentationMocks" ? "page" : undefined}
            className="topbar__nav-item"
            onClick={onNavigatePresentationMocks}
            type="button"
          >
            Surface Mocks
          </button>
          <button
            aria-current={
              activeView === "encounters" ||
              activeView === "encounter" ||
              activeView === "encounterEdit"
                ? "page"
                : undefined
            }
            className="topbar__nav-item"
            onClick={onNavigateEncounters}
            type="button"
          >
            Encounters
          </button>
          <button
            aria-current={
              activeView === "lists" ||
              activeView === "list" ||
              activeView === "listEdit"
                ? "page"
                : undefined
            }
            className="topbar__nav-item"
            onClick={onNavigateLists}
            type="button"
          >
            Lists
          </button>
        </nav>
        <div className="topbar__actions">
          <span className={`status-pill status-pill--${status}`}>{status}</span>
          <button
            className="icon-button"
            aria-pressed={diagnosticsOpen}
            onClick={() => setDiagnosticsOpen(!diagnosticsOpen)}
            title="Diagnostics"
            type="button"
          >
            <Activity size={18} />
          </button>
          <button
            className="icon-button"
            onClick={workspace.refresh}
            title="Refresh"
            type="button"
          >
            <RefreshCw size={18} />
          </button>
          <button
            aria-label={colorSchemeTitle}
            className="icon-button"
            onClick={() => onColorSchemeChange(nextColorScheme)}
            title={colorSchemeTitle}
            type="button"
          >
            {colorSchemeIcon}
          </button>
        </div>
      </header>
      {workspace.errorMessage && (
        <div className="error-banner">{workspace.errorMessage}</div>
      )}
      {diagnosticsOpen && <DiagnosticsPanel workspace={workspace} />}
      {children}
    </div>
  );
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
