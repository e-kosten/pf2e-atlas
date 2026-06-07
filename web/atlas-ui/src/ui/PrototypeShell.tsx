import { Moon, RefreshCw, Sun } from "lucide-react";
import type { ColorSchemePreference, ResolvedColorScheme } from "./atlasTheme";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";
import type { LibraryPrototype } from "../state/searchState";

type PrototypeShellProps = {
  activeLibrary: LibraryPrototype;
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
  onLibraryChange: (library: LibraryPrototype) => void;
  resolvedColorScheme: ResolvedColorScheme;
  workspace: AtlasWorkspaceState;
  children: React.ReactNode;
};

export function PrototypeShell({
  activeLibrary,
  colorScheme,
  onColorSchemeChange,
  onLibraryChange,
  resolvedColorScheme,
  workspace,
  children,
}: PrototypeShellProps) {
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
    <div className="atlas-app" data-theme={resolvedColorScheme}>
      <header className="topbar">
        <div>
          <h1>PF2e Atlas</h1>
          <p>{readiness?.message ?? "Waiting for the local Atlas service."}</p>
        </div>
        <div className="topbar__actions">
          <span className={`status-pill status-pill--${status}`}>{status}</span>
          <div className="library-switch" role="tablist" aria-label="Component library">
            <button
              aria-selected={activeLibrary === "ant"}
              onClick={() => onLibraryChange("ant")}
              type="button"
            >
              AntD
            </button>
            <button
              aria-selected={activeLibrary === "mantine"}
              onClick={() => onLibraryChange("mantine")}
              type="button"
            >
              Mantine
            </button>
          </div>
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
      {children}
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

function cycleColorScheme(
  colorScheme: ColorSchemePreference,
): ColorSchemePreference {
  if (colorScheme === "system") {
    return "light";
  }
  if (colorScheme === "light") {
    return "dark";
  }
  return "system";
}
